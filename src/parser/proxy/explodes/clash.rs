use crate::parser::proxy::{
    Proxy, ProxyType, HTTP_DEFAULT_GROUP, SNELL_DEFAULT_GROUP, SOCKS_DEFAULT_GROUP,
    SSR_DEFAULT_GROUP, SS_DEFAULT_GROUP, TROJAN_DEFAULT_GROUP, V2RAY_DEFAULT_GROUP,
};
use serde_yaml::Value;
use std::collections::HashMap;

/// Parse a Clash YAML configuration into a vector of Proxy objects
pub fn explode_clash(content: &str, nodes: &mut Vec<Proxy>) -> bool {
    // Parse the YAML content
    let yaml: Value = match serde_yaml::from_str(content) {
        Ok(y) => y,
        Err(_) => return false,
    };

    // Extract proxies section
    let proxies = match yaml.get("proxies") {
        Some(Value::Sequence(seq)) => seq,
        _ => match yaml.get("Proxy") {
            Some(Value::Sequence(seq)) => seq,
            _ => return false,
        },
    };

    let mut success = false;

    // Process each proxy in the sequence
    for proxy in proxies {
        if let Some(node) = parse_clash_proxy(proxy) {
            nodes.push(node);
            success = true;
        }
    }

    success
}

/// Parse a single proxy from Clash YAML
fn parse_clash_proxy(proxy: &Value) -> Option<Proxy> {
    // Extract the proxy type
    let proxy_type = match proxy.get("type") {
        Some(Value::String(t)) => t.to_lowercase(),
        _ => return None,
    };

    // Extract common fields
    let name = proxy.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let server = proxy.get("server").and_then(|v| v.as_str()).unwrap_or("");
    let port_value = proxy.get("port").and_then(|v| v.as_u64()).unwrap_or(0);
    let port = port_value as u16;

    // Skip if missing essential information
    if name.is_empty() || server.is_empty() || port == 0 {
        return None;
    }

    // Extract common optional fields
    let udp = proxy.get("udp").and_then(|v| v.as_bool());
    let tfo = proxy.get("tfo").and_then(|v| v.as_bool());
    let skip_cert_verify = proxy.get("skip-cert-verify").and_then(|v| v.as_bool());

    // Process based on proxy type
    match proxy_type.as_str() {
        "ss" | "shadowsocks" => {
            parse_clash_ss(proxy, name, server, port, udp, tfo, skip_cert_verify)
        }
        "ssr" | "shadowsocksr" => {
            parse_clash_ssr(proxy, name, server, port, udp, tfo, skip_cert_verify)
        }
        "vmess" => parse_clash_vmess(proxy, name, server, port, udp, tfo, skip_cert_verify),
        "socks" | "socks5" => {
            parse_clash_socks(proxy, name, server, port, udp, tfo, skip_cert_verify)
        }
        "http" => parse_clash_http(proxy, name, server, port, false, tfo, skip_cert_verify),
        "https" => parse_clash_http(proxy, name, server, port, true, tfo, skip_cert_verify),
        "trojan" => parse_clash_trojan(proxy, name, server, port, udp, tfo, skip_cert_verify),
        "snell" => parse_clash_snell(proxy, name, server, port, udp, tfo, skip_cert_verify),
        _ => None,
    }
}

/// Parse a Shadowsocks proxy from Clash YAML
fn parse_clash_ss(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    udp: Option<bool>,
    tfo: Option<bool>,
    skip_cert_verify: Option<bool>,
) -> Option<Proxy> {
    // Extract SS-specific fields
    let password = proxy.get("password").and_then(|v| v.as_str()).unwrap_or("");
    let method = proxy.get("cipher").and_then(|v| v.as_str()).unwrap_or("");

    if password.is_empty() || method.is_empty() {
        return None;
    }

    // Extract plugin information
    let mut plugin = "";
    let mut plugin_opts = "";

    if let Some(Value::String(plugin_val)) = proxy.get("plugin") {
        plugin = plugin_val;

        if let Some(Value::String(plugin_opts_val)) = proxy.get("plugin-opts") {
            plugin_opts = plugin_opts_val;
        } else if let Some(Value::Mapping(opts_map)) = proxy.get("plugin-opts") {
            // Parse plugin options from mapping
            let mut opts = String::new();

            if let Some(Value::String(mode)) = opts_map.get(&Value::String("mode".to_string())) {
                opts.push_str(&format!("obfs={};", mode));
            }

            if let Some(Value::String(host)) = opts_map.get(&Value::String("host".to_string())) {
                opts.push_str(&format!("obfs-host={}", host));
            }

            plugin_opts = Box::leak(opts.into_boxed_str());
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
        skip_cert_verify,
        None,
        "",
    ))
}

