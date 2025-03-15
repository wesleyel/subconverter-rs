use crate::generator::config::subexport::{process_remark, ExtraSettings};
use crate::parser::proxy::{Proxy, ProxyType};
use base64::{engine::general_purpose, Engine as _};
use serde_json::{self, json, Value};
use std::collections::HashMap;

/// Convert proxies to SSD format
///
/// This function converts a list of proxies to the SSD configuration format.
///
/// # Arguments
/// * `nodes` - List of proxy nodes to convert
/// * `group_name` - Name of the group
/// * `ext` - Extra settings for conversion
pub fn proxy_to_ssd(nodes: &mut Vec<Proxy>, group_name: &str, ext: &mut ExtraSettings) -> String {
    // Create SSD JSON structure
    let mut ssd_json = json!({
        "airport": group_name,
        "port": 443,
        "encryption": "aes-128-gcm",
        "password": "password",
        "servers": []
    });

    // Process servers
    let mut servers = Vec::new();

    for node in nodes.iter_mut() {
        // Skip non-SS nodes
        if node.proxy_type != ProxyType::Shadowsocks {
            continue;
        }

        // Process remark
        let mut remark = node.remark.clone();
        process_remark(&mut remark, ext, false);

        // Create server object
        let mut server = json!({
            "server": node.server,
            "port": node.port,
            "encryption": node.cipher,
            "password": node.password,
            "remarks": remark
        });

        // Add plugin if present
        if !node.plugin.is_empty() && !node.plugin_opts.is_empty() {
            server["plugin"] = json!(node.plugin);
            server["plugin_options"] = json!(node.plugin_opts);
        }

        // Add common fields
        if let Some(udp) = node.udp {
            server["udp"] = json!(udp);
        }

        servers.push(server);
    }

    // Add servers to SSD JSON
    ssd_json["servers"] = json!(servers);

    // Convert to string and base64 encode
    if let Ok(json_str) = serde_json::to_string(&ssd_json) {
        format!("ssd://{}", general_purpose::STANDARD.encode(json_str))
    } else {
        String::new()
    }
}
