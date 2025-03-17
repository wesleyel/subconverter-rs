use crate::generator::config::subexport::{
    group_generate, process_remark, ExtraSettings, ProxyGroupConfigs,
};
use crate::parser::ruleset::RulesetContent;
use crate::{Proxy, ProxyType};
use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;

/// Convert proxies to Quantumult format
///
/// This function converts a list of proxies to the Quantumult configuration format,
/// using a base configuration as a template and applying rules from ruleset_content_array.
///
/// # Arguments
/// * `nodes` - List of proxy nodes to convert
/// * `base_conf` - Base Quantumult configuration as a string
/// * `ruleset_content_array` - Array of ruleset contents to apply
/// * `extra_proxy_group` - Extra proxy group configurations
/// * `ext` - Extra settings for conversion
pub fn proxy_to_quan(
    nodes: &mut Vec<Proxy>,
    base_conf: &str,
    ruleset_content_array: &mut Vec<RulesetContent>,
    extra_proxy_group: &ProxyGroupConfigs,
    ext: &mut ExtraSettings,
) -> String {
    let mut proxy_config = String::new();
    let mut server_config = String::new();
    let mut filter_config = String::new();
    let mut base_config = base_conf.to_string();

    // Process proxies
    if !nodes.is_empty() {
        for node in nodes.iter_mut() {
            // Skip unsupported proxy types
            match node.proxy_type {
                ProxyType::Shadowsocks
                | ProxyType::ShadowsocksR
                | ProxyType::VMess
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
                        "{} = shadowsocks, {}, {}, {}, {}",
                        remark, node.server, node.port, node.cipher, node.password
                    ));

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
                        "{} = shadowsocksr, {}, {}, {}, {}, {}, {}, {}, {}",
                        remark,
                        node.server,
                        node.port,
                        node.cipher,
                        node.password,
                        node.protocol,
                        node.protocol_param,
                        node.obfs,
                        node.obfs_param
                    ));
                }
                ProxyType::VMess => {
                    proxy_config.push_str(&format!(
                        "{} = vmess, {}, {}, chacha20-poly1305, \"{}\", group={}",
                        remark, node.server, node.port, node.uuid, node.uuid
                    ));

                    // Add obfs settings
                    if !node.network.is_empty() {
                        match node.network.as_str() {
                            "tcp" => {
                                proxy_config.push_str(", obfs=none");
                            }
                            "ws" => {
                                proxy_config.push_str(", obfs=ws");

                                if !node.path.is_empty() {
                                    proxy_config
                                        .push_str(&format!(", obfs-path=\"{}\"", node.path));
                                }

                                if !node.host.is_empty() {
                                    proxy_config.push_str(&format!(
                                        ", obfs-header=\"Host: {}\"",
                                        node.host
                                    ));
                                }
                            }
                            _ => {}
                        }
                    }

                    // Add TLS settings
                    if node.tls {
                        proxy_config.push_str(", over-tls=true");

                        if let Some(sni) = &node.sni {
                            if !sni.is_empty() {
                                proxy_config.push_str(&format!(", tls-host={}", sni));
                            }
                        }
                    }
                }
                ProxyType::HTTP | ProxyType::HTTPS => {
                    if node.proxy_type == ProxyType::HTTP {
                        proxy_config.push_str(&format!(
                            "{} = http, {}, {}",
                            remark, node.server, node.port
                        ));
                    } else {
                        proxy_config.push_str(&format!(
                            "{} = https, {}, {}",
                            remark, node.server, node.port
                        ));
                    }

                    // Add username/password if present
                    if !node.username.is_empty() {
                        proxy_config.push_str(&format!(", username={}", node.username));
                    }

                    if !node.password.is_empty() {
                        proxy_config.push_str(&format!(", password={}", node.password));
                    }
                }
                _ => continue,
            }

            // Add common fields
            if let Some(udp) = node.udp {
                proxy_config.push_str(&format!(
                    ", udp-relay={}",
                    if udp { "true" } else { "false" }
                ));
            }

            if let Some(tfo) = node.tfo {
                proxy_config.push_str(&format!(
                    ", fast-open={}",
                    if tfo { "true" } else { "false" }
                ));
            }

            if let Some(scv) = node.skip_cert_verify {
                proxy_config.push_str(&format!(
                    ", certificate={}",
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
                "select" | "url-test" | "fallback" => {}
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
                "select" => {
                    server_config.push_str(&format!("{}=select, {}\n", group.name, group_path));
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

                    server_config.push_str(&format!(
                        "{}=url-test, {}, url={}, interval={}\n",
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

                    server_config.push_str(&format!(
                        "{}=fallback, {}, url={}, interval={}\n",
                        group.name, group_path, test_url, interval
                    ));
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
                    filter_config.push_str(&rule_str);
                    filter_config.push('\n');
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
                "[SERVER]",
                "[POLICY]",
                "[FILTER]",
                "[REWRITE]",
                "[URL-REJECTION]",
                "[TCP]",
                "[GLOBAL]",
                "[HOST]",
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

    // Add SERVER section
    if !proxy_config.is_empty() {
        config.push_str("[SERVER]\n");
        config.push_str(&proxy_config);
        config.push('\n');
    }

    // Add POLICY section
    if !server_config.is_empty() {
        config.push_str("[POLICY]\n");
        config.push_str(&server_config);
        config.push('\n');
    }

    // Add FILTER section
    if !filter_config.is_empty() {
        config.push_str("[FILTER]\n");
        config.push_str(&filter_config);
        config.push('\n');
    }

    config
}
