use crate::parser::proxy::{
    Proxy, ProxyType, HTTP_DEFAULT_GROUP, SSR_DEFAULT_GROUP, SS_DEFAULT_GROUP,
    TROJAN_DEFAULT_GROUP, V2RAY_DEFAULT_GROUP,
};
use base64;
use std::collections::HashMap;
use url::Url;

/// Parse Quantumult configuration into a vector of Proxy objects
pub fn explode_quan(content: &str, nodes: &mut Vec<Proxy>) -> bool {
    // Split the content into lines
    let lines: Vec<&str> = content.lines().collect();

    let mut success = false;

    for line in lines {
        // Skip empty lines and comments
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        // Check if this is a proxy line
        if let Some(mut node) = parse_quan_line(line) {
            nodes.push(node);
            success = true;
        }
    }

    success
}

/// Parse a single line from Quantumult configuration
fn parse_quan_line(line: &str) -> Option<Proxy> {
    // Different formats for Quantumult configuration lines:

    // 1. Shadowsocks: [name] = shadowsocks, [server], [port], [method], [password], [options]
    if line.contains(" = shadowsocks") {
        return parse_quan_ss(line);
    }

    // 2. ShadowsocksR: [name] = shadowsocksr, [server], [port], [method], [password], [protocol], [protocol_param], [obfs], [obfs_param], [options]
    if line.contains(" = shadowsocksr") {
        return parse_quan_ssr(line);
    }

    // 3. VMess: [name] = vmess, [server], [port], [method], [uuid], [options]
    if line.contains(" = vmess") {
        return parse_quan_vmess(line);
    }

    // 4. HTTP/HTTPS: [name] = http, [server], [port], [username], [password], [options]
    if line.contains(" = http") {
        return parse_quan_http(line);
    }

    // 5. Trojan: [name] = trojan, [server], [port], [password], [options]
    if line.contains(" = trojan") {
        return parse_quan_trojan(line);
    }

    None
}

/// Parse a Quantumult Shadowsocks line
fn parse_quan_ss(line: &str) -> Option<Proxy> {
    // Format: [name] = shadowsocks, [server], [port], [method], [password], [options]
    let parts: Vec<&str> = line.splitn(2, " = ").collect();
    if parts.len() != 2 {
        return None;
    }

    let name = parts[0].trim();
    let config = parts[1].trim();

    let config_parts: Vec<&str> = config.split(',').map(|s| s.trim()).collect();
    if config_parts.len() < 5 {
        return None;
    }

    // Validate this is a Shadowsocks line
    if config_parts[0] != "shadowsocks" {
        return None;
    }

    // Extract basic parameters
    let server = config_parts[1];
    let port = match config_parts[2].parse::<u16>() {
        Ok(p) => p,
        Err(_) => return None,
    };
    let method = config_parts[3];
    let password = config_parts[4];

    // Default values
    let mut plugin = "";
    let mut plugin_opts = "";
    let mut udp = None;
    let mut tfo = None;
    let mut scv = None;

    // Parse additional options
    for i in 5..config_parts.len() {
        if config_parts[i].starts_with("obfs=") {
            plugin = "obfs";

            let obfs_parts: Vec<&str> = config_parts[i][5..].split(',').collect();
            if !obfs_parts.is_empty() {
                let mut opts = format!("obfs={}", obfs_parts[0]);

                if obfs_parts.len() > 1 {
                    opts.push_str(&format!(";obfs-host={}", obfs_parts[1]));
                }

                plugin_opts = Box::leak(opts.into_boxed_str());
            }
        } else if config_parts[i] == "udp-relay=true" {
            udp = Some(true);
        } else if config_parts[i] == "fast-open=true" {
            tfo = Some(true);
        } else if config_parts[i] == "tls-verification=false" {
            scv = Some(true);
        }
    }

    Some(Proxy::ss_construct(
        SS_DEFAULT_GROUP,
        name,
        server,
        port,
        password,
        method,
        plugin,
        plugin_opts,
        udp,
        tfo,
        scv,
        None,
        "",
    ))
}

