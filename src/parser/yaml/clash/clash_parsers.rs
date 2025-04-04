use crate::models::Proxy;
use crate::parser::yaml::clash::clash_proxy_types::ClashProxyYamlInput;

use super::ClashYamlInput;

/// Parse Clash configuration from YAML string
///
/// This function is the Rust equivalent of the C++ `explodeClash` function.
/// The key improvements in this Rust implementation are:
/// 1. Type safety through enum variants in ClashProxyYamlInput
/// 2. Proper error handling with Result type
/// 3. Automatic deserialization using serde
/// 4. Cleaner pattern matching compared to C++ if/else chains
pub fn parse_clash_yaml(content: &str) -> Result<Vec<Proxy>, String> {
    let clash_input: ClashYamlInput = match serde_yaml::from_str(content) {
        Ok(input) => input,
        Err(e) => return Err(format!("Failed to parse Clash YAML: {}", e)),
    };

    let mut proxies = Vec::new();

    for proxy in clash_input.extract_proxies() {
        match proxy {
            ClashProxyYamlInput::Shadowsocks(ss) => {
                proxies.push(ss.into());
            }
            ClashProxyYamlInput::ShadowsocksR(ssr) => {
                proxies.push(ssr.into());
            }
            ClashProxyYamlInput::VMess(vmess) => {
                proxies.push(vmess.into());
            }
            ClashProxyYamlInput::Trojan(trojan) => {
                proxies.push(trojan.into());
            }
            ClashProxyYamlInput::Http(http) => {
                proxies.push(http.into());
            }
            ClashProxyYamlInput::Socks5(socks5) => {
                proxies.push(socks5.into());
            }
            ClashProxyYamlInput::Snell(snell) => {
                proxies.push(snell.into());
            }
            ClashProxyYamlInput::WireGuard(wg) => {
                proxies.push(wg.into());
            }
            ClashProxyYamlInput::Hysteria(hysteria) => {
                proxies.push(hysteria.into());
            }
            ClashProxyYamlInput::Hysteria2(hysteria2) => {
                proxies.push(hysteria2.into());
            }
            ClashProxyYamlInput::VLess(vless) => {
                proxies.push(vless.into());
            }
            ClashProxyYamlInput::Unknown => {
                // Skip unknown proxy types
            }
        }
    }

    Ok(proxies)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shadowsocks() {
        let yaml = r#"
proxies:
  - type: ss
    name: "Test SS"
    server: "example.com"
    port: 8388
    cipher: "aes-256-gcm"
    password: "password"
    udp: true
    tfo: true
    skip-cert-verify: true
"#;
        let proxies = parse_clash_yaml(yaml).unwrap();
        assert_eq!(proxies.len(), 1);
        let proxy = &proxies[0];
        assert_eq!(proxy.proxy_type, ProxyType::Shadowsocks);
        assert_eq!(proxy.remark, "Test SS");
        assert_eq!(proxy.hostname, "example.com");
        assert_eq!(proxy.port, 8388);
        assert_eq!(proxy.encrypt_method, Some("aes-256-gcm".to_string()));
        assert_eq!(proxy.password, Some("password".to_string()));
        assert_eq!(proxy.udp, Some(true));
        assert_eq!(proxy.tcp_fast_open, Some(true));
        assert_eq!(proxy.allow_insecure, Some(true));
    }

    #[test]
    fn test_parse_vmess() {
        let yaml = r#"
proxies:
  - type: vmess
    name: "Test VMess"
    server: "example.com"
    port: 443
    uuid: "b831381d-6324-4d53-ad4f-8cda48b30811"
    alterId: 0
    cipher: "auto"
    udp: true
    tfo: true
    skip-cert-verify: true
    network: "ws"
    ws-path: "/path"
    ws-headers:
      Host: "example.com"
      Edge: "edge"
    tls: true
    servername: "example.com"
"#;
        let proxies = parse_clash_yaml(yaml).unwrap();
        assert_eq!(proxies.len(), 1);
        let proxy = &proxies[0];
        assert_eq!(proxy.proxy_type, ProxyType::VMess);
        assert_eq!(proxy.remark, "Test VMess");
        assert_eq!(proxy.hostname, "example.com");
        assert_eq!(proxy.port, 443);
        assert_eq!(
            proxy.user_id,
            Some("b831381d-6324-4d53-ad4f-8cda48b30811".to_string())
        );
        assert_eq!(proxy.alter_id, 0);
        assert_eq!(proxy.encrypt_method, Some("auto".to_string()));
        assert_eq!(proxy.udp, Some(true));
        assert_eq!(proxy.tcp_fast_open, Some(true));
        assert_eq!(proxy.allow_insecure, Some(true));
        assert_eq!(proxy.transfer_protocol, Some("ws".to_string()));
        assert_eq!(proxy.path, Some("/path".to_string()));
        assert_eq!(proxy.host, Some("example.com".to_string()));
        assert_eq!(proxy.edge, Some("edge".to_string()));
        assert_eq!(proxy.tls_secure, true);
        assert_eq!(proxy.server_name, Some("example.com".to_string()));
    }

    #[test]
    fn test_parse_trojan() {
        let yaml = r#"
proxies:
  - type: trojan
    name: "Test Trojan"
    server: "example.com"
    port: 443
    password: "password"
    udp: true
    tfo: true
    skip-cert-verify: true
    network: "ws"
    sni: "example.com"
"#;
        let proxies = parse_clash_yaml(yaml).unwrap();
        assert_eq!(proxies.len(), 1);
        let proxy = &proxies[0];
        assert_eq!(proxy.proxy_type, ProxyType::Trojan);
        assert_eq!(proxy.remark, "Test Trojan");
        assert_eq!(proxy.hostname, "example.com");
        assert_eq!(proxy.port, 443);
        assert_eq!(proxy.password, Some("password".to_string()));
        assert_eq!(proxy.udp, Some(true));
        assert_eq!(proxy.tcp_fast_open, Some(true));
        assert_eq!(proxy.allow_insecure, Some(true));
        assert_eq!(proxy.transfer_protocol, Some("ws".to_string()));
        assert_eq!(proxy.sni, Some("example.com".to_string()));
    }
}
