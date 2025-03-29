use crate::models::{Proxy, ProxyType};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

lazy_static! {
    static ref GROUPID_REGEX: Regex =
        Regex::new(r"^!!(?:GROUPID|INSERT)=([\d\-+!,]+)(?:!!(.*))?$").unwrap();
    static ref GROUP_REGEX: Regex = Regex::new(r"^!!(?:GROUP)=(.+?)(?:!!(.*))?$").unwrap();
    static ref TYPE_REGEX: Regex = Regex::new(r"^!!(?:TYPE)=(.+?)(?:!!(.*))?$").unwrap();
    static ref PORT_REGEX: Regex = Regex::new(r"^!!(?:PORT)=(.+?)(?:!!(.*))?$").unwrap();
    static ref SERVER_REGEX: Regex = Regex::new(r"^!!(?:SERVER)=(.+?)(?:!!(.*))?$").unwrap();
    static ref PROTOCOL_REGEX: Regex = Regex::new(r"^!!(?:PROTOCOL)=(.+?)(?:!!(.*))?$").unwrap();
    static ref UDPSUPPORT_REGEX: Regex =
        Regex::new(r"^!!(?:UDPSUPPORT)=(.+?)(?:!!(.*))?$").unwrap();
    static ref SECURITY_REGEX: Regex = Regex::new(r"^!!(?:SECURITY)=(.+?)(?:!!(.*))?$").unwrap();
    static ref REMARKS_REGEX: Regex = Regex::new(r"^!!(?:REMARKS)=(.+?)(?:!!(.*))?$").unwrap();
    static ref PROXY_TYPES: HashMap<ProxyType, &'static str> = {
        let mut m = HashMap::new();
        m.insert(ProxyType::Shadowsocks, "SS");
        m.insert(ProxyType::ShadowsocksR, "SSR");
        m.insert(ProxyType::VMess, "VMESS");
        m.insert(ProxyType::Trojan, "TROJAN");
        m.insert(ProxyType::Snell, "SNELL");
        m.insert(ProxyType::HTTP, "HTTP");
        m.insert(ProxyType::HTTPS, "HTTPS");
        m.insert(ProxyType::Socks5, "SOCKS5");
        m.insert(ProxyType::WireGuard, "WIREGUARD");
        m.insert(ProxyType::Hysteria, "HYSTERIA");
        m.insert(ProxyType::Hysteria2, "HYSTERIA2");
        m.insert(ProxyType::Unknown, "UNKNOWN");
        m
    };
}

