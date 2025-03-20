use crate::generator::config::group::group_generate;
use crate::generator::config::remark::process_remark;
use crate::generator::ruleconvert::ruleset_to_clash_str;
use crate::models::{
    ExtraSettings, ProxyGroupConfigs, ProxyGroupType,
};
use crate::models::{Proxy, ProxyType, RulesetContent};
use crate::utils::tribool::TriboolExt;
use crate::utils::url::get_url_arg;
use crate::utils::yaml::YamlNode;
use log::error;
use serde_json::{self, json, Map, Value as JsonValue};
use serde_yaml::{self, Mapping, Sequence, Value as YamlValue};
use std::collections::HashSet;

// Macro to simplify creating and setting proxies with JsonApplicable trait
macro_rules! apply_if_present {
    ($json:expr, $key:expr, $value:expr) => {
        $value.apply_to_json(&mut $json, $key);
    };
}

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
    // Style settings - in C++ this is used to set serialization style but in Rust we have less control
    // over the serialization format. We keep them for compatibility but their actual effect may differ.
    let proxy_block = ext.clash_proxies_style == "block";
    let proxy_compact = ext.clash_proxies_style == "compact";
    let group_block = ext.clash_proxy_groups_style == "block";
    let group_compact = ext.clash_proxy_groups_style == "compact";

    // Create JSON structure for the proxies
    let mut proxies_json = Vec::new();
    let mut remarks_list = Vec::new();

    // Process each node
    for node in nodes.iter_mut() {
        // Create a local copy of the node for processing
        let mut remark = node.remark.clone();

        // Add proxy type prefix if enabled
        if ext.append_proxy_type {
            remark = format!("[{}] {}", node.proxy_type.to_string(), remark);
        }

        // Process remark with optional remarks list
        process_remark(&mut remark, &remarks_list, false);
        remarks_list.push(remark.clone());

        // Define tribool values with defaults from ext and override with node-specific values if present
        // This matches C++ logic where tribool can be in three states: true, false, or undef
        let udp = node.udp.define(ext.udp);
        let tfo = node.tcp_fast_open.define(ext.tfo);
        let scv = node.allow_insecure.define(ext.skip_cert_verify);

        // Check if proxy type is supported
        let mut proxy_json = match node.proxy_type {
            ProxyType::Shadowsocks => handle_shadowsocks(node, &remark, &scv, ext),
            ProxyType::ShadowsocksR => handle_shadowsocksr(node, &remark, &scv, clash_r, ext),
            ProxyType::VMess => handle_vmess(node, &remark, &scv, ext),
            ProxyType::Trojan => handle_trojan(node, &remark, &scv),
            ProxyType::HTTP | ProxyType::HTTPS => handle_http(node, &remark, &scv),
            ProxyType::Socks5 => handle_socks5(node, &remark, &scv),
            ProxyType::Snell => handle_snell(node, &remark),
            ProxyType::WireGuard => handle_wireguard(node, &remark),
            ProxyType::Hysteria => handle_hysteria(node, &remark, &scv),
            ProxyType::Hysteria2 => handle_hysteria2(node, &remark, &scv),
            _ => continue,
        };

        // Add common fields using tribool logic from C++
        // In C++: only add field if tribool is not undefined
        if let Some(obj) = proxy_json.as_object_mut() {
            udp.apply_to_json(obj, "udp");
            tfo.apply_to_json(obj, "tfo");
            scv.apply_to_json(obj, "skip-cert-verify");
        }

        // Add to proxies array
        proxies_json.push(proxy_json);
    }

    if ext.nodelist {
        let mut provider = YamlValue::Mapping(Mapping::new());
        provider["proxies"] =
            serde_yaml::to_value(&proxies_json).unwrap_or(YamlValue::Sequence(Vec::new()));
        yaml_node.value = provider;
        return;
    }

    // Update the YAML node with proxies
    if let YamlValue::Mapping(ref mut map) = yaml_node.value {
        // Convert JSON proxies array to YAML
        let proxies_yaml_value =
            serde_yaml::to_value(&proxies_json).unwrap_or(YamlValue::Sequence(Vec::new()));
        if ext.clash_new_field_name {
            map.insert(YamlValue::String("proxies".to_string()), proxies_yaml_value);
        } else {
            map.insert(YamlValue::String("Proxy".to_string()), proxies_yaml_value);
        }
    }

    // Add proxy groups if present
    if !extra_proxy_group.is_empty() {
        // Get existing proxy groups if any
        let mut original_groups = if ext.clash_new_field_name {
            match yaml_node.value.get("proxy-groups") {
                Some(YamlValue::Sequence(seq)) => seq.clone(),
                _ => Sequence::new(),
            }
        } else {
            match yaml_node.value.get("Proxy Group") {
                Some(YamlValue::Sequence(seq)) => seq.clone(),
                _ => Sequence::new(),
            }
        };

        // Process each proxy group
        for group in extra_proxy_group {
            // Create the proxy group with basic properties
            let mut proxy_group_map = Mapping::new();
            proxy_group_map.insert(
                YamlValue::String("name".to_string()),
                YamlValue::String(group.name.clone()),
            );

            // Set type (special case for Smart type which becomes url-test)
            let type_str = if group.group_type == ProxyGroupType::Smart {
                "url-test"
            } else {
                group.type_str()
            };
            proxy_group_map.insert(
                YamlValue::String("type".to_string()),
                YamlValue::String(type_str.to_string()),
            );

            // Add fields based on proxy group type
            match group.group_type {
                ProxyGroupType::Select | ProxyGroupType::Relay => {
                    // No special fields for these types
                }
                ProxyGroupType::LoadBalance => {
                    // Add strategy for load balancing
                    proxy_group_map.insert(
                        YamlValue::String("strategy".to_string()),
                        YamlValue::String(group.strategy_str().to_string()),
                    );

                    // Continue with URL test fields (fall through)
                    if !group.lazy {
                        proxy_group_map.insert(
                            YamlValue::String("lazy".to_string()),
                            YamlValue::Bool(group.lazy),
                        );
                    }

                    proxy_group_map.insert(
                        YamlValue::String("url".to_string()),
                        YamlValue::String(group.url.clone()),
                    );

                    if group.interval > 0 {
                        proxy_group_map.insert(
                            YamlValue::String("interval".to_string()),
                            YamlValue::Number(group.interval.into()),
                        );
                    }

                    if group.tolerance > 0 {
                        proxy_group_map.insert(
                            YamlValue::String("tolerance".to_string()),
                            YamlValue::Number(group.tolerance.into()),
                        );
                    }
                }
                ProxyGroupType::Smart | ProxyGroupType::URLTest => {
                    // Add lazy if defined
                    if !group.lazy {
                        proxy_group_map.insert(
                            YamlValue::String("lazy".to_string()),
                            YamlValue::Bool(group.lazy),
                        );
                    }

                    // Add URL test fields
                    proxy_group_map.insert(
                        YamlValue::String("url".to_string()),
                        YamlValue::String(group.url.clone()),
                    );

                    if group.interval > 0 {
                        proxy_group_map.insert(
                            YamlValue::String("interval".to_string()),
                            YamlValue::Number(group.interval.into()),
                        );
                    }

                    if group.tolerance > 0 {
                        proxy_group_map.insert(
                            YamlValue::String("tolerance".to_string()),
                            YamlValue::Number(group.tolerance.into()),
                        );
                    }
                }
                ProxyGroupType::Fallback => {
                    // Add URL test fields
                    proxy_group_map.insert(
                        YamlValue::String("url".to_string()),
                        YamlValue::String(group.url.clone()),
                    );

                    if group.interval > 0 {
                        proxy_group_map.insert(
                            YamlValue::String("interval".to_string()),
                            YamlValue::Number(group.interval.into()),
                        );
                    }

                    if group.tolerance > 0 {
                        proxy_group_map.insert(
                            YamlValue::String("tolerance".to_string()),
                            YamlValue::Number(group.tolerance.into()),
                        );
                    }
                }
                _ => {
                    // Skip unsupported types
                    continue;
                }
            }

            // Add disable-udp if defined
            if group.disable_udp {
                proxy_group_map.insert(
                    YamlValue::String("disable-udp".to_string()),
                    YamlValue::Bool(group.disable_udp),
                );
            }

            // Add persistent if defined
            if group.persistent {
                proxy_group_map.insert(
                    YamlValue::String("persistent".to_string()),
                    YamlValue::Bool(group.persistent),
                );
            }

            // Add evaluate-before-use if defined
            if group.evaluate_before_use {
                proxy_group_map.insert(
                    YamlValue::String("evaluate-before-use".to_string()),
                    YamlValue::Bool(group.evaluate_before_use),
                );
            }

            // Get filtered proxies
            let mut filtered_nodes = Vec::new();
            for proxy_name in &group.proxies {
                group_generate(proxy_name, nodes, &mut filtered_nodes, true, ext);
            }

            // Add provider via "use" field if present, or filtered nodes
            if !group.using_provider.is_empty() {
                let provider_seq = group
                    .using_provider
                    .iter()
                    .map(|name| YamlValue::String(name.clone()))
                    .collect::<Vec<_>>();
                proxy_group_map.insert(
                    YamlValue::String("use".to_string()),
                    YamlValue::Sequence(provider_seq),
                );
            } else {
                // Add DIRECT if empty
                if filtered_nodes.is_empty() {
                    filtered_nodes.push("DIRECT".to_string());
                }
            }

            // Add proxies list
            if !filtered_nodes.is_empty() {
                let proxies_seq = filtered_nodes
                    .into_iter()
                    .map(|name| YamlValue::String(name))
                    .collect::<Vec<_>>();
                proxy_group_map.insert(
                    YamlValue::String("proxies".to_string()),
                    YamlValue::Sequence(proxies_seq),
                );
            }

            // Create the final YamlValue from the map
            let proxy_group = YamlValue::Mapping(proxy_group_map);

            // Check if this group should replace an existing one with the same name
            let mut replaced = false;
            for i in 0..original_groups.len() {
                if let Some(YamlValue::Mapping(map)) = original_groups.get(i) {
                    if let Some(YamlValue::String(name)) =
                        map.get(&YamlValue::String("name".to_string()))
                    {
                        if name == &group.name {
                            if let Some(elem) = original_groups.get_mut(i) {
                                *elem = proxy_group.clone();
                                replaced = true;
                                break;
                            }
                        }
                    }
                }
            }

            // If not replaced, add to the list
            if !replaced {
                original_groups.push(proxy_group);
            }
        }

        // Update the YAML node with proxy groups
        if let YamlValue::Mapping(ref mut map) = yaml_node.value {
            if ext.clash_new_field_name {
                map.insert(
                    YamlValue::String("proxy-groups".to_string()),
                    YamlValue::Sequence(original_groups),
                );
            } else {
                map.insert(
                    YamlValue::String("Proxy Group".to_string()),
                    YamlValue::Sequence(original_groups),
                );
            }
        }
    }
}

