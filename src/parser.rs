use once_cell::sync::Lazy;
use parse_rule::ParseRule;
use regex::Regex;
use std::collections::HashMap;

mod parse_rule;

static MAIN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)^\[\[(.*?)\]\](.*)$").unwrap());
static KV_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^=\]]+)=([^\]]*)\]").unwrap());

pub fn parse_log_block(
    log_block: &str,
    parse_rule: &ParseRule,
) -> Option<(HashMap<String, String>, String)> {
    if let Some(caps) = MAIN_RE.captures(log_block) {
        let header_content = caps.get(1).map_or("", |m| m.as_str());
        let attributes = parse_header(header_content);
        let raw_log = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

        Some((attributes, raw_log))
    } else {
        None
    }
}

fn parse_header(header_content: &str) -> HashMap<String, String> {
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