/// Match a rule against a proxy node
///
/// This function evaluates complex rule strings that can match different aspects of a proxy node.
/// Special rule formats begin with "!!" and can match against properties like group, type, port, etc.
///
/// Supported special rules:
/// - !!GROUP=<group_pattern> - Matches node's group against pattern
/// - !!GROUPID=<id_range> - Matches node's group ID against range
/// - !!INSERT=<id_range> - Like GROUPID but negates direction
/// - !!TYPE=<type_pattern> - Matches node's proxy type against pattern
/// - !!PORT=<port_range> - Matches node's port against range
/// - !!SERVER=<server_pattern> - Matches node's hostname against pattern
/// - !!PROTOCOL=<protocol_pattern> - Matches node's protocol against pattern
/// - !!UDPSUPPORT=<support_pattern> - Matches node's UDP support status
/// - !!SECURITY=<security_pattern> - Matches node's security features
/// - !!REMARKS=<remarks_pattern> - Matches node's remark against pattern
///
/// # Arguments
/// * `rule` - The rule to match
/// * `real_rule` - Output parameter that will contain the processed rule after special prefix handling
/// * `node` - The proxy node to match against
///
/// # Returns
/// * `true` if the rule matches the node
/// * `false` otherwise
pub fn apply_matcher(rule: &str, real_rule: &mut String, node: &Proxy) -> bool {
    if rule.starts_with("!!GROUP=") {
        if let Some(captures) = GROUP_REGEX.captures(rule) {
            let target = captures.get(1).map_or("", |m| m.as_str());
            *real_rule = captures.get(2).map_or("", |m| m.as_str()).to_string();
            return reg_find(&node.group, target);
        }
    } else if rule.starts_with("!!GROUPID=") || rule.starts_with("!!INSERT=") {
        let dir = if rule.starts_with("!!INSERT=") { -1 } else { 1 };
        if let Some(captures) = GROUPID_REGEX.captures(rule) {
            let target = captures.get(1).map_or("", |m| m.as_str());
            *real_rule = captures.get(2).map_or("", |m| m.as_str()).to_string();
            return match_range(target, dir * (node.group_id as i32));
        }
    } else if rule.starts_with("!!TYPE=") {
        if let Some(captures) = TYPE_REGEX.captures(rule) {
            let target = captures.get(1).map_or("", |m| m.as_str());
            *real_rule = captures.get(2).map_or("", |m| m.as_str()).to_string();
            if node.proxy_type == ProxyType::Unknown {
                return false;
            }

            let type_str = PROXY_TYPES.get(&node.proxy_type).unwrap_or(&"UNKNOWN");
            return reg_match(type_str, target);
        }
    } else if rule.starts_with("!!PORT=") {
        if let Some(captures) = PORT_REGEX.captures(rule) {
            let target = captures.get(1).map_or("", |m| m.as_str());
            *real_rule = captures.get(2).map_or("", |m| m.as_str()).to_string();
            return match_range(target, node.port as i32);
        }
    } else if rule.starts_with("!!SERVER=") {
        if let Some(captures) = SERVER_REGEX.captures(rule) {
            let target = captures.get(1).map_or("", |m| m.as_str());
            *real_rule = captures.get(2).map_or("", |m| m.as_str()).to_string();
            return reg_find(&node.hostname, target);
        }
    } else if rule.starts_with("!!PROTOCOL=") {
        if let Some(captures) = PROTOCOL_REGEX.captures(rule) {
            let target = captures.get(1).map_or("", |m| m.as_str());
            *real_rule = captures.get(2).map_or("", |m| m.as_str()).to_string();
            let protocol = match &node.protocol {
                Some(proto) => proto,
                None => return false,
            };
            return reg_find(protocol, target);
        }
    } else if rule.starts_with("!!UDPSUPPORT=") {
        if let Some(captures) = UDPSUPPORT_REGEX.captures(rule) {
            let target = captures.get(1).map_or("", |m| m.as_str());
            *real_rule = captures.get(2).map_or("", |m| m.as_str()).to_string();

            match node.udp {
                Some(true) => return reg_match("yes", target),
                Some(false) => return reg_match("no", target),
                None => return reg_match("undefined", target),
            }
        }
    } else if rule.starts_with("!!SECURITY=") {
        if let Some(captures) = SECURITY_REGEX.captures(rule) {
            let target = captures.get(1).map_or("", |m| m.as_str());
            *real_rule = captures.get(2).map_or("", |m| m.as_str()).to_string();

            // Build a string of security features
            let mut features = String::new();

            if node.tls_secure {
                features.push_str("TLS,");
            }

            if let Some(true) = node.allow_insecure {
                features.push_str("INSECURE,");
            }

            if let Some(true) = node.tls13 {
                features.push_str("TLS13,");
            }

            if !features.is_empty() {
                features.pop(); // Remove trailing comma
            } else {
                features.push_str("NONE");
            }

            return reg_find(&features, target);
        }
    } else if rule.starts_with("!!REMARKS=") {
        if let Some(captures) = REMARKS_REGEX.captures(rule) {
            let target = captures.get(1).map_or("", |m| m.as_str());
            *real_rule = captures.get(2).map_or("", |m| m.as_str()).to_string();
            return reg_find(&node.remark, target);
        }
    } else {
        *real_rule = rule.to_string();
    }

    true
}

