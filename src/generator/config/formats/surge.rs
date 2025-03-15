use crate::generator::config::ruleconvert;
use crate::generator::config::subexport::{
    group_generate, process_remark, ExtraSettings, ProxyGroupConfigs,
};
use crate::parser::proxy::{Proxy, ProxyType};
use crate::parser::ruleset::RulesetContent;
use std::collections::HashMap;

/// Convert proxies to Surge format
///
/// This function converts a list of proxies to the Surge configuration format,
/// using a base configuration as a template and applying rules from ruleset_content_array.
///
/// # Arguments
/// * `nodes` - List of proxy nodes to convert
/// * `base_conf` - Base Surge configuration as a string
/// * `ruleset_content_array` - Array of ruleset contents to apply
/// * `extra_proxy_group` - Extra proxy group configurations
/// * `surge_ver` - Surge version (3 or 4)
/// * `ext` - Extra settings for conversion
pub fn proxy_to_surge(
    nodes: &mut Vec<Proxy>,
    base_conf: &str,
    ruleset_content_array: &mut Vec<RulesetContent>,
    extra_proxy_group: &ProxyGroupConfigs,
    surge_ver: i32,
    ext: &mut ExtraSettings,
) -> String {
    let mut proxy_config = String::new();
    let mut group_config = String::new();
    let mut rule_config = String::new();
    let mut base_config = base_conf.to_string();

    // Process proxies
    if !nodes.is_empty() {
        proxy_config.push_str("[Proxy]\n");

        for node in nodes.iter_mut() {
            // Skip unsupported proxy types
            match node.proxy_type {
                ProxyType::Shadowsocks
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
                        } else if node.plugin == "v2ray-plugin" {
                            let mut plugin_opts = HashMap::new();
                            for opt in node.plugin_opts.split(';') {
                                let parts: Vec<&str> = opt.split('=').collect();
                                if parts.len() == 2 {
                                    plugin_opts.insert(parts[0], parts[1]);
                                }
                            }

                            if let Some(mode) = plugin_opts.get("mode") {
                                if mode == "websocket" {
                                    proxy_config.push_str(", obfs=ws");

                                    if let Some(host) = plugin_opts.get("host") {
                                        proxy_config.push_str(&format!(", obfs-host={}", host));
                                    }

                                    if let Some(path) = plugin_opts.get("path") {
                                        proxy_config.push_str(&format!(", obfs-uri={}", path));
                                    }

                                    if plugin_opts.get("tls").is_some() {
                                        proxy_config.push_str(", tls=true");
                                    }
                                }
                            }
                        }
                    }
                }
                ProxyType::VMess => {
                    proxy_config.push_str(&format!(
                        "{} = vmess, {}, {}, username={}",
                        remark, node.server, node.port, node.uuid
                    ));

                    // Add alterId if present
                    if node.alter_id > 0 {
                        proxy_config.push_str(&format!(", alterId={}", node.alter_id));
                    }

                    // Add encryption if present
                    if !node.cipher.is_empty() {
                        proxy_config.push_str(&format!(", encrypt-method={}", node.cipher));
                    }

                    // Add network settings
                    if !node.network.is_empty() {
                        match node.network.as_str() {
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
                                proxy_config.push_str(", obfs=http");

                                if !node.path.is_empty() {
                                    proxy_config.push_str(&format!(", obfs-uri={}", node.path));
                                }

                                if !node.host.is_empty() {
                                    proxy_config.push_str(&format!(", obfs-host={}", node.host));
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
                                proxy_config.push_str(&format!(", sni={}", sni));
                            }
                        }
                    }
                }
                ProxyType::Trojan => {
                    proxy_config.push_str(&format!(
                        "{} = trojan, {}, {}, password={}",
                        remark, node.server, node.port, node.password
                    ));

                    // Add SNI if present
                    if let Some(sni) = &node.sni {
                        if !sni.is_empty() {
                            proxy_config.push_str(&format!(", sni={}", sni));
                        }
                    }

                    // Add network settings
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
                ProxyType::Socks5 => {
                    proxy_config.push_str(&format!(
                        "{} = socks5, {}, {}",
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
                ProxyType::Snell => {
                    proxy_config.push_str(&format!(
                        "{} = snell, {}, {}, psk={}",
                        remark, node.server, node.port, node.password
                    ));

                    // Add version if present
                    if node.version > 0 {
                        proxy_config.push_str(&format!(", version={}", node.version));
                    }

                    // Add obfs settings
                    if !node.obfs.is_empty() {
                        proxy_config.push_str(&format!(", obfs={}", node.obfs));

                        if !node.host.is_empty() {
                            proxy_config.push_str(&format!(", obfs-host={}", node.host));
                        }
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
                proxy_config.push_str(&format!(", tfo={}", if tfo { "true" } else { "false" }));
            }

            if let Some(scv) = node.skip_cert_verify {
                proxy_config.push_str(&format!(
                    ", skip-cert-verify={}",
                    if scv { "true" } else { "false" }
                ));
            }

            proxy_config.push('\n');
        }

        proxy_config.push('\n');
    }

    // Process proxy groups
    if !extra_proxy_group.is_empty() {
        group_config.push_str("[Proxy Group]\n");

        for group in extra_proxy_group {
            group_config.push_str(&format!("{} = {}", group.name, group.type_field));

            // Add URL if present
            if !group.url.is_empty() {
                group_config.push_str(&format!(", {}", group.url));
            }

            // Add interval if present
            if group.interval > 0 {
                group_config.push_str(&format!(", interval={}", group.interval));
            }

            // Add proxies
            for proxy_name in &group.proxies {
                let mut filtered_nodes = Vec::new();
                group_generate(proxy_name, nodes, &mut filtered_nodes, false, ext);
                for node in filtered_nodes {
                    group_config.push_str(&format!(", {}", node));
                }
            }

            group_config.push('\n');
        }

        group_config.push('\n');
    }

    // Process rules
    if ext.enable_rule_generator && !ruleset_content_array.is_empty() {
        rule_config.push_str("[Rule]\n");

        for ruleset in ruleset_content_array {
            for rule in &ruleset.rules {
                let rule_str =
                    format!("{},{},{}", rule.rule_type, rule.rule_content, ruleset.group);
                rule_config.push_str(&rule_str);
                rule_config.push('\n');
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

    // Add proxy, group, and rule configs
    config.push_str(&proxy_config);
    config.push_str(&group_config);
    config.push_str(&rule_config);

    config
}
