use crate::generator::config::group::group_generate;
use crate::generator::config::remark::process_remark;
use crate::generator::ruleconvert::ruleset_to_clash_str;
use crate::generator::yaml::clash::{ClashProxy, ClashProxyCommon, CommonProxyOptions};
use crate::models::{ExtraSettings, ProxyGroupConfigs, ProxyGroupType};
use crate::models::{Proxy, ProxyType, RulesetContent};
use crate::utils::base64::base64_encode;
use crate::utils::replace_all_distinct;
use crate::utils::tribool::{OptionSetExt, TriboolExt};
use crate::utils::url::get_url_arg;
use log::error;
use serde::{Deserialize, Serialize};
use serde_yml::{self, Mapping, Sequence, Value as YamlValue};
use std::collections::HashMap;
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
    let mut yaml_node: YamlValue = match serde_yml::from_str(base_conf) {
        Ok(node) => node,
        Err(e) => {
            error!("Clash base loader failed with error: {}", e);
            return String::new();
        }
    };

    if yaml_node.is_null() {
        yaml_node = YamlValue::Mapping(Mapping::new());
    }

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
        return match serde_yml::to_string(&yaml_node) {
            Ok(result) => result,
            Err(_) => String::new(),
        };
    }

    // Handle rule generation if enabled
    if !ext.enable_rule_generator {
        return match serde_yml::to_string(&yaml_node) {
            Ok(result) => result,
            Err(_) => String::new(),
        };
    }

    // Handle managed config and clash script
    if !ext.managed_config_prefix.is_empty() || ext.clash_script {
        // Set mode if it exists
        if yaml_node.get("mode").is_some() {
            if let Some(ref mut map) = yaml_node.as_mapping_mut() {
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
        return match serde_yml::to_string(&yaml_node) {
            Ok(result) => result,
            Err(_) => String::new(),
        };
    }

    // Generate rules and return combined output
    let rules_str = ruleset_to_clash_str(
        &yaml_node,
        ruleset_content_array,
        ext.overwrite_original_rules,
        ext.clash_new_field_name,
    );

    let yaml_output = match serde_yml::to_string(&yaml_node) {
        Ok(result) => result,
        Err(_) => String::new(),
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
    yaml_node: &mut serde_yml::Value,
    _ruleset_content_array: &Vec<RulesetContent>,
    extra_proxy_group: &ProxyGroupConfigs,
    clash_r: bool,
    ext: &mut ExtraSettings,
) {
    // Style settings - in C++ this is used to set serialization style but in Rust we have less control
    // over the serialization format. We keep them for compatibility but their actual effect may differ.
    let _proxy_block = ext.clash_proxies_style == "block";
    let _proxy_compact = ext.clash_proxies_style == "compact";
    let _group_block = ext.clash_proxy_groups_style == "block";
    let _group_compact = ext.clash_proxy_groups_style == "compact";

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
        let mut clash_proxy = match node.proxy_type {
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
        if let Some(ref mut obj) = clash_proxy {
            obj.common_mut().udp.set_if_some(udp.clone());
            obj.common_mut().tfo.set_if_some(tfo.clone());
            obj.common_mut().skip_cert_verify.set_if_some(scv.clone());
        }

        // Add to proxies array
        proxies_json.push(clash_proxy);
    }

    if ext.nodelist {
        let mut provider = YamlValue::Mapping(Mapping::new());
        provider["proxies"] =
            serde_yml::to_value(&proxies_json).unwrap_or(YamlValue::Sequence(Vec::new()));
        *yaml_node = provider;
        return;
    }

    // Update the YAML node with proxies
    if let Some(ref mut map) = yaml_node.as_mapping_mut() {
        // Convert JSON proxies array to YAML
        let proxies_yaml_value =
            serde_yml::to_value(&proxies_json).unwrap_or(YamlValue::Sequence(Vec::new()));
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
            match yaml_node.get("proxy-groups") {
                Some(YamlValue::Sequence(seq)) => seq.clone(),
                _ => Sequence::new(),
            }
        } else {
            match yaml_node.get("Proxy Group") {
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
        if let Some(ref mut map) = yaml_node.as_mapping_mut() {
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

fn build_common_proxy_options(
    node: &Proxy,
    remark: &str,
    udp: &Option<bool>,
    tfo: &Option<bool>,
    scv: &Option<bool>,
) -> CommonProxyOptions {
    let mut common_builder =
        CommonProxyOptions::builder(remark.to_string(), node.hostname.clone(), node.port);

    if let Some(udp) = udp {
        common_builder = common_builder.udp(*udp);
    }
    if let Some(tfo) = tfo {
        common_builder = common_builder.tfo(*tfo);
    }

    // Add skip-cert-verify if defined
    if let Some(skip_cert_verify) = scv {
        common_builder = common_builder.skip_cert_verify(*skip_cert_verify);
    }
    common_builder.build()
}

// Helper functions for each proxy type
fn handle_shadowsocks(
    node: &Proxy,
    remark: &str,
    scv: &Option<bool>,
    ext: &ExtraSettings,
) -> Option<ClashProxy> {
    // Skip chacha20 encryption if filter_deprecated is enabled
    if ext.filter_deprecated && node.encrypt_method.as_deref() == Some("chacha20") {
        return None;
    }

    let plugin_options =
        replace_all_distinct(node.plugin_option.as_deref().unwrap_or(""), &";", &"&");

    let mut proxy =
        ClashProxy::new_shadowsocks(build_common_proxy_options(node, remark, &None, &None, scv));

    if let ClashProxy::Shadowsocks {
        plugin,
        plugin_opts,
        cipher,
        password,
        ..
    } = &mut proxy
    {
        *cipher = node.encrypt_method.clone().unwrap_or("".to_string());
        *password = node.password.clone().unwrap_or("".to_string());

        let mut opts = HashMap::new();
        match node.plugin.as_deref() {
            Some("simple-obfs" | "obfs-local") => {
                *plugin = Some("obfs".to_string());

                let obfs_mode = get_url_arg(&plugin_options, "obfs");
                let obfs_host = get_url_arg(&plugin_options, "obfs-host");

                opts.insert("mode".to_string(), serde_yml::Value::String(obfs_mode));
                if !obfs_host.is_empty() {
                    opts.insert("host".to_string(), serde_yml::Value::String(obfs_host));
                }
                *plugin_opts = Some(opts);
            }
            Some("v2ray-plugin") => {
                *plugin = Some("v2ray-plugin".to_string());

                let mode = get_url_arg(&plugin_options, "mode");
                if !mode.is_empty() {
                    opts.insert("mode".to_string(), serde_yml::Value::String(mode));
                }

                let host = get_url_arg(&plugin_options, "host");
                if !host.is_empty() {
                    opts.insert("host".to_string(), serde_yml::Value::String(host));
                }

                let path = get_url_arg(&plugin_options, "path");
                if !path.is_empty() {
                    opts.insert("path".to_string(), serde_yml::Value::String(path));
                }

                if plugin_options.contains("tls") {
                    opts.insert("tls".to_string(), serde_yml::Value::Bool(true));
                }

                if plugin_options.contains("mux") {
                    opts.insert("mux".to_string(), serde_yml::Value::Bool(true));
                }

                if let Some(skip_cert_verify) = scv {
                    opts.insert(
                        "skip-cert-verify".to_string(),
                        serde_yml::Value::Bool(*skip_cert_verify),
                    );
                }
                *plugin_opts = Some(opts);
            }
            _ => {}
        }
    }

    Some(proxy)
}

fn handle_shadowsocksr(
    node: &Proxy,
    remark: &str,
    scv: &Option<bool>,
    clash_r: bool,
    ext: &ExtraSettings,
) -> Option<ClashProxy> {
    let mut proxy =
        ClashProxy::new_shadowsocksr(build_common_proxy_options(node, remark, &None, &None, scv));
    // Skip if not using ClashR or if using deprecated features
    if ext.filter_deprecated {
        if !clash_r {
            return None;
        }

        let encrypt_method = node.encrypt_method.as_deref().unwrap_or("");
        if !CLASH_SSR_CIPHERS.contains(encrypt_method) {
            return None;
        }

        let protocol = node.protocol.as_deref().unwrap_or("");
        if !CLASHR_PROTOCOLS.contains(protocol) {
            return None;
        }

        let obfs = node.obfs.as_deref().unwrap_or("");
        if !CLASHR_OBFS.contains(obfs) {
            return None;
        }
    }

    if let ClashProxy::ShadowsocksR {
        cipher,
        password,
        protocol,
        obfs,
        protocol_param,
        obfs_param,
        protocolparam,
        obfsparam,
        ..
    } = &mut proxy
    {
        *cipher = match node.encrypt_method.as_deref() {
            None => "dummy".to_string(),
            Some("none") => "dummy".to_string(),
            Some(encrypt_method) => encrypt_method.to_string(),
        };
        *password = node.password.as_deref().unwrap_or("").to_string();
        *protocol = node.protocol.as_deref().unwrap_or("").to_string();
        *obfs = node.obfs.as_deref().unwrap_or("").to_string();

        if clash_r {
            *protocolparam = Some(node.protocol_param.as_deref().unwrap_or("").to_string());
            *obfsparam = Some(node.obfs_param.as_deref().unwrap_or("").to_string());
        } else {
            *protocol_param = Some(node.protocol_param.as_deref().unwrap_or("").to_string());
            *obfs_param = Some(node.obfs_param.as_deref().unwrap_or("").to_string());
        }
    }

    Some(proxy)
}

fn handle_vmess(
    node: &Proxy,
    remark: &str,
    scv: &Option<bool>,
    _ext: &ExtraSettings,
) -> Option<ClashProxy> {
    let mut proxy =
        ClashProxy::new_vmess(build_common_proxy_options(node, remark, &None, &None, scv));

    proxy.common_mut().tls = Some(node.tls_secure);
    proxy.common_mut().skip_cert_verify.set_if_some(scv.clone());

    if let ClashProxy::VMess {
        uuid,
        alter_id,
        cipher,
        network,
        ws_opts,
        ws_path,
        ws_headers,
        http_opts,
        h2_opts,
        grpc_opts,
        servername,
        ..
    } = &mut proxy
    {
        *servername = node.server_name.clone();
        *uuid = node.user_id.as_deref().unwrap_or("").to_string();
        *alter_id = node.alter_id as u32;
        *cipher = node.encrypt_method.as_deref().unwrap_or("").to_string();

        match node.transfer_protocol.as_deref() {
            Some("tcp") => {}
            Some("ws") => {
                *network = Some("ws".to_string());
                let mut opts = HashMap::new();
                if let Some(path) = &node.path {
                    opts.insert("path".to_string(), serde_yml::Value::String(path.clone()));
                    *ws_path = Some(path.clone());
                }
                let mut headers = serde_yml::mapping::Mapping::new();
                if let Some(host) = &node.host {
                    headers.insert(
                        serde_yml::Value::String("Host".to_string()),
                        serde_yml::Value::String(host.clone()),
                    );
                }
                if let Some(edge) = &node.edge {
                    headers.insert(
                        serde_yml::Value::String("Edge".to_string()),
                        serde_yml::Value::String(edge.clone()),
                    );
                }
                if !headers.is_empty() {
                    opts.insert(
                        "headers".to_string(),
                        serde_yml::Value::Mapping(headers.clone()),
                    );
                    *ws_headers = Some(serde_yml::Value::Mapping(headers));
                }
                *ws_opts = Some(opts);
            }
            Some("http") => {
                *network = Some("http".to_string());
                let mut opts = HashMap::new();
                opts.insert(
                    "method".to_string(),
                    serde_yml::Value::String("GET".to_string()),
                );
                if let Some(path) = &node.path {
                    opts.insert("path".to_string(), serde_yml::Value::String(path.clone()));
                }
                let mut headers = serde_yml::mapping::Mapping::new();
                if let Some(host) = &node.host {
                    headers.insert(
                        serde_yml::Value::String("Host".to_string()),
                        serde_yml::Value::String(host.clone()),
                    );
                }
                if let Some(edge) = &node.edge {
                    headers.insert(
                        serde_yml::Value::String("Edge".to_string()),
                        serde_yml::Value::String(edge.clone()),
                    );
                }
                opts.insert("headers".to_string(), serde_yml::Value::Mapping(headers));
                *http_opts = Some(opts);
            }
            Some("h2") => {
                *network = Some("h2".to_string());
                let mut opts = HashMap::new();
                if let Some(path) = &node.path {
                    opts.insert("path".to_string(), serde_yml::Value::String(path.clone()));
                }
                if let Some(host) = &node.host {
                    opts.insert("host".to_string(), serde_yml::Value::String(host.clone()));
                }

                *h2_opts = Some(opts);
            }
            Some("grpc") => {
                *network = Some("grpc".to_string());
                *servername = node.host.clone();
                let mut opts = HashMap::new();
                if let Some(path) = &node.path {
                    opts.insert(
                        "grpc-service-name".to_string(),
                        serde_yml::Value::String(path.clone()),
                    );
                }
                *grpc_opts = Some(opts);
            }
            _ => {}
        }
    }

    Some(proxy)
}

fn handle_trojan(node: &Proxy, remark: &str, scv: &Option<bool>) -> Option<ClashProxy> {
    let mut proxy =
        ClashProxy::new_trojan(build_common_proxy_options(node, remark, &None, &None, scv));

    proxy.common_mut().sni.set_if_some(node.host.clone());
    proxy.common_mut().skip_cert_verify.set_if_some(scv.clone());

    if let ClashProxy::Trojan {
        password,
        network,
        ws_opts,
        grpc_opts,
        ..
    } = &mut proxy
    {
        *password = node.password.as_deref().unwrap_or("").to_string();
        match node.transfer_protocol.as_deref() {
            Some("tcp") => {}
            Some("grpc") => {
                *network = Some("grpc".to_string());
                if let Some(path) = &node.path {
                    *grpc_opts = Some(HashMap::from([(
                        "grpc-service-name".to_string(),
                        serde_yml::Value::String(path.clone()),
                    )]));
                }
            }
            Some("ws") => {
                *network = Some("ws".to_string());
                #[derive(Debug, Default, Serialize, Deserialize)]
                #[serde()]
                struct WsOpts {
                    #[serde(skip_serializing_if = "Option::is_none")]
                    path: Option<String>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    host: Option<String>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    headers: Option<HashMap<String, String>>,
                }
                let mut opts = WsOpts::default();
                opts.path = node.path.clone();
                if let Some(host) = &node.host {
                    opts.headers = Some(HashMap::from([("Host".to_string(), host.clone())]));
                }
                *ws_opts = Some(serde_yml::to_value(opts).unwrap());
            }
            _ => {}
        }
    }

    Some(proxy)
}

fn handle_http(node: &Proxy, remark: &str, scv: &Option<bool>) -> Option<ClashProxy> {
    let mut proxy =
        ClashProxy::new_http(build_common_proxy_options(node, remark, &None, &None, scv));
    if let ClashProxy::Http {
        username, password, ..
    } = &mut proxy
    {
        *username = Some(node.username.as_deref().unwrap_or("").to_string());
        *password = Some(node.password.as_deref().unwrap_or("").to_string());
    }

    // Set TLS for HTTPS
    if node.proxy_type == ProxyType::HTTPS {
        proxy.common_mut().tls = Some(true);
    }

    proxy.common_mut().skip_cert_verify.set_if_some(scv.clone());

    Some(proxy)
}

fn handle_socks5(node: &Proxy, remark: &str, scv: &Option<bool>) -> Option<ClashProxy> {
    let mut proxy =
        ClashProxy::new_socks5(build_common_proxy_options(node, remark, &None, &None, scv));

    if let ClashProxy::Socks5 {
        username, password, ..
    } = &mut proxy
    {
        *username = node.username.clone();
        *password = node.password.clone();
    }

    proxy.common_mut().skip_cert_verify.set_if_some(scv.clone());
    Some(proxy)
}

fn handle_snell(node: &Proxy, remark: &str) -> Option<ClashProxy> {
    // Skip Snell v4+ if exists - exactly matching C++ behavior
    if node.snell_version >= 4 {
        return None;
    }

    let mut proxy = ClashProxy::new_snell(build_common_proxy_options(
        node, remark, &None, &None, &None,
    ));

    if let ClashProxy::Snell {
        psk,
        version,

        obfs_opts,
        ..
    } = &mut proxy
    {
        *psk = node.password.as_deref().unwrap_or("").to_string();
        *version = Some(node.snell_version as u32);

        let mut opts = HashMap::new();

        if let Some(obfs) = &node.obfs {
            opts.insert("mode".to_string(), serde_yml::Value::String(obfs.clone()));
        }
        if let Some(obfs_host) = &node.host {
            opts.insert(
                "host".to_string(),
                serde_yml::Value::String(obfs_host.clone()),
            );
        }
        *obfs_opts = Some(opts);
    }

    Some(proxy)
}

fn handle_wireguard(node: &Proxy, remark: &str) -> Option<ClashProxy> {
    let mut proxy = ClashProxy::new_wireguard(build_common_proxy_options(
        node, remark, &None, &None, &None,
    ));

    if let ClashProxy::WireGuard {
        public_key,
        private_key,
        ip,
        ipv6,
        preshared_key,
        dns,
        mtu,
        allowed_ips,
        keepalive,
        ..
    } = &mut proxy
    {
        *public_key = node.public_key.as_deref().unwrap_or("").to_string();
        *private_key = node.private_key.as_deref().unwrap_or("").to_string();
        *ip = node.self_ip.as_deref().unwrap_or("").to_string();
        *ipv6 = node.self_ipv6.clone();
        *preshared_key = node.pre_shared_key.clone();
        if !node.dns_servers.is_empty() {
            *dns = Some(node.dns_servers.iter().map(|s| s.clone()).collect());
        }
        if node.mtu > 0 {
            *mtu = Some(node.mtu as u32);
        }
        if !node.allowed_ips.is_empty() {
            *allowed_ips = node
                .allowed_ips
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }
        if node.keep_alive > 0 {
            *keepalive = Some(node.keep_alive as u32);
        }
    }
    Some(proxy)
}

fn handle_hysteria(node: &Proxy, remark: &str, scv: &Option<bool>) -> Option<ClashProxy> {
    let mut proxy =
        ClashProxy::new_hysteria(build_common_proxy_options(node, remark, &None, &None, scv));

    if let ClashProxy::Hysteria {
        ports,
        protocol,
        obfs_protocol,
        up_speed,
        down_speed,
        auth,
        auth_str,
        obfs,
        fingerprint,
        alpn,
        ca,
        ca_str,
        recv_window_conn,
        recv_window,
        disable_mtu_discovery,
        hop_interval,
        ..
    } = &mut proxy
    {
        *ports = node.ports.clone();
        *protocol = node.protocol.clone();
        *obfs_protocol = node.obfs.clone();
        *up_speed = Some(node.up_speed);
        *down_speed = Some(node.down_speed);
        if let Some(auth_str) = &node.auth_str {
            *auth = Some(base64_encode(&auth_str));
        }
        *auth_str = node.auth_str.clone();
        *obfs = node.obfs.clone();
        *fingerprint = node.fingerprint.clone();
        *alpn = Some(node.alpn.iter().map(|s| s.clone()).collect());
        *ca = node.ca.clone();
        *ca_str = node.ca_str.clone();
        if node.recv_window_conn > 0 {
            *recv_window_conn = Some(node.recv_window_conn);
        }
        if node.recv_window > 0 {
            *recv_window = Some(node.recv_window);
        }
        *disable_mtu_discovery = node.disable_mtu_discovery;

        if node.hop_interval > 0 {
            *hop_interval = Some(node.hop_interval);
        }
    }

    proxy.common_mut().sni.set_if_some(node.sni.clone());
    proxy.common_mut().skip_cert_verify.set_if_some(scv.clone());
    proxy
        .common_mut()
        .tfo
        .set_if_some(node.tcp_fast_open.clone());
    Some(proxy)
}

fn handle_hysteria2(node: &Proxy, remark: &str, scv: &Option<bool>) -> Option<ClashProxy> {
    let mut proxy =
        ClashProxy::new_hysteria2(build_common_proxy_options(node, remark, &None, &None, scv));

    if let ClashProxy::Hysteria2 {
        ports,
        hop_interval,
        up,
        down,
        password,
        obfs,
        obfs_password,
        fingerprint,
        alpn,
        ca,
        ca_str,
        cwnd,
        ..
    } = &mut proxy
    {
        *ports = node.ports.clone();
        if node.up_speed > 0 {
            *up = Some(format!("{}Mbps", node.up_speed));
        }
        if node.down_speed > 0 {
            *down = Some(format!("{}Mbps", node.down_speed));
        }
        *password = node.password.clone();
        *obfs = node.obfs.clone();
        *obfs_password = node.obfs_param.clone();
        *fingerprint = node.fingerprint.clone();
        *alpn = Some(node.alpn.iter().map(|s| s.clone()).collect());
        *ca = node.ca.clone();
        *ca_str = node.ca_str.clone();
        *cwnd = Some(node.cwnd);

        if node.hop_interval > 0 {
            *hop_interval = Some(node.hop_interval);
        }
    }

    proxy.common_mut().sni.set_if_some(node.sni.clone());
    proxy.common_mut().skip_cert_verify.set_if_some(scv.clone());
    proxy.common_mut().tfo.set_if_some(node.tcp_fast_open);

    Some(proxy)
}
