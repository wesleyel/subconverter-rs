use crate::parser::proxy::{Proxy, ProxyType};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::collections::HashMap;
use url::Url;

/// Parse a Hysteria link into a Proxy object
pub fn explode_hysteria(hysteria: &str, node: &mut Proxy) -> bool {
    // Check if the link starts with hysteria://
    if !hysteria.starts_with("hysteria://") {
        return false;
    }

    // Try to parse as URL
    let url = match Url::parse(hysteria) {
        Ok(url) => url,
        Err(_) => return false,
    };

    // Extract host and port
    let host = match url.host_str() {
        Some(host) => host,
        None => return false,
    };
    let port = url.port().unwrap_or(443);

    // Extract parameters from the query string
    let mut params = HashMap::new();
    for (key, value) in url.query_pairs() {
        params.insert(key.to_string(), value.to_string());
    }

    // Extract auth string
    let auth = params.get("auth").map(|s| s.as_str()).unwrap_or("");

    // Extract protocol
    let protocol = params.get("protocol").map(|s| s.as_str()).unwrap_or("udp");

    // Extract up and down speeds
    let up = params.get("up").map(|s| s.as_str()).unwrap_or("10");
    let down = params.get("down").map(|s| s.as_str()).unwrap_or("50");
    let up_speed = up.parse::<u32>().unwrap_or(10);
    let down_speed = down.parse::<u32>().unwrap_or(50);

    // Extract ALPN
    let alpn_str = params.get("alpn").map(|s| s.as_str()).unwrap_or("");
    let mut alpn = std::collections::HashSet::new();
    if !alpn_str.is_empty() {
        for a in alpn_str.split(',') {
            alpn.insert(a.trim().to_string());
        }
    }

    // Extract obfs
    let obfs = params.get("obfs").map(|s| s.as_str()).unwrap_or("");

    // Extract SNI
    let sni = params.get("peer").map(|s| s.as_str()).unwrap_or(host);

    // Extract insecure
    let insecure = params
        .get("insecure")
        .map(|s| s == "1" || s.to_lowercase() == "true")
        .unwrap_or(false);

    // Extract remark from the fragment
    let remark = url.fragment().unwrap_or("");
    let formatted_remark = if remark.is_empty() {
        format!("{} ({})", host, port)
    } else {
        remark.to_string()
    };

    // Create the proxy object
    *node = Proxy {
        proxy_type: ProxyType::Hysteria,
        group: "Hysteria".to_string(),
        remark: formatted_remark,
        hostname: host.to_string(),
        port,
        auth_str: Some(auth.to_string()),
        sni: Some(sni.to_string()),
        allow_insecure: Some(insecure),
        alpn,
        up: Some(up.to_string()),
        up_speed,
        down: Some(down.to_string()),
        down_speed,
        obfs: Some(obfs.to_string()),
        protocol: Some(protocol.to_string()),
        ..Default::default()
    };

    true
}

/// Parse a Hysteria2 link into a Proxy object
pub fn explode_hysteria2(hysteria2: &str, node: &mut Proxy) -> bool {
    // Check if the link starts with hysteria2://
    if !hysteria2.starts_with("hysteria2://") {
        return false;
    }

    // Try to parse as URL
    let url = match Url::parse(hysteria2) {
        Ok(url) => url,
        Err(_) => return false,
    };

    // Extract password
    let password = url.username();

    // Extract host and port
    let host = match url.host_str() {
        Some(host) => host,
        None => return false,
    };
    let port = url.port().unwrap_or(443);

    // Extract parameters from the query string
    let mut params = HashMap::new();
    for (key, value) in url.query_pairs() {
        params.insert(key.to_string(), value.to_string());
    }

    // Extract SNI
    let sni = params.get("sni").map(|s| s.as_str()).unwrap_or(host);

    // Extract insecure
    let insecure = params
        .get("insecure")
        .map(|s| s == "1" || s.to_lowercase() == "true")
        .unwrap_or(false);

    // Extract obfs
    let obfs = params.get("obfs").map(|s| s.as_str()).unwrap_or("");
    let obfs_password = params
        .get("obfs-password")
        .map(|s| s.as_str())
        .unwrap_or("");

    // Extract bandwidth
    let bandwidth = params.get("bandwidth").map(|s| s.as_str()).unwrap_or("");

    // Extract remark from the fragment
    let remark = url.fragment().unwrap_or("");
    let formatted_remark = if remark.is_empty() {
        format!("{} ({})", host, port)
    } else {
        remark.to_string()
    };

    // Create the proxy object
    *node = Proxy {
        proxy_type: ProxyType::Hysteria2,
        group: "Hysteria2".to_string(),
        remark: formatted_remark,
        hostname: host.to_string(),
        port,
        password: Some(password.to_string()),
        sni: Some(sni.to_string()),
        allow_insecure: Some(insecure),
        obfs: Some(obfs.to_string()),
        obfs_param: Some(obfs_password.to_string()),
        up: Some(bandwidth.to_string()),
        ..Default::default()
    };

    true
}
