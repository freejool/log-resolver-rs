// @generated automatically by Diesel CLI.

diesel::table! {
    log_parser_field (id) {
        id -> Unsigned<Bigint>,
        log_parser_rule_id -> Unsigned<Bigint>,
        #[max_length = 255]
        name -> Nullable<Varchar>,
        #[max_length = 255]
        name_in_capture -> Varchar,
        #[sql_name = "type"]
        type_ -> Integer,
        #[max_length = 255]
        format_pattern -> Nullable<Varchar>,
        #[max_length = 1024]
        default_val -> Nullable<Varchar>,
        is_sensitive -> Nullable<Bool>,
    }
}

diesel::table! {
    log_parser_pattern (id) {
        id -> Unsigned<Bigint>,
        log_parser_rule_id -> Unsigned<Bigint>,
        #[max_length = 255]
        name -> Nullable<Varchar>,
        pattern -> Nullable<Text>,
    }
}

diesel::table! {
    log_parser_rule (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        name -> Nullable<Varchar>,
        status -> Bool,
        #[max_length = 255]
        chinese_name -> Nullable<Varchar>,
    }
}

diesel::table! {
    subsys_log_parser (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        subsys_code -> Varchar,
        log_parser_rule_id -> Unsigned<Bigint>,
        #[max_length = 255]
        file_name -> Nullable<Varchar>,
        status -> Bool,
        #[max_length = 255]
        log_split -> Nullable<Varchar>,
        #[max_length = 255]
        source_topic -> Varchar,
    }
}

diesel::table! {
    sys_subsys_config (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        sys_code -> Varchar,
        #[max_length = 255]
        sys_name -> Nullable<Varchar>,
        #[max_length = 255]
        subsys_code -> Varchar,
        #[max_length = 255]
        subsys_name -> Nullable<Varchar>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    log_parser_field,
    log_parser_pattern,
    log_parser_rule,
    subsys_log_parser,
    sys_subsys_config,
);
