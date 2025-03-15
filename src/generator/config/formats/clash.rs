use crate::generator::config::subexport::{
    group_generate, process_remark, ExtraSettings, ProxyGroupConfigs,
};
use crate::parser::proxy::{Proxy, ProxyType};
use crate::parser::ruleset::RulesetContent;
use crate::utils::yaml::YamlNode;
use serde_yaml::{self, Mapping, Value};

/// Convert proxies to Clash format
///
/// This function converts a list of proxies to the Clash configuration format,
/// using a base configuration as a template and applying rules from ruleset_content_array.
///
/// # Arguments
/// * `nodes` - List of proxy nodes to convert
/// * `base_conf` - Base Clash configuration as a string
/// * `ruleset_content_array` - Array of ruleset contents to apply
/// * `extra_proxy_group` - Extra proxy group configurations
/// * `clash_r` - Whether to use ClashR format
/// * `ext` - Extra settings for conversion
pub fn proxy_to_clash(
    nodes: &mut Vec<Proxy>,
    base_conf: &str,
    ruleset_content_array: &mut Vec<RulesetContent>,
    extra_proxy_group: &ProxyGroupConfigs,
    clash_r: bool,
    ext: &mut ExtraSettings,
) -> String {
    // Parse the base configuration
    let mut yaml_node = match YamlNode::from_str(base_conf) {
        Ok(node) => node,
        Err(_) => return String::new(),
    };

    // Apply conversion to the YAML node
    proxy_to_clash_yaml(nodes, &mut yaml_node, extra_proxy_group, clash_r, ext);

    // Convert back to string
    match yaml_node.to_string() {
        Ok(result) => result,
        Err(_) => String::new(),
    }
}