/// Parse a Quantumult ShadowsocksR line
fn parse_quan_ssr(line: &str) -> Option<Proxy> {
    // Format: [name] = shadowsocksr, [server], [port], [method], [password], [protocol], [protocol_param], [obfs], [obfs_param], [options]
    let parts: Vec<&str> = line.splitn(2, " = ").collect();
    if parts.len() != 2 {
        return None;
    }

    let name = parts[0].trim();
    let config = parts[1].trim();

    let config_parts: Vec<&str> = config.split(',').map(|s| s.trim()).collect();
    if config_parts.len() < 9 {
        return None;
    }

    // Validate this is a ShadowsocksR line
    if config_parts[0] != "shadowsocksr" {
        return None;
    }

    // Extract basic parameters
    let server = config_parts[1];
    let port = match config_parts[2].parse::<u16>() {
        Ok(p) => p,
        Err(_) => return None,
    };
    let method = config_parts[3];
    let password = config_parts[4];
    let protocol = config_parts[5];
    let protocol_param = config_parts[6];
    let obfs = config_parts[7];
    let obfs_param = config_parts[8];

    // Default values
    let mut udp = None;
    let mut tfo = None;
    let mut scv = None;

    // Parse additional options
    for i in 9..config_parts.len() {
        if config_parts[i] == "udp-relay=true" {
            udp = Some(true);
        } else if config_parts[i] == "fast-open=true" {
            tfo = Some(true);
        } else if config_parts[i] == "tls-verification=false" {
            scv = Some(true);
        }
    }

    Some(Proxy::ssr_construct(
        SSR_DEFAULT_GROUP,
        name,
        server,
        port,
        protocol,
        method,
        obfs,
        password,
        obfs_param,
        protocol_param,
        udp,
        tfo,
        scv,
        "",
    ))
}

/// Parse a Quantumult VMess line
fn parse_quan_vmess(line: &str) -> Option<Proxy> {
    // Format: [name] = vmess, [server], [port], [method], [uuid], [options]
    let parts: Vec<&str> = line.splitn(2, " = ").collect();
    if parts.len() != 2 {
        return None;
    }

    let name = parts[0].trim();
    let config = parts[1].trim();

    let config_parts: Vec<&str> = config.split(',').map(|s| s.trim()).collect();
    if config_parts.len() < 5 {
        return None;
    }

    // Validate this is a VMess line
    if config_parts[0] != "vmess" {
        return None;
    }

    // Extract basic parameters
    let server = config_parts[1];
    let port = match config_parts[2].parse::<u16>() {
        Ok(p) => p,
        Err(_) => return None,
    };
    let method = config_parts[3];
    let uuid = config_parts[4];

    // Default values
    let mut aid = 0u16;
    let mut net = "tcp".to_string();
    let mut tls = false;
    let mut host = String::new();
    let mut path = String::new();
    let mut sni = None;
    // let mut alpn = Vec::new();
    let mut udp = None;
    let mut tfo = None;
    let mut scv = None;

    // Parse additional options
    for i in 5..config_parts.len() {
        if config_parts[i].starts_with("obfs=") {
            net = config_parts[i][5..].to_string();
        } else if config_parts[i].starts_with("obfs-path=") {
            path = config_parts[i][10..].to_string();
        } else if config_parts[i].starts_with("obfs-header=") {
            let header_str = &config_parts[i][12..];
            if let Ok(header_map) = serde_json::from_str::<HashMap<String, String>>(header_str) {
                if let Some(host_value) = header_map.get("Host") {
                    host = host_value.clone();
                }
            }
        } else if config_parts[i].starts_with("alterId=") {
            aid = config_parts[i][8..].parse::<u16>().unwrap_or(0);
        } else if config_parts[i].starts_with("over-tls=true") {
            tls = true;
        } else if config_parts[i].starts_with("tls-host=") {
            sni = Some(config_parts[i][9..].to_string());
        } else if config_parts[i] == "udp-relay=true" {
            udp = Some(true);
        } else if config_parts[i] == "fast-open=true" {
            tfo = Some(true);
        } else if config_parts[i] == "tls-verification=false" {
            scv = Some(true);
        }
    }

    // Create the proxy object
    Some(Proxy::vmess_construct(
        &V2RAY_DEFAULT_GROUP.to_string(),
        &name.to_string(),
        &server.to_string(),
        port,
        "",
        &uuid.to_string(),
        aid,
        &net,
        method,
        &path,
        &host,
        "",
        if tls { "tls" } else { "" },
        &sni.unwrap_or_else(String::new),
        udp,
        tfo,
        scv,
        None,
        "",
    ))
}