/// Match a number against a range specification
///
/// Range specification can include:
/// * Single numbers: "1", "2"
/// * Ranges: "1-10", "100-200"
/// * Negation: "!1-10" (everything except 1-10)
/// * Multiple ranges: "1-10,20-30,50"
///
/// # Arguments
/// * `range` - The range specification string
/// * `target` - The target number to match
///
/// # Returns
/// * `true` if the target matches the range
/// * `false` otherwise
pub fn match_range(range: &str, target: i32) -> bool {
    let mut negate = false;
    let mut matched = false;

    for range_part in range.split(',') {
        let mut part = range_part.trim();

        if part.starts_with('!') {
            negate = true;
            part = &part[1..];
        }

        if part.contains('-') {
            let bounds: Vec<&str> = part.split('-').collect();
            if bounds.len() == 2 {
                let lower = bounds[0].parse::<i32>().unwrap_or(i32::MIN);
                let upper = bounds[1].parse::<i32>().unwrap_or(i32::MAX);

                if target >= lower && target <= upper {
                    matched = true;
                    break;
                }
            }
        } else if let Ok(exact) = part.parse::<i32>() {
            if target == exact {
                matched = true;
                break;
            }
        }
    }

    if negate {
        !matched
    } else {
        matched
    }
}

/// Check if a string matches a regular expression pattern
///
/// # Arguments
/// * `text` - The text to search
/// * `pattern` - The regex pattern to match
///
/// # Returns
/// * `true` if the pattern is found in the text
/// * `false` otherwise
pub fn reg_find(text: &str, pattern: &str) -> bool {
    if pattern.is_empty() {
        return true;
    }

    match Regex::new(&format!("(?i){}", pattern)) {
        Ok(re) => re.is_match(text),
        Err(_) => false,
    }
}

