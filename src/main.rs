use anyhow::anyhow;
use chrono::prelude::*;
use diesel::MysqlConnection;
use diesel::sql_types::ops::Mul;
use encoding_rs;
use encoding_rs::Encoding;
use env_logger;
use log::{info, warn};
use log_resolver_rs::dao::{
    log_parser_field_dao, log_parser_pattern_dao, log_parser_rule_dao,
    subsys_log_parser_config_dao, sys_subsys_config_dao,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::any;
use std::borrow::Cow;
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut context = ApplicationContext::from(
        <diesel::MysqlConnection as diesel::Connection>::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url)),
    );

    // loop {
    let records: Vec<Record> = poll_records().unwrap();
    log::info!("{}", records.len());
    for record in records {
        log::info!("{} {}", record.key, String::from_utf8_lossy(&record.value));
        let logs = parse_log(&mut context, &record.value)?;
        log::debug!("{logs:?}");
    }
    Ok(())
}
static MAIN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)^\[\[(.*?)\]\](.*)$").unwrap());
static KV_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^=\[\]]+)=([^\[\]]*)\]").unwrap());
static DELIMITER: &'static [u8] = b"]]";
fn parse_header_kv(header_content: &str) -> HashMap<String, String> {
    let mut attributes = HashMap::new();
    for kv_caps in KV_RE.captures_iter(header_content) {
        if let (Some(key_match), Some(value_match)) = (kv_caps.get(1), kv_caps.get(2)) {
            log::debug!(
                "key: {}, value: {}",
                key_match.as_str(),
                value_match.as_str()
            );
            attributes.insert(
                key_match.as_str().to_string(),
                value_match.as_str().to_string(),
            );
        }
    }
    attributes
}
#[derive(Debug)]
pub struct ParsedLog<'a> {
    pub header: LogHeader,
    pub log_content: Cow<'a, str>,
}

#[derive(Debug, Clone)]
pub struct LogHeader {
    pub subsys_code: String,
    pub encode: &'static encoding_rs::Encoding,
    pub attr: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Log<'a> {
    pub date_time: DateTime<Local>,
    pub log_header: LogHeader,
    pub log_content: Cow<'a, str>,
}

impl LogHeader {
    fn from_bytes(header_bytes: &[u8]) -> anyhow::Result<Self> {
        let header_str = std::str::from_utf8(header_bytes)?; // 如果非 ASCII 会在此处报错
        log::debug!("header: {:?}", header_str);
        let headers = parse_header_kv(header_str);
        let encoding_label_opt = headers.get("encode").map(|s| s.to_string());

        let encoding_label = encoding_label_opt.as_deref().unwrap_or("UTF-8");

        let encoding = get_encoding_from_label(encoding_label).unwrap();

        let subsys_code = get_subsys_code(&headers).unwrap_or("null".to_string());
        log::debug!("{subsys_code}");

        Ok(LogHeader {
            subsys_code,
            encode: encoding,
            attr: headers,
        })
    }
}
fn parse_log<'a>(
    context: &mut ApplicationContext,
    raw_log: &'a Vec<u8>,
) -> anyhow::Result<Vec<Log<'a>>> {
    // 1: 找到头部和内容分隔符的位置
    let conn = context.conn();
    let delimiter_pos = raw_log
        .windows(DELIMITER.len())
        .position(|window| window == DELIMITER);
    let (header_bytes, log_content_bytes) = match delimiter_pos {
        Some(pos) => {
            let header = &raw_log[..pos + DELIMITER.len()];
            let content = &raw_log[pos + DELIMITER.len()..];
            (header, content)
        }
        None => return Err(anyhow!("log header delimiter not found")),
    };

    // 2: 解析头部
    let log_header = LogHeader::from_bytes(header_bytes)?;

    // 3: 解码日志字符串
    let (decoded_log_cow, actual_encoding, had_errors) =
        log_header.encode.decode(log_content_bytes);
    log::debug!(
        "Decoded log: {:?}, encoding: {:?}, had_errors: {:?}",
        decoded_log_cow,
        actual_encoding,
        had_errors
    );
    if had_errors {
        log::warn!(
            "Error while decoding log content from {}",
            log_header.subsys_code
        );
    }

    let sys_subsys_config =
        sys_subsys_config_dao::query_by_subsys_code(conn, &log_header.subsys_code).unwrap();
    let subsys_log_parser_config_list =
        subsys_log_parser_config_dao::query_by_subsys_code(conn, &log_header.subsys_code);

    let logs: anyhow::Result<Vec<Log<'_>>> = subsys_log_parser_config_list.into_iter().try_fold(
        vec![],
        |mut v, subsys_log_parser_config| {
            let logs = apply_parse_config(
                conn,
                &log_header,
                &decoded_log_cow,
                subsys_log_parser_config,
            )?;
            v.extend(logs);
            Ok(v)
        },
    );

    logs
}

