use crate::generator::config::subexport::{
    group_generate, process_remark, ExtraSettings, ProxyGroupConfigs,
};
use crate::parser::ruleset::RulesetContent;
use crate::{Proxy, ProxyType};
use std::collections::HashMap;

/// Convert proxies to Loon format
///
/// This function converts a list of proxies to the Loon configuration format,
/// using a base configuration as a template and applying rules from ruleset_content_array.
///
/// # Arguments
/// * `nodes` - List of proxy nodes to convert
/// * `base_conf` - Base Loon configuration as a string
/// * `ruleset_content_array` - Array of ruleset contents to apply
/// * `extra_proxy_group` - Extra proxy group configurations
/// * `ext` - Extra settings for conversion
pub fn proxy_to_loon(
    nodes: &mut Vec<Proxy>,
    base_conf: &str,
    ruleset_content_array: &mut Vec<RulesetContent>,
    extra_proxy_group: &ProxyGroupConfigs,
    ext: &mut ExtraSettings,
) -> String {
    let mut proxy_config = String::new();
    let mut group_config = String::new();
    let mut rule_config = String::new();
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
                | ProxyType::HTTPS
                | ProxyType::Socks5
                | ProxyType::Snell => {}
                _ => continue,
            }

            // Process remark
            let mut remark = node.remark.clone();
            process_remark(&mut remark, ext, true);

            // Add proxy based on type
            match node.proxy_type {
                ProxyType::Shadowsocks => {
                    proxy_config.push_str(&format!(
                        "{} = shadowsocks,{},{},{},{}",
                        remark, node.server, node.port, node.cipher, node.password
                    ));

                    // Add plugin if present
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
                                proxy_config.push_str(&format!(",{}", obfs));

                                if let Some(host) = plugin_opts.get("obfs-host") {
                                    proxy_config.push_str(&format!(",{}", host));
                                }
                            }
                        }
                    }
                }
                ProxyType::ShadowsocksR => {
                    proxy_config.push_str(&format!(
                        "{} = shadowsocksr,{},{},{},{},{},{},{}",
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
                        "{} = vmess,{},{},{}",
                        remark, node.server, node.port, node.uuid
                    ));

                    // Add alterId if present
                    if node.alter_id > 0 {
                        proxy_config.push_str(&format!(",{}", node.alter_id));
                    } else {
                        proxy_config.push_str(",0");
                    }

                    // Add encryption if present
                    if !node.cipher.is_empty() {
                        proxy_config.push_str(&format!(",{}", node.cipher));
                    } else {
                        proxy_config.push_str(",auto");
                    }

                    // Add transport settings
                    if !node.network.is_empty() {
                        proxy_config.push_str(&format!(",{}", node.network));

                        match node.network.as_str() {
                            "ws" => {
                                if !node.host.is_empty() {
                                    proxy_config.push_str(&format!(",{}", node.host));
                                } else {
                                    proxy_config.push(',');
                                }

                                if !node.path.is_empty() {
                                    proxy_config.push_str(&format!(",{}", node.path));
                                }
                            }
                            "h2" => {
                                if !node.host.is_empty() {
                                    proxy_config.push_str(&format!(",{}", node.host));
                                } else {
                                    proxy_config.push(',');
                                }

                                if !node.path.is_empty() {
                                    proxy_config.push_str(&format!(",{}", node.path));
                                }
                            }
                            _ => {}
                        }
                    } else {
                        proxy_config.push_str(",tcp");
                    }

                    // Add TLS settings
                    if node.tls {
                        proxy_config.push_str(",tls");

                        if let Some(sni) = &node.sni {
                            if !sni.is_empty() {
                                proxy_config.push_str(&format!(",{}", sni));
                            }
                        }
                    }
                }
                ProxyType::Trojan => {
                    proxy_config.push_str(&format!(
                        "{} = trojan,{},{},{}",
                        remark, node.server, node.port, node.password
                    ));

                    // Add SNI if present
                    if let Some(sni) = &node.sni {
                        if !sni.is_empty() {
                            proxy_config.push_str(&format!(",{}", sni));
                        }
                    }
                }
                ProxyType::HTTP | ProxyType::HTTPS => {
                    if node.proxy_type == ProxyType::HTTP {
                        proxy_config
                            .push_str(&format!("{} = http,{},{}", remark, node.server, node.port));
                    } else {
                        proxy_config
                            .push_str(&format!("{} = https,{},{}", remark, node.server, node.port));
                    }

                    // Add username/password if present
                    if !node.username.is_empty() {
                        proxy_config.push_str(&format!(",{}", node.username));

                        if !node.password.is_empty() {
                            proxy_config.push_str(&format!(",{}", node.password));
                        }
                    }
                }
                ProxyType::Socks5 => {
                    proxy_config.push_str(&format!(
                        "{} = socks5,{},{}",
                        remark, node.server, node.port
                    ));

                    // Add username/password if present
                    if !node.username.is_empty() {
                        proxy_config.push_str(&format!(",{}", node.username));

                        if !node.password.is_empty() {
                            proxy_config.push_str(&format!(",{}", node.password));
                        }
                    }
                }
                ProxyType::Snell => {
                    proxy_config.push_str(&format!(
                        "{} = snell,{},{},{}",
                        remark, node.server, node.port, node.password
                    ));

                    // Add version if present
                    if node.version > 0 {
                        proxy_config.push_str(&format!(",{}", node.version));
                    }
                }
                _ => continue,
            }

            // Add common fields
            if let Some(udp) = node.udp {
                proxy_config.push_str(&format!(",udp={}", if udp { "true" } else { "false" }));
            }

            if let Some(tfo) = node.tfo {
                proxy_config.push_str(&format!(
                    ",fast-open={}",
                    if tfo { "true" } else { "false" }
                ));
            }

            if let Some(scv) = node.skip_cert_verify {
                proxy_config.push_str(&format!(
                    ",skip-cert-verify={}",
                    if scv { "true" } else { "false" }
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
                "select" | "url-test" | "fallback" | "load-balance" => {}
                _ => continue,
            }

            // Add group based on type
            match group.type_field.as_str() {
                "select" => {
                    group_config.push_str(&format!("{} = select", group.name));
                }
                "url-test" => {
                    group_config.push_str(&format!("{} = url-test", group.name));

                    // Add URL if present
                    if !group.url.is_empty() {
                        group_config.push_str(&format!(",url={}", group.url));
                    }

                    // Add interval if present
                    if group.interval > 0 {
                        group_config.push_str(&format!(",interval={}", group.interval));
                    }
                }
                "fallback" => {
                    group_config.push_str(&format!("{} = fallback", group.name));

                    // Add URL if present
                    if !group.url.is_empty() {
                        group_config.push_str(&format!(",url={}", group.url));
                    }

                    // Add interval if present
                    if group.interval > 0 {
                        group_config.push_str(&format!(",interval={}", group.interval));
                    }
                }
                "load-balance" => {
                    group_config.push_str(&format!("{} = load-balance", group.name));

                    // Add URL if present
                    if !group.url.is_empty() {
                        group_config.push_str(&format!(",url={}", group.url));
                    }

                    // Add interval if present
                    if group.interval > 0 {
                        group_config.push_str(&format!(",interval={}", group.interval));
                    }
                }
                _ => {}
            }

            // Add proxies
            for proxy_name in &group.proxies {
                let mut filtered_nodes = Vec::new();
                group_generate(proxy_name, nodes, &mut filtered_nodes, false, ext);
                for node in filtered_nodes {
                    group_config.push_str(&format!(",{}", node));
                }
            }

            group_config.push('\n');
        }
    }

    // Process rules
    if ext.enable_rule_generator && !ruleset_content_array.is_empty() {
        for ruleset in ruleset_content_array {
            for rule in &ruleset.rules {
                let mut rule_str = String::new();

                match rule.rule_type.as_str() {
                    "DOMAIN" => {
                        rule_str = format!("DOMAIN,{},{}", rule.rule_content, ruleset.group);
                    }
                    "DOMAIN-SUFFIX" => {
                        rule_str = format!("DOMAIN-SUFFIX,{},{}", rule.rule_content, ruleset.group);
                    }
                    "DOMAIN-KEYWORD" => {
                        rule_str =
                            format!("DOMAIN-KEYWORD,{},{}", rule.rule_content, ruleset.group);
                    }
                    "IP-CIDR" => {
                        rule_str = format!("IP-CIDR,{},{}", rule.rule_content, ruleset.group);
                    }
                    "IP-CIDR6" => {
                        rule_str = format!("IP-CIDR6,{},{}", rule.rule_content, ruleset.group);
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
                    rule_config.push_str(&rule_str);
                    rule_config.push('\n');
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
            // Remove original [Proxy], [Proxy Group], and [Rule] sections
            let sections = ["[Proxy]", "[Proxy Group]", "[Rule]"];

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

    // Add proxy section
    if !proxy_config.is_empty() {
        config.push_str("[Proxy]\n");
        config.push_str(&proxy_config);
        config.push('\n');
    }

    // Add proxy group section
    if !group_config.is_empty() {
        config.push_str("[Proxy Group]\n");
        config.push_str(&group_config);
        config.push('\n');
    }

    // Add rule section
    if !rule_config.is_empty() {
        config.push_str("[Rule]\n");
        config.push_str(&rule_config);
        config.push('\n');
    }

    config
}
