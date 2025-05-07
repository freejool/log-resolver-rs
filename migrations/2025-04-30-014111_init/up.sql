create table if not exists sys_subsys_config
(
    id          bigint unsigned auto_increment primary key,
    sys_code    varchar(255) not null,
    sys_name    varchar(255) null,
    subsys_code varchar(255) not null,
    subsys_name varchar(255) null,
    constraint sys_subsys_uindex unique (sys_code, subsys_code)
);

insert into sys_subsys_config value (1, 'SYS_TEST', null, 'SUBSYS_TEST', null);

create table if not exists subsys_log_parser
(
    id                 bigint unsigned auto_increment primary key,
    subsys_code        varchar(255)    not null, -- 外键
    log_parser_rule_id bigint unsigned not null, -- 外键
    file_name          varchar(255)    null,     -- 正则表达式，空则匹配所有
    status             tinyint(1)      not null, -- 是否启用，1为启用，0为禁用
    log_split          varchar(255)    null,     -- 日志分割正则，将在每一个match的开头位置分割
    source_topic       varchar(255)    not null, -- 将发往的topic
    constraint subsys_log_parser_uindex unique (log_parser_rule_id, file_name, subsys_code)
);
insert into subsys_log_parser value (1, 'SUBSYS_TEST', 1, null, true, '\n', 'TOPIC');

create table if not exists log_parser_rule
(                                   --
    id           bigint unsigned auto_increment primary key,
    name         varchar(255) null, --
    status       tinyint(1)   not null,
    chinese_name varchar(255) null  --
);
insert into log_parser_rule value (1, 'default', true, null);

create table if not exists log_parser_pattern
(
    id                 bigint unsigned auto_increment primary key,
    log_parser_rule_id bigint unsigned not null, -- 外键
    name               varchar(255)    null,
    pattern            text            null      -- 正则，所有named capture应有对应的field
);
insert into log_parser_pattern value (1, 1, null,
                                      '(?P<dateTime>^\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2}\.\d{3,6})\s*|\s*(?P<level>INFO|ERROR|DEBUG)|$');

create table if not exists log_parser_field
(                                                -- 自定义的字段解析
    id                 bigint unsigned auto_increment primary key,
    log_parser_rule_id bigint unsigned not null, -- 外键
    name               varchar(255)    null,
    name_in_capture    varchar(255)    not null, -- 在log_parser_pattern中pattern中的name
    type               int             not null, -- 正则表达式 0:常规,10:日期
    format_pattern     varchar(255)    null,     -- 和type对应，如果是10则是时间格式
    default_val        varchar(1024)   null,     -- 默认值
    is_sensitive       tinyint(1)      null
);
insert into log_parser_field value (1, 1, null, 'dateTime', 10, 'yyyy-MM-dd hh:mm:ss.SSS', null, null);
insert into log_parser_field value (2, 1, null, 'level', 0, null, null, null);
