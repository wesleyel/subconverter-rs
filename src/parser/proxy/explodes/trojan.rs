use crate::parser::proxy::Proxy;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::collections::HashMap;
use url::Url;

/// Parse a Trojan link into a Proxy object
pub fn explode_trojan(trojan: &str, node: &mut Proxy) -> bool {
    // Check if the link starts with trojan://
    if !trojan.starts_with("trojan://") {
        return false;
    }

    // Try to parse as URL
    let url = match Url::parse(trojan) {
        Ok(url) => url,
        Err(_) => return false,
    };

    // Extract password
    let password = url.username();
    if password.is_empty() {
        return false;
    }

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
    let sni = params.get("sni").map(|s| s.to_string());

    // Extract TLS verification setting
    let skip_cert_verify = params
        .get("allowInsecure")
        .map(|s| s == "1" || s.to_lowercase() == "true");

    // Extract ALPN
    let alpn = params.get("alpn").map(|s| s.to_string());

    // Extract remark from the fragment
    let remark = url.fragment().unwrap_or("");
    let formatted_remark = if remark.is_empty() {
        format!("{} ({})", host, port)
    } else {
        remark.to_string()
    };

    // Create the proxy object
    *node = Proxy::trojan_construct(
        "Trojan".to_string(),
        formatted_remark,
        host.to_string(),
        port,
        password.to_string(),
        None,             // network
        None,             // host
        None,             // path
        true,             // tls_secure
        None,             // udp
        None,             // tfo
        skip_cert_verify, // allow_insecure
        None,             // tls13
        None,             // underlying_proxy
    );

    true
}

/// Parse a Trojan-Go link into a Proxy object
pub fn explode_trojan_go(trojan_go: &str, node: &mut Proxy) -> bool {
    // Check if the link starts with trojan-go://
    if !trojan_go.starts_with("trojan-go://") {
        return false;
    }

    // Try to parse as URL
    let url = match Url::parse(trojan_go) {
        Ok(url) => url,
        Err(_) => return false,
    };

    // Extract password
    let password = url.username();
    if password.is_empty() {
        return false;
    }

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

    // Extract network, host, path
    let network = params.get("type").map(|s| s.to_string());
    let host_param = params.get("host").map(|s| s.to_string());
    let path = params.get("path").map(|s| s.to_string());

    // Extract TLS verification setting
    let skip_cert_verify = params
        .get("allowInsecure")
        .map(|s| s == "1" || s.to_lowercase() == "true");

    // Extract remark from the fragment
    let remark = url.fragment().unwrap_or("");
    let formatted_remark = if remark.is_empty() {
        format!("{} ({})", host, port)
    } else {
        remark.to_string()
    };

    // Create the proxy object
    *node = Proxy::trojan_construct(
        "Trojan".to_string(),
        formatted_remark,
        host.to_string(),
        port,
        password.to_string(),
        network,
        host_param,
        path,
        true,             // tls_secure
        None,             // udp
        None,             // tfo
        skip_cert_verify, // allow_insecure
        None,             // tls13
        None,             // underlying_proxy
    );

    true
}
