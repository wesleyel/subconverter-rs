use crate::parser::proxy::{Proxy, ProxyType, SSR_DEFAULT_GROUP};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::collections::HashMap;
use url::Url;

/// Parse a ShadowsocksR link into a Proxy object
pub fn explode_ssr(ssr: &str, node: &mut Proxy) -> bool {
    // Check if the link starts with ssr://
    if !ssr.starts_with("ssr://") {
        return false;
    }

    // Extract the base64 part
    let encoded = &ssr[6..];

    // Decode base64
    let decoded = match STANDARD.decode(encoded) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(s) => s,
            Err(_) => return false,
        },
        Err(_) => return false,
    };

    // Split the decoded string by ":"
    let parts: Vec<&str> = decoded.split(':').collect();
    if parts.len() < 6 {
        return false;
    }

    // Extract the main components
    let server = parts[0];
    let port_str = parts[1];
    let protocol = parts[2];
    let method = parts[3];
    let obfs = parts[4];

    // The remaining part may contain the password and parameters
    let remaining = parts[5..].join(":");
    let remaining_parts: Vec<&str> = remaining.split('/').collect();
    if remaining_parts.is_empty() {
        return false;
    }

    // Extract password (base64 encoded)
    let password_encoded = remaining_parts[0];
    let password = match STANDARD.decode(password_encoded) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(s) => s,
            Err(_) => return false,
        },
        Err(_) => return false,
    };

    // Parse port
    let port = match port_str.parse::<u16>() {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Default values for optional parameters
    let mut obfs_param = String::new();
    let mut protocol_param = String::new();
    let mut remark = format!("{} ({})", server, port);

    // Parse query parameters if present
    if remaining_parts.len() > 1 && !remaining_parts[1].is_empty() {
        let query = format!("?{}", remaining_parts[1]);
        if let Ok(url) = Url::parse(&format!("http://localhost/{}", query)) {
            for (key, value) in url.query_pairs() {
                let value_decoded = match STANDARD.decode(value.as_bytes()) {
                    Ok(decoded) => match String::from_utf8(decoded) {
                        Ok(s) => s,
                        Err(_) => continue,
                    },
                    Err(_) => continue,
                };

                match key.as_ref() {
                    "obfsparam" => obfs_param = value_decoded,
                    "protoparam" => protocol_param = value_decoded,
                    "remarks" => remark = value_decoded,
                    _ => {}
                }
            }
        }
    }

    // Create the proxy object
    *node = Proxy::ssr_construct(
        "SSR",
        &remark,
        server,
        port,
        protocol,
        method,
        obfs,
        &password,
        &obfs_param,
        &protocol_param,
        None,
        None,
        None,
        "",
    );

    true
}

