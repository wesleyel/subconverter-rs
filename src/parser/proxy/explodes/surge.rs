use crate::parser::proxy::{
    Proxy, ProxyType, HTTP_DEFAULT_GROUP, SNELL_DEFAULT_GROUP, SOCKS_DEFAULT_GROUP,
    SS_DEFAULT_GROUP, TROJAN_DEFAULT_GROUP,
};
use std::collections::HashMap;

/// Parse a Surge configuration into a vector of Proxy objects
pub fn explode_surge(content: &str, nodes: &mut Vec<Proxy>) -> bool {
    // Split the content into lines
    let lines: Vec<&str> = content.lines().collect();

    // Track the section we're currently in
    let mut in_proxy_section = false;
    let mut success = false;

    for line in lines {
        // Skip empty lines and comments
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Check section headers
        if line.starts_with('[') && line.ends_with(']') {
            in_proxy_section = line == "[Proxy]";
            continue;
        }

        // Only process lines in the [Proxy] section
        if !in_proxy_section {
            continue;
        }

        // Split by = to get name and configuration
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() != 2 {
            continue;
        }

        let name = parts[0].trim();
        let config = parts[1].trim();

        // Parse the proxy based on the configuration format
        let mut node = Proxy::default();

        if config.starts_with("ss,") || config.starts_with("shadowsocks,") {
            if parse_surge_ss(config, name, &mut node) {
                nodes.push(node);
                success = true;
            }
        } else if config.starts_with("http") || config.starts_with("https") {
            if parse_surge_http(config, name, &mut node) {
                nodes.push(node);
                success = true;
            }
        } else if config.starts_with("socks5") || config.starts_with("socks5-tls") {
            if parse_surge_socks(config, name, &mut node) {
                nodes.push(node);
                success = true;
            }
        } else if config.starts_with("trojan") {
            if parse_surge_trojan(config, name, &mut node) {
                nodes.push(node);
                success = true;
            }
        } else if config.starts_with("snell") {
            if parse_surge_snell(config, name, &mut node) {
                nodes.push(node);
                success = true;
            }
        }
    }

    success
}

