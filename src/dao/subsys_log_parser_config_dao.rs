use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::models::*;
use crate::schema;

pub fn query_by_subsys_code(
    conn: &mut diesel::MysqlConnection,
    subsys_code: &str,
) -> Vec<SubsysLogParser> {
    log::debug!("query_by_subsys_code: {}", subsys_code);
    let subsys_log_parser = crate::schema::subsys_log_parser::dsl::subsys_log_parser
        .filter(schema::subsys_log_parser::subsys_code.eq(subsys_code))
        .filter(schema::subsys_log_parser::status.eq(true))
        .select(SubsysLogParser::as_select())
        .get_results(conn); // This allows for returning an Option<Post>, otherwise it will throw an error

    match subsys_log_parser {
        Ok(post) => post,
        Err(error) => {
            log::error!("Error loading post: {:?}", error);
            Vec::new()
        }
    }
}