/// Parse a ShadowsocksR proxy from Clash YAML
fn parse_clash_ssr(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    udp: Option<bool>,
    tfo: Option<bool>,
    skip_cert_verify: Option<bool>,
) -> Option<Proxy> {
    // Extract SSR-specific fields
    let password = proxy.get("password").and_then(|v| v.as_str()).unwrap_or("");
    let method = proxy.get("cipher").and_then(|v| v.as_str()).unwrap_or("");
    let protocol = proxy.get("protocol").and_then(|v| v.as_str()).unwrap_or("");
    let protocol_param = proxy
        .get("protocol-param")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let obfs = proxy.get("obfs").and_then(|v| v.as_str()).unwrap_or("");
    let obfs_param = proxy
        .get("obfs-param")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if password.is_empty() || method.is_empty() || protocol.is_empty() || obfs.is_empty() {
        return None;
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
        skip_cert_verify,
        "",
    ))
}

/// Parse a VMess proxy from Clash YAML
fn parse_clash_vmess(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    udp: Option<bool>,
    tfo: Option<bool>,
    skip_cert_verify: Option<bool>,
) -> Option<Proxy> {
    // Extract VMess-specific fields
    let uuid = proxy.get("uuid").and_then(|v| v.as_str()).unwrap_or("");
    let alter_id_val = proxy.get("alterId").and_then(|v| v.as_u64()).unwrap_or(0);
    let alter_id = alter_id_val as u16;
    let cipher = proxy
        .get("cipher")
        .and_then(|v| v.as_str())
        .unwrap_or("auto");

    if uuid.is_empty() {
        return None;
    }

    // Get network settings
    let network = proxy
        .get("network")
        .and_then(|v| v.as_str())
        .unwrap_or("tcp");

    // Get TLS settings
    let tls = proxy.get("tls").and_then(|v| v.as_bool()).unwrap_or(false);
    let sni = proxy
        .get("servername")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Create a basic proxy object for VMess
    let mut node = Proxy::default();
    node.proxy_type = ProxyType::VMess;
    node.group = V2RAY_DEFAULT_GROUP.to_string();
    node.remark = name.to_string();
    node.hostname = server.to_string();
    node.port = port;
    node.user_id = Some(uuid.to_string());
    node.alter_id = alter_id;
    node.encrypt_method = Some(cipher.to_string());
    node.transfer_protocol = Some(network.to_string());
    node.tls_secure = tls;
    node.fake_type = Some("".to_string()); // Default empty type

    // Add optional parameters if present
    if !sni.is_empty() {
        node.server_name = Some(sni.to_string());
    }

    node.udp = udp;
    node.tcp_fast_open = tfo;
    node.allow_insecure = skip_cert_verify;

    // Parse network specific options
    let mut host = String::new();
    let mut path = String::new();

    // Handle WebSocket options
    if let Some(ws_opts) = proxy.get("ws-opts").and_then(|v| v.as_mapping()) {
        if let Some(path_val) = ws_opts
            .get(&Value::String("path".to_string()))
            .and_then(|v| v.as_str())
        {
            path = path_val.to_string();
        }

        if let Some(headers) = ws_opts
            .get(&Value::String("headers".to_string()))
            .and_then(|v| v.as_mapping())
        {
            if let Some(host_val) = headers
                .get(&Value::String("Host".to_string()))
                .and_then(|v| v.as_str())
            {
                host = host_val.to_string();
            }
        }
    }
    // Handle HTTP/2 options
    else if let Some(h2_opts) = proxy.get("h2-opts").and_then(|v| v.as_mapping()) {
        if let Some(path_val) = h2_opts
            .get(&Value::String("path".to_string()))
            .and_then(|v| v.as_str())
        {
            path = path_val.to_string();
        }

        if let Some(hosts) = h2_opts
            .get(&Value::String("host".to_string()))
            .and_then(|v| v.as_sequence())
        {
            if !hosts.is_empty() {
                if let Some(first_host) = hosts.get(0).and_then(|v| v.as_str()) {
                    host = first_host.to_string();
                }
            }
        }
    }
    // Handle HTTP options
    else if let Some(http_opts) = proxy.get("http-opts").and_then(|v| v.as_mapping()) {
        if let Some(paths) = http_opts
            .get(&Value::String("path".to_string()))
            .and_then(|v| v.as_sequence())
        {
            if !paths.is_empty() {
                if let Some(first_path) = paths.get(0).and_then(|v| v.as_str()) {
                    path = first_path.to_string();
                }
            }
        }

        if let Some(hosts) = http_opts
            .get(&Value::String("host".to_string()))
            .and_then(|v| v.as_sequence())
        {
            if !hosts.is_empty() {
                if let Some(first_host) = hosts.get(0).and_then(|v| v.as_str()) {
                    host = first_host.to_string();
                }
            }
        }
    }
    // Handle gRPC options
    else if let Some(grpc_opts) = proxy.get("grpc-opts").and_then(|v| v.as_mapping()) {
        if let Some(service_name) = grpc_opts
            .get(&Value::String("grpc-service-name".to_string()))
            .and_then(|v| v.as_str())
        {
            path = service_name.to_string();
        }
    }

    // Set host and path
    node.host = Some(host);
    node.path = Some(if path.is_empty() {
        "/".to_string()
    } else {
        path
    });

    Some(node)
}

