use crate::parser::proxy::{Proxy, ProxyType};
use base64::{engine::general_purpose::STANDARD, Engine};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use url::Url;

/// Parse a VMess link into a Proxy object
pub fn explode_vmess(vmess: &str, node: &mut Proxy) -> bool {
    // Check if the link starts with vmess://
    if !vmess.starts_with("vmess://") {
        return false;
    }

    // Extract the base64 part
    let encoded = &vmess[8..];

    // Decode base64
    let decoded = match STANDARD.decode(encoded) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(s) => s,
            Err(_) => return false,
        },
        Err(_) => return false,
    };

    // Try to parse as JSON
    let json: Value = match serde_json::from_str(&decoded) {
        Ok(json) => json,
        Err(_) => return false,
    };

    // Determine protocol version
    let version = json["v"].as_u64().unwrap_or(1);

    // Extract common fields
    let add = json["add"].as_str().unwrap_or("").to_string();
    let port = json["port"]
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            json["port"]
                .as_u64()
                .map_or_else(|| "0".to_string(), |p| p.to_string())
        });
    let id = json["id"].as_str().unwrap_or("").to_string();
    let aid = json["aid"]
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            json["aid"]
                .as_u64()
                .map_or_else(|| "0".to_string(), |a| a.to_string())
        });
    let net = json["net"].as_str().unwrap_or("tcp").to_string();
    let type_field = json["type"].as_str().unwrap_or("").to_string();
    let mut host = json["host"].as_str().unwrap_or("").to_string();
    let mut path = json["path"].as_str().unwrap_or("").to_string();
    let tls = json["tls"].as_str().unwrap_or("").to_string();
    let sni = json["sni"].as_str().unwrap_or("").to_string();

    // Extract remark (ps field)
    let remark = json["ps"].as_str().unwrap_or("").to_string();

    // Parse port and aid as integers
    let port = port.parse::<u16>().unwrap_or(0);
    let aid = aid.parse::<u16>().unwrap_or(0);

    // Handle host and path for different versions
    if version == 2 {
        if !host.is_empty() {
            let host_str = host.clone();
            let parts: Vec<&str> = host_str.split(';').collect();
            if parts.len() == 2 {
                host = parts[0].to_string();
                path = parts[1].to_string();
            }
        }
    }

    // Create the proxy object
    *node = Proxy::vmess_construct(
        "VMess",
        &remark,
        &add,
        port,
        &type_field,
        &id,
        aid,
        &net,
        "auto",
        &path,
        &host,
        "",
        &tls,
        &sni,
        None,
        None,
        None,
        None,
        "",
    );

    true
}

/// Parse a standard VMess link into a Proxy object
/// Format: vmess[+tls]://uuid-alterId@hostname:port[/?network=ws&host=xxx&path=yyy]
pub fn explode_std_vmess(vmess: &str, node: &mut Proxy) -> bool {
    // Check if the link starts with vmess:// or vmess+tls://
    if !vmess.starts_with("vmess://") && !vmess.starts_with("vmess+") {
        return false;
    }

    // Extract the protocol part and check TLS
    let protocol_end = match vmess.find("://") {
        Some(pos) => pos,
        None => return false,
    };

    let protocol = vmess[..protocol_end].to_string();
    let tls = protocol.contains("+tls");

    // Extract the rest of the URL
    let url_part = &vmess[protocol_end + 3..];

    // Split URL and fragment (remark)
    let (url_without_fragment, remark) = match url_part.find('#') {
        Some(pos) => (url_part[..pos].to_string(), url_part[pos + 1..].to_string()),
        None => (url_part.to_string(), String::new()),
    };

    // Parse the URL-like string
    let re = Regex::new(
        r"^([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})-(\d+)@([^:]+):(\d+)(.*)$",
    )
    .unwrap();

    let caps = match re.captures(&url_without_fragment) {
        Some(c) => c,
        None => return false,
    };

    let id = caps.get(1).map_or("", |m| m.as_str()).to_string();
    let aid = caps
        .get(2)
        .map_or("0", |m| m.as_str())
        .parse::<u16>()
        .unwrap_or(0);
    let host = caps.get(3).map_or("", |m| m.as_str()).to_string();
    let port = caps
        .get(4)
        .map_or("0", |m| m.as_str())
        .parse::<u16>()
        .unwrap_or(0);
    let query = caps.get(5).map_or("", |m| m.as_str()).to_string();

    // Default values
    let mut net = "tcp".to_string();
    let mut path = "/".to_string();
    let mut host_header = host.clone();
    let mut tls_str = if tls {
        "tls".to_string()
    } else {
        String::new()
    };
    let mut sni = String::new();

    // Parse query parameters
    if !query.is_empty() && query.starts_with("/?") {
        for param in query[2..].split('&') {
            let mut kv = param.split('=');
            if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                match k {
                    "network" => net = v.to_string(),
                    "host" => host_header = v.to_string(),
                    "path" => path = v.to_string(),
                    "tls" => tls_str = v.to_string(),
                    "sni" => sni = v.to_string(),
                    _ => {}
                }
            }
        }
    }

    // Create formatted remark if empty
    let formatted_remark = if remark.is_empty() {
        format!("{} ({})", host, port)
    } else {
        remark
    };

    // Create the proxy object
    *node = Proxy::vmess_construct(
        "VMess",
        &formatted_remark,
        &host,
        port,
        "",
        &id,
        aid,
        &net,
        "auto",
        &path,
        &host_header,
        "",
        &tls_str,
        &sni,
        None,
        None,
        None,
        None,
        "",
    );

    true
}

