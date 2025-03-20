use crate::models::{Proxy, SS_DEFAULT_GROUP};
use crate::utils::url::url_decode;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde_json::Value;

/// Parse a Shadowsocks link into a Proxy object
/// Based on the C++ implementation in explodeSS function
pub fn explode_ss(ss: &str, node: &mut Proxy) -> bool {
    // Check if the link starts with ss://
    if !ss.starts_with("ss://") {
        return false;
    }

    // Extract the content part after ss://
    let mut ss_content = ss[5..].to_string();
    // Replace "/?" with "?" like in C++ replaceAllDistinct
    ss_content = ss_content.replace("/?", "?");

    // Extract fragment (remark) if present
    let mut ps = String::new();
    if let Some(hash_pos) = ss_content.find('#') {
        ps = url_decode(&ss_content[hash_pos + 1..]);
        ss_content = ss_content[..hash_pos].to_string();
    }

    // Extract plugin and other query parameters
    let mut plugin = String::new();
    let mut plugin_opts = String::new();
    let mut group = SS_DEFAULT_GROUP.to_string();

    if let Some(query_pos) = ss_content.find('?') {
        let addition = ss_content[query_pos + 1..].to_string();
        ss_content = ss_content[..query_pos].to_string();

        // Parse query parameters
        for (key, value) in url::form_urlencoded::parse(addition.as_bytes()) {
            if key == "plugin" {
                let plugins = url_decode(&value);
                if let Some(semicolon_pos) = plugins.find(';') {
                    plugin = plugins[..semicolon_pos].to_string();
                    plugin_opts = plugins[semicolon_pos + 1..].to_string();
                } else {
                    plugin = plugins;
                }
            } else if key == "group" {
                if !value.is_empty() {
                    group = crate::utils::base64::url_safe_base64_decode(&value);
                }
            }
        }
    }

    // Parse the main part of the URL
    let mut method = String::new();
    let mut password = String::new();
    let mut server = String::new();
    let mut port = 0;

    if ss_content.contains('@') {
        // SIP002 format (method:password@server:port)
        let parts: Vec<&str> = ss_content.split('@').collect();
        if parts.len() < 2 {
            return false;
        }

        let secret = parts[0];
        let server_port = parts[1];

        // Parse server and port
        let server_port_parts: Vec<&str> = server_port.split(':').collect();
        if server_port_parts.len() < 2 {
            return false;
        }
        server = server_port_parts[0].to_string();
        port = match server_port_parts[1].parse::<u16>() {
            Ok(p) => p,
            Err(_) => return false,
        };

        // Decode the secret part
        let decoded_secret = crate::utils::base64::url_safe_base64_decode(secret);
        let method_pass: Vec<&str> = decoded_secret.split(':').collect();
        if method_pass.len() < 2 {
            return false;
        }
        method = method_pass[0].to_string();
        password = method_pass[1..].join(":"); // In case password contains colons
    } else {
        // Legacy format
        let decoded = crate::utils::base64::url_safe_base64_decode(&ss_content);
        if decoded.is_empty() {
            return false;
        }

        // Parse method:password@server:port
        let parts: Vec<&str> = decoded.split('@').collect();
        if parts.len() < 2 {
            return false;
        }

        let method_pass = parts[0];
        let server_port = parts[1];

        // Parse method and password
        let method_pass_parts: Vec<&str> = method_pass.split(':').collect();
        if method_pass_parts.len() < 2 {
            return false;
        }
        method = method_pass_parts[0].to_string();
        password = method_pass_parts[1..].join(":"); // In case password contains colons

        // Parse server and port
        let server_port_parts: Vec<&str> = server_port.split(':').collect();
        if server_port_parts.len() < 2 {
            return false;
        }
        server = server_port_parts[0].to_string();
        port = match server_port_parts[1].parse::<u16>() {
            Ok(p) => p,
            Err(_) => return false,
        };
    }

    // Skip if port is 0
    if port == 0 {
        return false;
    }

    // Use server:port as remark if none provided
    if ps.is_empty() {
        ps = format!("{} ({})", server, port);
    }

    // Create the proxy
    *node = Proxy::ss_construct(
        &group,
        &ps,
        &server,
        port,
        &password,
        &method,
        &plugin,
        &plugin_opts,
        None,
        None,
        None,
        None,
        "",
    );

    true
}

