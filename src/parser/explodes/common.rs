use crate::Proxy;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use std::collections::HashMap;

/// Explode a proxy link into a Proxy object
///
/// This function detects the type of proxy link and calls the appropriate parser
pub fn explode(link: &str, node: &mut Proxy) -> bool {
    // Trim the link
    let link = link.trim();

    // Check for empty link
    if link.is_empty() {
        return false;
    }

    // Detect link type and call appropriate parser
    if link.starts_with("vmess://") {
        // Try standard VMess parser first
        if super::vmess::explode_vmess(link, node) {
            return true;
        }

        // Try alternative VMess formats if standard parser fails
        if super::vmess::explode_std_vmess(link, node) {
            return true;
        }

        if super::vmess::explode_shadowrocket(link, node) {
            return true;
        }

        if super::vmess::explode_kitsunebi(link, node) {
            return true;
        }

        return false;
    } else if link.starts_with("ss://") {
        super::ss::explode_ss(link, node)
    } else if link.starts_with("ssr://") {
        // super::ssr::explode_ssr(link, node)
        false
    } else if link.starts_with("socks://") || link.starts_with("socks5://") {
        super::socks::explode_socks(link, node)
    } else if link.starts_with("http://") || link.starts_with("https://") {
        // Try HTTP parser first
        if super::http::explode_http(link, node) {
            return true;
        }

        // If that fails, try HTTP subscription format
        super::httpsub::explode_http_sub(link, node)
    } else if link.starts_with("trojan://") {
        super::trojan::explode_trojan(link, node)
    } else if link.starts_with("snell://") {
        super::snell::explode_snell(link, node)
    } else if link.starts_with("wg://") || link.starts_with("wireguard://") {
        super::wireguard::explode_wireguard(link, node)
    } else if link.starts_with("hysteria://") {
        super::hysteria::explode_hysteria(link, node)
    } else if link.starts_with("hysteria2://") {
        super::hysteria2::explode_hysteria2(link, node)
    } else if link.starts_with("vmess+") {
        false
        // super::vmess::explode_std_vmess(link, node)
    } else {
        false
    }
}

/// Explode a subscription content into a vector of Proxy objects
///
/// This function parses a subscription content (which may contain multiple proxy links)
/// and returns a vector of Proxy objects
pub fn explode_sub(sub: &str, nodes: &mut Vec<Proxy>) -> bool {
    // Trim the subscription content
    let sub = sub.trim();

    // Check for empty subscription
    if sub.is_empty() {
        return false;
    }

    // Try to decode as base64
    let decoded = match STANDARD.decode(sub) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => sub.to_string(),
        },
        Err(_) => sub.to_string(),
    };

    // Split by newlines and parse each line
    let lines: Vec<&str> = decoded.lines().collect();

    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut node = Proxy::default();
        if explode(line, &mut node) {
            nodes.push(node);
        }
    }

    !nodes.is_empty()
}

/// Explode a configuration file content into a vector of Proxy objects
///
/// This function tries to parse various configuration file formats
/// and returns a vector of Proxy objects
pub fn explode_conf_content(content: &str, nodes: &mut Vec<Proxy>) -> i32 {
    // Trim the content
    let content = content.trim();

    // Check for empty content
    if content.is_empty() {
        return -1;
    }

    // Try to parse as JSON
    if content.starts_with('{') {
        // Try to parse as V2Ray configuration
        if super::vmess::explode_vmess_conf(content, nodes) {
            return 0;
        }

        // Try Netch configuration
        if content.contains("\"server\"") && content.contains("\"port\"") {
            if super::netch::explode_netch_conf(content, nodes) {
                return 0;
            }
        }

        return -1;
    }

    // Try to parse as YAML/Clash
    if content.contains("proxies:") || content.contains("Proxy:") {
        if super::clash::explode_clash(content, nodes) {
            return 0;
        }
        return -1;
    }

    // Try to parse as SSD
    if content.starts_with("ssd://") {
        if super::ss::explode_ssd(content, nodes) {
            return 0;
        }
        return -1;
    }

    // Try to parse as SSTap configuration
    if content.contains("\"servers\":") || content.contains("\"configs\":") {
        if super::sstap::explode_sstap(content, nodes) {
            return 0;
        }
    }

    // Try to parse as Surge configuration
    if content.contains("[Proxy]") {
        if super::surge::explode_surge(content, nodes) {
            return 0;
        }
    }

    // Try to parse as Quantumult configuration
    if content.contains(" = vmess")
        || content.contains(" = shadowsocks")
        || content.contains(" = shadowsocksr")
        || content.contains(" = http")
        || content.contains(" = trojan")
    {
        if super::quan::explode_quan(content, nodes) {
            return 0;
        }
    }

    // Try to parse as subscription
    if explode_sub(content, nodes) {
        return 0;
    }

    -1
}
