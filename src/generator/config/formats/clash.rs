use crate::generator::config::group::group_generate;
use crate::generator::config::subexport::{process_remark, ExtraSettings, ProxyGroupConfigs};
use crate::generator::ruleconvert::ruleset_to_clash_str;
use crate::models::{Proxy, ProxyType, RulesetContent};
use crate::utils::yaml::YamlNode;
use log::error;
use serde_json::{self, json, Map, Value as JsonValue};
use serde_yaml::{self, Mapping, Value as YamlValue};
use std::collections::HashSet;

// Lists of supported protocols and encryption methods for filtering in ClashR
lazy_static::lazy_static! {
    static ref CLASH_SSR_CIPHERS: HashSet<&'static str> = {
        let mut ciphers = HashSet::new();
        ciphers.insert("aes-128-cfb");
        ciphers.insert("aes-192-cfb");
        ciphers.insert("aes-256-cfb");
        ciphers.insert("aes-128-ctr");
        ciphers.insert("aes-192-ctr");
        ciphers.insert("aes-256-ctr");
        ciphers.insert("aes-128-ofb");
        ciphers.insert("aes-192-ofb");
        ciphers.insert("aes-256-ofb");
        ciphers.insert("des-cfb");
        ciphers.insert("bf-cfb");
        ciphers.insert("cast5-cfb");
        ciphers.insert("rc4-md5");
        ciphers.insert("chacha20");
        ciphers.insert("chacha20-ietf");
        ciphers.insert("salsa20");
        ciphers.insert("camellia-128-cfb");
        ciphers.insert("camellia-192-cfb");
        ciphers.insert("camellia-256-cfb");
        ciphers.insert("idea-cfb");
        ciphers.insert("rc2-cfb");
        ciphers.insert("seed-cfb");
        ciphers
    };

    static ref CLASHR_PROTOCOLS: HashSet<&'static str> = {
        let mut protocols = HashSet::new();
        protocols.insert("origin");
        protocols.insert("auth_sha1_v4");
        protocols.insert("auth_aes128_md5");
        protocols.insert("auth_aes128_sha1");
        protocols.insert("auth_chain_a");
        protocols.insert("auth_chain_b");
        protocols
    };

    static ref CLASHR_OBFS: HashSet<&'static str> = {
        let mut obfs = HashSet::new();
        obfs.insert("plain");
        obfs.insert("http_simple");
        obfs.insert("http_post");
        obfs.insert("random_head");
        obfs.insert("tls1.2_ticket_auth");
        obfs.insert("tls1.2_ticket_fastauth");
        obfs
    };
}

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
        Err(e) => {
            error!("Clash base loader failed with error: {}", e);
            return String::new();
        }
    };

    // Apply conversion to the YAML node
    proxy_to_clash_yaml(
        nodes,
        &mut yaml_node,
        ruleset_content_array,
        extra_proxy_group,
        clash_r,
        ext,
    );

    // If nodelist mode is enabled, just return the YAML node
    if ext.nodelist {
        return match yaml_node.to_string() {
            Ok(result) => result,
            Err(_) => String::new(),
        };
    }

    // Handle rule generation if enabled
    if !ext.enable_rule_generator {
        return match yaml_node.to_string() {
            Ok(result) => result,
            Err(_) => String::new(),
        };
    }

    // Handle managed config and clash script
    if !ext.managed_config_prefix.is_empty() || ext.clash_script {
        // Set mode if it exists
        if yaml_node.value.get("mode").is_some() {
            if let YamlValue::Mapping(ref mut map) = yaml_node.value {
                map.insert(
                    YamlValue::String("mode".to_string()),
                    YamlValue::String(
                        if ext.clash_script {
                            if ext.clash_new_field_name {
                                "script"
                            } else {
                                "Script"
                            }
                        } else {
                            if ext.clash_new_field_name {
                                "rule"
                            } else {
                                "Rule"
                            }
                        }
                        .to_string(),
                    ),
                );
            }
        }

        // TODO: Implement renderClashScript
        // For now, just return the YAML
        return match yaml_node.to_string() {
            Ok(result) => result,
            Err(_) => String::new(),
        };
    }

    // Generate rules and return combined output
    let rules_str = ruleset_to_clash_str(
        &yaml_node.value,
        ruleset_content_array,
        ext.overwrite_original_rules,
        ext.clash_new_field_name,
    );

    let yaml_output = match yaml_node.to_string() {
        Ok(result) => result,
        Err(_) => return String::new(),
    };

    format!("{}{}", yaml_output, rules_str)
}

