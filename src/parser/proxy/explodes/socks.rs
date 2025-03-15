use crate::parser::proxy::Proxy;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::collections::HashMap;
use url::Url;

/// Parse a SOCKS link into a Proxy object
pub fn explode_socks(socks: &str, node: &mut Proxy) -> bool {
    // Check if the link starts with socks:// or socks5://
    if !socks.starts_with("socks://") && !socks.starts_with("socks5://") {
        return false;
    }

    // Try to parse as URL
    let url = match Url::parse(socks) {
        Ok(url) => url,
        Err(_) => return false,
    };

    // Extract host and port
    let host = match url.host_str() {
        Some(host) => host,
        None => return false,
    };
    let port = url.port().unwrap_or(1080);

    // Extract username and password
    let username = url.username();
    let password = url.password().unwrap_or("");

    // Extract parameters from the query string
    let mut params = HashMap::new();
    for (key, value) in url.query_pairs() {
        params.insert(key.to_string(), value.to_string());
    }

    // Extract UDP relay setting
    let udp = params
        .get("udp")
        .map(|s| s == "1" || s.to_lowercase() == "true")
        .unwrap_or(false);

    // Extract TLS verification setting
    let skip_cert_verify = params
        .get("skip-cert-verify")
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
    *node = Proxy::socks_construct(
        "SOCKS",
        &formatted_remark,
        host,
        port,
        username,
        password,
        Some(udp),
        None,
        Some(skip_cert_verify),
        "",
    );

    true
}