/// Parse a Surge Shadowsocks configuration line
fn parse_surge_ss(config: &str, name: &str, node: &mut Proxy) -> bool {
    // Split the configuration into parts
    let parts: Vec<&str> = config.split(',').map(|s| s.trim()).collect();

    // Check minimum required parts
    if parts.len() < 4 {
        return false;
    }

    // Extract the server and port
    let server = parts[1];
    let port_str = parts[2];
    let port = match port_str.parse::<u16>() {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Extract the encryption method and password
    let method = parts[3];

    // Default values
    let mut password = "";
    let mut plugin = "";
    let mut plugin_opts = "";
    let mut udp = None;
    let mut tfo = None;
    let mut scv = None;

    // Parse additional parameters
    for i in 4..parts.len() {
        if parts[i].starts_with("password=") {
            password = &parts[i][9..];
        } else if parts[i].starts_with("plugin=") {
            plugin = &parts[i][7..];
        } else if parts[i].starts_with("plugin-opts=") {
            plugin_opts = &parts[i][12..];
        } else if parts[i] == "udp=true" {
            udp = Some(true);
        } else if parts[i] == "tfo=true" {
            tfo = Some(true);
        } else if parts[i] == "skip-cert-verify=true" {
            scv = Some(true);
        }
    }

    // Create the proxy object
    *node = Proxy::ss_construct(
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
    );

    true
}

/// Parse a Surge HTTP/HTTPS configuration line
fn parse_surge_http(config: &str, name: &str, node: &mut Proxy) -> bool {
    // Split the configuration into parts
    let parts: Vec<&str> = config.split(',').map(|s| s.trim()).collect();

    // Check minimum required parts
    if parts.len() < 3 {
        return false;
    }

    // Extract the server and port
    let server = parts[1];
    let port_str = parts[2];
    let port = match port_str.parse::<u16>() {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Determine if it's HTTPS
    let is_https = parts[0] == "https";

    // Default values
    let mut username = "";
    let mut password = "";
    let mut tfo = None;
    let mut scv = None;

    // Parse additional parameters
    for i in 3..parts.len() {
        if parts[i].starts_with("username=") {
            username = &parts[i][9..];
        } else if parts[i].starts_with("password=") {
            password = &parts[i][9..];
        } else if parts[i] == "tfo=true" {
            tfo = Some(true);
        } else if parts[i] == "skip-cert-verify=true" {
            scv = Some(true);
        }
    }

    // Create the proxy object
    *node = Proxy::http_construct(
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
    );

    true
}

/// Parse a Surge SOCKS5 configuration line
fn parse_surge_socks(config: &str, name: &str, node: &mut Proxy) -> bool {
    // Split the configuration into parts
    let parts: Vec<&str> = config.split(',').map(|s| s.trim()).collect();

    // Check minimum required parts
    if parts.len() < 3 {
        return false;
    }

    // Extract the server and port
    let server = parts[1];
    let port_str = parts[2];
    let port = match port_str.parse::<u16>() {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Default values
    let mut username = "";
    let mut password = "";
    let mut udp = None;
    let mut tfo = None;
    let mut scv = None;

    // Parse additional parameters
    for i in 3..parts.len() {
        if parts[i].starts_with("username=") {
            username = &parts[i][9..];
        } else if parts[i].starts_with("password=") {
            password = &parts[i][9..];
        } else if parts[i] == "udp=true" {
            udp = Some(true);
        } else if parts[i] == "tfo=true" {
            tfo = Some(true);
        } else if parts[i] == "skip-cert-verify=true" {
            scv = Some(true);
        }
    }

    // Create the proxy object
    *node = Proxy::socks_construct(
        SOCKS_DEFAULT_GROUP,
        name,
        server,
        port,
        username,
        password,
        udp,
        tfo,
        scv,
        "",
    );

    true
}

/// Parse a Surge Trojan configuration line
fn parse_surge_trojan(config: &str, name: &str, node: &mut Proxy) -> bool {
    // Split the configuration into parts
    let parts: Vec<&str> = config.split(',').map(|s| s.trim()).collect();

    // Check minimum required parts
    if parts.len() < 4 {
        return false;
    }

    // Extract the server and port
    let server = parts[1];
    let port_str = parts[2];
    let port = match port_str.parse::<u16>() {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Extract password
    let mut password = parts[3];
    if password.starts_with("password=") {
        password = &password[9..];
    }

    // Default values
    let mut sni = None;
    let mut udp = None;
    let mut tfo = None;
    let mut scv = None;

    // Parse additional parameters
    for i in 4..parts.len() {
        if parts[i].starts_with("sni=") {
            sni = Some(parts[i][4..].to_string());
        } else if parts[i] == "udp=true" {
            udp = Some(true);
        } else if parts[i] == "tfo=true" {
            tfo = Some(true);
        } else if parts[i] == "skip-cert-verify=true" {
            scv = Some(true);
        }
    }

    // Create the proxy object
    *node = Proxy::trojan_construct(
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
    );

    true
}

/// Parse a Surge Snell configuration line
fn parse_surge_snell(config: &str, name: &str, node: &mut Proxy) -> bool {
    // Split the configuration into parts
    let parts: Vec<&str> = config.split(',').map(|s| s.trim()).collect();

    // Check minimum required parts
    if parts.len() < 4 {
        return false;
    }

    // Extract the server and port
    let server = parts[1];
    let port_str = parts[2];
    let port = match port_str.parse::<u16>() {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Default values
    let mut password = "";
    let mut obfs = String::new();
    let mut obfs_host = String::new();
    let mut version = 1u16;
    let mut udp = None;
    let mut tfo = None;
    let mut scv = None;

    // Parse additional parameters
    for i in 3..parts.len() {
        if parts[i].starts_with("psk=") {
            password = &parts[i][4..];
        } else if parts[i].starts_with("obfs=") {
            obfs = parts[i][5..].to_string();
        } else if parts[i].starts_with("obfs-host=") {
            obfs_host = parts[i][10..].to_string();
        } else if parts[i].starts_with("version=") {
            version = parts[i][8..].parse::<u16>().unwrap_or(1);
        } else if parts[i] == "udp=true" {
            udp = Some(true);
        } else if parts[i] == "tfo=true" {
            tfo = Some(true);
        } else if parts[i] == "skip-cert-verify=true" {
            scv = Some(true);
        }
    }

    // Create the proxy object
    *node = Proxy::snell_construct(
        SNELL_DEFAULT_GROUP.to_string(),
        name.to_string(),
        server.to_string(),
        port,
        password.to_string(),
        obfs,
        obfs_host,
        version,
        udp,
        tfo,
        scv,
        None,
    );

    true
}
