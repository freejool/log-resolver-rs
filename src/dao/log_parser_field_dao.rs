use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::models::*;
use crate::schema;

pub fn query_by_log_parser_rule_id<T>(
    conn: &mut diesel::MysqlConnection,
    id: T,
) -> Vec<LogParserField>
where
    T: Into<u64>,
{
    let idu64 = id.into();
    log::debug!("id: {:?}", idu64);
    let log_parser_field = crate::schema::log_parser_field::dsl::log_parser_field
        .filter(schema::log_parser_field::log_parser_rule_id.eq(idu64))
        .select(LogParserField::as_select())
        .get_results(conn);

    match log_parser_field {
        Ok(l) => l,
        Err(error) => {
            log::error!("Error loading log_parser_rule: {:?}", error);
            Vec::new()
        }
    }
}
