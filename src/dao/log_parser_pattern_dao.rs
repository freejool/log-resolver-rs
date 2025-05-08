use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::models::*;
use crate::schema;

pub fn query_by_log_parser_rule_id<T>(
    conn: &mut diesel::MysqlConnection,
    id: T,
) -> Vec<LogParserPattern>
where
    T: Into<u64>,
{
    let idu64 = id.into();
    log::debug!("id: {:?}", idu64);
    let log_parser_pattern = crate::schema::log_parser_pattern::dsl::log_parser_pattern
        .filter(schema::log_parser_pattern::log_parser_rule_id.eq(idu64))
        .select(LogParserPattern::as_select())
        .get_results(conn);

    match log_parser_pattern {
        Ok(l) => l,
        Err(error) => {
            log::error!("Error loading log_parser_rule: {:?}", error);
            Vec::new()
        }
    }
}