/// Parse a Shadowrocket format VMess link
pub fn explode_shadowrocket(rocket: &str, node: &mut Proxy) -> bool {
    // Check if the link starts with vmess://
    if !rocket.starts_with("vmess://") {
        return false;
    }

    // Try to parse as URL
    let url = match Url::parse(rocket) {
        Ok(url) => url,
        Err(_) => return false,
    };

    // Extract host and port
    let host = url.host_str().unwrap_or("").to_string();
    let port = url.port().unwrap_or(0);
    if port == 0 {
        return false;
    }

    // Extract username (contains encoded config)
    let username = url.username().to_string();
    if username.is_empty() {
        return false;
    }

    // Decode the username
    let decoded = match STANDARD.decode(username) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(s) => s,
            Err(_) => return false,
        },
        Err(_) => return false,
    };

    // Parse the decoded string
    let parts: Vec<&str> = decoded.split(':').collect();
    if parts.len() < 6 {
        return false;
    }

    let method = parts[0].to_string();
    let id = parts[1].to_string();
    let aid = parts[2].parse::<u16>().unwrap_or(0);

    // Extract parameters from the query string
    let mut net = "tcp".to_string();
    let mut path = "/".to_string();
    let mut host_header = host.clone();
    let mut tls = String::new();
    let mut sni = String::new();

    for (key, value) in url.query_pairs() {
        let value = value.to_string();
        match key.as_ref() {
            "obfs" => net = value,
            "path" => path = value,
            "obfsParam" => host_header = value,
            "tls" => {
                tls = if value == "1" {
                    "tls".to_string()
                } else {
                    String::new()
                }
            }
            "peer" => sni = value,
            _ => {}
        }
    }

    // Extract remark from the fragment
    let remark = url.fragment().unwrap_or("").to_string();
    let formatted_remark = if remark.is_empty() {
        format!("{} ({})", host, port)
    } else {
        remark
    };

    // Create the proxy object
    *node = Proxy::vmess_construct(
        "VMess",
        &formatted_remark,
        &host,
        port,
        "",
        &id,
        aid,
        &net,
        &method,
        &path,
        &host_header,
        "",
        &tls,
        &sni,
        None,
        None,
        None,
        None,
        "",
    );

    true
}

