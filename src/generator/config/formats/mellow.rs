use crate::generator::config::subexport::{
    group_generate, process_remark, ExtraSettings, ProxyGroupConfigs,
};
use crate::parser::proxy::{Proxy, ProxyType};
use crate::parser::ruleset::RulesetContent;
use crate::utils::ini::IniReader;
use std::collections::HashMap;

/// Convert proxies to Mellow format
///
/// This function converts a list of proxies to the Mellow configuration format,
/// using a base configuration as a template and applying rules from ruleset_content_array.
///
/// # Arguments
/// * `nodes` - List of proxy nodes to convert
/// * `base_conf` - Base Mellow configuration as a string
/// * `ruleset_content_array` - Array of ruleset contents to apply
/// * `extra_proxy_group` - Extra proxy group configurations
/// * `ext` - Extra settings for conversion
pub fn proxy_to_mellow(
    nodes: &mut Vec<Proxy>,
    base_conf: &str,
    ruleset_content_array: &mut Vec<RulesetContent>,
    extra_proxy_group: &ProxyGroupConfigs,
    ext: &mut ExtraSettings,
) -> String {
    let mut proxy_config = String::new();
    let mut route_config = String::new();
    let mut rule_config = String::new();
    let mut base_config = base_conf.to_string();

    // Process proxies
    if !nodes.is_empty() {
        proxy_config.push_str("[Endpoint]\n");

        for node in nodes.iter_mut() {
            // Skip unsupported proxy types
            match node.proxy_type {
                ProxyType::Shadowsocks
                | ProxyType::ShadowsocksR
                | ProxyType::VMess
                | ProxyType::Socks5
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
                        "{} = ss, {}, {}, encrypt-method={}, password={}",
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
                                proxy_config.push_str(&format!(", obfs={}", obfs));

                                if let Some(host) = plugin_opts.get("obfs-host") {
                                    proxy_config.push_str(&format!(", obfs-host={}", host));
                                }
                            }
                        }
                    }
                }
                ProxyType::ShadowsocksR => {
                    proxy_config.push_str(&format!("{} = ssr, {}, {}, encrypt-method={}, password={}, obfs={}, obfs-param={}, protocol={}, protocol-param={}", 
                        remark, node.server, node.port, node.cipher, node.password,
                        node.obfs, node.obfs_param, node.protocol, node.protocol_param));
                }
                ProxyType::VMess => {
                    proxy_config.push_str(&format!(
                        "{} = vmess, {}, {}, uuid={}, security={}",
                        remark,
                        node.server,
                        node.port,
                        node.uuid,
                        if node.cipher.is_empty() {
                            "auto"
                        } else {
                            &node.cipher
                        }
                    ));

                    // Add alterId if present
                    if node.alter_id > 0 {
                        proxy_config.push_str(&format!(", alter-id={}", node.alter_id));
                    }

                    // Add network settings
                    if !node.network.is_empty() {
                        proxy_config.push_str(&format!(", network={}", node.network));

                        match node.network.as_str() {
                            "ws" => {
                                if !node.path.is_empty() {
                                    proxy_config.push_str(&format!(", ws-path={}", node.path));
                                }

                                if !node.host.is_empty() {
                                    proxy_config.push_str(&format!(", ws-host={}", node.host));
                                }
                            }
                            "h2" => {
                                if !node.path.is_empty() {
                                    proxy_config.push_str(&format!(", h2-path={}", node.path));
                                }

                                if !node.host.is_empty() {
                                    proxy_config.push_str(&format!(", h2-host={}", node.host));
                                }
                            }
                            _ => {}
                        }
                    }

                    // Add TLS settings
                    if node.tls {
                        proxy_config.push_str(", tls=true");

                        if let Some(sni) = &node.sni {
                            if !sni.is_empty() {
                                proxy_config.push_str(&format!(", tls-server-name={}", sni));
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
                            "{} = http, {}, {}, tls=true",
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
                ProxyType::Socks5 => {
                    proxy_config.push_str(&format!(
                        "{} = socks, {}, {}",
                        remark, node.server, node.port
                    ));

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

            proxy_config.push('\n');
        }

        proxy_config.push('\n');
    }

    // Process proxy groups
    if !extra_proxy_group.is_empty() {
        route_config.push_str("[Routing]\n");

        // Add default route
        route_config.push_str("default = direct\n");

        // Add proxy groups
        for group in extra_proxy_group {
            // Skip unsupported group types
            match group.type_field.as_str() {
                "select" | "url-test" | "fallback" | "load-balance" => {}
                _ => continue,
            }

            // Create proxy list for this group
            let mut proxy_list = Vec::new();

            for proxy_name in &group.proxies {
                let mut filtered_nodes = Vec::new();
                group_generate(proxy_name, nodes, &mut filtered_nodes, false, ext);
                for node in filtered_nodes {
                    proxy_list.push(node);
                }
            }

            // Add group based on type
            match group.type_field.as_str() {
                "select" => {
                    route_config.push_str(&format!(
                        "proxy-{} = select, {}\n",
                        group.name,
                        proxy_list.join(", ")
                    ));
                }
                "url-test" => {
                    route_config.push_str(&format!(
                        "proxy-{} = url-test, {}",
                        group.name,
                        proxy_list.join(", ")
                    ));

                    // Add URL if present
                    if !group.url.is_empty() {
                        route_config.push_str(&format!(", url={}", group.url));
                    }

                    // Add interval if present
                    if group.interval > 0 {
                        route_config.push_str(&format!(", interval={}", group.interval));
                    }

                    route_config.push('\n');
                }
                "fallback" => {
                    route_config.push_str(&format!(
                        "proxy-{} = fallback, {}",
                        group.name,
                        proxy_list.join(", ")
                    ));

                    // Add URL if present
                    if !group.url.is_empty() {
                        route_config.push_str(&format!(", url={}", group.url));
                    }

                    // Add interval if present
                    if group.interval > 0 {
                        route_config.push_str(&format!(", interval={}", group.interval));
                    }

                    route_config.push('\n');
                }
                "load-balance" => {
                    route_config.push_str(&format!(
                        "proxy-{} = load-balance, {}\n",
                        group.name,
                        proxy_list.join(", ")
                    ));
                }
                _ => {}
            }
        }

        route_config.push('\n');
    }

    // Process rules
    if ext.enable_rule_generator && !ruleset_content_array.is_empty() {
        rule_config.push_str("[Rule]\n");

        for ruleset in ruleset_content_array {
            for rule in &ruleset.rules {
                let mut rule_str = String::new();

                match rule.rule_type.as_str() {
                    "DOMAIN" => {
                        rule_str = format!("domain:{}, {}", rule.rule_content, ruleset.group);
                    }
                    "DOMAIN-SUFFIX" => {
                        rule_str =
                            format!("domain-suffix:{}, {}", rule.rule_content, ruleset.group);
                    }
                    "DOMAIN-KEYWORD" => {
                        rule_str =
                            format!("domain-keyword:{}, {}", rule.rule_content, ruleset.group);
                    }
                    "IP-CIDR" => {
                        rule_str = format!("ip:{}, {}", rule.rule_content, ruleset.group);
                    }
                    "IP-CIDR6" => {
                        rule_str = format!("ip:{}, {}", rule.rule_content, ruleset.group);
                    }
                    "GEOIP" => {
                        rule_str = format!("geoip:{}, {}", rule.rule_content, ruleset.group);
                    }
                    "FINAL" => {
                        rule_str = format!("final, {}", ruleset.group);
                    }
                    _ => {}
                }

                if !rule_str.is_empty() {
                    rule_config.push_str(&rule_str);
                    rule_config.push('\n');
                }
            }
        }

        rule_config.push('\n');
    }

    // Combine all sections
    let mut config = String::new();

    // Add base config if not empty
    if !base_config.is_empty() {
        // Check if we need to overwrite original sections
        if ext.overwrite_original_rules {
            // Remove original [Endpoint], [Routing], and [Rule] sections
            let sections = ["[Endpoint]", "[Routing]", "[Rule]"];

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

    // Add proxy, route, and rule configs
    config.push_str(&proxy_config);
    config.push_str(&route_config);
    config.push_str(&rule_config);

    config
}

/// Convert proxies to Mellow format with INI reader
///
/// This function modifies an INI reader in place to add Mellow configuration
/// for the provided proxy nodes.
///
/// # Arguments
/// * `nodes` - List of proxy nodes to convert
/// * `ini` - INI reader to modify
/// * `ruleset_content_array` - Array of ruleset contents to apply
/// * `extra_proxy_group` - Extra proxy group configurations
/// * `ext` - Extra settings for conversion
pub fn proxy_to_mellow_ini(
    nodes: &mut Vec<Proxy>,
    ini: &mut IniReader,
    ruleset_content_array: &mut Vec<RulesetContent>,
    extra_proxy_group: &ProxyGroupConfigs,
    ext: &mut ExtraSettings,
) {
    // Placeholder for implementation
    // Here will be code to:
    // 1. Process proxy nodes into Mellow format
    // 2. Add them to the INI object
    // 3. Add proxy groups from extra_proxy_group
    // 4. Add rules if rule generator is enabled
}
