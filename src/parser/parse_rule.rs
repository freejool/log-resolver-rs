use std::collections::HashMap;

use regex;
use time::OffsetDateTime;

pub struct ParseRule {
    subsys_code: String,
    split_pattern: regex::Regex,
    date_time_pattern: regex::Regex,
    attribute_parse_rules: AttributeParseRule,
}

pub struct AttributeParseRule {
    pattern: regex::Regex,
}

pub struct Log {
    date_time: OffsetDateTime,
    raw_log: String,
    attibutes: HashMap<String, String>,
    subsys_code: String,
}