// Helper functions for each proxy type
fn handle_shadowsocks(
    node: &Proxy,
    remark: &str,
    scv: &Option<bool>,
    ext: &ExtraSettings,
) -> JsonValue {
    // Skip chacha20 encryption if filter_deprecated is enabled
    if ext.filter_deprecated && node.encrypt_method.as_deref() == Some("chacha20") {
        return JsonValue::Null;
    }

    let mut proxy = json!({
        "name": remark,
        "type": "ss",
        "server": node.hostname,
        "port": node.port,
        "cipher": node.encrypt_method.as_deref().unwrap_or(""),
        "password": node.password.as_deref().unwrap_or("")
    });

    // Add plugin if present
    if let Some(plugin) = &node.plugin {
        if !plugin.is_empty() {
            let plugin_option = node.plugin_option.as_deref().unwrap_or("");

            match plugin.as_str() {
                "simple-obfs" | "obfs-local" => {
                    proxy["plugin"] = json!("obfs");

                    let obfs_mode = get_url_arg(plugin_option, "obfs");
                    let obfs_host = get_url_arg(plugin_option, "obfs-host");

                    let mut plugin_opts = Map::new();
                    plugin_opts.insert("mode".to_string(), JsonValue::String(obfs_mode));
                    if !obfs_host.is_empty() {
                        plugin_opts.insert("host".to_string(), JsonValue::String(obfs_host));
                    }

                    scv.apply_to_json(&mut plugin_opts, "skip-cert-verify");
                    proxy["plugin-opts"] = JsonValue::Object(plugin_opts);
                }
                "v2ray-plugin" => {
                    proxy["plugin"] = json!("v2ray-plugin");

                    let mut plugin_opts = Map::new();

                    let mode = get_url_arg(plugin_option, "mode");
                    if !mode.is_empty() {
                        plugin_opts.insert("mode".to_string(), JsonValue::String(mode));
                    }

                    let host = get_url_arg(plugin_option, "host");
                    if !host.is_empty() {
                        plugin_opts.insert("host".to_string(), JsonValue::String(host));
                    }

                    let path = get_url_arg(plugin_option, "path");
                    if !path.is_empty() {
                        plugin_opts.insert("path".to_string(), JsonValue::String(path));
                    }

                    if plugin_option.contains("tls") {
                        plugin_opts.insert("tls".to_string(), JsonValue::Bool(true));
                    }

                    if plugin_option.contains("mux") {
                        plugin_opts.insert("mux".to_string(), JsonValue::Bool(true));
                    }

                    scv.apply_to_json(&mut plugin_opts, "skip-cert-verify");
                    proxy["plugin-opts"] = JsonValue::Object(plugin_opts);
                }
                _ => {}
            }
        }
    }

    proxy
}