/// Parse a Quantumult HTTP/HTTPS line
fn parse_quan_http(line: &str) -> Option<Proxy> {
    // Format: [name] = http, [server], [port], [username], [password], [options]
    let parts: Vec<&str> = line.splitn(2, " = ").collect();
    if parts.len() != 2 {
        return None;
    }

    let name = parts[0].trim();
    let config = parts[1].trim();

    let config_parts: Vec<&str> = config.split(',').map(|s| s.trim()).collect();
    if config_parts.len() < 3 {
        return None;
    }

    // Validate this is an HTTP line
    if config_parts[0] != "http" {
        return None;
    }

    // Extract basic parameters
    let server = config_parts[1];
    let port = match config_parts[2].parse::<u16>() {
        Ok(p) => p,
        Err(_) => return None,
    };

    // Default values
    let mut username = "";
    let mut password = "";
    let mut is_https = false;
    let mut tfo = None;
    let mut scv = None;

    // Parse additional parameters
    if config_parts.len() > 3 {
        username = config_parts[3];
    }

    if config_parts.len() > 4 {
        password = config_parts[4];
    }

    // Parse additional options
    for i in 5..config_parts.len() {
        if config_parts[i] == "over-tls=true" {
            is_https = true;
        } else if config_parts[i] == "fast-open=true" {
            tfo = Some(true);
        } else if config_parts[i] == "tls-verification=false" {
            scv = Some(true);
        }
    }

    Some(Proxy::http_construct(
        HTTP_DEFAULT_GROUP,
        name,
        server,
        port,
        username,
        password,
        is_https,
        tfo,
        scv,
        None,
        "",
    ))
}

/// Parse a Quantumult Trojan line
fn parse_quan_trojan(line: &str) -> Option<Proxy> {
    // Format: [name] = trojan, [server], [port], [password], [options]
    let parts: Vec<&str> = line.splitn(2, " = ").collect();
    if parts.len() != 2 {
        return None;
    }

    let name = parts[0].trim();
    let config = parts[1].trim();

    let config_parts: Vec<&str> = config.split(',').map(|s| s.trim()).collect();
    if config_parts.len() < 4 {
        return None;
    }

    // Validate this is a Trojan line
    if config_parts[0] != "trojan" {
        return None;
    }

    // Extract basic parameters
    let server = config_parts[1];
    let port = match config_parts[2].parse::<u16>() {
        Ok(p) => p,
        Err(_) => return None,
    };
    let password = config_parts[3];

    // Default values
    let mut sni = None;
    let mut udp = None;
    let mut tfo = None;
    let mut scv = None;

    // Parse additional options
    for i in 4..config_parts.len() {
        if config_parts[i].starts_with("tls-host=") {
            sni = Some(config_parts[i][9..].to_string());
        } else if config_parts[i] == "udp-relay=true" {
            udp = Some(true);
        } else if config_parts[i] == "fast-open=true" {
            tfo = Some(true);
        } else if config_parts[i] == "tls-verification=false" {
            scv = Some(true);
        }
    }

    Some(Proxy::trojan_construct(
        TROJAN_DEFAULT_GROUP.to_string(),
        name.to_string(),
        server.to_string(),
        port,
        password.to_string(),
        None,
        sni,
        None,
        true,
        udp,
        tfo,
        scv,
        None,
        None,
    ))
}
