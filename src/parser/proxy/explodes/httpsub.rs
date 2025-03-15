use crate::parser::proxy::{Proxy, ProxyType, HTTP_DEFAULT_GROUP};
use url::Url;

/// Parse an HTTP subscription link into a Proxy object
pub fn explode_http_sub(link: &str, node: &mut Proxy) -> bool {
    // Parse URL
    let url = match Url::parse(link) {
        Ok(u) => u,
        Err(_) => return false,
    };

    // Determine if it's HTTP or HTTPS
    let is_https = url.scheme() == "https";

    // Extract hostname and port
    let hostname = match url.host_str() {
        Some(h) => h,
        None => return false,
    };

    let port = url.port().unwrap_or(if is_https { 443 } else { 80 });

    // Extract username and password
    let username = url.username();
    let password = url.password().unwrap_or("");

    // Create remark
    let mut remark = format!("{} ({})", hostname, port);
    if let Some(fragment) = url.fragment() {
        if !fragment.is_empty() {
            remark = fragment.to_string();
        }
    }

    // Create Proxy object
    *node = Proxy::http_construct(
        HTTP_DEFAULT_GROUP,
        &remark,
        hostname,
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