/// Parse a Kitsunebi format VMess link
pub fn explode_kitsunebi(kit: &str, node: &mut Proxy) -> bool {
    // Check if the link starts with vmess://
    if !kit.starts_with("vmess://") {
        return false;
    }

    // Extract the base64 part
    let encoded = &kit[8..];

    // Decode base64
    let decoded = match STANDARD.decode(encoded) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(s) => s,
            Err(_) => return false,
        },
        Err(_) => return false,
    };

    // Split by line breaks
    let lines: Vec<&str> = decoded.lines().collect();
    if lines.is_empty() {
        return false;
    }

    // Parse the first line (main config)
    let parts: Vec<&str> = lines[0].split(',').collect();
    if parts.len() < 4 {
        return false;
    }

    let add = parts[0].to_string();
    let port = parts[1].parse::<u16>().unwrap_or(0);
    let id = parts[2].to_string();
    let aid = parts[3].parse::<u16>().unwrap_or(0);

    // Default values
    let mut net = "tcp".to_string();
    let mut path = "/".to_string();
    let mut host = add.clone();
    let mut tls = String::new();
    let mut sni = String::new();
    let mut remark = format!("{} ({})", add, port);

    // Parse additional parameters
    for i in 4..parts.len() {
        let kv: Vec<&str> = parts[i].split('=').collect();
        if kv.len() != 2 {
            continue;
        }

        let value = kv[1].to_string();
        match kv[0] {
            "net" => net = value,
            "path" => path = value,
            "host" => host = value,
            "tls" => tls = value,
            "sni" => sni = value,
            "remarks" | "remark" => remark = value,
            _ => {}
        }
    }

    // Create the proxy object
    *node = Proxy::vmess_construct(
        "VMess", &remark, &add, port, "", &id, aid, &net, "auto", &path, &host, "", &tls, &sni,
        None, None, None, None, "",
    );

    true
}

/// Parse a VMess configuration file into a vector of Proxy objects
pub fn explode_vmess_conf(content: &str, nodes: &mut Vec<Proxy>) -> bool {
    // Try to parse as JSON
    let json: Value = match serde_json::from_str(content) {
        Ok(json) => json,
        Err(_) => return false,
    };

    // Check if it's a V2Ray configuration
    if !json["outbounds"].is_array() {
        return false;
    }

    // Extract outbounds
    let outbounds = json["outbounds"].as_array().unwrap();
    let mut success = false;

    for outbound in outbounds {
        // Check if it's a VMess outbound
        if outbound["protocol"].as_str().unwrap_or("") != "vmess" {
            continue;
        }

        // Extract settings
        let settings = &outbound["settings"];
        if !settings["vnext"].is_array() {
            continue;
        }

        // Extract vnext
        let vnext = settings["vnext"].as_array().unwrap();

        for server in vnext {
            let address = server["address"].as_str().unwrap_or("").to_string();
            let port = server["port"].as_u64().unwrap_or(0) as u16;

            // Extract users
            if !server["users"].is_array() {
                continue;
            }

            let users = server["users"].as_array().unwrap();

            for user in users {
                let id = user["id"].as_str().unwrap_or("").to_string();
                let alter_id = user["alterId"].as_u64().unwrap_or(0) as u16;
                let security = user["security"].as_str().unwrap_or("auto").to_string();

                // Extract stream settings
                let stream_settings = &outbound["streamSettings"];
                let network = stream_settings["network"]
                    .as_str()
                    .unwrap_or("tcp")
                    .to_string();
                let security_type = stream_settings["security"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();

                // Extract network-specific settings
                let mut host = String::new();
                let mut path = String::new();
                let mut edge = String::new();
                let mut tls = String::new();
                let mut sni = String::new();

                match network.as_str() {
                    "ws" => {
                        let ws_settings = &stream_settings["wsSettings"];
                        path = ws_settings["path"].as_str().unwrap_or("").to_string();

                        if let Some(headers) = ws_settings["headers"].as_object() {
                            if let Some(host_val) = headers.get("Host") {
                                host = host_val.as_str().unwrap_or("").to_string();
                            }
                        }
                    }
                    "h2" => {
                        let h2_settings = &stream_settings["httpSettings"];
                        path = h2_settings["path"].as_str().unwrap_or("").to_string();

                        if let Some(hosts) = h2_settings["host"].as_array() {
                            if !hosts.is_empty() {
                                host = hosts[0].as_str().unwrap_or("").to_string();
                            }
                        }
                    }
                    _ => {}
                }

                if security_type == "tls" {
                    tls = "tls".to_string();
                    let tls_settings = &stream_settings["tlsSettings"];
                    sni = tls_settings["serverName"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();
                }

                // Create formatted remark for the node
                let formatted_remark = format!("{} ({})", address, port);

                // Create the proxy object
                let mut node = Proxy::vmess_construct(
                    "VMess",
                    &formatted_remark,
                    &address,
                    port,
                    "",
                    &id,
                    alter_id,
                    &network,
                    &security,
                    &path,
                    &host,
                    &edge,
                    &tls,
                    &sni,
                    None,
                    None,
                    None,
                    None,
                    "",
                );

                nodes.push(node);
                success = true;
            }
        }
    }

    success
}