/// Parse a SSD (Shadowsocks subscription) link into a vector of Proxy objects
pub fn explode_ssd(link: &str, nodes: &mut Vec<Proxy>) -> bool {
    // Check if the link starts with ssd://
    if !link.starts_with("ssd://") {
        return false;
    }

    // Extract the base64 part
    let encoded = &link[6..];

    // Decode base64
    let decoded = match STANDARD.decode(encoded) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => return false,
        },
        Err(_) => return false,
    };

    // Parse as JSON
    let json: Value = match serde_json::from_str(&decoded) {
        Ok(json) => json,
        Err(_) => return false,
    };

    // Extract common fields
    let airport = json["airport"].as_str().unwrap_or("");
    let port = json["port"].as_u64().unwrap_or(0) as u16;
    let encryption = json["encryption"].as_str().unwrap_or("");
    let password = json["password"].as_str().unwrap_or("");

    // Extract servers
    if !json["servers"].is_array() {
        return false;
    }

    let servers = json["servers"].as_array().unwrap();

    for server in servers {
        let server_host = server["server"].as_str().unwrap_or("");
        let server_port = server["port"].as_u64().unwrap_or(port as u64) as u16;
        let server_encryption = server["encryption"].as_str().unwrap_or(encryption);
        let server_password = server["password"].as_str().unwrap_or(password);
        let server_remark = server["remarks"].as_str().unwrap_or("");
        let server_plugin = server["plugin"].as_str().unwrap_or("");
        let server_plugin_opts = server["plugin_options"].as_str().unwrap_or("");

        // Create formatted remark
        let formatted_remark = format!("{} - {}", airport, server_remark);

        // Create the proxy object
        let node = Proxy::ss_construct(
            SS_DEFAULT_GROUP,
            &formatted_remark,
            server_host,
            server_port,
            server_password,
            server_encryption,
            server_plugin,
            server_plugin_opts,
            None,
            None,
            None,
            None,
            "",
        );

        nodes.push(node);
    }

    !nodes.is_empty()
}

/// Parse Android Shadowsocks configuration into a vector of Proxy objects
pub fn explode_ss_android(content: &str, nodes: &mut Vec<Proxy>) -> bool {
    // Try to parse as JSON
    let json: Value = match serde_json::from_str(content) {
        Ok(json) => json,
        Err(_) => {
            // If not JSON, try to parse as text format
            let mut success = false;

            for line in content.lines() {
                if line.starts_with("ss://") {
                    let mut node = Proxy::default();
                    if explode_ss(line, &mut node) {
                        nodes.push(node);
                        success = true;
                    }
                } else if line.starts_with("ssr://") {
                    // Handle SSR links
                } else if line.contains("=ss-base64=") {
                    let parts: Vec<&str> = line.split("=ss-base64=").collect();
                    if parts.len() >= 2 {
                        let encoded = parts[1].trim();
                        let decoded = match STANDARD.decode(encoded) {
                            Ok(bytes) => match String::from_utf8(bytes) {
                                Ok(s) => s,
                                Err(_) => continue,
                            },
                            Err(_) => continue,
                        };

                        // Process decoded content
                    }
                }
            }

            return success;
        }
    };

    // Check if it contains profiles
    if !json["configs"].is_array() && !json["proxies"].is_array() {
        return false;
    }

    // Determine which field to use
    let configs = if json["configs"].is_array() {
        json["configs"].as_array().unwrap()
    } else {
        json["proxies"].as_array().unwrap()
    };

    let mut index = nodes.len();

    for config in configs {
        // Extract fields
        let server = config["server"].as_str().unwrap_or("");
        if server.is_empty() {
            continue;
        }

        let port_num = config["server_port"].as_u64().unwrap_or(0) as u16;
        if port_num == 0 {
            continue;
        }

        let method = config["method"].as_str().unwrap_or("");
        let password = config["password"].as_str().unwrap_or("");

        // Get remark, try both "remarks" and "name" fields
        let remark = if config["remarks"].is_string() {
            config["remarks"].as_str().unwrap_or("").to_string()
        } else if config["name"].is_string() {
            config["name"].as_str().unwrap_or("").to_string()
        } else {
            format!("{} ({})", server, port_num)
        };

        // Get plugin and plugin_opts
        let plugin = config["plugin"].as_str().unwrap_or("");
        let plugin_opts = config["plugin_opts"].as_str().unwrap_or("");

        // Create the proxy object
        let mut node = Proxy::ss_construct(
            SS_DEFAULT_GROUP,
            &remark,
            server,
            port_num,
            password,
            method,
            plugin,
            plugin_opts,
            None,
            None,
            None,
            None,
            "",
        );

        node.id = index as u32;
        nodes.push(node);
        index += 1;
    }

    !nodes.is_empty()
}

