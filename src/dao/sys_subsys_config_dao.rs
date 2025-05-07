use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::models::*;
use crate::schema;

pub fn query_by_subsys_code(
    conn: &mut diesel::MysqlConnection,
    subsys_code: &str,
) -> Option<SysSubsysConfig> {
    log::debug!("query_by_subsys_code: {}", subsys_code);
    let sys_subsys_config = crate::schema::sys_subsys_config::dsl::sys_subsys_config
        .filter(schema::sys_subsys_config::subsys_code.eq(subsys_code))
        .select(SysSubsysConfig::as_select())
        .first(conn)
        .optional(); // This allows for returning an Option<Post>, otherwise it will throw an error

    match sys_subsys_config {
        Ok(post) => post,
        Err(error) => {
            log::error!("Error loading post: {:?}", error);
            None
        }
    }
}