/// Convert proxies to Clash format with YAML node
///
/// This function modifies a YAML node in place to add Clash configuration
/// for the provided proxy nodes.
///
/// # Arguments
/// * `nodes` - List of proxy nodes to convert
/// * `yaml_node` - YAML node to modify
/// * `extra_proxy_group` - Extra proxy group configurations
/// * `clash_r` - Whether to use ClashR format
/// * `ext` - Extra settings for conversion
pub fn proxy_to_clash_yaml(
    nodes: &mut Vec<Proxy>,
    yaml_node: &mut YamlNode,
    extra_proxy_group: &ProxyGroupConfigs,
    clash_r: bool,
    ext: &mut ExtraSettings,
) {
    // Create proxies array if it doesn't exist
    if yaml_node.value.get("proxies").is_none() {
        let proxies = Value::Sequence(Vec::new());
        if let Value::Mapping(ref mut map) = yaml_node.value {
            map.insert(Value::String("proxies".to_string()), proxies);
        }
    }

    // Get proxies array
    let proxies = match yaml_node.value.get_mut("proxies") {
        Some(Value::Sequence(seq)) => seq,
        _ => return,
    };

    // Process each node
    for node in nodes.iter_mut() {
        // Skip unsupported proxy types
        match node.proxy_type {
            ProxyType::Shadowsocks
            | ProxyType::ShadowsocksR
            | ProxyType::VMess
            | ProxyType::Trojan
            | ProxyType::Snell
            | ProxyType::HTTP
            | ProxyType::HTTPS
            | ProxyType::Socks5 => {}
            _ => continue,
        }

        // Process remark
        let mut remark = node.remark.clone();
        process_remark(&mut remark, ext, true);

        // Create proxy object
        let mut proxy = Mapping::new();
        proxy.insert(Value::String("name".to_string()), Value::String(remark));

        // Add type-specific fields
        match node.proxy_type {
            ProxyType::Shadowsocks => {
                proxy.insert(
                    Value::String("type".to_string()),
                    Value::String("ss".to_string()),
                );
                proxy.insert(
                    Value::String("server".to_string()),
                    Value::String(node.server.clone()),
                );
                proxy.insert(
                    Value::String("port".to_string()),
                    Value::Number(node.port.into()),
                );
                proxy.insert(
                    Value::String("cipher".to_string()),
                    Value::String(node.cipher.clone()),
                );
                proxy.insert(
                    Value::String("password".to_string()),
                    Value::String(node.password.clone()),
                );

                // Add plugin if present
                if !node.plugin.is_empty() {
                    proxy.insert(
                        Value::String("plugin".to_string()),
                        Value::String(node.plugin.clone()),
                    );
                    proxy.insert(
                        Value::String("plugin-opts".to_string()),
                        Value::String(node.plugin_opts.clone()),
                    );
                }
            }
            ProxyType::ShadowsocksR => {
                if clash_r {
                    proxy.insert(
                        Value::String("type".to_string()),
                        Value::String("ssr".to_string()),
                    );
                    proxy.insert(
                        Value::String("server".to_string()),
                        Value::String(node.server.clone()),
                    );
                    proxy.insert(
                        Value::String("port".to_string()),
                        Value::Number(node.port.into()),
                    );
                    proxy.insert(
                        Value::String("cipher".to_string()),
                        Value::String(node.cipher.clone()),
                    );
                    proxy.insert(
                        Value::String("password".to_string()),
                        Value::String(node.password.clone()),
                    );
                    proxy.insert(
                        Value::String("protocol".to_string()),
                        Value::String(node.protocol.clone()),
                    );
                    proxy.insert(
                        Value::String("protocolparam".to_string()),
                        Value::String(node.protocol_param.clone()),
                    );
                    proxy.insert(
                        Value::String("obfs".to_string()),
                        Value::String(node.obfs.clone()),
                    );
                    proxy.insert(
                        Value::String("obfsparam".to_string()),
                        Value::String(node.obfs_param.clone()),
                    );
                } else {
                    // Skip SSR nodes if ClashR is not enabled
                    continue;
                }
            }
            ProxyType::VMess => {
                proxy.insert(
                    Value::String("type".to_string()),
                    Value::String("vmess".to_string()),
                );
                proxy.insert(
                    Value::String("server".to_string()),
                    Value::String(node.server.clone()),
                );
                proxy.insert(
                    Value::String("port".to_string()),
                    Value::Number(node.port.into()),
                );
                proxy.insert(
                    Value::String("uuid".to_string()),
                    Value::String(node.uuid.clone()),
                );
                proxy.insert(
                    Value::String("alterId".to_string()),
                    Value::Number(node.alter_id.into()),
                );

                // Add cipher
                if !node.cipher.is_empty() {
                    proxy.insert(
                        Value::String("cipher".to_string()),
                        Value::String(node.cipher.clone()),
                    );
                } else {
                    proxy.insert(
                        Value::String("cipher".to_string()),
                        Value::String("auto".to_string()),
                    );
                }

                // Add network settings
                if !node.network.is_empty() {
                    proxy.insert(
                        Value::String("network".to_string()),
                        Value::String(node.network.clone()),
                    );
                }

                // Add TLS settings
                if node.tls {
                    proxy.insert(Value::String("tls".to_string()), Value::Bool(true));
                    if let Some(sni) = &node.sni {
                        if !sni.is_empty() {
                            proxy.insert(
                                Value::String("servername".to_string()),
                                Value::String(sni.clone()),
                            );
                        }
                    }
                }

                // Add network specific settings
                match node.network.as_str() {
                    "ws" => {
                        let mut ws_opts = Mapping::new();
                        if !node.path.is_empty() {
                            ws_opts.insert(
                                Value::String("path".to_string()),
                                Value::String(node.path.clone()),
                            );
                        }
                        if !node.host.is_empty() {
                            let mut headers = Mapping::new();
                            headers.insert(
                                Value::String("Host".to_string()),
                                Value::String(node.host.clone()),
                            );
                            ws_opts.insert(
                                Value::String("headers".to_string()),
                                Value::Mapping(headers),
                            );
                        }
                        proxy.insert(
                            Value::String("ws-opts".to_string()),
                            Value::Mapping(ws_opts),
                        );
                    }
                    "h2" => {
                        let mut h2_opts = Mapping::new();
                        if !node.path.is_empty() {
                            h2_opts.insert(
                                Value::String("path".to_string()),
                                Value::String(node.path.clone()),
                            );
                        }
                        if !node.host.is_empty() {
                            let hosts = vec![Value::String(node.host.clone())];
                            h2_opts
                                .insert(Value::String("host".to_string()), Value::Sequence(hosts));
                        }
                        proxy.insert(
                            Value::String("h2-opts".to_string()),
                            Value::Mapping(h2_opts),
                        );
                    }
                    "http" => {
                        let mut http_opts = Mapping::new();
                        if !node.path.is_empty() {
                            http_opts.insert(
                                Value::String("path".to_string()),
                                Value::String(node.path.clone()),
                            );
                        }
                        if !node.host.is_empty() {
                            let hosts = vec![Value::String(node.host.clone())];
                            http_opts
                                .insert(Value::String("host".to_string()), Value::Sequence(hosts));
                        }
                        proxy.insert(
                            Value::String("http-opts".to_string()),
                            Value::Mapping(http_opts),
                        );
                    }
                    _ => {}
                }
            }
            ProxyType::Trojan => {
                proxy.insert(
                    Value::String("type".to_string()),
                    Value::String("trojan".to_string()),
                );
                proxy.insert(
                    Value::String("server".to_string()),
                    Value::String(node.server.clone()),
                );
                proxy.insert(
                    Value::String("port".to_string()),
                    Value::Number(node.port.into()),
                );
                proxy.insert(
                    Value::String("password".to_string()),
                    Value::String(node.password.clone()),
                );

                // Add SNI
                if let Some(sni) = &node.sni {
                    if !sni.is_empty() {
                        proxy.insert(Value::String("sni".to_string()), Value::String(sni.clone()));
                    }
                }

                // Add network settings
                if !node.network.is_empty() && node.network != "tcp" {
                    proxy.insert(
                        Value::String("network".to_string()),
                        Value::String(node.network.clone()),
                    );

                    // Add network specific settings
                    match node.network.as_str() {
                        "ws" => {
                            let mut ws_opts = Mapping::new();
                            if !node.path.is_empty() {
                                ws_opts.insert(
                                    Value::String("path".to_string()),
                                    Value::String(node.path.clone()),
                                );
                            }
                            if !node.host.is_empty() {
                                let mut headers = Mapping::new();
                                headers.insert(
                                    Value::String("Host".to_string()),
                                    Value::String(node.host.clone()),
                                );
                                ws_opts.insert(
                                    Value::String("headers".to_string()),
                                    Value::Mapping(headers),
                                );
                            }
                            proxy.insert(
                                Value::String("ws-opts".to_string()),
                                Value::Mapping(ws_opts),
                            );
                        }
                        _ => {}
                    }
                }
            }
            ProxyType::HTTP | ProxyType::HTTPS => {
                proxy.insert(
                    Value::String("type".to_string()),
                    Value::String("http".to_string()),
                );
                proxy.insert(
                    Value::String("server".to_string()),
                    Value::String(node.server.clone()),
                );
                proxy.insert(
                    Value::String("port".to_string()),
                    Value::Number(node.port.into()),
                );

                // Add username/password if present
                if !node.username.is_empty() {
                    proxy.insert(
                        Value::String("username".to_string()),
                        Value::String(node.username.clone()),
                    );
                }
                if !node.password.is_empty() {
                    proxy.insert(
                        Value::String("password".to_string()),
                        Value::String(node.password.clone()),
                    );
                }

                // Add TLS for HTTPS
                if node.proxy_type == ProxyType::HTTPS {
                    proxy.insert(Value::String("tls".to_string()), Value::Bool(true));
                }
            }
            ProxyType::Socks5 => {
                proxy.insert(
                    Value::String("type".to_string()),
                    Value::String("socks5".to_string()),
                );
                proxy.insert(
                    Value::String("server".to_string()),
                    Value::String(node.server.clone()),
                );
                proxy.insert(
                    Value::String("port".to_string()),
                    Value::Number(node.port.into()),
                );

                // Add username/password if present
                if !node.username.is_empty() {
                    proxy.insert(
                        Value::String("username".to_string()),
                        Value::String(node.username.clone()),
                    );
                }
                if !node.password.is_empty() {
                    proxy.insert(
                        Value::String("password".to_string()),
                        Value::String(node.password.clone()),
                    );
                }
            }
            ProxyType::Snell => {
                proxy.insert(
                    Value::String("type".to_string()),
                    Value::String("snell".to_string()),
                );
                proxy.insert(
                    Value::String("server".to_string()),
                    Value::String(node.server.clone()),
                );
                proxy.insert(
                    Value::String("port".to_string()),
                    Value::Number(node.port.into()),
                );
                proxy.insert(
                    Value::String("psk".to_string()),
                    Value::String(node.password.clone()),
                );

                // Add version if present
                if node.version > 0 {
                    proxy.insert(
                        Value::String("version".to_string()),
                        Value::Number(node.version.into()),
                    );
                }

                // Add obfs settings
                if !node.obfs.is_empty() {
                    proxy.insert(
                        Value::String("obfs".to_string()),
                        Value::String(node.obfs.clone()),
                    );
                    if !node.host.is_empty() {
                        proxy.insert(
                            Value::String("obfs-host".to_string()),
                            Value::String(node.host.clone()),
                        );
                    }
                }
            }
            _ => continue,
        }

        // Add common fields
        if let Some(udp) = node.udp {
            proxy.insert(Value::String("udp".to_string()), Value::Bool(udp));
        }
        if let Some(tfo) = node.tfo {
            proxy.insert(Value::String("tfo".to_string()), Value::Bool(tfo));
        }
        if let Some(scv) = node.skip_cert_verify {
            proxy.insert(
                Value::String("skip-cert-verify".to_string()),
                Value::Bool(scv),
            );
        }

        // Add the proxy to the list
        proxies.push(Value::Mapping(proxy));
    }

    // Add proxy groups if present
    if !extra_proxy_group.is_empty() {
        // Create proxy-groups array if it doesn't exist
        if yaml_node.value.get("proxy-groups").is_none() {
            let proxy_groups = Value::Sequence(Vec::new());
            if let Value::Mapping(ref mut map) = yaml_node.value {
                map.insert(Value::String("proxy-groups".to_string()), proxy_groups);
            }
        }

        // Get proxy-groups array
        let proxy_groups = match yaml_node.value.get_mut("proxy-groups") {
            Some(Value::Sequence(seq)) => seq,
            _ => return,
        };

        // Process each proxy group
        for group in extra_proxy_group {
            let mut proxy_group = Mapping::new();
            proxy_group.insert(
                Value::String("name".to_string()),
                Value::String(group.name.clone()),
            );
            proxy_group.insert(
                Value::String("type".to_string()),
                Value::String(group.type_field.clone()),
            );

            // Add URL if present
            if !group.url.is_empty() {
                proxy_group.insert(
                    Value::String("url".to_string()),
                    Value::String(group.url.clone()),
                );
            }

            // Add interval if present
            if group.interval > 0 {
                proxy_group.insert(
                    Value::String("interval".to_string()),
                    Value::Number(group.interval.into()),
                );
            }

            // Add proxies
            let mut proxies_list = Vec::new();
            for proxy_name in &group.proxies {
                let mut filtered_nodes = Vec::new();
                group_generate(proxy_name, nodes, &mut filtered_nodes, false, ext);
                for node in filtered_nodes {
                    proxies_list.push(Value::String(node));
                }
            }
            proxy_group.insert(
                Value::String("proxies".to_string()),
                Value::Sequence(proxies_list),
            );

            // Add the proxy group to the list
            proxy_groups.push(Value::Mapping(proxy_group));
        }
    }

    // Add rules if rule generator is enabled
    if ext.enable_rule_generator && !ruleset_content_array.is_empty() {
        // Create rules array if it doesn't exist
        if yaml_node.value.get("rules").is_none() {
            let rules = Value::Sequence(Vec::new());
            if let Value::Mapping(ref mut map) = yaml_node.value {
                map.insert(Value::String("rules".to_string()), rules);
            }
        }

        // Get rules array
        let rules = match yaml_node.value.get_mut("rules") {
            Some(Value::Sequence(seq)) => seq,
            _ => return,
        };

        // Clear existing rules if overwrite is enabled
        if ext.overwrite_original_rules {
            rules.clear();
        }

        // Process each ruleset
        for ruleset in ruleset_content_array {
            for rule in &ruleset.rules {
                let rule_str =
                    format!("{},{},{}", rule.rule_type, rule.rule_content, ruleset.group);
                rules.push(Value::String(rule_str));
            }
        }
    }
}