fn handle_shadowsocksr(
    node: &Proxy,
    remark: &str,
    scv: &Option<bool>,
    clash_r: bool,
    ext: &ExtraSettings,
) -> JsonValue {
    // Skip if not using ClashR or if using deprecated features
    if ext.filter_deprecated {
        if !clash_r {
            return JsonValue::Null;
        }

        let encrypt_method = node.encrypt_method.as_deref().unwrap_or("");
        if !CLASH_SSR_CIPHERS.contains(encrypt_method) {
            return JsonValue::Null;
        }

        let protocol = node.protocol.as_deref().unwrap_or("");
        if !CLASHR_PROTOCOLS.contains(protocol) {
            return JsonValue::Null;
        }

        let obfs = node.obfs.as_deref().unwrap_or("");
        if !CLASHR_OBFS.contains(obfs) {
            return JsonValue::Null;
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

fn handle_vmess(node: &Proxy, remark: &str, scv: &Option<bool>, ext: &ExtraSettings) -> JsonValue {
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

    // Add TLS settings and other common fields
    if let Some(obj) = proxy.as_object_mut() {
        apply_proxy_fields(obj, node, scv);
    }

    // Add network settings
    if let Some(protocol) = &node.transfer_protocol {
        match protocol.as_str() {
            "tcp" => {}
            "ws" => {
                proxy["network"] = json!("ws");

                // Use new field names if enabled - exactly matches C++ behavior
                if ext.clash_new_field_name {
                    let mut ws_opts = Map::new();
                    if let Some(path) = &node.path {
                        ws_opts.insert("path".to_string(), JsonValue::String(path.clone()));
                    }

                    if node.host.as_ref().map_or(false, |h| !h.is_empty())
                        || node.edge.as_ref().map_or(false, |e| !e.is_empty())
                    {
                        let mut headers = Map::new();
                        if let Some(host) = &node.host {
                            headers.insert("Host".to_string(), JsonValue::String(host.clone()));
                        }
                        if let Some(edge) = &node.edge {
                            headers.insert("Edge".to_string(), JsonValue::String(edge.clone()));
                        }

                        if !headers.is_empty() {
                            ws_opts.insert("headers".to_string(), JsonValue::Object(headers));
                        }
                    }

                    if !ws_opts.is_empty() {
                        proxy["ws-opts"] = JsonValue::Object(ws_opts);
                    }
                } else {
                    // Legacy field names - exactly matches C++ behavior
                    if let Some(path) = &node.path {
                        proxy["ws-path"] = json!(path);
                    }

                    if node.host.as_ref().map_or(false, |h| !h.is_empty())
                        || node.edge.as_ref().map_or(false, |e| !e.is_empty())
                    {
                        let mut headers = Map::new();
                        if let Some(host) = &node.host {
                            headers.insert("Host".to_string(), JsonValue::String(host.clone()));
                        }
                        if let Some(edge) = &node.edge {
                            headers.insert("Edge".to_string(), JsonValue::String(edge.clone()));
                        }

                        if !headers.is_empty() {
                            proxy["ws-headers"] = JsonValue::Object(headers);
                        }
                    }
                }
            }
            "http" => {
                proxy["network"] = json!("http");

                let mut http_opts = Map::new();
                http_opts.insert("method".to_string(), JsonValue::String("GET".to_string()));

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
            _ => {}
        }
    }

    proxy
}

fn handle_trojan(node: &Proxy, remark: &str, scv: &Option<bool>) -> JsonValue {
    let mut proxy = json!({
        "name": remark,
        "type": "trojan",
        "server": node.hostname,
        "port": node.port,
        "password": node.password.as_deref().unwrap_or("")
    });

    // Add SNI from Host field as per C++ implementation
    if let Some(host) = &node.host {
        if !host.is_empty() {
            proxy["sni"] = json!(host);
        }
    }

    // Add skip-cert-verify if defined
    if let Some(obj) = proxy.as_object_mut() {
        scv.apply_to_json(obj, "skip-cert-verify");
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
                        headers.insert("Host".to_string(), JsonValue::String(host.clone()));
                        ws_opts.insert("headers".to_string(), JsonValue::Object(headers));
                    }
                }

                proxy["ws-opts"] = JsonValue::Object(ws_opts);
            }
            _ => {}
        }
    }

    proxy
}

