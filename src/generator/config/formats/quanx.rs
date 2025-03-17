use crate::generator::config::subexport::{
    group_generate, process_remark, ExtraSettings, ProxyGroupConfigs,
};
use crate::{Proxy, ProxyType};

use crate::parser::ruleset::RulesetContent;
use std::collections::HashMap;

/// Convert proxies to QuantumultX format
///
/// This function converts a list of proxies to the QuantumultX configuration format,
/// using a base configuration as a template and applying rules from ruleset_content_array.
///
/// # Arguments
/// * `nodes` - List of proxy nodes to convert
/// * `base_conf` - Base QuantumultX configuration as a string
/// * `ruleset_content_array` - Array of ruleset contents to apply
/// * `extra_proxy_group` - Extra proxy group configurations
/// * `ext` - Extra settings for conversion
pub fn proxy_to_quanx(
    nodes: &mut Vec<Proxy>,
    base_conf: &str,
    ruleset_content_array: &mut Vec<RulesetContent>,
    extra_proxy_group: &ProxyGroupConfigs,
    ext: &mut ExtraSettings,
) -> String {
    let mut proxy_config = String::new();
    let mut server_remote_config = String::new();
    let mut filter_remote_config = String::new();
    let mut rewrite_remote_config = String::new();
    let mut server_local_config = String::new();
    let mut filter_local_config = String::new();
    let mut rewrite_local_config = String::new();
    let mut server_tag = String::new();
    let mut base_config = base_conf.to_string();

    // Process proxies
    if !nodes.is_empty() {
        for node in nodes.iter_mut() {
            // Skip unsupported proxy types
            match node.proxy_type {
                ProxyType::Shadowsocks
                | ProxyType::ShadowsocksR
                | ProxyType::VMess
                | ProxyType::Trojan
                | ProxyType::HTTP
                | ProxyType::HTTPS => {}
                _ => continue,
            }

            // Process remark
            let mut remark = node.remark.clone();
            process_remark(&mut remark, ext, true);

            // Add proxy based on type
            match node.proxy_type {
                ProxyType::Shadowsocks => {
                    proxy_config.push_str(&format!(
                        "shadowsocks={}, {}, {}, {}, ",
                        node.server, node.port, node.cipher, node.password
                    ));

                    // Add tag
                    proxy_config.push_str(&format!("tag={}", remark));

                    // Add obfs if present
                    if !node.plugin.is_empty() {
                        if node.plugin == "obfs-local" || node.plugin == "simple-obfs" {
                            let mut plugin_opts = HashMap::new();
                            for opt in node.plugin_opts.split(';') {
                                let parts: Vec<&str> = opt.split('=').collect();
                                if parts.len() == 2 {
                                    plugin_opts.insert(parts[0], parts[1]);
                                }
                            }

                            if let Some(obfs) = plugin_opts.get("obfs") {
                                proxy_config.push_str(&format!(", obfs={}", obfs));

                                if let Some(host) = plugin_opts.get("obfs-host") {
                                    proxy_config.push_str(&format!(", obfs-host={}", host));
                                }
                            }
                        }
                    }
                }
                ProxyType::ShadowsocksR => {
                    proxy_config.push_str(&format!(
                        "shadowsocks={}, {}, {}, {}, ",
                        node.server, node.port, node.cipher, node.password
                    ));

                    // Add tag
                    proxy_config.push_str(&format!("tag={}", remark));

                    // Add SSR specific parameters
                    proxy_config.push_str(&format!(", ssr-protocol={}", node.protocol));

                    if !node.protocol_param.is_empty() {
                        proxy_config
                            .push_str(&format!(", ssr-protocol-param={}", node.protocol_param));
                    }

                    proxy_config.push_str(&format!(", obfs={}", node.obfs));

                    if !node.obfs_param.is_empty() {
                        proxy_config.push_str(&format!(", obfs-host={}", node.obfs_param));
                    }
                }
                ProxyType::VMess => {
                    proxy_config.push_str(&format!("vmess={}, {}, ", node.server, node.port));

                    // Add method
                    if !node.cipher.is_empty() {
                        proxy_config.push_str(&format!("method={}, ", node.cipher));
                    } else {
                        proxy_config.push_str("method=auto, ");
                    }

                    // Add password (UUID)
                    proxy_config.push_str(&format!("password={}, ", node.uuid));

                    // Add tag
                    proxy_config.push_str(&format!("tag={}", remark));

                    // Add obfs settings
                    if !node.network.is_empty() {
                        match node.network.as_str() {
                            "tcp" => {
                                proxy_config.push_str(", obfs=none");
                            }
                            "ws" => {
                                proxy_config.push_str(", obfs=ws");

                                if !node.path.is_empty() {
                                    proxy_config.push_str(&format!(", obfs-uri={}", node.path));
                                }

                                if !node.host.is_empty() {
                                    proxy_config.push_str(&format!(", obfs-host={}", node.host));
                                }
                            }
                            "h2" => {
                                proxy_config.push_str(", obfs=h2");

                                if !node.path.is_empty() {
                                    proxy_config.push_str(&format!(", obfs-uri={}", node.path));
                                }

                                if !node.host.is_empty() {
                                    proxy_config.push_str(&format!(", obfs-host={}", node.host));
                                }
                            }
                            "grpc" => {
                                proxy_config.push_str(", obfs=grpc");

                                if !node.path.is_empty() {
                                    proxy_config.push_str(&format!(", obfs-uri={}", node.path));
                                }
                            }
                            _ => {}
                        }
                    }

                    // Add alterId if present
                    if node.alter_id > 0 {
                        proxy_config.push_str(&format!(", alterId={}", node.alter_id));
                    }

                    // Add TLS settings
                    if node.tls {
                        proxy_config.push_str(", tls=1");

                        if let Some(sni) = &node.sni {
                            if !sni.is_empty() {
                                proxy_config.push_str(&format!(", tls-host={}", sni));
                            }
                        }
                    } else {
                        proxy_config.push_str(", tls=0");
                    }
                }
                ProxyType::Trojan => {
                    proxy_config.push_str(&format!(
                        "trojan={}, {}, {}, ",
                        node.server, node.port, node.password
                    ));

                    // Add tag
                    proxy_config.push_str(&format!("tag={}", remark));

                    // Add TLS settings
                    if let Some(sni) = &node.sni {
                        if !sni.is_empty() {
                            proxy_config.push_str(&format!(", tls-host={}", sni));
                        }
                    }

                    // Add obfs settings for WebSocket
                    if !node.network.is_empty() && node.network == "ws" {
                        proxy_config.push_str(", obfs=ws");

                        if !node.path.is_empty() {
                            proxy_config.push_str(&format!(", obfs-uri={}", node.path));
                        }

                        if !node.host.is_empty() {
                            proxy_config.push_str(&format!(", obfs-host={}", node.host));
                        }
                    }
                }
                ProxyType::HTTP | ProxyType::HTTPS => {
                    if node.proxy_type == ProxyType::HTTP {
                        proxy_config.push_str(&format!("http={}, {}, ", node.server, node.port));
                    } else {
                        proxy_config.push_str(&format!("https={}, {}, ", node.server, node.port));
                    }

                    // Add username/password if present
                    if !node.username.is_empty() {
                        proxy_config.push_str(&format!("username={}, ", node.username));
                    }

                    if !node.password.is_empty() {
                        proxy_config.push_str(&format!("password={}, ", node.password));
                    }

                    // Add tag
                    proxy_config.push_str(&format!("tag={}", remark));
                }
                _ => continue,
            }

            // Add common fields
            if let Some(udp) = node.udp {
                proxy_config.push_str(&format!(", udp={}", if udp { "true" } else { "false" }));
            }

            if let Some(tfo) = node.tfo {
                proxy_config.push_str(&format!(
                    ", fast-open={}",
                    if tfo { "true" } else { "false" }
                ));
            }

            if let Some(scv) = node.skip_cert_verify {
                proxy_config.push_str(&format!(
                    ", tls-verification={}",
                    if scv { "false" } else { "true" }
                ));
            }

            proxy_config.push('\n');
        }
    }

    // Process proxy groups
    if !extra_proxy_group.is_empty() {
        for group in extra_proxy_group {
            // Skip unsupported group types
            match group.type_field.as_str() {
                "static" | "url-test" | "fallback" | "load-balance" => {}
                _ => continue,
            }

            // Create policy path for this group
            let mut group_path = String::new();

            for proxy_name in &group.proxies {
                let mut filtered_nodes = Vec::new();
                group_generate(proxy_name, nodes, &mut filtered_nodes, false, ext);
                for node in filtered_nodes {
                    group_path.push_str(&node);
                    group_path.push_str(", ");
                }
            }

            // Remove trailing comma and space
            if group_path.ends_with(", ") {
                group_path.truncate(group_path.len() - 2);
            }

            // Add group based on type
            match group.type_field.as_str() {
                "static" => {
                    server_tag.push_str(&format!("static={}, {}\n", group.name, group_path));
                }
                "url-test" => {
                    let mut test_url = "http://www.gstatic.com/generate_204";
                    let mut interval = 600;

                    if !group.url.is_empty() {
                        test_url = &group.url;
                    }

                    if group.interval > 0 {
                        interval = group.interval;
                    }

                    server_tag.push_str(&format!(
                        "url-test={}, {}, {}, {}\n",
                        group.name, group_path, test_url, interval
                    ));
                }
                "fallback" => {
                    let mut test_url = "http://www.gstatic.com/generate_204";
                    let mut interval = 600;

                    if !group.url.is_empty() {
                        test_url = &group.url;
                    }

                    if group.interval > 0 {
                        interval = group.interval;
                    }

                    server_tag.push_str(&format!(
                        "fallback={}, {}, {}, {}\n",
                        group.name, group_path, test_url, interval
                    ));
                }
                "load-balance" => {
                    server_tag.push_str(&format!("round-robin={}, {}\n", group.name, group_path));
                }
                _ => {}
            }
        }
    }

    // Process rules
    if ext.enable_rule_generator && !ruleset_content_array.is_empty() {
        for ruleset in ruleset_content_array {
            for rule in &ruleset.rules {
                let mut rule_str = String::new();

                match rule.rule_type.as_str() {
                    "DOMAIN" => {
                        rule_str = format!("HOST,{},{}", rule.rule_content, ruleset.group);
                    }
                    "DOMAIN-SUFFIX" => {
                        rule_str = format!("HOST-SUFFIX,{},{}", rule.rule_content, ruleset.group);
                    }
                    "DOMAIN-KEYWORD" => {
                        rule_str = format!("HOST-KEYWORD,{},{}", rule.rule_content, ruleset.group);
                    }
                    "IP-CIDR" => {
                        rule_str = format!("IP-CIDR,{},{}", rule.rule_content, ruleset.group);
                    }
                    "IP-CIDR6" => {
                        rule_str = format!("IP6-CIDR,{},{}", rule.rule_content, ruleset.group);
                    }
                    "GEOIP" => {
                        rule_str = format!("GEOIP,{},{}", rule.rule_content, ruleset.group);
                    }
                    "USER-AGENT" => {
                        rule_str = format!("USER-AGENT,{},{}", rule.rule_content, ruleset.group);
                    }
                    "FINAL" => {
                        rule_str = format!("FINAL,{}", ruleset.group);
                    }
                    _ => {}
                }

                if !rule_str.is_empty() {
                    filter_local_config.push_str(&rule_str);
                    filter_local_config.push('\n');
                }
            }
        }
    }

    // Combine all sections
    let mut config = String::new();

    // Add base config if not empty
    if !base_config.is_empty() {
        // Check if we need to overwrite original sections
        if ext.overwrite_original_rules {
            // Remove original sections
            let sections = [
                "[server_local]",
                "[server_remote]",
                "[filter_local]",
                "[filter_remote]",
                "[rewrite_local]",
                "[rewrite_remote]",
                "[task_local]",
                "[policy]",
            ];

            for section in &sections {
                if let Some(start) = base_config.find(section) {
                    if let Some(next_section) = base_config[start + section.len()..].find('[') {
                        let end = start + section.len() + next_section - 1;
                        base_config.replace_range(start..end, "");
                    } else {
                        base_config.truncate(start);
                    }
                }
            }
        }

        config.push_str(&base_config);

        // Add a newline if the base config doesn't end with one
        if !base_config.ends_with('\n') {
            config.push('\n');
        }
    }

    // Add server_local section
    if !proxy_config.is_empty() {
        config.push_str("[server_local]\n");
        config.push_str(&proxy_config);
        config.push('\n');
    }

    // Add server_remote section
    if !server_remote_config.is_empty() {
        config.push_str("[server_remote]\n");
        config.push_str(&server_remote_config);
        config.push('\n');
    }

    // Add policy section
    if !server_tag.is_empty() {
        config.push_str("[policy]\n");
        config.push_str(&server_tag);
        config.push('\n');
    }

    // Add filter_local section
    if !filter_local_config.is_empty() {
        config.push_str("[filter_local]\n");
        config.push_str(&filter_local_config);
        config.push('\n');
    }

    // Add filter_remote section
    if !filter_remote_config.is_empty() {
        config.push_str("[filter_remote]\n");
        config.push_str(&filter_remote_config);
        config.push('\n');
    }

    // Add rewrite_local section
    if !rewrite_local_config.is_empty() {
        config.push_str("[rewrite_local]\n");
        config.push_str(&rewrite_local_config);
        config.push('\n');
    }

    // Add rewrite_remote section
    if !rewrite_remote_config.is_empty() {
        config.push_str("[rewrite_remote]\n");
        config.push_str(&rewrite_remote_config);
        config.push('\n');
    }

    config
}
