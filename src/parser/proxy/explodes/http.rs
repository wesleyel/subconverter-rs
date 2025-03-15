use crate::parser::proxy::{Proxy, ProxyType};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::collections::HashMap;
use url::Url;

/// Parse an HTTP/HTTPS link into a Proxy object
pub fn explode_http(http: &str, node: &mut Proxy) -> bool {
    // Check if the link starts with http:// or https://
    if !http.starts_with("http://") && !http.starts_with("https://") {
        return false;
    }

    // Try to parse as URL
    let url = match Url::parse(http) {
        Ok(url) => url,
        Err(_) => return false,
    };

    // Extract host and port
    let host = match url.host_str() {
        Some(host) => host,
        None => return false,
    };
    let port = url.port().unwrap_or(if http.starts_with("https://") {
        443
    } else {
        80
    });

    // Extract username and password
    let username = url.username();
    let password = url.password().unwrap_or("");

    // Determine if TLS is enabled
    let tls = http.starts_with("https://");

    // Extract parameters from the query string
    let mut params = HashMap::new();
    for (key, value) in url.query_pairs() {
        params.insert(key.to_string(), value.to_string());
    }

    // Extract TLS verification setting
    let skip_cert_verify = params
        .get("skip-cert-verify")
        .map(|s| s == "1" || s.to_lowercase() == "true")
        .unwrap_or(false);

    // Extract TLS13 setting
    let tls13 = params
        .get("tls13")
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
    *node = Proxy::http_construct(
        "HTTP",
        &formatted_remark,
        host,
        port,
        username,
        password,
        tls,
        None,
        Some(skip_cert_verify),
        if tls13 { Some(true) } else { None },
        "",
    );

    true
}

/// Parse an HTTP subscription link into a Proxy object
pub fn explode_http_sub(http: &str, node: &mut Proxy) -> bool {
    // Format: http(s)=username:password@server:port#remark

    // Check if the link starts with http= or https=
    let is_https = http.starts_with("https=");
    if !http.starts_with("http=") && !is_https {
        return false;
    }

    // Extract the main part
    let main_part = if is_https { &http[6..] } else { &http[5..] };

    // Split by # to extract remark
    let parts: Vec<&str> = main_part.split('#').collect();
    let main_part = parts[0];
    let remark = if parts.len() > 1 { parts[1] } else { "" };

    // Split by @ to extract username:password and server:port
    let parts: Vec<&str> = main_part.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let user_pass = parts[0];
    let server_port = parts[1];

    // Split username:password
    let user_pass_parts: Vec<&str> = user_pass.split(':').collect();
    let username = if user_pass_parts.is_empty() {
        ""
    } else {
        user_pass_parts[0]
    };
    let password = if user_pass_parts.len() > 1 {
        user_pass_parts[1]
    } else {
        ""
    };

    // Split server:port
    let server_port_parts: Vec<&str> = server_port.split(':').collect();
    if server_port_parts.is_empty() {
        return false;
    }

    let server = server_port_parts[0];
    let port = if server_port_parts.len() > 1 {
        server_port_parts[1]
            .parse::<u16>()
            .unwrap_or(if is_https { 443 } else { 80 })
    } else {
        if is_https {
            443
        } else {
            80
        }
    };

    // Create formatted remark string if needed
    let formatted_remark = if remark.is_empty() {
        format!("{} ({})", server, port)
    } else {
        remark.to_string()
    };

    // Create the proxy object with proper type conversions
    *node = Proxy::http_construct(
        if is_https { "HTTPS" } else { "HTTP" },
        &formatted_remark,
        server,
        port,
        username,
        password,
        is_https,
        None,
        None,
        None,
        "",
    );

    true
}
