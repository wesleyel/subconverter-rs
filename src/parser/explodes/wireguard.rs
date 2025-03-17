use crate::{Proxy, ProxyType};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::collections::{HashMap, HashSet};
use url::Url;

/// Parse a WireGuard link into a Proxy object
pub fn explode_wireguard(wireguard: &str, node: &mut Proxy) -> bool {
    // Check if the link starts with wireguard://
    if !wireguard.starts_with("wireguard://") {
        return false;
    }

    // Try to parse as URL
    let url = match Url::parse(wireguard) {
        Ok(url) => url,
        Err(_) => return false,
    };

    // Extract parameters from the query string
    let mut params = HashMap::new();
    for (key, value) in url.query_pairs() {
        params.insert(key.to_string(), value.to_string());
    }

    // Extract required fields
    let private_key = match params.get("privateKey") {
        Some(key) => key,
        None => return false,
    };

    let public_key = match params.get("publicKey") {
        Some(key) => key,
        None => return false,
    };

    // Extract host and port
    let host = match url.host_str() {
        Some(host) => host,
        None => return false,
    };
    let port = url.port().unwrap_or(51820);

    // Extract optional fields
    let preshared_key = params.get("presharedKey").map(|s| s.as_str()).unwrap_or("");
    let self_ip = params
        .get("selfIP")
        .map(|s| s.as_str())
        .unwrap_or("10.0.0.2");
    let self_ipv6 = params.get("selfIPv6").map(|s| s.as_str()).unwrap_or("");
    let mtu = params
        .get("mtu")
        .map(|s| s.parse::<u16>().unwrap_or(1420))
        .unwrap_or(1420);
    let keep_alive = params
        .get("keepAlive")
        .map(|s| s.parse::<u16>().unwrap_or(25))
        .unwrap_or(25);

    // Extract DNS servers
    let dns_str = params.get("dns").map(|s| s.as_str()).unwrap_or("");
    let dns_servers: Vec<String> = if dns_str.is_empty() {
        vec!["1.1.1.1".to_string()]
    } else {
        dns_str.split(',').map(|s| s.trim().to_string()).collect()
    };

    // Extract remark from the fragment
    let remark = url.fragment().unwrap_or("");
    let formatted_remark = if remark.is_empty() {
        format!("{} ({})", host, port)
    } else {
        remark.to_string()
    };

    // Create the proxy object
    *node = Proxy::wireguard_construct(
        "WireGuard".to_string(),
        formatted_remark,
        host.to_string(),
        port,
        self_ip.to_string(),
        self_ipv6.to_string(),
        private_key.to_string(),
        public_key.to_string(),
        preshared_key.to_string(),
        dns_servers,
        Some(mtu),
        Some(keep_alive),
        "https://www.gstatic.com/generate_204".to_string(),
        "".to_string(),
        None,
        None,
    );

    true
}

/// Parse WireGuard peers from configuration text
pub fn parse_peers(config: &str, node: &mut Proxy) -> bool {
    if !config.contains("[Interface]") && !config.contains("[Peer]") {
        return false;
    }

    let mut section = "";
    let mut self_ip = String::new();
    let mut self_ipv6 = String::new();
    let mut private_key = String::new();
    let mut public_key = String::new();
    let mut preshared_key = String::new();
    let mut dns_servers = Vec::new();
    let mut mtu = None;
    let mut keep_alive = None;
    let mut host = String::new();
    let mut port = 51820u16; // Default WireGuard port

    for line in config.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Check section headers
        if line.starts_with('[') && line.ends_with(']') {
            section = line;
            continue;
        }

        // Split by '='
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() < 2 {
            continue;
        }

        let key = parts[0].trim();
        let value = parts[1..].join("=").trim().to_string();

        match section {
            "[Interface]" => {
                match key {
                    "PrivateKey" => private_key = value,
                    "Address" => {
                        // Address can contain IPv4 and IPv6 separated by comma
                        let addresses: Vec<&str> = value.split(',').collect();
                        for addr in addresses {
                            let addr = addr.trim();
                            if addr.contains(':') {
                                self_ipv6 = addr.to_string();
                            } else {
                                self_ip = addr.to_string();
                            }
                        }
                    }
                    "DNS" => {
                        // DNS can be comma separated
                        for dns in value.split(',') {
                            dns_servers.push(dns.trim().to_string());
                        }
                    }
                    "MTU" => mtu = value.parse::<u16>().ok(),
                    _ => {}
                }
            }
            "[Peer]" => {
                match key {
                    "PublicKey" => public_key = value,
                    "PresharedKey" => preshared_key = value,
                    "Endpoint" => {
                        // Endpoint is usually in format host:port
                        let endpoint_parts: Vec<&str> = value.split(':').collect();
                        if endpoint_parts.len() >= 1 {
                            host = endpoint_parts[0].to_string();
                        }
                        if endpoint_parts.len() >= 2 {
                            port = endpoint_parts[1].parse::<u16>().unwrap_or(51820);
                        }
                    }
                    "PersistentKeepalive" => keep_alive = value.parse::<u16>().ok(),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // Validate required fields
    if public_key.is_empty() || private_key.is_empty() || self_ip.is_empty() {
        return false;
    }

    // Create remark
    let remark = format!("{} ({})", host, port);

    // Create the proxy object
    *node = Proxy::wireguard_construct(
        "WireGuard".to_string(),
        remark,
        host,
        port,
        self_ip,
        self_ipv6,
        private_key,
        public_key,
        preshared_key,
        dns_servers,
        mtu,
        keep_alive,
        "".to_string(), // test_url
        "".to_string(), // client_id
        None,           // udp
        None,           // underlying_proxy
    );

    true
}
