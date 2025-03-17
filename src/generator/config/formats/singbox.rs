use crate::generator::config::subexport::{
    group_generate, process_remark, ExtraSettings, ProxyGroupConfigs,
};
use crate::parser::ruleset::RulesetContent;
use crate::{Proxy, ProxyType};
use serde_json::{self, json, Value};

/// Convert proxies to SingBox format
///
/// This function converts a list of proxies to the SingBox configuration format,
/// using a base configuration as a template and applying rules from ruleset_content_array.
///
/// # Arguments
/// * `nodes` - List of proxy nodes to convert
/// * `base_conf` - Base SingBox configuration as a string
/// * `ruleset_content_array` - Array of ruleset contents to apply
/// * `extra_proxy_group` - Extra proxy group configurations
/// * `ext` - Extra settings for conversion
pub fn proxy_to_singbox(
    nodes: &mut Vec<Proxy>,
    base_conf: &str,
    ruleset_content_array: &mut Vec<RulesetContent>,
    extra_proxy_group: &ProxyGroupConfigs,
    ext: &mut ExtraSettings,
) -> String {
    // Parse the base configuration
    let mut config: Value = match serde_json::from_str(base_conf) {
        Ok(v) => v,
        Err(_) => json!({}),
    };

    // Ensure outbounds array exists
    if !config.get("outbounds").is_some() {
        config["outbounds"] = json!([]);
    }

    // Process proxies
    let mut proxy_outbounds = Vec::new();
    let mut proxy_names = Vec::new();

    for node in nodes.iter_mut() {
        // Skip unsupported proxy types
        match node.proxy_type {
            ProxyType::Shadowsocks
            | ProxyType::ShadowsocksR
            | ProxyType::VMess
            | ProxyType::Trojan
            | ProxyType::Socks5
            | ProxyType::HTTP
            | ProxyType::HTTPS => {}
            _ => continue,
        }

        // Process remark
        let mut remark = node.remark.clone();
        process_remark(&mut remark, ext, true);
        proxy_names.push(remark.clone());

        // Create outbound based on type
        match node.proxy_type {
            ProxyType::Shadowsocks => {
                let mut outbound = json!({
                    "type": "shadowsocks",
                    "tag": remark,
                    "server": node.server,
                    "server_port": node.port,
                    "method": node.cipher,
                    "password": node.password
                });

                // Add plugin if present
                if !node.plugin.is_empty() {
                    if node.plugin == "obfs-local" || node.plugin == "simple-obfs" {
                        let mut plugin_opts = std::collections::HashMap::new();
                        for opt in node.plugin_opts.split(';') {
                            let parts: Vec<&str> = opt.split('=').collect();
                            if parts.len() == 2 {
                                plugin_opts.insert(parts[0], parts[1]);
                            }
                        }

                        if let Some(obfs) = plugin_opts.get("obfs") {
                            outbound["plugin"] = json!("obfs");
                            outbound["plugin_opts"] = json!({
                                "mode": obfs
                            });

                            if let Some(host) = plugin_opts.get("obfs-host") {
                                outbound["plugin_opts"]["host"] = json!(host);
                            }
                        }
                    }
                }

                proxy_outbounds.push(outbound);
            }
            ProxyType::ShadowsocksR => {
                let outbound = json!({
                    "type": "shadowsocksr",
                    "tag": remark,
                    "server": node.server,
                    "server_port": node.port,
                    "method": node.cipher,
                    "password": node.password,
                    "protocol": node.protocol,
                    "protocol_param": node.protocol_param,
                    "obfs": node.obfs,
                    "obfs_param": node.obfs_param
                });

                proxy_outbounds.push(outbound);
            }
            ProxyType::VMess => {
                let mut outbound = json!({
                    "type": "vmess",
                    "tag": remark,
                    "server": node.server,
                    "server_port": node.port,
                    "uuid": node.uuid,
                    "alter_id": node.alter_id,
                    "security": if node.cipher.is_empty() { "auto" } else { &node.cipher }
                });

                // Add TLS settings
                if node.tls {
                    outbound["tls"] = json!({
                        "enabled": true,
                        "server_name": node.sni.clone().unwrap_or_default(),
                        "insecure": node.skip_cert_verify.unwrap_or(false)
                    });
                }

                // Add transport settings
                if !node.network.is_empty() {
                    match node.network.as_str() {
                        "tcp" => {
                            outbound["transport"] = json!({
                                "type": "tcp"
                            });
                        }
                        "ws" => {
                            let mut transport = json!({
                                "type": "ws"
                            });

                            if !node.path.is_empty() {
                                transport["path"] = json!(node.path);
                            }

                            if !node.host.is_empty() {
                                transport["headers"] = json!({
                                    "Host": node.host
                                });
                            }

                            outbound["transport"] = transport;
                        }
                        "h2" => {
                            let mut transport = json!({
                                "type": "http"
                            });

                            if !node.path.is_empty() {
                                transport["path"] = json!(node.path);
                            }

                            if !node.host.is_empty() {
                                transport["host"] = json!([node.host]);
                            }

                            outbound["transport"] = transport;
                        }
                        "grpc" => {
                            let mut transport = json!({
                                "type": "grpc"
                            });

                            if !node.path.is_empty() {
                                transport["service_name"] = json!(node.path);
                            }

                            outbound["transport"] = transport;
                        }
                        _ => {}
                    }
                }

                proxy_outbounds.push(outbound);
            }
            ProxyType::Trojan => {
                let mut outbound = json!({
                    "type": "trojan",
                    "tag": remark,
                    "server": node.server,
                    "server_port": node.port,
                    "password": node.password,
                    "tls": {
                        "enabled": true,
                        "server_name": node.sni.clone().unwrap_or_default(),
                        "insecure": node.skip_cert_verify.unwrap_or(false)
                    }
                });

                // Add transport settings
                if !node.network.is_empty() && node.network != "tcp" {
                    match node.network.as_str() {
                        "ws" => {
                            let mut transport = json!({
                                "type": "ws"
                            });

                            if !node.path.is_empty() {
                                transport["path"] = json!(node.path);
                            }

                            if !node.host.is_empty() {
                                transport["headers"] = json!({
                                    "Host": node.host
                                });
                            }

                            outbound["transport"] = transport;
                        }
                        "grpc" => {
                            let mut transport = json!({
                                "type": "grpc"
                            });

                            if !node.path.is_empty() {
                                transport["service_name"] = json!(node.path);
                            }

                            outbound["transport"] = transport;
                        }
                        _ => {}
                    }
                }

                proxy_outbounds.push(outbound);
            }
            ProxyType::HTTP | ProxyType::HTTPS => {
                let mut outbound = json!({
                    "type": "http",
                    "tag": remark,
                    "server": node.server,
                    "server_port": node.port
                });

                // Add username/password if present
                if !node.username.is_empty() {
                    outbound["username"] = json!(node.username);
                }

                if !node.password.is_empty() {
                    outbound["password"] = json!(node.password);
                }

                // Add TLS for HTTPS
                if node.proxy_type == ProxyType::HTTPS {
                    outbound["tls"] = json!({
                        "enabled": true,
                        "insecure": node.skip_cert_verify.unwrap_or(false)
                    });
                }

                proxy_outbounds.push(outbound);
            }
            ProxyType::Socks5 => {
                let mut outbound = json!({
                    "type": "socks",
                    "tag": remark,
                    "server": node.server,
                    "server_port": node.port
                });

                // Add username/password if present
                if !node.username.is_empty() {
                    outbound["username"] = json!(node.username);
                }

                if !node.password.is_empty() {
                    outbound["password"] = json!(node.password);
                }

                proxy_outbounds.push(outbound);
            }
            _ => continue,
        }
    }

    // Process proxy groups
    let mut selector_outbounds = Vec::new();
    let mut urltest_outbounds = Vec::new();

    for group in extra_proxy_group {
        let mut outbound_tags = Vec::new();

        // Generate proxy list for this group
        for proxy_name in &group.proxies {
            let mut filtered_nodes = Vec::new();
            group_generate(proxy_name, nodes, &mut filtered_nodes, false, ext);
            for node in filtered_nodes {
                outbound_tags.push(node);
            }
        }

        // Create appropriate outbound based on group type
        match group.type_field.as_str() {
            "select" => {
                let selector = json!({
                    "type": "selector",
                    "tag": group.name,
                    "outbounds": outbound_tags,
                    "default": outbound_tags.first().unwrap_or(&String::new())
                });
                selector_outbounds.push(selector);
            }
            "url-test" | "fallback" | "load-balance" => {
                let mut urltest = json!({
                    "type": "urltest",
                    "tag": group.name,
                    "outbounds": outbound_tags
                });

                if !group.url.is_empty() {
                    urltest["url"] = json!(group.url);
                } else {
                    urltest["url"] = json!("https://www.gstatic.com/generate_204");
                }

                if group.interval > 0 {
                    urltest["interval"] = json!(format!("{}s", group.interval));
                } else {
                    urltest["interval"] = json!("300s");
                }

                urltest_outbounds.push(urltest);
            }
            _ => {}
        }
    }

    // Add direct, block, and dns outbounds if they don't exist
    let default_outbounds = vec![
        json!({
            "type": "direct",
            "tag": "direct"
        }),
        json!({
            "type": "block",
            "tag": "block"
        }),
        json!({
            "type": "dns",
            "tag": "dns-out"
        }),
    ];

    // Combine all outbounds
    let mut all_outbounds = Vec::new();

    // Add proxy outbounds
    for outbound in proxy_outbounds {
        all_outbounds.push(outbound);
    }

    // Add selector outbounds
    for outbound in selector_outbounds {
        all_outbounds.push(outbound);
    }

    // Add urltest outbounds
    for outbound in urltest_outbounds {
        all_outbounds.push(outbound);
    }

    // Add default outbounds
    for outbound in default_outbounds {
        all_outbounds.push(outbound);
    }

    // Update outbounds in config
    config["outbounds"] = json!(all_outbounds);

    // Process rules if rule generator is enabled
    if ext.enable_rule_generator && !ruleset_content_array.is_empty() {
        // Ensure route exists
        if !config.get("route").is_some() {
            config["route"] = json!({});
        }

        // Ensure rules array exists
        if !config["route"].get("rules").is_some() {
            config["route"]["rules"] = json!([]);
        }

        // Clear existing rules if overwrite is enabled
        if ext.overwrite_original_rules {
            config["route"]["rules"] = json!([]);
        }

        // Get rules array
        let rules = config["route"]["rules"].as_array_mut().unwrap();

        // Add default rules
        rules.push(json!({
            "inbound": ["dns-in"],
            "outbound": "dns-out"
        }));

        // Process each ruleset
        for ruleset in ruleset_content_array {
            for rule in &ruleset.rules {
                match rule.rule_type.as_str() {
                    "DOMAIN" => {
                        rules.push(json!({
                            "domain": [rule.rule_content.clone()],
                            "outbound": ruleset.group
                        }));
                    }
                    "DOMAIN-SUFFIX" => {
                        rules.push(json!({
                            "domain_suffix": [rule.rule_content.clone()],
                            "outbound": ruleset.group
                        }));
                    }
                    "DOMAIN-KEYWORD" => {
                        rules.push(json!({
                            "domain_keyword": [rule.rule_content.clone()],
                            "outbound": ruleset.group
                        }));
                    }
                    "IP-CIDR" | "IP-CIDR6" => {
                        rules.push(json!({
                            "ip_cidr": [rule.rule_content.clone()],
                            "outbound": ruleset.group
                        }));
                    }
                    "GEOIP" => {
                        rules.push(json!({
                            "geoip": [rule.rule_content.clone()],
                            "outbound": ruleset.group
                        }));
                    }
                    "FINAL" => {
                        rules.push(json!({
                            "outbound": ruleset.group
                        }));
                    }
                    _ => {}
                }
            }
        }
    }

    // Convert to string with pretty formatting
    match serde_json::to_string_pretty(&config) {
        Ok(result) => result,
        Err(_) => String::new(),
    }
}

/// Format an interval value for SingBox
///
/// # Arguments
/// * `interval` - Interval value in seconds
fn format_sing_box_interval(interval: u32) -> String {
    // Format the interval as per SingBox requirements
    if interval == 0 {
        return String::new();
    }

    format!("{}s", interval)
}

/// Build a transport object for SingBox
///
/// # Arguments
/// * `proxy` - Proxy node to build transport for
fn build_sing_box_transport(proxy: &Proxy) -> Value {
    // Placeholder implementation
    json!({})
}
