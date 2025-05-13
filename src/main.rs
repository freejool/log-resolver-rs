use anyhow::anyhow;
use chrono::prelude::*;
use diesel::MysqlConnection;
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
use std::borrow::Cow;
use std::collections::HashMap;

fn main() {
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
        let log_header = parse_log(&mut context, &record.value);
        log::info!("{:?}", log_header);
    }
    // }
}
static MAIN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)^\[\[(.*?)\]\](.*)$").unwrap());
static KV_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^=\[\]]+)=([^\[\]]*)\]").unwrap());

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

#[derive(Debug)]
pub struct LogHeader {
    pub subsys_code: String,
    pub encode: &'static encoding_rs::Encoding,
    pub attr: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Log<'a> {
    pub date_time: DateTime<Local>,
    pub subsys_code: String,
    pub attr: HashMap<String, String>,
    pub log_content: Cow<'a, str>,
}

impl Log<'_> {
    fn new() -> Self {
        Log {
            date_time: Local::now(),
            subsys_code: "".to_string(),
            attr: HashMap::new(),
            log_content: Cow::Borrowed(""),
        }
    }

    fn get_attr_mut(&mut self) -> HashMap<String, String> {
        self.attr.clone()
    }

    fn set_date_time(&mut self, date_time: DateTime<Local>) {
        self.date_time = date_time;
    }
}

fn parse_log<'a>(
    context: &mut ApplicationContext,
    raw_log: &'a Vec<u8>,
) -> anyhow::Result<ParsedLog<'a>> {
    let conn = context.conn();
    let delimiter = b"]]";
    let delimiter_pos = raw_log
        .windows(delimiter.len())
        .position(|window| window == delimiter);
    let (header_bytes, log_content_bytes) = match delimiter_pos {
        Some(pos) => {
            let header = &raw_log[..pos + delimiter.len()];
            let content = &raw_log[pos + delimiter.len()..];
            (header, content)
        }
        None => return Err(anyhow!("log header delimiter not found")),
    };

    // 2. 解析头部 (保证是 ASCII)
    // 使用 from_utf8 验证并转换为 &str，因为 ASCII 是 UTF-8 的子集
    let header_str = std::str::from_utf8(header_bytes)?; // 如果非 ASCII 会在此处报错
    log::debug!("header: {:?}", header_str);
    let mut headers = parse_header_kv(header_str);

    // 3. 从头部获取编码信息 (假设 key 是 'encode')
    let encoding_label_opt = headers.get("encode").map(|s| s.to_string()); // 克隆出来，避免生命周期问题

    let encoding_label = encoding_label_opt.as_deref().unwrap_or("UTF-8"); // 默认 UTF-8

    // 4. 根据标签获取编码器
    let encoding = get_encoding_from_label(encoding_label).unwrap();

    // 5. 使用获取到的编码解码日志原文
    // decode() 返回 (Cow<'_, str>, &'static Encoding, bool)
    // 第一个元素是解码后的字符串，可能是借用的也可能是拥有的 (Cow)
    // 第二个元素是实际使用的编码 (可能与请求的不同，例如 BOM 检测)
    // 第三个元素表示是否有解码错误
    let (decoded_log_cow, actual_encoding, had_errors) = encoding.decode(log_content_bytes);
    log::debug!(
        "Decoded log: {:?}, encoding: {:?}, had_errors: {:?}",
        decoded_log_cow,
        actual_encoding,
        had_errors
    );
    let subsys_code = get_subsys_code(&headers).unwrap_or("null".to_string());
    if had_errors {
        warn!("Error while decoding log content from {subsys_code}");
    }
    log::debug!("{subsys_code}");
    let sys_subsys_config =
        sys_subsys_config_dao::query_by_subsys_code(conn, &subsys_code).unwrap();
    let subsys_log_parser_config_list =
        subsys_log_parser_config_dao::query_by_subsys_code(conn, &subsys_code);
    for subsys_log_parser_config in subsys_log_parser_config_list {
        let parser_rule_id = subsys_log_parser_config.log_parser_rule_id;
        if let Some(log_parser_rule) = log_parser_rule_dao::query_by_id(conn, parser_rule_id) {
            let log_parser_pattern_list =
                log_parser_pattern_dao::query_by_log_parser_rule_id(conn, parser_rule_id);
            let log_parser_field_list =
                log_parser_field_dao::query_by_log_parser_rule_id(conn, parser_rule_id);
            for log_parser_pattern in log_parser_pattern_list {
                let pattern =
                    regex::Regex::new(log_parser_pattern.pattern.unwrap_or_default().as_str())
                        .unwrap();
                log::info!("{}", pattern);
                if let Some(captures) = pattern.captures(&decoded_log_cow) {
                    log::debug!(
                        "  整体匹配到的内容: \"{}\"",
                        captures.get(0).map_or("", |m| m.as_str())
                    );
                    let mut named_captures = HashMap::<String, String>::new();
                    for group_name_option in pattern.capture_names() {
                        if let Some(group_name) = group_name_option {
                            let group_value = captures.name(group_name).unwrap();
                            let log_parser_field = log_parser_field_dao::query_by_log_parser_rule_id_and_name_in_capture(conn, id, name_in_capture);
                            if let Some(log_parser_field) = log_parser_field {
                                match log_parser_field.type_ {
                                    10 => {
                                        let dateTime = chrono::DateTime::parse_from_str(
                                            group_value.as_str(),
                                            log_parser_field.format_pattern.unwrap().as_str(),
                                        )?;
                                        let log = Log {
                                            date_time: dateTime.into(),
                                            subsys_code: subsys_code,
                                            attr: headers,
                                            log_content: decoded_log_cow.into(),
                                        };
                                    }
                                    0 => {
                                        let log = Log {
                                            date_time: Local::now(),
                                            subsys_code: subsys_code,
                                            attr: headers,
                                            log_content: decoded_log_cow.into(),
                                        };
                                    }
                                    t => {
                                        anyhow::bail!("unsupported group type: {}", t);
                                    }
                                }
                            }
                            named_captures.insert(
                                group_name.to_string(),
                                group_value.map_or(String::new(), |m| m.as_str().to_string()),
                            );
                        }
                    }
                }
            }
        }

        log::info!("{:?}", subsys_log_parser_config);
    }
    headers.insert("sys_code".to_string(), sys_subsys_config.sys_code);

    Ok(ParsedLog {
        log_content: decoded_log_cow,
        header: LogHeader {
            encode: actual_encoding,
            subsys_code,
            attr: headers,
        },
    })
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