fn apply_parse_config<'a>(
    conn: &mut MysqlConnection,
    log_header: &LogHeader,
    decoded_log_cow: &Cow<'a, str>,
    subsys_log_parser_config: log_resolver_rs::models::SubsysLogParser,
) -> anyhow::Result<Vec<Log<'a>>> {
    let parser_rule_id = subsys_log_parser_config.log_parser_rule_id;
    // let log_parser_rule = log_parser_rule_dao::query_by_id(conn, parser_rule_id).unwrap();
    let log_parser_pattern_list =
        log_parser_pattern_dao::query_by_log_parser_rule_id(conn, parser_rule_id);
    // 每个pattern都产生一个Log
    let logs: Vec<Log<'_>> = log_parser_pattern_list
        .into_iter()
        .filter_map(|log_parser_pattern| {
            let pattern =
                regex::Regex::new(log_parser_pattern.pattern.unwrap_or_default().as_str()).ok()?;

            pattern.captures(decoded_log_cow).map(|captures| {
                let mut log = Log {
                    date_time: Local::now(),
                    log_header: log_header.clone(),
                    log_content: decoded_log_cow.clone(),
                };
                // let mut attr = log.log_header.attr;

                pattern.capture_names().flatten().for_each(|group_name| {
                    let group_value = captures.name(group_name).unwrap();
                    if let Some(log_parser_field) =
                        log_parser_field_dao::query_by_log_parser_rule_id_and_name_in_capture(
                            conn,
                            parser_rule_id,
                            group_name,
                        )
                    {
                        match log_parser_field.type_ {
                            10 => {
                                chrono::DateTime::parse_from_str(
                                    group_value.as_str(),
                                    log_parser_field.format_pattern.unwrap().as_str(),
                                )
                                .map(|dt| log.date_time = dt.into())
                                .ok();
                            }
                            0 => {
                                log.log_header.attr.insert(
                                    group_name.to_string(),
                                    group_value.as_str().to_string(),
                                );
                            }
                            _ => log::warn!("unsupported group type"),
                        }
                    } else {
                        log.log_header
                            .attr
                            .insert(group_name.to_string(), group_value.as_str().to_string());
                    }
                });

                log
            })
        })
        .collect();
    log::info!("{:?}", subsys_log_parser_config);
    Ok(logs)
}

fn get_subsys_code(headers: &HashMap<String, String>) -> Option<String> {
    headers
        .get("fields0.SUBSYSCODE")
        .or_else(|| headers.get("subsyscode"))
        .cloned()
}

// 预定义常见的编码标签映射 (可选，提高查找效率)
static ENCODING_MAP: Lazy<HashMap<&'static str, &'static Encoding>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("utf-8", encoding_rs::UTF_8);
    m.insert("utf8", encoding_rs::UTF_8);
    m.insert("gbk", encoding_rs::GBK);
    m.insert("gb18030", encoding_rs::GB18030);
    // 添加其他你可能需要支持的编码
    m.insert("latin1", encoding_rs::WINDOWS_1252); // alias for latin1
    m.insert("windows-1252", encoding_rs::WINDOWS_1252);
    m
});

// 查找编码的辅助函数
fn get_encoding_from_label(label: &str) -> Option<&'static Encoding> {
    // 优先使用预定义 Map (更快)
    if let Some(encoding) = ENCODING_MAP.get(label.to_lowercase().as_str()) {
        return Some(encoding);
    }
    // 其次尝试 encoding_rs 的动态查找
    Encoding::for_label(label.as_bytes())
}

fn poll_records() -> anyhow::Result<Vec<Record>> {
    let records = vec![
        Record {
            key: "".to_string(),
            value: Vec::from(
                "[[subsyscode=SUBSYS_TEST][encode=utf-8]]2025-01-01 22:22:22.222 |INFO| MSG",
            ),
            timestamp: 1000000,
        },
        // Record {
        //     key: "".to_string(),
        //     value: Vec::from(""),
        //     timestamp: 1000001,
        // },
        // Record {
        //     key: "".to_string(),
        //     value: Vec::from(""),
        //     timestamp: 1000002,
        // },
    ];
    Ok(records)
}

struct Record {
    key: String,
    value: Vec<u8>,
    timestamp: u64,
}

pub struct ApplicationContext {
    // 数据库连接
    conn: diesel::MysqlConnection,
}

impl ApplicationContext {
    pub fn conn(&mut self) -> &mut MysqlConnection {
        &mut self.conn
    }

    pub fn from(conn: MysqlConnection) -> Self {
        Self { conn }
    }
}