/// Check if a string fully matches a regular expression pattern
///
/// # Arguments
/// * `text` - The text to match
/// * `pattern` - The regex pattern to match
///
/// # Returns
/// * `true` if the pattern fully matches the text
/// * `false` otherwise
pub fn reg_match(text: &str, pattern: &str) -> bool {
    if pattern.is_empty() {
        return true;
    }

    match Regex::new(&format!("(?i)^{}$", pattern)) {
        Ok(re) => re.is_match(text),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProxyType;

    // Helper function to create a test proxy
    fn create_test_proxy() -> Proxy {
        Proxy {
            id: 1,
            group_id: 2,
            group: "TestGroup".to_string(),
            remark: "TestRemark".to_string(),
            hostname: "example.com".to_string(),
            port: 8080,
            proxy_type: ProxyType::Shadowsocks,
            protocol: Some("origin".to_string()),
            udp: Some(true),
            tls_secure: true,
            tls13: Some(true),
            ..Default::default()
        }
    }

    #[test]
    fn test_match_range_simple() {
        assert!(match_range("5", 5));
        assert!(!match_range("5", 6));
    }

    #[test]
    fn test_match_range_with_ranges() {
        assert!(match_range("1-10", 5));
        assert!(!match_range("1-10", 11));
    }

    #[test]
    fn test_match_range_with_negation() {
        assert!(!match_range("!5", 5));
        assert!(match_range("!5", 6));
        assert!(!match_range("!1-10", 5));
        assert!(match_range("!1-10", 11));
    }

    #[test]
    fn test_match_range_with_multiple() {
        assert!(match_range("1-5,10-15", 3));
        assert!(match_range("1-5,10-15", 12));
        assert!(!match_range("1-5,10-15", 7));
    }

    #[test]
    fn test_match_range_complex() {
        assert!(match_range("!1-5,10,15-20", 12));
        assert!(!match_range("!1-5,10,15-20", 10));
        assert!(!match_range("!1-5,10,15-20", 3));
        assert!(match_range("!1-5,10,15-20", 6));
    }

    #[test]
    fn test_reg_find() {
        assert!(reg_find("This is a test", "test"));
        assert!(reg_find("This is a test", "TEST")); // Case insensitive
        assert!(!reg_find("This is a test", "banana"));
        assert!(reg_find("This is a test", "")); // Empty pattern always matches
    }

    #[test]
    fn test_reg_match() {
        assert!(reg_match("12345", r"^\d+$"));
        assert!(!reg_match("12345a", r"^\d+$"));
        assert!(reg_match("HELLO", r"(?i)hello"));
    }

    #[test]
    fn test_apply_matcher_group() {
        let node = create_test_proxy();
        let mut real_rule = String::new();

        assert!(apply_matcher("!!GROUP=TestGroup", &mut real_rule, &node));
        assert_eq!(real_rule, "");

        real_rule.clear();
        assert!(!apply_matcher("!!GROUP=OtherGroup", &mut real_rule, &node));
    }

    #[test]
    fn test_apply_matcher_type() {
        let node = create_test_proxy();
        let mut real_rule = String::new();

        assert!(apply_matcher("!!TYPE=SS", &mut real_rule, &node));
        assert_eq!(real_rule, "");

        real_rule.clear();
        assert!(!apply_matcher("!!TYPE=VMess", &mut real_rule, &node));
    }

    #[test]
    fn test_apply_matcher_port() {
        let node = create_test_proxy();
        let mut real_rule = String::new();

        assert!(apply_matcher("!!PORT=8080", &mut real_rule, &node));
        assert_eq!(real_rule, "");

        real_rule.clear();
        assert!(apply_matcher("!!PORT=8000-9000", &mut real_rule, &node));

        real_rule.clear();
        assert!(!apply_matcher("!!PORT=443", &mut real_rule, &node));
    }

    #[test]
    fn test_apply_matcher_server() {
        let node = create_test_proxy();
        let mut real_rule = String::new();

        assert!(apply_matcher("!!SERVER=example", &mut real_rule, &node));
        assert_eq!(real_rule, "");

        real_rule.clear();
        assert!(!apply_matcher("!!SERVER=google", &mut real_rule, &node));
    }

    #[test]
    fn test_apply_matcher_protocol() {
        let node = create_test_proxy();
        let mut real_rule = String::new();

        assert!(apply_matcher("!!PROTOCOL=origin", &mut real_rule, &node));
        assert_eq!(real_rule, "");

        real_rule.clear();
        assert!(!apply_matcher(
            "!!PROTOCOL=auth_sha1",
            &mut real_rule,
            &node
        ));
    }

    #[test]
    fn test_apply_matcher_udp_support() {
        let node = create_test_proxy();
        let mut real_rule = String::new();

        assert!(apply_matcher("!!UDPSUPPORT=yes", &mut real_rule, &node));
        assert_eq!(real_rule, "");

        real_rule.clear();
        assert!(!apply_matcher("!!UDPSUPPORT=no", &mut real_rule, &node));

        // Test with undefined UDP support
        let mut node_no_udp = node.clone();
        node_no_udp.udp = None;

        real_rule.clear();
        assert!(apply_matcher(
            "!!UDPSUPPORT=undefined",
            &mut real_rule,
            &node_no_udp
        ));
    }

    #[test]
    fn test_apply_matcher_security() {
        let node = create_test_proxy();
        let mut real_rule = String::new();

        assert!(apply_matcher("!!SECURITY=TLS", &mut real_rule, &node));
        assert_eq!(real_rule, "");

        real_rule.clear();
        assert!(apply_matcher("!!SECURITY=TLS13", &mut real_rule, &node));

        real_rule.clear();
        assert!(!apply_matcher("!!SECURITY=INSECURE", &mut real_rule, &node));

        // Test with insecure allowed
        let mut node_insecure = node.clone();
        node_insecure.allow_insecure = Some(true);

        real_rule.clear();
        assert!(apply_matcher(
            "!!SECURITY=INSECURE",
            &mut real_rule,
            &node_insecure
        ));
    }

    #[test]
    fn test_apply_matcher_remarks() {
        let node = create_test_proxy();
        let mut real_rule = String::new();

        assert!(apply_matcher("!!REMARKS=Test", &mut real_rule, &node));
        assert_eq!(real_rule, "");

        real_rule.clear();
        assert!(!apply_matcher("!!REMARKS=Premium", &mut real_rule, &node));
    }

    #[test]
    fn test_apply_matcher_with_trailing_rule() {
        let node = create_test_proxy();
        let mut real_rule = String::new();

        assert!(apply_matcher(
            "!!GROUP=TestGroup!!.+",
            &mut real_rule,
            &node
        ));
        assert_eq!(real_rule, ".+");

        // The trailing rule ".+" will be used with node.remark in the parent function
    }
}