/// Parse a SOCKS5 proxy from Clash YAML
fn parse_clash_socks(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    udp: Option<bool>,
    tfo: Option<bool>,
    skip_cert_verify: Option<bool>,
) -> Option<Proxy> {
    // Extract SOCKS-specific fields
    let username = proxy.get("username").and_then(|v| v.as_str()).unwrap_or("");
    let password = proxy.get("password").and_then(|v| v.as_str()).unwrap_or("");

    Some(Proxy::socks_construct(
        SOCKS_DEFAULT_GROUP,
        name,
        server,
        port,
        username,
        password,
        udp,
        tfo,
        skip_cert_verify,
        "",
    ))
}

/// Parse an HTTP/HTTPS proxy from Clash YAML
fn parse_clash_http(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    is_https: bool,
    tfo: Option<bool>,
    skip_cert_verify: Option<bool>,
) -> Option<Proxy> {
    // Extract HTTP-specific fields
    let username = proxy.get("username").and_then(|v| v.as_str()).unwrap_or("");
    let password = proxy.get("password").and_then(|v| v.as_str()).unwrap_or("");

    Some(Proxy::http_construct(
        HTTP_DEFAULT_GROUP,
        name,
        server,
        port,
        username,
        password,
        is_https,
        tfo,
        skip_cert_verify,
        None,
        "",
    ))
}

/// Parse a Trojan proxy from Clash YAML
fn parse_clash_trojan(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    udp: Option<bool>,
    tfo: Option<bool>,
    skip_cert_verify: Option<bool>,
) -> Option<Proxy> {
    // Extract Trojan-specific fields
    let password = proxy.get("password").and_then(|v| v.as_str()).unwrap_or("");

    if password.is_empty() {
        return None;
    }

    // Get SNI
    let sni = proxy.get("sni").and_then(|v| v.as_str()).unwrap_or("");
    let alpn = proxy.get("alpn").and_then(|v| v.as_str()).unwrap_or("");

    // Create a direct Proxy struct
    let mut node = Proxy {
        proxy_type: ProxyType::Trojan,
        group: TROJAN_DEFAULT_GROUP.to_string(),
        remark: name.to_string(),
        hostname: server.to_string(),
        port,
        password: Some(password.to_string()),
        tls_secure: true,
        server_name: if sni.is_empty() {
            None
        } else {
            Some(sni.to_string())
        },
        udp,
        tcp_fast_open: tfo,
        allow_insecure: skip_cert_verify,
        ..Default::default()
    };

    Some(node)
}

