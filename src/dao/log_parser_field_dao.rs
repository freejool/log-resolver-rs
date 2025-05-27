use diesel::{BoolExpressionMethods, OptionalExtension};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::models::*;
use crate::schema;

pub fn query_by_log_parser_rule_id_and_name_in_capture<T>(
    conn: &mut diesel::MysqlConnection,
    id: T,
    name_in_capture: &str,
) -> Option<LogParserField>
where
    T: Into<u64>,
{
    let idu64 = id.into();
    log::debug!("id: {:?} name_in_capture: {name_in_capture}", idu64);
    let log_parser_field = schema::log_parser_field::dsl::log_parser_field
        .filter(
            schema::log_parser_field::log_parser_rule_id
                .eq(idu64)
                .and(schema::log_parser_field::name_in_capture.eq(name_in_capture)),
        )
        .select(LogParserField::as_select())
        .first(conn)
        .optional();

    match log_parser_field {
        Ok(l) => l,
        Err(error) => {
            log::error!("Error loading log_parser_rule: {:?}", error);
            None
        }
    }
}
