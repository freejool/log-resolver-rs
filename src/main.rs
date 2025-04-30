use encoding_rs;
use encoding_rs;
use encoding_rs::Encoding;
use env_logger;
use log_resolver_rs::{dao::post, util::IsEmpty};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{collections::HashMap, error::Error, hash::Hash, os::macos::raw};

fn main() {
    env_logger::init();
    loop {
        let records: Vec<Record> = poll_records().unwrap();
        for record in records {
            log::info!(
                "{} {}",
                record.key,
                String::from_utf8(record.value).unwrap()
            );
            let log_header = parse_log(record.value);
        }
    }
}
static MAIN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)^\[\[(.*?)\]\](.*)$").unwrap());
static KV_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^=\]]+)=([^\]]*)\]").unwrap());

fn parse_header_kv(header_content: &str) -> HashMap<String, String> {
    let mut attributes = HashMap::new();
    for kv_caps in KV_RE.captures_iter(header_content) {
        if let (Some(key_match), Some(value_match)) = (kv_caps.get(1), kv_caps.get(2)) {
            attributes.insert(
                key_match.as_str().to_string(),
                value_match.as_str().to_string(),
            );
        }
    }
    attributes
}
#[derive(Debug)]
pub struct ParsedLog {
    pub headers: HashMap<String, String>,
    pub log_content: String,
}

struct LogHeader {
    subsys_code: String,
    encode: encoding_rs::Encoding,
    attr: HashMap<String, String>,
}
enum Header {
    ENCODING,
}

fn parse_log(raw_log: Vec<u8>) -> Result<LogHeader, Box<dyn Error>> {
    let delimiter = b"]]";
    let delimiter_pos = raw_log
        .windows(delimiter.len())
        .position(|window| window == delimiter);
    let (header_bytes, log_content_bytes) = match delimiter_pos {
        Some(pos) => {
            let header = &raw_log[..pos];
            let content = &raw_log[pos + delimiter.len()..];
            (header, content)
        }
        None => return Err(Error),
    };

    // 2. 解析头部 (保证是 ASCII)
    // 使用 from_utf8 验证并转换为 &str，因为 ASCII 是 UTF-8 的子集
    let header_str = str::from_utf8(header_bytes)?; // 如果非 ASCII 会在此处报错
    let headers = parse_header_kv(header_str);

    // 3. 从头部获取编码信息 (假设 key 是 'encode')
    let encoding_label_opt = headers.get("encode").map(|s| s.to_string()); // 克隆出来，避免生命周期问题

    let encoding_label = encoding_label_opt.as_deref().unwrap_or("UTF-8"); // 默认 UTF-8

    // 4. 根据标签获取编码器
    let encoding = get_encoding_from_label(encoding_label).ok_or_else(|| Error)?;

    // 5. 使用获取到的编码解码日志原文
    // decode() 返回 (Cow<'_, str>, &'static Encoding, bool)
    // 第一个元素是解码后的字符串，可能是借用的也可能是拥有的 (Cow)
    // 第二个元素是实际使用的编码 (可能与请求的不同，例如 BOM 检测)
    // 第三个元素表示是否有解码错误
    let (decoded_log_cow, actual_encoding, had_errors) = encoding.decode(log_content_bytes);
}

// 预定义常见的编码标签映射 (可选，提高查找效率)
static ENCODING_MAP: HashMap<&'static str, &'static Encoding> = {
    let mut m = HashMap::new();
    m.insert("utf-8", encoding_rs::UTF_8);
    m.insert("utf8", encoding_rs::UTF_8);
    m.insert("gbk", encoding_rs::GBK);
    m.insert("gb18030", encoding_rs::GB18030);
    // 添加其他你可能需要支持的编码
    m.insert("latin1", encoding_rs::WINDOWS_1252); // alias for latin1
    m.insert("windows-1252", encoding_rs::WINDOWS_1252);
    m
};

// 查找编码的辅助函数
fn get_encoding_from_label(label: &str) -> Option<&'static Encoding> {
    // 优先使用预定义 Map (更快)
    if let Some(encoding) = ENCODING_MAP.get(label.to_lowercase().as_str()) {
        return Some(encoding);
    }
    // 其次尝试 encoding_rs 的动态查找
    Encoding::for_label(label.as_bytes())
}

fn poll_records() -> Result<Vec<Record>, Box<dyn Error>> {
    let records = vec![
        Record {
            key: "".to_string(),
            value: Vec::from(""),
            timestamp: 1000000,
        },
        Record {
            key: "".to_string(),
            value: Vec::from(""),
            timestamp: 1000001,
        },
        Record {
            key: "".to_string(),
            value: Vec::from(""),
            timestamp: 1000002,
        },
    ];
    Ok(records)
}

struct Record {
    key: String,
    value: Vec<u8>,
    timestamp: u64,
}
