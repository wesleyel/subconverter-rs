use crate::parser::proxy::{Proxy, ProxyType, SSR_DEFAULT_GROUP, SS_DEFAULT_GROUP};
use base64;
use regex::Regex;
use url::Url;

/// Parse a SSTap JSON configuration into a vector of Proxy objects
pub fn explode_sstap(content: &str, nodes: &mut Vec<Proxy>) -> bool {
    // Check if the content is in the right format
    if !content.contains("\"configs\":") && !content.contains("\"servers\":") {
        return false;
    }

    // Use regex to extract server configurations
    let mut success = false;

    // Pattern for finding SS configurations
    let ss_pattern = Regex::new(r#"\{"server":"([^"]+)","server_port":(\d+),"password":"([^"]+)","method":"([^"]+)"(?:,"plugin":"([^"]+)")?(?:,"plugin_opts":"([^"]+)")?\}"#).unwrap();

    // Pattern for finding SSR configurations
    let ssr_pattern = Regex::new(r#"\{"server":"([^"]+)","server_port":(\d+),"protocol":"([^"]+)","method":"([^"]+)","obfs":"([^"]+)","password":"([^"]+)"(?:,"obfsparam":"([^"]*)")?(?:,"protoparam":"([^"]*)")?\}"#).unwrap();

    // Extract SS configurations
    for cap in ss_pattern.captures_iter(content) {
        let server = cap.get(1).map_or("", |m| m.as_str());
        let port = cap
            .get(2)
            .map_or("0", |m| m.as_str())
            .parse::<u16>()
            .unwrap_or(0);
        let password = cap.get(3).map_or("", |m| m.as_str());
        let method = cap.get(4).map_or("", |m| m.as_str());
        let plugin = cap.get(5).map_or("", |m| m.as_str());
        let plugin_opts = cap.get(6).map_or("", |m| m.as_str());

        // Skip invalid configurations
        if server.is_empty() || port == 0 || password.is_empty() || method.is_empty() {
            continue;
        }

        // Create remark
        let remark = format!("{} ({})", server, port);

        // Create the proxy object
        let mut node = Proxy::ss_construct(
            SS_DEFAULT_GROUP,
            &remark,
            server,
            port,
            password,
            method,
            plugin,
            plugin_opts,
            None,
            None,
            None,
            None,
            "",
        );

        nodes.push(node);
        success = true;
    }

    // Extract SSR configurations
    for cap in ssr_pattern.captures_iter(content) {
        let server = cap.get(1).map_or("", |m| m.as_str());
        let port = cap
            .get(2)
            .map_or("0", |m| m.as_str())
            .parse::<u16>()
            .unwrap_or(0);
        let protocol = cap.get(3).map_or("", |m| m.as_str());
        let method = cap.get(4).map_or("", |m| m.as_str());
        let obfs = cap.get(5).map_or("", |m| m.as_str());
        let password = cap.get(6).map_or("", |m| m.as_str());
        let obfs_param = cap.get(7).map_or("", |m| m.as_str());
        let proto_param = cap.get(8).map_or("", |m| m.as_str());

        // Skip invalid configurations
        if server.is_empty() || port == 0 || password.is_empty() || method.is_empty() {
            continue;
        }

        // Create remark
        let remark = format!("{} ({})", server, port);

        // Create the proxy object
        let mut node = Proxy::ssr_construct(
            SSR_DEFAULT_GROUP,
            &remark,
            server,
            port,
            protocol,
            method,
            obfs,
            password,
            obfs_param,
            proto_param,
            None,
            None,
            None,
            "",
        );

        nodes.push(node);
        success = true;
    }

    success
}
