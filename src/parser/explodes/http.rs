use crate::models::{Proxy, HTTP_DEFAULT_GROUP};
use url::Url;
use urlencoding::decode as url_decode;

/// Parse an HTTP/HTTPS link into a Proxy object
/// Matches C++ explodeHTTP implementation
pub fn explode_http(link: &str, node: &mut Proxy) -> bool {
    // Try to parse as URL if it has a scheme, otherwise add a dummy scheme
    let url_str = if link.contains("://") {
        link.to_string()
    } else {
        format!("http://_dummy_host_/?{}", link)
    };

    // Parse URL
    let url = match Url::parse(&url_str) {
        Ok(u) => u,
        Err(_) => return false,
    };

    // Extract query parameters
    let mut server = String::new();
    let mut port = String::new();
    let mut username = String::new();
    let mut password = String::new();
    let mut remarks = String::new();
    let mut group = String::new();

    // Parse query parameters
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "server" => server = value.to_string(),
            "port" => port = value.to_string(),
            "user" => {
                username = match url_decode(&value) {
                    Ok(s) => s.to_string(),
                    Err(_) => value.to_string(),
                }
            }
            "pass" => {
                password = match url_decode(&value) {
                    Ok(s) => s.to_string(),
                    Err(_) => value.to_string(),
                }
            }
            "remarks" => {
                remarks = match url_decode(&value) {
                    Ok(s) => s.to_string(),
                    Err(_) => value.to_string(),
                }
            }
            "group" => {
                group = match url_decode(&value) {
                    Ok(s) => s.to_string(),
                    Err(_) => value.to_string(),
                }
            }
            _ => {}
        }
    }

    // Use default group if none specified
    let group = if group.is_empty() {
        HTTP_DEFAULT_GROUP
    } else {
        &group
    };

    // Use server:port as remark if none specified
    let remarks = if remarks.is_empty() {
        format!("{}:{}", server, port)
    } else {
        remarks
    };

    // Skip invalid port
    if port == "0" {
        return false;
    }

    // Determine if TLS is enabled
    let is_https = link.contains("/https");

    // Parse port to u16
    let port_num = match port.parse::<u16>() {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Create the proxy object
    *node = Proxy::http_construct(
        group, &remarks, &server, port_num, &username, &password, is_https, None, None, None, "",
    );

    true
}