/// Convert proxies to Clash format with YAML node
///
/// This function modifies a YAML node in place to add Clash configuration
/// for the provided proxy nodes.
///
/// # Arguments
/// * `nodes` - List of proxy nodes to convert
/// * `yaml_node` - YAML node to modify
/// * `ruleset_content_array` - Array of ruleset contents to apply
/// * `extra_proxy_group` - Extra proxy group configurations
/// * `clash_r` - Whether to use ClashR format
/// * `ext` - Extra settings for conversion
pub fn proxy_to_clash_yaml(
    nodes: &mut Vec<Proxy>,
    yaml_node: &mut YamlNode,
    ruleset_content_array: &Vec<RulesetContent>,
    extra_proxy_group: &ProxyGroupConfigs,
    clash_r: bool,
    ext: &mut ExtraSettings,
) {
    // Style settings
    let proxy_block = ext.clash_proxies_style == "block";
    let proxy_compact = ext.clash_proxies_style == "compact";
    let group_block = ext.clash_proxy_groups_style == "block";
    let group_compact = ext.clash_proxy_groups_style == "compact";

    // Create JSON structure for the proxies
    let mut proxies_json = Vec::new();

    // Process each node
    for node in nodes.iter_mut() {
        // Process remark
        let mut remark = node.remark.clone();

        // Add proxy type to remark if enabled
        if ext.append_proxy_type {
            remark = format!("[{}] {}", node.proxy_type.to_string(), remark);
        }

        process_remark(&mut remark, ext, true);

        // Check if proxy type is supported
        let mut proxy_json = match node.proxy_type {
            ProxyType::Shadowsocks => {
                // Skip chacha20 encryption if filter_deprecated is enabled
                if ext.filter_deprecated && node.encrypt_method.as_deref() == Some("chacha20") {
                    continue;
                }

                let mut proxy = json!({
                    "name": remark,
                    "type": "ss",
                    "server": node.hostname,
                    "port": node.port,
                    "cipher": node.encrypt_method.as_deref().unwrap_or(""),
                    "password": node.password.as_deref().unwrap_or("")
                });

                // Handle numeric passwords (in Rust we don't need special handling for JSON tags)

                // Add plugin if present
                if let Some(plugin) = &node.plugin {
                    if !plugin.is_empty() {
                        let plugin_option = node.plugin_option.as_deref().unwrap_or("");

                        match plugin.as_str() {
                            "simple-obfs" | "obfs-local" => {
                                proxy["plugin"] = json!("obfs");

                                // TODO: Implement urlDecode and getUrlArg functions
                                // For now just use the plugin_option directly
                                let mut plugin_opts = Map::new();
                                plugin_opts.insert(
                                    "mode".to_string(),
                                    JsonValue::String(plugin_option.to_string()),
                                );

                                if let Some(host) = &node.host {
                                    plugin_opts.insert(
                                        "host".to_string(),
                                        JsonValue::String(host.clone()),
                                    );
                                }

                                proxy["plugin-opts"] = JsonValue::Object(plugin_opts);
                            }
                            "v2ray-plugin" => {
                                proxy["plugin"] = json!("v2ray-plugin");

                                let mut plugin_opts = Map::new();

                                // TODO: Parse plugin options properly with getUrlArg
                                plugin_opts.insert(
                                    "mode".to_string(),
                                    JsonValue::String(plugin_option.to_string()),
                                );

                                if let Some(host) = &node.host {
                                    plugin_opts.insert(
                                        "host".to_string(),
                                        JsonValue::String(host.clone()),
                                    );
                                }

                                if let Some(path) = &node.path {
                                    plugin_opts.insert(
                                        "path".to_string(),
                                        JsonValue::String(path.clone()),
                                    );
                                }

                                if plugin_option.contains("tls") {
                                    plugin_opts.insert("tls".to_string(), JsonValue::Bool(true));
                                }

                                if plugin_option.contains("mux") {
                                    plugin_opts.insert("mux".to_string(), JsonValue::Bool(true));
                                }

                                if let Some(allow_insecure) = node.allow_insecure {
                                    plugin_opts.insert(
                                        "skip-cert-verify".to_string(),
                                        JsonValue::Bool(allow_insecure),
                                    );
                                }

                                proxy["plugin-opts"] = JsonValue::Object(plugin_opts);
                            }
                            _ => {}
                        }
                    }
                }

                proxy
            }
            ProxyType::ShadowsocksR => {
                // Skip if not using ClashR or if using deprecated features
                if ext.filter_deprecated {
                    if !clash_r {
                        continue;
                    }

                    let encrypt_method = node.encrypt_method.as_deref().unwrap_or("");
                    if !CLASH_SSR_CIPHERS.contains(encrypt_method) {
                        continue;
                    }

                    let protocol = node.protocol.as_deref().unwrap_or("");
                    if !CLASHR_PROTOCOLS.contains(protocol) {
                        continue;
                    }

                    let obfs = node.obfs.as_deref().unwrap_or("");
                    if !CLASHR_OBFS.contains(obfs) {
                        continue;
                    }
                }

                let encrypt_method = node.encrypt_method.as_deref().unwrap_or("");
                let cipher = if encrypt_method == "none" {
                    "dummy"
                } else {
                    encrypt_method
                };

                let mut proxy = json!({
                    "name": remark,
                    "type": "ssr",
                    "server": node.hostname,
                    "port": node.port,
                    "cipher": cipher,
                    "password": node.password.as_deref().unwrap_or(""),
                    "protocol": node.protocol.as_deref().unwrap_or(""),
                    "obfs": node.obfs.as_deref().unwrap_or("")
                });

                // ClashR uses different field names than regular Clash
                if clash_r {
                    proxy["protocolparam"] = json!(node.protocol_param.as_deref().unwrap_or(""));
                    proxy["obfsparam"] = json!(node.obfs_param.as_deref().unwrap_or(""));
                } else {
                    proxy["protocol-param"] = json!(node.protocol_param.as_deref().unwrap_or(""));
                    proxy["obfs-param"] = json!(node.obfs_param.as_deref().unwrap_or(""));
                }

                proxy
            }
            ProxyType::VMess => {
                let mut proxy = json!({
                    "name": remark,
                    "type": "vmess",
                    "server": node.hostname,
                    "port": node.port,
                    "uuid": node.user_id.as_deref().unwrap_or(""),
                    "alterId": node.alter_id
                });

                // Add cipher
                let encrypt_method = node.encrypt_method.as_deref().unwrap_or("");
                proxy["cipher"] = json!(if encrypt_method.is_empty() {
                    "auto"
                } else {
                    encrypt_method
                });

                // Add TLS settings
                if node.tls_secure {
                    proxy["tls"] = json!(true);

                    if let Some(server_name) = &node.server_name {
                        if !server_name.is_empty() {
                            proxy["servername"] = json!(server_name);
                        }
                    }
                }

                if let Some(allow_insecure) = node.allow_insecure {
                    proxy["skip-cert-verify"] = json!(allow_insecure);
                }

                // Add network settings
                if let Some(protocol) = &node.transfer_protocol {
                    match protocol.as_str() {
                        "tcp" => {}
                        "ws" => {
                            proxy["network"] = json!("ws");

                            // Use new field names if enabled
                            if ext.clash_new_field_name {
                                let mut ws_opts = Map::new();

                                if let Some(path) = &node.path {
                                    ws_opts.insert(
                                        "path".to_string(),
                                        JsonValue::String(path.clone()),
                                    );
                                }

                                if node.host.as_ref().map_or(false, |h| !h.is_empty())
                                    || node.edge.as_ref().map_or(false, |e| !e.is_empty())
                                {
                                    let mut headers = Map::new();

                                    if let Some(host) = &node.host {
                                        if !host.is_empty() {
                                            headers.insert(
                                                "Host".to_string(),
                                                JsonValue::String(host.clone()),
                                            );
                                        }
                                    }

                                    if let Some(edge) = &node.edge {
                                        if !edge.is_empty() {
                                            headers.insert(
                                                "Edge".to_string(),
                                                JsonValue::String(edge.clone()),
                                            );
                                        }
                                    }

                                    ws_opts
                                        .insert("headers".to_string(), JsonValue::Object(headers));
                                }

                                proxy["ws-opts"] = JsonValue::Object(ws_opts);
                            } else {
                                // Legacy field names
                                if let Some(path) = &node.path {
                                    proxy["ws-path"] = json!(path);
                                }

                                if node.host.as_ref().map_or(false, |h| !h.is_empty())
                                    || node.edge.as_ref().map_or(false, |e| !e.is_empty())
                                {
                                    let mut headers = Map::new();

                                    if let Some(host) = &node.host {
                                        if !host.is_empty() {
                                            headers.insert(
                                                "Host".to_string(),
                                                JsonValue::String(host.clone()),
                                            );
                                        }
                                    }

                                    if let Some(edge) = &node.edge {
                                        if !edge.is_empty() {
                                            headers.insert(
                                                "Edge".to_string(),
                                                JsonValue::String(edge.clone()),
                                            );
                                        }
                                    }

                                    proxy["ws-headers"] = JsonValue::Object(headers);
                                }
                            }
                        }
                        "http" => {
                            proxy["network"] = json!("http");

                            let mut http_opts = Map::new();
                            http_opts
                                .insert("method".to_string(), JsonValue::String("GET".to_string()));

                            if let Some(path) = &node.path {
                                http_opts.insert(
                                    "path".to_string(),
                                    JsonValue::Array(vec![JsonValue::String(path.clone())]),
                                );
                            }

                            if node.host.as_ref().map_or(false, |h| !h.is_empty())
                                || node.edge.as_ref().map_or(false, |e| !e.is_empty())
                            {
                                let mut headers = Map::new();

                                if let Some(host) = &node.host {
                                    if !host.is_empty() {
                                        headers.insert(
                                            "Host".to_string(),
                                            JsonValue::Array(vec![JsonValue::String(host.clone())]),
                                        );
                                    }
                                }

                                if let Some(edge) = &node.edge {
                                    if !edge.is_empty() {
                                        headers.insert(
                                            "Edge".to_string(),
                                            JsonValue::Array(vec![JsonValue::String(edge.clone())]),
                                        );
                                    }
                                }

                                http_opts.insert("headers".to_string(), JsonValue::Object(headers));
                            }

                            proxy["http-opts"] = JsonValue::Object(http_opts);
                        }
                        "h2" => {
                            proxy["network"] = json!("h2");

                            let mut h2_opts = Map::new();

                            if let Some(path) = &node.path {
                                h2_opts.insert("path".to_string(), JsonValue::String(path.clone()));
                            }

                            if let Some(host) = &node.host {
                                if !host.is_empty() {
                                    h2_opts.insert(
                                        "host".to_string(),
                                        JsonValue::Array(vec![JsonValue::String(host.clone())]),
                                    );
                                }
                            }

                            proxy["h2-opts"] = JsonValue::Object(h2_opts);
                        }
                        "grpc" => {
                            proxy["network"] = json!("grpc");

                            if let Some(host) = &node.host {
                                if !host.is_empty() {
                                    proxy["servername"] = json!(host);
                                }
                            }

                            let mut grpc_opts = Map::new();

                            if let Some(path) = &node.path {
                                grpc_opts.insert(
                                    "grpc-service-name".to_string(),
                                    JsonValue::String(path.clone()),
                                );
                            }

                            proxy["grpc-opts"] = JsonValue::Object(grpc_opts);
                        }
                        _ => continue,
                    }
                }

                proxy
            }
            ProxyType::Trojan => {
                let mut proxy = json!({
                    "name": remark,
                    "type": "trojan",
                    "server": node.hostname,
                    "port": node.port,
                    "password": node.password.as_deref().unwrap_or("")
                });

                if let Some(host) = &node.host {
                    if !host.is_empty() {
                        proxy["sni"] = json!(host);
                    }
                }

                if let Some(allow_insecure) = node.allow_insecure {
                    proxy["skip-cert-verify"] = json!(allow_insecure);
                }

                // Handle network protocols
                if let Some(protocol) = &node.transfer_protocol {
                    match protocol.as_str() {
                        "tcp" => {}
                        "grpc" => {
                            proxy["network"] = json!("grpc");

                            if let Some(path) = &node.path {
                                if !path.is_empty() {
                                    let mut grpc_opts = Map::new();
                                    grpc_opts.insert(
                                        "grpc-service-name".to_string(),
                                        JsonValue::String(path.clone()),
                                    );
                                    proxy["grpc-opts"] = JsonValue::Object(grpc_opts);
                                }
                            }
                        }
                        "ws" => {
                            proxy["network"] = json!("ws");

                            let mut ws_opts = Map::new();

                            if let Some(path) = &node.path {
                                ws_opts.insert("path".to_string(), JsonValue::String(path.clone()));
                            }

                            if let Some(host) = &node.host {
                                if !host.is_empty() {
                                    let mut headers = Map::new();
                                    headers.insert(
                                        "Host".to_string(),
                                        JsonValue::String(host.clone()),
                                    );
                                    ws_opts
                                        .insert("headers".to_string(), JsonValue::Object(headers));
                                }
                            }

                            proxy["ws-opts"] = JsonValue::Object(ws_opts);
                        }
                        _ => {}
                    }
                }

                proxy
            }
            ProxyType::HTTP | ProxyType::HTTPS => {
                let mut proxy = json!({
                    "name": remark,
                    "type": "http",
                    "server": node.hostname,
                    "port": node.port
                });

                if let Some(username) = &node.username {
                    if !username.is_empty() {
                        proxy["username"] = json!(username);
                    }
                }

                if let Some(password) = &node.password {
                    if !password.is_empty() {
                        proxy["password"] = json!(password);
                    }
                }

                // Set TLS for HTTPS
                if node.proxy_type == ProxyType::HTTPS {
                    proxy["tls"] = json!(true);
                }

                if let Some(allow_insecure) = node.allow_insecure {
                    proxy["skip-cert-verify"] = json!(allow_insecure);
                }

                proxy
            }
            ProxyType::Socks5 => {
                let mut proxy = json!({
                    "name": remark,
                    "type": "socks5",
                    "server": node.hostname,
                    "port": node.port
                });

                if let Some(username) = &node.username {
                    if !username.is_empty() {
                        proxy["username"] = json!(username);
                    }
                }

                if let Some(password) = &node.password {
                    if !password.is_empty() {
                        proxy["password"] = json!(password);
                    }
                }

                if let Some(allow_insecure) = node.allow_insecure {
                    proxy["skip-cert-verify"] = json!(allow_insecure);
                }

                proxy
            }
            ProxyType::Snell => {
                // Skip Snell v4+ if exists
                if node.snell_version >= 4 {
                    continue;
                }

                let mut proxy = json!({
                    "name": remark,
                    "type": "snell",
                    "server": node.hostname,
                    "port": node.port,
                    "psk": node.password.as_deref().unwrap_or("")
                });

                if node.snell_version > 0 {
                    proxy["version"] = json!(node.snell_version);
                }

                if let Some(obfs) = &node.obfs {
                    if !obfs.is_empty() {
                        proxy["obfs"] = json!(obfs);

                        if let Some(host) = &node.host {
                            if !host.is_empty() {
                                proxy["obfs-host"] = json!(host);
                            }
                        }
                    }
                }

                proxy
            }
            ProxyType::WireGuard => {
                let mut proxy = json!({
                    "name": remark,
                    "type": "wireguard",
                    "server": node.hostname,
                    "port": node.port
                });

                if let Some(public_key) = &node.public_key {
                    proxy["public-key"] = json!(public_key);
                }

                if let Some(private_key) = &node.private_key {
                    proxy["private-key"] = json!(private_key);
                }

                if let Some(self_ip) = &node.self_ip {
                    proxy["ip"] = json!(self_ip);
                }

                if let Some(self_ipv6) = &node.self_ipv6 {
                    if !self_ipv6.is_empty() {
                        proxy["ipv6"] = json!(self_ipv6);
                    }
                }

                if let Some(pre_shared_key) = &node.pre_shared_key {
                    if !pre_shared_key.is_empty() {
                        proxy["preshared-key"] = json!(pre_shared_key);
                    }
                }

                if !node.dns_servers.is_empty() {
                    let mut dns_servers = Vec::new();
                    for server in &node.dns_servers {
                        dns_servers.push(JsonValue::String(server.clone()));
                    }
                    proxy["dns"] = JsonValue::Array(dns_servers);
                }

                if node.mtu > 0 {
                    proxy["mtu"] = json!(node.mtu);
                }

                if !node.allowed_ips.is_empty() {
                    let allowed_ips: Vec<JsonValue> = node
                        .allowed_ips
                        .split(',')
                        .map(|ip| JsonValue::String(ip.trim().to_string()))
                        .collect();
                    proxy["allowed-ips"] = JsonValue::Array(allowed_ips);
                } else {
                    // Default allowed IPs
                    proxy["allowed-ips"] = JsonValue::Array(vec![
                        JsonValue::String("0.0.0.0/0".to_string()),
                        JsonValue::String("::/0".to_string()),
                    ]);
                }

                if node.keep_alive > 0 {
                    proxy["keepalive"] = json!(node.keep_alive);
                }

                proxy
            }
            ProxyType::Hysteria => {
                let mut proxy = json!({
                    "name": remark,
                    "type": "hysteria",
                    "server": node.hostname,
                    "port": node.port
                });

                // Add auth fields
                if let Some(auth_type) = &node.protocol {
                    match auth_type.as_str() {
                        "auth" => {
                            // Auth string format
                            if let Some(password) = &node.password {
                                proxy["auth_str"] = json!(password);
                            }
                        }
                        "base64" => {
                            // Base64 auth string
                            if let Some(password) = &node.password {
                                proxy["auth_str"] = json!(password);
                            }
                        }
                        "none" => {
                            // No auth
                        }
                        _ => {}
                    }
                }

                // Add obfs fields
                if let Some(obfs) = &node.obfs {
                    if !obfs.is_empty() {
                        proxy["obfs"] = json!(obfs);
                    }
                }

                // Add TLS settings
                proxy["tls"] = json!(true); // Hysteria always uses TLS

                if let Some(server_name) = &node.server_name {
                    if !server_name.is_empty() {
                        proxy["sni"] = json!(server_name);
                    }
                }

                if let Some(allow_insecure) = node.allow_insecure {
                    proxy["skip-cert-verify"] = json!(allow_insecure);
                }

                // Add protocol-specific fields
                if let Some(up_mbps) = &node.quic_secret {
                    // In the Hysteria protocol, quic_secret might be repurposed to store up_mbps
                    if let Ok(up_mbps_value) = up_mbps.parse::<u32>() {
                        proxy["up_mbps"] = json!(up_mbps_value);
                    }
                }

                if let Some(down_mbps) = &node.quic_secure {
                    // In the Hysteria protocol, quic_secure might be repurposed to store down_mbps
                    if let Ok(down_mbps_value) = down_mbps.parse::<u32>() {
                        proxy["down_mbps"] = json!(down_mbps_value);
                    }
                }

                // Add ALPN protocols if specified
                if let Some(alpn) = &node.edge {
                    // In Hysteria, edge might be repurposed to store ALPN
                    let alpn_array: Vec<JsonValue> = alpn
                        .split(',')
                        .map(|protocol| JsonValue::String(protocol.trim().to_string()))
                        .collect();
                    proxy["alpn"] = JsonValue::Array(alpn_array);
                } else {
                    // Default ALPN
                    proxy["alpn"] = JsonValue::Array(vec![JsonValue::String("h3".to_string())]);
                }

                proxy
            }
            ProxyType::Hysteria2 => {
                let mut proxy = json!({
                    "name": remark,
                    "type": "hysteria2",
                    "server": node.hostname,
                    "port": node.port
                });

                // Add password/auth
                if let Some(password) = &node.password {
                    proxy["password"] = json!(password);
                }

                // Add TLS settings
                proxy["tls"] = json!(true); // Hysteria2 always uses TLS

                if let Some(server_name) = &node.server_name {
                    if !server_name.is_empty() {
                        proxy["sni"] = json!(server_name);
                    }
                }

                if let Some(allow_insecure) = node.allow_insecure {
                    proxy["skip-cert-verify"] = json!(allow_insecure);
                }

                // Add bandwidth settings
                if let Some(up_mbps) = &node.quic_secret {
                    // In the Hysteria2 protocol, quic_secret might be repurposed to store up_mbps
                    if let Ok(up_mbps_value) = up_mbps.parse::<u32>() {
                        proxy["up"] = json!(up_mbps_value);
                    }
                }

                if let Some(down_mbps) = &node.quic_secure {
                    // In the Hysteria2 protocol, quic_secure might be repurposed to store down_mbps
                    if let Ok(down_mbps_value) = down_mbps.parse::<u32>() {
                        proxy["down"] = json!(down_mbps_value);
                    }
                }

                // Add obfs settings
                if let Some(obfs) = &node.obfs {
                    if !obfs.is_empty() {
                        proxy["obfs"] = json!(obfs);

                        if let Some(obfs_password) = &node.obfs_param {
                            proxy["obfs-password"] = json!(obfs_password);
                        }
                    }
                }

                // Add ALPN protocols if specified
                if let Some(alpn) = &node.edge {
                    // In Hysteria2, edge might be repurposed to store ALPN
                    let alpn_array: Vec<JsonValue> = alpn
                        .split(',')
                        .map(|protocol| JsonValue::String(protocol.trim().to_string()))
                        .collect();
                    proxy["alpn"] = JsonValue::Array(alpn_array);
                } else {
                    // Default ALPN for Hysteria2
                    proxy["alpn"] = JsonValue::Array(vec![JsonValue::String("h3".to_string())]);
                }

                proxy
            }
            _ => continue,
        };

        // Add common fields
        if let Some(udp) = node.udp {
            proxy_json["udp"] = JsonValue::Bool(udp);
        }

        if let Some(tfo) = node.tcp_fast_open {
            proxy_json["tfo"] = JsonValue::Bool(tfo);
        }

        if let Some(scv) = node.allow_insecure {
            proxy_json["skip-cert-verify"] = JsonValue::Bool(scv);
        }

        // Add to proxies array
        proxies_json.push(proxy_json);
    }

    // Update the YAML node with proxies
    if let YamlValue::Mapping(ref mut map) = yaml_node.value {
        // Convert JSON proxies array to YAML
        let proxies_yaml_value =
            serde_yaml::to_value(&proxies_json).unwrap_or(YamlValue::Sequence(Vec::new()));
        map.insert(YamlValue::String("proxies".to_string()), proxies_yaml_value);
    }

    // Add proxy groups if present
    if !extra_proxy_group.is_empty() {
        let mut proxy_groups_json = Vec::new();

        // Process each proxy group
        for group in extra_proxy_group {
            let mut proxy_group = json!({
                "name": group.name,
                "type": group.type_field
            });

            // Add URL if present
            if !group.url.is_empty() {
                proxy_group["url"] = json!(group.url);
            }

            // Add interval if present
            if group.interval > 0 {
                proxy_group["interval"] = json!(group.interval);
            }

            // Add proxies
            let mut proxies_list = Vec::new();
            for proxy_name in &group.proxies {
                let mut filtered_nodes = Vec::new();
                group_generate(proxy_name, nodes, &mut filtered_nodes, false, ext);
                for node in filtered_nodes {
                    proxies_list.push(JsonValue::String(node));
                }
            }
            proxy_group["proxies"] = JsonValue::Array(proxies_list);

            // Add to proxy groups array
            proxy_groups_json.push(proxy_group);
        }

        // Update the YAML node with proxy groups
        if let YamlValue::Mapping(ref mut map) = yaml_node.value {
            // Convert JSON proxy groups array to YAML
            let proxy_groups_yaml_value =
                serde_yaml::to_value(&proxy_groups_json).unwrap_or(YamlValue::Sequence(Vec::new()));
            map.insert(
                YamlValue::String("proxy-groups".to_string()),
                proxy_groups_yaml_value,
            );
        }
    }
}
