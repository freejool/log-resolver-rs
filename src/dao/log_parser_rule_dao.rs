use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::models::*;
use crate::schema;

pub fn query_by_id<T>(conn: &mut diesel::MysqlConnection, id: T) -> Option<LogParserRule>
where
    T: Into<u64>,
{
    let idu64 = id.into();
    log::debug!("query_by_id: {:?}", idu64);
    let log_parser_rule = crate::schema::log_parser_rule::dsl::log_parser_rule
        .filter(schema::log_parser_rule::id.eq(idu64))
        .select(LogParserRule::as_select())
        .first(conn)
        .optional();

    match log_parser_rule {
        Ok(post) => post,
        Err(error) => {
            log::error!("Error loading log_parser_rule: {:?}", error);
            None
        }
    }
}