fn handle_http(node: &Proxy, remark: &str, scv: &Option<bool>) -> JsonValue {
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

    // Add skip-cert-verify if defined
    if let Some(obj) = proxy.as_object_mut() {
        scv.apply_to_json(obj, "skip-cert-verify");
    }

    proxy
}

fn handle_socks5(node: &Proxy, remark: &str, scv: &Option<bool>) -> JsonValue {
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

    // Add skip-cert-verify if defined
    if let Some(obj) = proxy.as_object_mut() {
        scv.apply_to_json(obj, "skip-cert-verify");
    }

    proxy
}

fn handle_snell(node: &Proxy, remark: &str) -> JsonValue {
    // Skip Snell v4+ if exists - exactly matching C++ behavior
    if node.snell_version >= 4 {
        return JsonValue::Null;
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

    // Handling obfs differs slightly between C++ and Rust
    // C++ uses obfs-opts structure, while in clash config it's often the direct field
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

fn handle_wireguard(node: &Proxy, remark: &str) -> JsonValue {
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

fn handle_hysteria(node: &Proxy, remark: &str, scv: &Option<bool>) -> JsonValue {
    let mut proxy = json!({
        "name": remark,
        "type": "hysteria",
        "server": node.hostname,
        "port": node.port
    });

    // Add auth fields
    if let Some(auth_type) = &node.protocol {
        match auth_type.as_str() {
            "auth" | "base64" => {
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

    // Add protocol fields
    if let Some(obfs) = &node.obfs {
        proxy["obfs"] = json!(obfs);
    }

    // Hysteria always uses TLS
    proxy["tls"] = json!(true);
    if let Some(server_name) = &node.server_name {
        proxy["sni"] = json!(server_name);
    }

    // Add skip-cert-verify if defined
    if let Some(obj) = proxy.as_object_mut() {
        scv.apply_to_json(obj, "skip-cert-verify");
    }

    // Add bandwidth settings
    if let Some(up_mbps) = &node.quic_secret {
        if let Ok(up_mbps_value) = up_mbps.parse::<u32>() {
            proxy["up_mbps"] = json!(up_mbps_value);
        }
    }

    if let Some(down_mbps) = &node.quic_secure {
        if let Ok(down_mbps_value) = down_mbps.parse::<u32>() {
            proxy["down_mbps"] = json!(down_mbps_value);
        }
    }

    // Add remaining Hysteria fields
    if let Some(fingerprint) = &node.fingerprint {
        proxy["fingerprint"] = json!(fingerprint);
    }
    if let Some(ca) = &node.ca {
        proxy["ca"] = json!(ca);
    }
    if let Some(ca_str) = &node.ca_str {
        proxy["ca-str"] = json!(ca_str);
    }
    if node.recv_window_conn > 0 {
        proxy["recv-window-conn"] = json!(node.recv_window_conn);
    }
    if node.recv_window > 0 {
        proxy["recv-window"] = json!(node.recv_window);
    }

    // Add disable-mtu-discovery if defined
    if let Some(disable_mtu_discovery) = &node.disable_mtu_discovery {
        proxy["disable-mtu-discovery"] = json!(disable_mtu_discovery);
    }

    // Add hop-interval if defined
    if node.hop_interval > 0 {
        proxy["hop-interval"] = json!(node.hop_interval);
    }

    // Add ALPN protocols if specified
    if let Some(alpn) = &node.edge {
        // In Hysteria, edge might be repurposed to store ALPN
        let alpn_array: Vec<JsonValue> = alpn
            .split(',')
            .map(|protocol| JsonValue::String(protocol.trim().to_string()))
            .collect();
        if !alpn_array.is_empty() {
            proxy["alpn"] = JsonValue::Array(alpn_array);
        }
    } else {
        // Default ALPN
        proxy["alpn"] = JsonValue::Array(vec![JsonValue::String("h3".to_string())]);
    }

    proxy
}

fn handle_hysteria2(node: &Proxy, remark: &str, scv: &Option<bool>) -> JsonValue {
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
        proxy["sni"] = json!(server_name);
    }

    // Add skip-cert-verify if defined
    if let Some(obj) = proxy.as_object_mut() {
        scv.apply_to_json(obj, "skip-cert-verify");
    }

    // Add bandwidth settings
    if let Some(up_mbps) = &node.quic_secret {
        if let Ok(up_mbps_value) = up_mbps.parse::<u32>() {
            proxy["up"] = json!(up_mbps_value);
        }
    }

    if let Some(down_mbps) = &node.quic_secure {
        if let Ok(down_mbps_value) = down_mbps.parse::<u32>() {
            proxy["down"] = json!(down_mbps_value);
        }
    }

    // Add obfs settings
    if let Some(obfs) = &node.obfs {
        proxy["obfs"] = json!(obfs);
    }
    if let Some(obfs_param) = &node.obfs_param {
        proxy["obfs-password"] = json!(obfs_param);
    }

    // Add other fields
    if let Some(fingerprint) = &node.fingerprint {
        proxy["fingerprint"] = json!(fingerprint);
    }
    if let Some(ca) = &node.ca {
        proxy["ca"] = json!(ca);
    }
    if let Some(ca_str) = &node.ca_str {
        proxy["ca-str"] = json!(ca_str);
    }
    if node.cwnd > 0 {
        proxy["cwnd"] = json!(node.cwnd);
    }

    // Add ALPN protocols if specified
    if let Some(alpn) = &node.edge {
        // In Hysteria2, edge might be repurposed to store ALPN
        let alpn_array: Vec<JsonValue> = alpn
            .split(',')
            .map(|protocol| JsonValue::String(protocol.trim().to_string()))
            .collect();
        if !alpn_array.is_empty() {
            proxy["alpn"] = JsonValue::Array(alpn_array);
        }
    } else {
        // Default ALPN for Hysteria2
        proxy["alpn"] = JsonValue::Array(vec![JsonValue::String("h3".to_string())]);
    }

    proxy
}

/// Helper function to apply common optional fields to a proxy
fn apply_proxy_fields(proxy: &mut Map<String, JsonValue>, node: &Proxy, scv: &Option<bool>) {
    // Apply common optional fields
    if let Some(obfs) = &node.obfs {
        if !obfs.is_empty() {
            proxy.insert("obfs".to_string(), JsonValue::String(obfs.clone()));
        }
    }

    // For TLS-based protocols
    if node.tls_secure {
        proxy.insert("tls".to_string(), JsonValue::Bool(true));
        if let Some(server_name) = &node.server_name {
            if !server_name.is_empty() {
                proxy.insert("sni".to_string(), JsonValue::String(server_name.clone()));
            }
        }
    }

    // Apply cert verification if defined
    scv.apply_to_json(proxy, "skip-cert-verify");
}
