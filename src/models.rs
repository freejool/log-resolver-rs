
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::schema;


#[derive(Queryable, Selectable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = schema::sys_subsys_config)]
// 显式检查 MySQL 后端有助于捕获类型不匹配
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct SysSubsysConfig {
    // Diesel 使用特定的 SQL 类型来表示无符号整数
    #[diesel(sql_type = Unsigned<BigInt>)]
    pub id: u64,
    pub sys_code: String,
    pub sys_name: Option<String>,
    pub subsys_code: String,
    pub subsys_name: Option<String>,
}


#[derive(Queryable, Selectable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = schema::subsys_log_parser)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct SubsysLogParser {
    #[diesel(sql_type = Unsigned<BigInt>)]
    pub id: u64,
    pub subsys_code: String,
    #[diesel(sql_type = diesel::sql_types::Unsigned<diesel::sql_types::BigInt>)]
    pub log_parser_rule_id: u64,
    pub file_name: Option<String>,
    pub status: bool,
    pub log_split: Option<String>,
    pub source_topic: String,
}


#[derive(Queryable, Selectable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = schema::log_parser_rule)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct LogParserRule {
    #[diesel(sql_type = diesel::sql_types::Unsigned<diesel::sql_types::BigInt>)]
    pub id: u64,
    pub name: Option<String>,
    pub status: bool,
    pub chinese_name: Option<String>,
}

#[derive(Queryable, Selectable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = schema::log_parser_pattern)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct LogParserPattern {
    #[diesel(sql_type = diesel::sql_types::Unsigned<diesel::sql_types::BigInt>)]
    pub id: u64,
    #[diesel(sql_type = diesel::sql_types::Unsigned<diesel::sql_types::BigInt>)]
    pub log_parser_rule_id: u64,
    pub name: Option<String>,
    pub pattern: Option<String>,
}

#[derive(Queryable, Selectable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = schema::log_parser_field)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct LogParserField {
    #[diesel(sql_type = Unsigned<BigInt>)]
    pub id: u64,
    #[diesel(sql_type = Unsigned<BigInt>)]
    pub log_parser_rule_id: u64,
    pub name: Option<String>,
    pub name_in_capture: String,
    // 'type' is a Rust keyword, rename it and map using column_name
    pub type_: i32, // Mapped from INT NOT NULL
    pub format_pattern: Option<String>,
    pub default_val: Option<String>,
    pub is_sensitive: Option<bool>,
}