/// Parse a ShadowsocksR configuration file into a vector of Proxy objects
pub fn explode_ssr_conf(content: &str, nodes: &mut Vec<Proxy>) -> bool {
    // Try to parse as JSON
    let json: Value = match serde_json::from_str(content) {
        Ok(json) => json,
        Err(_) => return false,
    };

    // Check if it's a ShadowsocksR configuration
    if !json["configs"].is_array() {
        return false;
    }

    // Extract configs
    let configs = json["configs"].as_array().unwrap();

    for config in configs {
        let server = config["server"].as_str().unwrap_or("");
        let port = config["server_port"].as_u64().unwrap_or(0) as u16;
        let protocol = config["protocol"].as_str().unwrap_or("");
        let method = config["method"].as_str().unwrap_or("");
        let obfs = config["obfs"].as_str().unwrap_or("");
        let password = config["password"].as_str().unwrap_or("");
        let obfs_param = config["obfsparam"].as_str().unwrap_or("");
        let proto_param = config["protocolparam"].as_str().unwrap_or("");
        let remarks = config["remarks"].as_str().unwrap_or("");
        let group = config["group"].as_str().unwrap_or("");

        // Create formatted remark and group
        let group_str = if group.is_empty() {
            SSR_DEFAULT_GROUP.to_string()
        } else {
            group.to_string()
        };
        let remark_str = if remarks.is_empty() {
            format!("{} ({})", server, port)
        } else {
            remarks.to_string()
        };

        // Create the proxy object
        let node = Proxy::ssr_construct(
            &group_str,
            &remark_str,
            server,
            port,
            protocol,
            method,
            obfs,
            password,
            obfs_param,
            proto_param,
            None,
            None,
            None,
            "",
        );

        nodes.push(node);
    }

    !nodes.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine};

    #[test]
    fn test_explode_ssr_valid_link() {
        let mut node = Proxy::default();

        // This is a valid SSR link with known parameters
        let ssr_link = "ssr://ZXhhbXBsZS5jb206ODM4ODphdXRoX2FlczEyOF9tZDU6YWVzLTI1Ni1jZmI6dGxzMS4yX3RpY2tldF9hdXRoOmRHVnpkQT09Lz9vYmZzcGFyYW09ZEdWemRBPT0mcHJvdG9wYXJhbT1kR1Z6ZEE9PSZyZW1hcmtzPVZHVnpkQ0JUVTFJPSZncm91cD1WR1Z6ZENCVFUxST0=";

        // Parse the link
        let result = explode_ssr(ssr_link, &mut node);

        // Verify the result
        assert!(result);
        assert_eq!(node.proxy_type, ProxyType::ShadowsocksR);
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 8388);
        assert_eq!(node.protocol.as_deref().unwrap_or(""), "auth_aes128_md5");
        assert_eq!(node.encrypt_method.as_deref().unwrap_or(""), "aes-256-cfb");
        assert_eq!(node.obfs.as_deref().unwrap_or(""), "tls1.2_ticket_auth");
        assert_eq!(node.password.as_deref().unwrap_or(""), "test");
        assert_eq!(node.obfs_param.as_deref().unwrap_or(""), "test");
        assert_eq!(node.protocol_param.as_deref().unwrap_or(""), "test");
        assert_eq!(node.remark, "Test SSR");
        assert_eq!(node.group, "Test SSR");
    }

    #[test]
    fn test_explode_ssr_invalid_prefix() {
        let mut node = Proxy::default();
        let result = explode_ssr("ss://invalid", &mut node);
        assert!(!result);
    }

    #[test]
    fn test_explode_ssr_invalid_base64() {
        let mut node = Proxy::default();
        let result = explode_ssr("ssr://invalid!base64", &mut node);
        assert!(!result);
    }

    #[test]
    fn test_explode_ssr_missing_parts() {
        let mut node = Proxy::default();
        // Only server:port:protocol
        let link = format!(
            "ssr://{}",
            STANDARD.encode("example.com:8388:auth_aes128_md5")
        );
        let result = explode_ssr(&link, &mut node);
        assert!(!result);
    }

    #[test]
    fn test_explode_ssr_default_group() {
        let mut node = Proxy::default();
        let server = "example.com";
        let port = 8388;
        let protocol = "auth_aes128_md5";
        let method = "aes-256-cfb";
        let obfs = "tls1.2_ticket_auth";
        let password = "password123";

        // Encode the password
        let password_b64 = STANDARD.encode(password);

        // Construct the SSR link without group
        let ssr_link = format!(
            "{}:{}:{}:{}:{}:{}",
            server, port, protocol, method, obfs, password_b64
        );

        // Base64 encode the entire link
        let ssr_link_b64 = format!("ssr://{}", STANDARD.encode(&ssr_link));

        // Parse the link
        let result = explode_ssr(&ssr_link_b64, &mut node);

        // Verify the result
        assert!(result);
        assert_eq!(node.group, SSR_DEFAULT_GROUP);
        assert_eq!(node.remark, format!("{} ({})", server, port));
    }

    #[test]
    fn test_explode_ssr_conf_valid() {
        let mut nodes = Vec::new();
        let content = r#"{
            "configs": [
                {
                    "server": "example1.com",
                    "server_port": 8388,
                    "protocol": "auth_aes128_md5",
                    "method": "aes-256-cfb",
                    "obfs": "tls1.2_ticket_auth",
                    "password": "password1",
                    "obfsparam": "obfs.param1",
                    "protocolparam": "proto.param1",
                    "remarks": "Server 1",
                    "group": "Group 1"
                },
                {
                    "server": "example2.com",
                    "server_port": 8389,
                    "protocol": "auth_chain_a",
                    "method": "chacha20",
                    "obfs": "http_simple",
                    "password": "password2",
                    "obfsparam": "obfs.param2",
                    "protocolparam": "proto.param2",
                    "remarks": "Server 2",
                    "group": "Group 2"
                }
            ]
        }"#;

        let result = explode_ssr_conf(content, &mut nodes);

        assert!(result);
        assert_eq!(nodes.len(), 2);

        // Check first node
        assert_eq!(nodes[0].proxy_type, ProxyType::ShadowsocksR);
        assert_eq!(nodes[0].hostname, "example1.com");
        assert_eq!(nodes[0].port, 8388);
        assert_eq!(
            nodes[0].protocol.as_deref().unwrap_or(""),
            "auth_aes128_md5"
        );
        assert_eq!(
            nodes[0].encrypt_method.as_deref().unwrap_or(""),
            "aes-256-cfb"
        );
        assert_eq!(nodes[0].obfs.as_deref().unwrap_or(""), "tls1.2_ticket_auth");
        assert_eq!(nodes[0].password.as_deref().unwrap_or(""), "password1");
        assert_eq!(nodes[0].obfs_param.as_deref().unwrap_or(""), "obfs.param1");
        assert_eq!(
            nodes[0].protocol_param.as_deref().unwrap_or(""),
            "proto.param1"
        );
        assert_eq!(nodes[0].remark, "Server 1");
        assert_eq!(nodes[0].group, "Group 1");

        // Check second node
        assert_eq!(nodes[1].proxy_type, ProxyType::ShadowsocksR);
        assert_eq!(nodes[1].hostname, "example2.com");
        assert_eq!(nodes[1].port, 8389);
        assert_eq!(nodes[1].protocol.as_deref().unwrap_or(""), "auth_chain_a");
        assert_eq!(nodes[1].encrypt_method.as_deref().unwrap_or(""), "chacha20");
        assert_eq!(nodes[1].obfs.as_deref().unwrap_or(""), "http_simple");
        assert_eq!(nodes[1].password.as_deref().unwrap_or(""), "password2");
        assert_eq!(nodes[1].obfs_param.as_deref().unwrap_or(""), "obfs.param2");
        assert_eq!(
            nodes[1].protocol_param.as_deref().unwrap_or(""),
            "proto.param2"
        );
        assert_eq!(nodes[1].remark, "Server 2");
        assert_eq!(nodes[1].group, "Group 2");
    }

    #[test]
    fn test_explode_ssr_conf_invalid_json() {
        let mut nodes = Vec::new();
        let content = "invalid json";
        let result = explode_ssr_conf(content, &mut nodes);
        assert!(!result);
        assert_eq!(nodes.len(), 0);
    }

    #[test]
    fn test_explode_ssr_conf_missing_configs() {
        let mut nodes = Vec::new();
        let content = r#"{ "not_configs": [] }"#;
        let result = explode_ssr_conf(content, &mut nodes);
        assert!(!result);
        assert_eq!(nodes.len(), 0);
    }

    #[test]
    fn test_explode_ssr_conf_empty_configs() {
        let mut nodes = Vec::new();
        let content = r#"{ "configs": [] }"#;
        let result = explode_ssr_conf(content, &mut nodes);
        assert!(!result);
        assert_eq!(nodes.len(), 0);
    }

    #[test]
    fn test_explode_ssr_conf_default_values() {
        let mut nodes = Vec::new();
        let content = r#"{
            "configs": [
                {
                    "server": "example.com",
                    "server_port": 8388
                }
            ]
        }"#;

        let result = explode_ssr_conf(content, &mut nodes);

        assert!(result);
        assert_eq!(nodes.len(), 1);

        assert_eq!(nodes[0].proxy_type, ProxyType::ShadowsocksR);
        assert_eq!(nodes[0].hostname, "example.com");
        assert_eq!(nodes[0].port, 8388);
        assert_eq!(nodes[0].protocol.as_deref().unwrap_or(""), "");
        assert_eq!(nodes[0].encrypt_method.as_deref().unwrap_or(""), "");
        assert_eq!(nodes[0].obfs.as_deref().unwrap_or(""), "");
        assert_eq!(nodes[0].password.as_deref().unwrap_or(""), "");
        assert_eq!(nodes[0].obfs_param.as_deref().unwrap_or(""), "");
        assert_eq!(nodes[0].protocol_param.as_deref().unwrap_or(""), "");
        assert_eq!(nodes[0].remark, "example.com (8388)");
        assert_eq!(nodes[0].group, SSR_DEFAULT_GROUP);
    }
}