/// Parse a Shadowsocks configuration file into a vector of Proxy objects
pub fn explode_ss_conf(content: &str, nodes: &mut Vec<Proxy>) -> bool {
    // Try to parse as JSON
    let json: Value = match serde_json::from_str(content) {
        Ok(json) => json,
        Err(_) => return false,
    };

    // Check for different configuration formats
    if json["configs"].is_array() || json["proxies"].is_array() {
        return explode_ss_android(content, nodes);
    }

    // Check for single server configuration
    if json["server"].is_string() && json["server_port"].is_u64() {
        let index = nodes.len();

        // Extract fields
        let server = json["server"].as_str().unwrap_or("");
        let port_num = json["server_port"].as_u64().unwrap_or(0) as u16;
        if server.is_empty() || port_num == 0 {
            return false;
        }

        let method = json["method"].as_str().unwrap_or("");
        let password = json["password"].as_str().unwrap_or("");

        // Get remark
        let remark = if json["remarks"].is_string() {
            json["remarks"].as_str().unwrap_or("")
        } else {
            &format!("{} ({})", server, port_num)
        };

        // Get plugin and plugin_opts
        let plugin = json["plugin"].as_str().unwrap_or("");
        let plugin_opts = json["plugin_opts"].as_str().unwrap_or("");

        // Create the proxy object
        let mut node = Proxy::ss_construct(
            SS_DEFAULT_GROUP,
            remark,
            server,
            port_num,
            password,
            method,
            plugin,
            plugin_opts,
            None,
            None,
            None,
            None,
            "",
        );

        node.id = index as u32;
        nodes.push(node);

        return true;
    }

    // Check for server list configuration
    if json["servers"].is_array() {
        let servers = json["servers"].as_array().unwrap();
        let mut index = nodes.len();

        for server_json in servers {
            // Extract fields
            let server = server_json["server"].as_str().unwrap_or("");
            let port_num = server_json["server_port"].as_u64().unwrap_or(0) as u16;
            if server.is_empty() || port_num == 0 {
                continue;
            }

            let method = server_json["method"].as_str().unwrap_or("");
            let password = server_json["password"].as_str().unwrap_or("");

            // Get remark
            let remark = if server_json["remarks"].is_string() {
                server_json["remarks"].as_str().unwrap_or("")
            } else {
                &format!("{} ({})", server, port_num)
            };

            // Get plugin and plugin_opts
            let plugin = server_json["plugin"].as_str().unwrap_or("");
            let plugin_opts = server_json["plugin_opts"].as_str().unwrap_or("");

            // Create the proxy object
            let mut node = Proxy::ss_construct(
                SS_DEFAULT_GROUP,
                remark,
                server,
                port_num,
                password,
                method,
                plugin,
                plugin_opts,
                None,
                None,
                None,
                None,
                "",
            );

            node.id = index as u32;
            nodes.push(node);
            index += 1;
        }

        return !nodes.is_empty();
    }

    false
}

#[cfg(test)]
mod tests {
    use crate::ProxyType;

    use super::*;

    #[test]
    fn test_explode_ss_legacy_format() {
        // Legacy format: ss://base64(method:password@server:port)
        let legacy_ss = "ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpwYXNzd29yZEAxMjcuMC4wLjE6ODA4MA==";
        let mut node = Proxy::default();
        let result = explode_ss(legacy_ss, &mut node);

        assert!(result);
        assert_eq!(node.proxy_type, ProxyType::Shadowsocks);
        assert_eq!(node.hostname, "127.0.0.1");
        assert_eq!(node.port, 8080);
        assert_eq!(
            node.encrypt_method,
            Some("chacha20-ietf-poly1305".to_string())
        );
        assert_eq!(node.password, Some("password".to_string()));
    }

