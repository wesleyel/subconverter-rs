use crate::generator::config::subexport::{process_remark, ExtraSettings};
use crate::utils::url::url_encode;
use crate::{Proxy, ProxyType};
use base64::{engine::general_purpose, Engine as _};
use serde_json::{self, json, Value};
use std::collections::HashMap;

/// Convert a proxy to a single URI
///
/// This function converts a proxy to its URI representation.
///
/// # Arguments
/// * `node` - Proxy node to convert
/// * `ext` - Extra settings for conversion
pub fn proxy_to_uri(node: &mut Proxy, ext: &mut ExtraSettings) -> String {
    // Process remark
    let mut remark = node.remark.clone();
    process_remark(&mut remark, ext, false);

    // Create URI based on proxy type
    match node.proxy_type {
        ProxyType::Shadowsocks => {
            // Format: ss://BASE64(method:password)@server:port/?plugin=plugin_data#remark
            let user_info = format!("{}:{}", node.cipher, node.password);
            let encoded_user_info = general_purpose::STANDARD.encode(user_info);

            let mut uri = format!("ss://{}@{}:{}", encoded_user_info, node.server, node.port);

            // Add plugin if present
            if !node.plugin.is_empty() && !node.plugin_opts.is_empty() {
                uri.push_str(&format!(
                    "/?plugin={}",
                    url_encode(&format!("{};{}", node.plugin, node.plugin_opts))
                ));
            }

            // Add remark
            uri.push_str(&format!("#{}", url_encode(&remark)));

            uri
        }
        ProxyType::ShadowsocksR => {
            // Format: ssr://BASE64(server:port:protocol:method:obfs:BASE64(password)/?remarks=BASE64(remark)&protoparam=BASE64(protocol_param)&obfsparam=BASE64(obfs_param))
            let mut plain_text = format!(
                "{}:{}:{}:{}:{}:{}",
                node.server,
                node.port,
                node.protocol,
                node.cipher,
                node.obfs,
                general_purpose::STANDARD.encode(&node.password)
            );

            // Add parameters
            let mut params = Vec::new();

            if !remark.is_empty() {
                params.push(format!(
                    "remarks={}",
                    general_purpose::STANDARD.encode(&remark)
                ));
            }

            if !node.protocol_param.is_empty() {
                params.push(format!(
                    "protoparam={}",
                    general_purpose::STANDARD.encode(&node.protocol_param)
                ));
            }

            if !node.obfs_param.is_empty() {
                params.push(format!(
                    "obfsparam={}",
                    general_purpose::STANDARD.encode(&node.obfs_param)
                ));
            }

            if !params.is_empty() {
                plain_text.push_str(&format!("/?{}", params.join("&")));
            }

            // Base64 encode the entire URI
            format!("ssr://{}", general_purpose::STANDARD.encode(plain_text))
        }
        ProxyType::VMess => {
            // Format: vmess://BASE64(JSON)
            let mut vmess_json = json!({
                "v": "2",
                "ps": remark,
                "add": node.server,
                "port": node.port,
                "id": node.uuid,
                "aid": node.alter_id,
                "net": if node.network.is_empty() { "tcp" } else { &node.network },
                "type": if node.header_type.is_empty() { "none" } else { &node.header_type },
                "host": node.host,
                "path": node.path,
                "tls": if node.tls { "tls" } else { "none" }
            });

            // Add SNI if present
            if let Some(sni) = &node.sni {
                if !sni.is_empty() {
                    vmess_json["sni"] = json!(sni);
                }
            }

            // Add cipher if present
            if !node.cipher.is_empty() {
                vmess_json["cipher"] = json!(node.cipher);
            }

            // Convert to string and base64 encode
            if let Ok(json_str) = serde_json::to_string(&vmess_json) {
                format!("vmess://{}", general_purpose::STANDARD.encode(json_str))
            } else {
                String::new()
            }
        }
        ProxyType::Trojan => {
            // Format: trojan://password@server:port?sni=sni&allowInsecure=1#remark
            let mut uri = format!("trojan://{}@{}:{}", node.password, node.server, node.port);

            // Add parameters
            let mut params = Vec::new();

            if let Some(sni) = &node.sni {
                if !sni.is_empty() {
                    params.push(format!("sni={}", sni));
                }
            }

            if let Some(skip_cert_verify) = node.skip_cert_verify {
                if skip_cert_verify {
                    params.push("allowInsecure=1".to_string());
                }
            }

            if !node.network.is_empty() && node.network == "ws" {
                params.push("type=ws".to_string());

                if !node.host.is_empty() {
                    params.push(format!("host={}", url_encode(&node.host)));
                }

                if !node.path.is_empty() {
                    params.push(format!("path={}", url_encode(&node.path)));
                }
            }

            if !params.is_empty() {
                uri.push_str(&format!("?{}", params.join("&")));
            }

            // Add remark
            uri.push_str(&format!("#{}", url_encode(&remark)));

            uri
        }
        ProxyType::Socks5 => {
            // Format: socks5://username:password@server:port#remark
            let mut uri = String::from("socks5://");

            // Add username/password if present
            if !node.username.is_empty() {
                uri.push_str(&node.username);

                if !node.password.is_empty() {
                    uri.push_str(&format!(":{}", node.password));
                }

                uri.push('@');
            }

            // Add server and port
            uri.push_str(&format!("{}:{}", node.server, node.port));

            // Add remark
            uri.push_str(&format!("#{}", url_encode(&remark)));

            uri
        }
        ProxyType::HTTP | ProxyType::HTTPS => {
            // Format: http(s)://username:password@server:port#remark
            let protocol = if node.proxy_type == ProxyType::HTTP {
                "http"
            } else {
                "https"
            };
            let mut uri = format!("{}://", protocol);

            // Add username/password if present
            if !node.username.is_empty() {
                uri.push_str(&node.username);

                if !node.password.is_empty() {
                    uri.push_str(&format!(":{}", node.password));
                }

                uri.push('@');
            }

            // Add server and port
            uri.push_str(&format!("{}:{}", node.server, node.port));

            // Add remark
            uri.push_str(&format!("#{}", url_encode(&remark)));

            uri
        }
        ProxyType::Snell => {
            // Format: snell://PSK@server:port?version=version#remark
            let mut uri = format!("snell://{}@{}:{}", node.password, node.server, node.port);

            // Add parameters
            let mut params = Vec::new();

            if node.version > 0 {
                params.push(format!("version={}", node.version));
            }

            if !node.obfs.is_empty() {
                params.push(format!("obfs={}", node.obfs));

                if !node.host.is_empty() {
                    params.push(format!("obfs-host={}", url_encode(&node.host)));
                }
            }

            if !params.is_empty() {
                uri.push_str(&format!("?{}", params.join("&")));
            }

            // Add remark
            uri.push_str(&format!("#{}", url_encode(&remark)));

            uri
        }
        _ => String::new(),
    }
}

/// Convert proxies to a list of URIs
///
/// This function converts a list of proxies to their URI representations.
///
/// # Arguments
/// * `nodes` - List of proxy nodes to convert
/// * `ext` - Extra settings for conversion
pub fn proxy_to_single(nodes: &mut Vec<Proxy>, ext: &mut ExtraSettings) -> String {
    let mut result = String::new();

    for node in nodes.iter_mut() {
        let uri = proxy_to_uri(node, ext);
        if !uri.is_empty() {
            result.push_str(&uri);
            result.push('\n');
        }
    }

    result
}
