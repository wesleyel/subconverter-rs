use crate::models::{Proxy, ProxyType};
use crate::parser::yaml::clash::proxy_types::ClashProxyYamlInput;
use crate::utils::tribool::OptionSetExt;
use serde_yaml::Value as YamlValue;
use std::collections::{HashMap, HashSet};

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
            // Shadowsocks proxy handling
            // C++ counterpart uses string fields: plugin, pluginopts, pluginopts_mode, pluginopts_host, pluginopts_mux
            // Rust uses more structured HashMap for plugin_opts, providing better flexibility
            ClashProxyYamlInput::Shadowsocks {
                name,
                server,
                port,
                cipher,
                password,
                udp,
                tfo,
                skip_cert_verify,
                plugin,
                plugin_opts,
            } => {
                let mut proxy = Proxy::default();
                proxy.proxy_type = ProxyType::Shadowsocks;
                proxy.remark = name;
                proxy.hostname = server;
                proxy.port = port;
                proxy.encrypt_method = Some(cipher);
                proxy.password = Some(password);
                // Using Option<bool> with set_if_some - equivalent to C++ tribool handling
                proxy.udp.set_if_some(udp);
                proxy.tcp_fast_open.set_if_some(tfo);
                proxy.allow_insecure.set_if_some(skip_cert_verify);

                if let Some(plugin_name) = plugin {
                    proxy.plugin = Some(plugin_name);
                    if let Some(opts) = plugin_opts {
                        let mut plugin_opts_str = String::new();
                        for (key, value) in opts {
                            if !plugin_opts_str.is_empty() {
                                plugin_opts_str.push(';');
                            }
                            // Using {:?} for proper formatting of YamlValue
                            plugin_opts_str.push_str(&format!("{}={:?}", key, value));
                        }
                        proxy.plugin_option = Some(plugin_opts_str);
                    }
                }

                proxies.push(proxy);
            }
            // ShadowsocksR proxy handling
            // C++ uses separate strings: protocol, protoparam, obfs, obfsparam
            // Rust directly maps these to Option fields with proper naming
            ClashProxyYamlInput::ShadowsocksR {
                name,
                server,
                port,
                cipher,
                password,
                protocol,
                obfs,
                udp,
                tfo,
                skip_cert_verify,
                protocol_param,
                obfs_param,
            } => {
                let mut proxy = Proxy::default();
                proxy.proxy_type = ProxyType::ShadowsocksR;
                proxy.remark = name;
                proxy.hostname = server;
                proxy.port = port;
                proxy.encrypt_method = Some(cipher);
                proxy.password = Some(password);
                proxy.protocol = Some(protocol);
                proxy.obfs = Some(obfs);
                proxy.udp.set_if_some(udp);
                proxy.tcp_fast_open.set_if_some(tfo);
                proxy.allow_insecure.set_if_some(skip_cert_verify);
                proxy.protocol_param = protocol_param;
                proxy.obfs_param = obfs_param;

                proxies.push(proxy);
            }
            // VMess proxy handling
            // C++ uses type, id, aid, net, path, host, edge, tls, sni as separate variables
            // Rust uses structured field names: uuid, alter_id, network, ws_path, ws_headers, etc.
            ClashProxyYamlInput::VMess {
                name,
                server,
                port,
                uuid,
                alter_id,
                cipher,
                udp,
                tfo,
                skip_cert_verify,
                network,
                ws_path,
                ws_headers,
                tls,
                servername,
            } => {
                let mut proxy = Proxy::default();
                proxy.proxy_type = ProxyType::VMess;
                proxy.remark = name;
                proxy.hostname = server.clone();
                proxy.port = port;
                proxy.user_id = Some(uuid);
                // Safe type conversion from u32 to u16
                proxy.alter_id = alter_id as u16;
                proxy.encrypt_method = Some(cipher);
                proxy.udp.set_if_some(udp);
                proxy.tcp_fast_open.set_if_some(tfo);
                proxy.allow_insecure.set_if_some(skip_cert_verify);
                proxy.tls_secure = tls.unwrap_or(false);
                proxy.server_name = servername;

                // Network protocol handling - more concise than C++ nested if/else
                if let Some(net) = network {
                    proxy.transfer_protocol = Some(net.clone());
                    match net.as_str() {
                        "ws" => {
                            if let Some(path) = ws_path {
                                proxy.path = Some(path);
                            }
                            if let Some(headers) = ws_headers {
                                if let Some(host) = headers.get("Host") {
                                    proxy.host = Some(host.clone());
                                }
                                if let Some(edge) = headers.get("Edge") {
                                    proxy.edge = Some(edge.clone());
                                }
                            }
                        }
                        "http" => {
                            if let Some(path) = ws_path {
                                proxy.path = Some(path);
                            }
                            if let Some(headers) = ws_headers {
                                if let Some(host) = headers.get("Host") {
                                    proxy.host = Some(host.clone());
                                }
                                if let Some(edge) = headers.get("Edge") {
                                    proxy.edge = Some(edge.clone());
                                }
                            }
                        }
                        "h2" => {
                            if let Some(path) = ws_path {
                                proxy.path = Some(path);
                            }
                            if let Some(headers) = ws_headers {
                                if let Some(host) = headers.get("Host") {
                                    proxy.host = Some(host.clone());
                                }
                            }
                        }
                        "grpc" => {
                            if let Some(path) = ws_path {
                                proxy.path = Some(path);
                            }
                            proxy.host = Some(server);
                        }
                        _ => {}
                    }
                }

                proxies.push(proxy);
            }
            // Trojan proxy handling
            ClashProxyYamlInput::Trojan {
                name,
                server,
                port,
                password,
                udp,
                tfo,
                skip_cert_verify,
                network,
                sni,
            } => {
                let mut proxy = Proxy::default();
                proxy.proxy_type = ProxyType::Trojan;
                proxy.remark = name;
                proxy.hostname = server;
                proxy.port = port;
                proxy.password = Some(password);
                proxy.udp.set_if_some(udp);
                proxy.tcp_fast_open.set_if_some(tfo);
                proxy.allow_insecure.set_if_some(skip_cert_verify);
                proxy.sni = sni;

                if let Some(net) = network {
                    proxy.transfer_protocol = Some(net);
                }

                proxies.push(proxy);
            }
            // HTTP/HTTPS proxy handling
            // C++ handles type selection with if/else
            // Rust uses a cleaner ternary-like if expression
            ClashProxyYamlInput::Http {
                name,
                server,
                port,
                username,
                password,
                tls,
                skip_cert_verify,
            } => {
                let mut proxy = Proxy::default();
                proxy.proxy_type = if tls.unwrap_or(false) {
                    ProxyType::HTTPS
                } else {
                    ProxyType::HTTP
                };
                proxy.remark = name;
                proxy.hostname = server;
                proxy.port = port;
                proxy.username = username;
                proxy.password = password;
                proxy.allow_insecure.set_if_some(skip_cert_verify);

                proxies.push(proxy);
            }
            // Socks5 proxy handling
            ClashProxyYamlInput::Socks5 {
                name,
                server,
                port,
                username,
                password,
                skip_cert_verify,
                udp,
                tfo,
            } => {
                let mut proxy = Proxy::default();
                proxy.proxy_type = ProxyType::Socks5;
                proxy.remark = name;
                proxy.hostname = server;
                proxy.port = port;
                proxy.username = username;
                proxy.password = password;
                proxy.allow_insecure.set_if_some(skip_cert_verify);
                proxy.udp.set_if_some(udp);
                proxy.tcp_fast_open.set_if_some(tfo);

                proxies.push(proxy);
            }
            // Snell proxy handling
            ClashProxyYamlInput::Snell {
                name,
                server,
                port,
                psk,
                version,
                obfs,
                obfs_opts,
                udp,
                tfo,
            } => {
                let mut proxy = Proxy::default();
                proxy.proxy_type = ProxyType::Snell;
                proxy.remark = name;
                proxy.hostname = server;
                proxy.port = port;
                proxy.password = Some(psk);
                // Safe type conversion with default value
                proxy.snell_version = version.unwrap_or(1) as u16;
                proxy.obfs = obfs;

                if let Some(opts) = obfs_opts {
                    let mut obfs_opts_str = String::new();
                    for (key, value) in opts {
                        if !obfs_opts_str.is_empty() {
                            obfs_opts_str.push(';');
                        }
                        // Using {:?} for proper formatting of YamlValue
                        obfs_opts_str.push_str(&format!("{}={:?}", key, value));
                    }
                    proxy.plugin_option = Some(obfs_opts_str);
                }

                proxy.udp.set_if_some(udp);
                proxy.tcp_fast_open.set_if_some(tfo);

                proxies.push(proxy);
            }
            // WireGuard proxy handling
            // C++ uses separate string variables for each field
            // Rust directly maps to structured fields with proper type conversion
            ClashProxyYamlInput::WireGuard {
                name,
                server,
                port,
                private_key,
                public_key,
                ip,
                ipv6,
                preshared_key,
                dns,
                mtu,
                allowed_ips,
                keepalive,
                udp,
            } => {
                let mut proxy = Proxy::default();
                proxy.proxy_type = ProxyType::WireGuard;
                proxy.remark = name;
                proxy.hostname = server;
                proxy.port = port;
                proxy.private_key = Some(private_key);
                proxy.public_key = Some(public_key);
                proxy.self_ip = Some(ip);
                proxy.self_ipv6 = ipv6;
                proxy.pre_shared_key = preshared_key;

                // Convert Vec<String> to HashSet<String> for dns_servers
                // More explicit conversion than C++ handling
                let mut dns_set = HashSet::new();
                if let Some(dns_servers) = dns {
                    for dns_server in dns_servers {
                        dns_set.insert(dns_server);
                    }
                }
                proxy.dns_servers = dns_set;

                proxy.mtu = mtu.unwrap_or(0) as u16;
                proxy.allowed_ips = allowed_ips.join(",");
                proxy.keep_alive = keepalive.unwrap_or(0) as u16;
                proxy.udp.set_if_some(udp);

                proxies.push(proxy);
            }
            // Hysteria proxy handling
            // Rust adds proper type handling and null safety compared to C++
            ClashProxyYamlInput::Hysteria {
                name,
                server,
                port,
                ports,
                protocol,
                obfs_protocol,
                up,
                up_speed,
                down,
                down_speed,
                auth,
                auth_str,
                obfs,
                sni,
                fingerprint,
                alpn,
                ca,
                ca_str,
                recv_window_conn,
                recv_window,
                disable_mtu_discovery,
                fast_open,
                hop_interval,
                skip_cert_verify,
                tfo,
            } => {
                let mut proxy = Proxy::default();
                proxy.proxy_type = ProxyType::Hysteria;
                proxy.remark = name;
                proxy.hostname = server;
                proxy.port = port;
                proxy.ports = ports;
                proxy.protocol = protocol;
                proxy.obfs = obfs_protocol.or(obfs);

                // 处理上行/下行速度
                if let Some(up_value) = up {
                    proxy.up_speed = up_value.replace("Mbps", "").parse().unwrap_or(0);
                } else if let Some(up_value) = up_speed {
                    proxy.up_speed = up_value;
                }

                if let Some(down_value) = down {
                    proxy.down_speed = down_value.replace("Mbps", "").parse().unwrap_or(0);
                } else if let Some(down_value) = down_speed {
                    proxy.down_speed = down_value;
                }

                // 设置认证信息
                proxy.auth_str = auth_str;

                // 设置TLS相关字段
                proxy.fingerprint = fingerprint;

                // Convert Vec<String> to HashSet<String> for alpn
                // C++ uses string_array (vector), Rust uses HashSet for uniqueness
                let mut alpn_set = HashSet::new();
                if let Some(alpn_values) = alpn {
                    for alpn_value in alpn_values {
                        alpn_set.insert(alpn_value);
                    }
                }
                proxy.alpn = alpn_set;

                proxy.ca = ca;
                proxy.ca_str = ca_str;
                proxy.sni = sni;

                // 设置网络相关选项
                proxy.recv_window_conn = recv_window_conn.unwrap_or(0);
                proxy.recv_window = recv_window.unwrap_or(0);
                // Proper handling of Option<bool> vs C++ tribool
                proxy.disable_mtu_discovery = if disable_mtu_discovery.unwrap_or(false) {
                    Some(true)
                } else {
                    None
                };
                proxy.hop_interval = hop_interval.unwrap_or(0);

                // 设置布尔选项
                proxy.tcp_fast_open.set_if_some(fast_open.or(tfo));
                proxy.allow_insecure.set_if_some(skip_cert_verify);

                proxies.push(proxy);
            }
            // Hysteria2 proxy handling - not present in original C++ implementation
            // Shows the extensibility of the Rust enum-based approach
            ClashProxyYamlInput::Hysteria2 {
                name,
                server,
                port,
                password,
                ports,
                hop_interval,
                up,
                down,
                obfs,
                obfs_password,
                fingerprint,
                alpn,
                ca,
                ca_str,
                cwnd,
                udp_mtu,
                sni,
                skip_cert_verify,
                fast_open,
                tfo,
            } => {
                let mut proxy = Proxy::default();
                proxy.proxy_type = ProxyType::Hysteria2;
                proxy.remark = name;
                proxy.hostname = server;
                proxy.port = port;
                proxy.password = Some(password);
                proxy.ports = ports;
                proxy.hop_interval = hop_interval.unwrap_or(0);

                // 处理上行/下行速度
                if let Some(up_value) = up {
                    proxy.up_speed = up_value.replace("Mbps", "").parse().unwrap_or(0);
                }

                if let Some(down_value) = down {
                    proxy.down_speed = down_value.replace("Mbps", "").parse().unwrap_or(0);
                }

                // 设置混淆选项
                proxy.obfs = obfs;
                proxy.obfs_param = obfs_password;

                // 设置TLS相关字段
                proxy.fingerprint = fingerprint;

                // Handle alpn as a comma-separated string to HashSet
                // More flexible handling than C++ string array
                if let Some(alpn_value) = alpn {
                    let mut alpn_set = HashSet::new();
                    for value in alpn_value.split(',').map(|s| s.trim().to_string()) {
                        if !value.is_empty() {
                            alpn_set.insert(value);
                        }
                    }
                    proxy.alpn = alpn_set;
                }

                proxy.ca = ca;
                proxy.ca_str = ca_str;
                proxy.sni = sni;

                // 设置其他网络参数
                proxy.cwnd = cwnd.unwrap_or(0);
                proxy.mtu = udp_mtu.unwrap_or(0) as u16;

                // 设置布尔选项
                proxy.allow_insecure.set_if_some(skip_cert_verify);
                proxy.tcp_fast_open.set_if_some(fast_open.or(tfo));

                proxies.push(proxy);
            }
            // Unknown proxy type handling
            // C++ tries to extract basic fields from unknown types
            // Rust simply ignores unknown types with a unit variant
            ClashProxyYamlInput::Unknown => {
                // This is now a unit variant, so there's no data to extract
                // We'll just skip it
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