/// Parse a Snell proxy from Clash YAML
fn parse_clash_snell(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    udp: Option<bool>,
    tfo: Option<bool>,
    skip_cert_verify: Option<bool>,
) -> Option<Proxy> {
    // Extract Snell-specific fields
    let psk = proxy.get("psk").and_then(|v| v.as_str()).unwrap_or("");

    if psk.is_empty() {
        return None;
    }

    // Get obfs settings
    let version = proxy.get("version").and_then(|v| v.as_u64()).unwrap_or(1) as u16;
    let obfs = proxy.get("obfs").and_then(|v| v.as_str()).unwrap_or("");
    let obfs_host = proxy
        .get("obfs-host")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Create a direct Proxy struct
    let mut node = Proxy {
        proxy_type: ProxyType::Snell,
        group: SNELL_DEFAULT_GROUP.to_string(),
        remark: name.to_string(),
        hostname: server.to_string(),
        port,
        password: Some(psk.to_string()),
        obfs: Some(obfs.to_string()),
        host: Some(obfs_host.to_string()),
        snell_version: version,
        udp,
        tcp_fast_open: tfo,
        allow_insecure: skip_cert_verify,
        ..Default::default()
    };

    Some(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

    // Helper function to create a basic YAML Value for VMess testing
    fn create_vmess_yaml() -> Value {
        let yaml_str = r#"
        {
          "name": "VMess Test",
          "type": "vmess",
          "server": "example.com",
          "port": 443,
          "uuid": "a3c14e2a-a37f-11ec-b909-0242ac120002",
          "alterId": 0,
          "cipher": "auto",
          "network": "ws",
          "tls": true,
          "servername": "example.com",
          "ws-opts": {
            "path": "/path",
            "headers": {
              "Host": "example.com"
            }
          },
          "udp": true
        }
        "#;
        serde_json::from_str(yaml_str).unwrap()
    }

    #[test]
    fn test_parse_clash_vmess() {
        let vmess_yaml = create_vmess_yaml();
        let node = parse_clash_vmess(
            &vmess_yaml,
            "VMess Test",
            "example.com",
            443,
            Some(true),
            None,
            None,
        )
        .unwrap();

        assert_eq!(node.proxy_type, ProxyType::VMess);
        assert_eq!(node.remark, "VMess Test");
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 443);
        assert_eq!(
            node.user_id,
            Some("a3c14e2a-a37f-11ec-b909-0242ac120002".to_string())
        );
        assert_eq!(node.alter_id, 0);
        assert_eq!(node.encrypt_method, Some("auto".to_string()));
        assert_eq!(node.transfer_protocol, Some("ws".to_string()));
        assert_eq!(node.tls_secure, true);
        assert_eq!(node.path, Some("/path".to_string()));
        assert_eq!(node.host, Some("example.com".to_string()));
        assert_eq!(node.server_name, Some("example.com".to_string()));
        assert_eq!(node.udp, Some(true));
    }

    #[test]
    fn test_explode_clash() {
        let yaml_str = r#"
        proxies:
          - name: "SS Test"
            type: ss
            server: example.com
            port: 8388
            cipher: aes-256-gcm
            password: "password123"
            udp: true
          - name: "VMess Test"
            type: vmess
            server: example.org
            port: 443
            uuid: a3c14e2a-a37f-11ec-b909-0242ac120002
            alterId: 0
            cipher: auto
            network: ws
            tls: true
            servername: example.org
            ws-opts:
              path: /path
              headers:
                Host: example.org
        "#;

        let mut nodes = Vec::new();
        let result = explode_clash(yaml_str, &mut nodes);

        assert!(result);
        assert_eq!(nodes.len(), 2);

        // Check SS node
        let ss_node = &nodes[0];
        assert_eq!(ss_node.proxy_type, ProxyType::Shadowsocks);
        assert_eq!(ss_node.remark, "SS Test");
        assert_eq!(ss_node.hostname, "example.com");
        assert_eq!(ss_node.port, 8388);
        assert_eq!(ss_node.encrypt_method, Some("aes-256-gcm".to_string()));
        assert_eq!(ss_node.password, Some("password123".to_string()));
        assert_eq!(ss_node.udp, Some(true));

        // Check VMess node
        let vmess_node = &nodes[1];
        assert_eq!(vmess_node.proxy_type, ProxyType::VMess);
        assert_eq!(vmess_node.remark, "VMess Test");
        assert_eq!(vmess_node.hostname, "example.org");
        assert_eq!(vmess_node.port, 443);
        assert_eq!(
            vmess_node.user_id,
            Some("a3c14e2a-a37f-11ec-b909-0242ac120002".to_string())
        );
        assert_eq!(vmess_node.transfer_protocol, Some("ws".to_string()));
        assert_eq!(vmess_node.tls_secure, true);
    }
}
