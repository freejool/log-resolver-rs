use log_resolver_rs::parser::parse_log_block;

// --- 主函数：示例用法 ---
fn main() {
    let log_data = r#"[[version=0.1.2][hostname=localhost][ip=180.23.1.1][subsyscode=null][encode-UTF-8][filename=executor.log][file_offset=203311111][data_length=143][file_line=8587][file_line_count=3][block_index=4073][path=/host/applogs/openbank-22222/executor.log][compress_algorithm=null][topic=hzbuls][pattern=(?=((\r|\n)\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2}))][fields0.CLUSTERNAME=prd-wy-k8sca][fields0.HOSTNAME=openbank-ceuexe-5455c6b48b-rn9mm][fields0.SERVICEGROUP=cm][fields0.SUBSYSCODE=SUBSYS_OPENBANK_CEUEXE][fields0.container_path=/applogs/openbank-ceuexe-5455c6b48b-rn9mm][fields0.encode=UTF-8][fields0.files=*.log][fields0.k8s_container_name=openbankceuexe-hsbt-executor-ceu-arm][fields0.k8s_node_name=192.168.154.53-share][fields0.k8s_pod=openbank-ceuexe-5455c6b48b-rn9mm][fields0.k8s_pod_namespace=hzbank-openbankapp][fields0.k8s_pod_uid=cbdfeaae-479a-4d18-a525-aeb3538a8638][fields0.path=/host/applogs/openbank-ceuexe-5455c6b48b-rn9mm][fields0.pattern=(?=((\r|\n)\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2}))][fields0.topic=hzbuls]][2025-04-25 09:02:20.023][pool-3-thread-3][INFO ][com.netflix.config.ChainedDynamicProperty] Flipping property: default.ribbon.ActiveConnectionsLimit to use NEXT property: niws.loadbalancer.availabilityFilteringRule.activeConnectionsLimit = 2147483647
[2025-04-25 09:02:20.038][pool-3-thread-3][INFO ][com.hzbank.hsbt.executor.core.thread.ExecutorRegistryThread] >>>>>>>>>>> executor registry success, registryParam:RegistryParam [registGroup=EXECUTOR, registryKey=SUBSYS_OPENBANK_CEUEXE, registryValue=138.135.16.240:9995, registryMaxThreadNum=50, registryCoreThreadNum=50, registryAvailableThreadNum=50], registryResult:ReturnT [code=200, msg=null, content=null]"#;

    match parse_log_block(log_data) {
        Some((attributes, message)) => {
            println!("--- 解析成功 ---");

            println!("\n--- 属性 (Attributes Map) ---");
            // 为了方便查看，对属性按key排序输出
            let mut sorted_attributes: Vec<_> = attributes.iter().collect();
            sorted_attributes.sort_by_key(|(k, _)| *k);
            for (key, value) in sorted_attributes {
                println!("{}: {}", key, value);
            }

            println!("\n--- 日志原文 (Log Message) ---");
            println!("{}", message);
        }
        None => {
            println!("解析失败：日志格式不匹配。");
        }
    }

    // 测试一个格式错误的例子
    let invalid_log_data = "这是一个不包含元数据的普通日志行";
    println!("\n--- 测试无效日志 ---");
    match parse_log_block(invalid_log_data) {
        Some(_) => println!("错误：不应解析成功！"),
        None => println!("解析失败：日志格式不匹配。（预期行为）"),
    }
}