    #[test]
    fn test_explode_ss_sip002_format() {
        // SIP002 format: ss://base64(method:password)@server:port
        let sip002_ss = "ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpwYXNzd29yZA==@example.com:8388";
        let mut node = Proxy::default();
        let result = explode_ss(sip002_ss, &mut node);

        assert!(result);
        assert_eq!(node.proxy_type, ProxyType::Shadowsocks);
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 8388);
        assert_eq!(
            node.encrypt_method,
            Some("chacha20-ietf-poly1305".to_string())
        );
        assert_eq!(node.password, Some("password".to_string()));
    }

    #[test]
    fn test_explode_ss_with_fragment() {
        // SIP002 format with fragment for remark
        let ss_with_fragment =
            "ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpwYXNzd29yZA==@example.com:8388#Example%20Server";
        let mut node = Proxy::default();
        let result = explode_ss(ss_with_fragment, &mut node);

        assert!(result);
        assert_eq!(node.proxy_type, ProxyType::Shadowsocks);
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 8388);
        assert_eq!(node.remark, "Example Server");
    }

    #[test]
    fn test_explode_ss_with_plugin() {
        // SIP002 format with plugin
        let ss_with_plugin = "ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpwYXNzd29yZA==@example.com:8388/?plugin=obfs-local;obfs=http;obfs-host=example.com#Example%20Plugin";
        let mut node = Proxy::default();
        let result = explode_ss(ss_with_plugin, &mut node);

        assert!(result);
        assert_eq!(node.proxy_type, ProxyType::Shadowsocks);
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 8388);
        assert_eq!(node.plugin, Some("obfs-local".to_string()));
        assert_eq!(
            node.plugin_option,
            Some("obfs=http;obfs-host=example.com".to_string())
        );
        assert_eq!(node.remark, "Example Plugin");
    }

    #[test]
    fn test_explode_ss_invalid_url() {
        // Invalid URL
        let invalid_ss = "ss://invalid";
        let mut node = Proxy::default();
        let result = explode_ss(invalid_ss, &mut node);

        assert!(!result);
    }

    #[test]
    fn test_explode_ssd() {
        // Simple SSD format
        let ssd_content = "ssd://eyJhaXJwb3J0IjoiRXhhbXBsZSBTZXJ2ZXIiLCJwb3J0Ijo4Mzg4LCJlbmNyeXB0aW9uIjoiY2hhY2hhMjAtaWV0Zi1wb2x5MTMwNSIsInBhc3N3b3JkIjoicGFzc3dvcmQiLCJzZXJ2ZXJzIjpbeyJzZXJ2ZXIiOiJleGFtcGxlLmNvbSIsInJlbWFya3MiOiJUZXN0IFNlcnZlciJ9LHsic2VydmVyIjoiZXhhbXBsZTIuY29tIiwicG9ydCI6ODM4OSwicmVtYXJrcyI6IlRlc3QgU2VydmVyIDIiLCJlbmNyeXB0aW9uIjoiYWVzLTI1Ni1nY20ifV19";
        let mut nodes = Vec::new();
        let result = explode_ssd(ssd_content, &mut nodes);

        assert!(result);
        assert_eq!(nodes.len(), 2);

        // Check first node
        assert_eq!(nodes[0].proxy_type, ProxyType::Shadowsocks);
        assert_eq!(nodes[0].hostname, "example.com");
        assert_eq!(nodes[0].port, 8388);
        assert_eq!(
            nodes[0].encrypt_method,
            Some("chacha20-ietf-poly1305".to_string())
        );
        assert_eq!(nodes[0].password, Some("password".to_string()));
        assert_eq!(nodes[0].remark, "Example Server - Test Server");

        // Check second node
        assert_eq!(nodes[1].proxy_type, ProxyType::Shadowsocks);
        assert_eq!(nodes[1].hostname, "example2.com");
        assert_eq!(nodes[1].port, 8389);
        assert_eq!(nodes[1].encrypt_method, Some("aes-256-gcm".to_string()));
        assert_eq!(nodes[1].password, Some("password".to_string()));
        assert_eq!(nodes[1].remark, "Example Server - Test Server 2");
    }

    #[test]
    fn test_explode_ss_android() {
        // Android Shadowsocks configuration
        let ss_android_content = r#"{
            "configs": [
                {
                    "server": "example.com",
                    "server_port": 8388,
                    "password": "password",
                    "method": "chacha20-ietf-poly1305",
                    "remarks": "Test Server 1"
                },
                {
                    "server": "example2.com",
                    "server_port": 8389,
                    "password": "password2",
                    "method": "aes-256-gcm",
                    "plugin": "obfs-local",
                    "plugin_opts": "obfs=http;obfs-host=example.com",
                    "remarks": "Test Server 2"
                }
            ]
        }"#;

        let mut nodes = Vec::new();
        let result = explode_ss_android(ss_android_content, &mut nodes);

        assert!(result);
        assert_eq!(nodes.len(), 2);

        // Check first node
        assert_eq!(nodes[0].proxy_type, ProxyType::Shadowsocks);
        assert_eq!(nodes[0].hostname, "example.com");
        assert_eq!(nodes[0].port, 8388);
        assert_eq!(
            nodes[0].encrypt_method,
            Some("chacha20-ietf-poly1305".to_string())
        );
        assert_eq!(nodes[0].password, Some("password".to_string()));
        assert_eq!(nodes[0].remark, "Test Server 1");

        // Check second node
        assert_eq!(nodes[1].proxy_type, ProxyType::Shadowsocks);
        assert_eq!(nodes[1].hostname, "example2.com");
        assert_eq!(nodes[1].port, 8389);
        assert_eq!(nodes[1].encrypt_method, Some("aes-256-gcm".to_string()));
        assert_eq!(nodes[1].password, Some("password2".to_string()));
        assert_eq!(nodes[1].plugin, Some("obfs-local".to_string()));
        assert_eq!(
            nodes[1].plugin_option,
            Some("obfs=http;obfs-host=example.com".to_string())
        );
        assert_eq!(nodes[1].remark, "Test Server 2");
    }

    #[test]
    fn test_explode_ss_conf() {
        // Test with a single server configuration
        let single_server_conf = r#"{
            "server": "example.com",
            "server_port": 8388,
            "password": "password",
            "method": "chacha20-ietf-poly1305",
            "remarks": "Single Server Config"
        }"#;

        let mut nodes = Vec::new();
        let result = explode_ss_conf(single_server_conf, &mut nodes);

        assert!(result);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].hostname, "example.com");
        assert_eq!(nodes[0].port, 8388);
        assert_eq!(nodes[0].remark, "Single Server Config");

        // Test with a server list configuration
        let server_list_conf = r#"{
            "servers": [
                {
                    "server": "example.com",
                    "server_port": 8388,
                    "password": "password",
                    "method": "chacha20-ietf-poly1305",
                    "remarks": "Server 1"
                },
                {
                    "server": "example2.com",
                    "server_port": 8389,
                    "password": "password2",
                    "method": "aes-256-gcm",
                    "remarks": "Server 2"
                }
            ]
        }"#;

        let mut nodes = Vec::new();
        let result = explode_ss_conf(server_list_conf, &mut nodes);

        assert!(result);
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].hostname, "example.com");
        assert_eq!(nodes[0].remark, "Server 1");
        assert_eq!(nodes[1].hostname, "example2.com");
        assert_eq!(nodes[1].remark, "Server 2");
    }

    // New tests to improve coverage

    #[test]
    fn test_explode_ss_with_ipv6() {
        // Test SIP002 format with IPv6 address
        let ss_with_ipv6 = "ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpwYXNzd29yZA==@[2001:db8::1]:8388";
        let mut node = Proxy::default();
        let result = explode_ss(ss_with_ipv6, &mut node);

        assert!(result);
        assert_eq!(node.proxy_type, ProxyType::Shadowsocks);
        assert_eq!(node.hostname, "[2001:db8::1]");
        assert_eq!(node.port, 8388);
        assert_eq!(
            node.encrypt_method,
            Some("chacha20-ietf-poly1305".to_string())
        );
        assert_eq!(node.password, Some("password".to_string()));
    }

    #[test]
    fn test_explode_ss_with_plain_credentials() {
        // Test URL with non-base64 encoded credentials
        let ss_with_plain = "ss://aes-256-gcm:password123@example.com:8388";
        let mut node = Proxy::default();
        let result = explode_ss(ss_with_plain, &mut node);

        assert!(result);
        assert_eq!(node.proxy_type, ProxyType::Shadowsocks);
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 8388);
        assert_eq!(node.encrypt_method, Some("aes-256-gcm".to_string()));
        assert_eq!(node.password, Some("password123".to_string()));
    }

    #[test]
    fn test_explode_ss_with_password_containing_colon() {
        // Test URL with password containing colon
        let ss_with_colon_in_pass =
            "ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpwYXNzOndvcmQ=@example.com:8388";
        let mut node = Proxy::default();
        let result = explode_ss(ss_with_colon_in_pass, &mut node);

        assert!(result);
        assert_eq!(node.proxy_type, ProxyType::Shadowsocks);
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 8388);
        assert_eq!(
            node.encrypt_method,
            Some("chacha20-ietf-poly1305".to_string())
        );
        assert_eq!(node.password, Some("pass:word".to_string()));
    }

    #[test]
    fn test_explode_ss_with_non_plugin_query() {
        // Test URL with query parameters other than plugin
        let ss_with_query = "ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpwYXNzd29yZA==@example.com:8388/?foo=bar&plugin=obfs-local;obfs=http";
        let mut node = Proxy::default();
        let result = explode_ss(ss_with_query, &mut node);

        assert!(result);
        assert_eq!(node.proxy_type, ProxyType::Shadowsocks);
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 8388);
        assert_eq!(node.plugin, Some("obfs-local".to_string()));
        assert_eq!(node.plugin_option, Some("obfs=http".to_string()));
        // Ignores non-plugin query params
    }

    #[test]
    fn test_explode_ss_malformed_plugin() {
        // Test URL with malformed plugin parameter
        let ss_with_malformed_plugin = "ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpwYXNzd29yZA==@example.com:8388/?plugin=obfs-local;";
        let mut node = Proxy::default();
        let result = explode_ss(ss_with_malformed_plugin, &mut node);

        assert!(result);
        assert_eq!(node.proxy_type, ProxyType::Shadowsocks);
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 8388);
        assert_eq!(node.plugin, Some("obfs-local".to_string()));
        assert_eq!(node.plugin_option, Some("".to_string()));
    }

    #[test]
    fn test_explode_ssd_malformed_json() {
        // Test SSD with malformed JSON
        let ssd_invalid = "ssd://eyJhaXJwb3J0IjoiRXhhbXBsZSBTZXJ2ZXIiLCJwb3J0Ijo4Mzg4LCJlbmNyeXB0aW9uIjoiY2hhY2hhMjAtaWV0Zi1wb2x5MTMwNSIsInBhc3N3b3JkIjoicGFzc3dvcmQiLCJzZXJ2ZXJ";
        let mut nodes = Vec::new();
        let result = explode_ssd(ssd_invalid, &mut nodes);

        assert!(!result);
        assert_eq!(nodes.len(), 0);
    }

    #[test]
    fn test_explode_ssd_missing_servers() {
        // Test SSD with missing servers field
        let ssd_missing_servers = "ssd://eyJhaXJwb3J0IjoiRXhhbXBsZSBTZXJ2ZXIiLCJwb3J0Ijo4Mzg4LCJlbmNyeXB0aW9uIjoiY2hhY2hhMjAtaWV0Zi1wb2x5MTMwNSIsInBhc3N3b3JkIjoicGFzc3dvcmQifQ==";
        let mut nodes = Vec::new();
        let result = explode_ssd(ssd_missing_servers, &mut nodes);

        assert!(!result);
        assert_eq!(nodes.len(), 0);
    }

    #[test]
    fn test_explode_ss_android_proxies_format() {
        // Test Android config with "proxies" array instead of "configs"
        let ss_android_proxies = r#"{
            "proxies": [
                {
                    "server": "example.com",
                    "server_port": 8388,
                    "password": "password",
                    "method": "chacha20-ietf-poly1305",
                    "name": "Test Proxy"
                }
            ]
        }"#;

        let mut nodes = Vec::new();
        let result = explode_ss_android(ss_android_proxies, &mut nodes);

        assert!(result);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].proxy_type, ProxyType::Shadowsocks);
        assert_eq!(nodes[0].hostname, "example.com");
        assert_eq!(nodes[0].port, 8388);
        assert_eq!(nodes[0].remark, "Test Proxy");
    }

    #[test]
    fn test_explode_ss_android_with_name_field() {
        // Test Android config using "name" field instead of "remarks"
        let ss_android_name = r#"{
            "configs": [
                {
                    "server": "example.com",
                    "server_port": 8388,
                    "password": "password",
                    "method": "chacha20-ietf-poly1305",
                    "name": "Custom Name"
                }
            ]
        }"#;

        let mut nodes = Vec::new();
        let result = explode_ss_android(ss_android_name, &mut nodes);

        assert!(result);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].remark, "Custom Name");
    }

    #[test]
    fn test_explode_ss_android_text_format() {
        // Test Android text format (non-JSON)
        let ss_android_text = r#"ss://YWVzLTI1Ni1nY206cGFzc3dvcmQxMjM=@example.com:8388
ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpwYXNzd29yZA==@example2.com:8389#TestServer"#;

        let mut nodes = Vec::new();
        let result = explode_ss_android(ss_android_text, &mut nodes);

        assert!(result);
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].hostname, "example.com");
        assert_eq!(nodes[1].hostname, "example2.com");
        assert_eq!(nodes[1].remark, "TestServer");
    }

    #[test]
    fn test_explode_ss_conf_invalid_json() {
        // Test SS conf with invalid JSON
        let invalid_json = r#"{
            "server": "example.com",
            "server_port": 8388,
            "method": "chacha20-ietf-poly1305",
            "password": "password",
            "remarks": "Test Server"
        "#; // Missing closing bracket

        let mut nodes = Vec::new();
        let result = explode_ss_conf(invalid_json, &mut nodes);

        assert!(!result);
        assert_eq!(nodes.len(), 0);
    }

    #[test]
    fn test_explode_ss_conf_missing_required_fields() {
        // Test SS conf with missing required fields
        let missing_fields = r#"{
            "server": "example.com",
            "method": "chacha20-ietf-poly1305",
            "password": "password"
        }"#; // Missing server_port

        let mut nodes = Vec::new();
        let result = explode_ss_conf(missing_fields, &mut nodes);

        assert!(!result);
        assert_eq!(nodes.len(), 0);
    }

    #[test]
    fn test_explode_ss_android_ss_base64_format() {
        // Test Android SS-Base64 format
        let ss_base64_format = r#"port=8388 password=password method=aes-256-gcm remarks=TestServer=ss-base64=MjA5LjU4LjE4OC42Nw=="#;

        let mut nodes = Vec::new();
        let result = explode_ss_android(ss_base64_format, &mut nodes);

        // Note: This will fail currently since the ss-base64 content processing is commented out
        // This test is included for completeness, but implementation needs to be completed
        assert!(!result);
    }

    #[test]
    fn test_explode_ss_special_characters_in_password() {
        // Test with special characters in password
        let special_chars_pass =
            "ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpwQHNzdzByZCFAIyQ=@example.com:8388";
        let mut node = Proxy::default();
        let result = explode_ss(special_chars_pass, &mut node);

        assert!(result);
        assert_eq!(node.password, Some("p@ssw0rd!@#$".to_string()));
    }

    #[test]
    fn test_explode_ss_multiple_semicolons_in_plugin_opts() {
        // Test with multiple semicolons in plugin options
        let multiple_semicolons = "ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpwYXNzd29yZA==@example.com:8388/?plugin=obfs-local;obfs=http;obfs-host=example.com;tls=1";
        let mut node = Proxy::default();
        let result = explode_ss(multiple_semicolons, &mut node);

        assert!(result);
        assert_eq!(node.plugin, Some("obfs-local".to_string()));
        assert_eq!(
            node.plugin_option,
            Some("obfs=http;obfs-host=example.com;tls=1".to_string())
        );
    }
}
