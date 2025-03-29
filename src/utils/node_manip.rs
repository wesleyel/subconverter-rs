use log::{debug, info};
use std::cmp::Ordering;

use crate::models::{
    extra_settings::ExtraSettings,
    proxy::{Proxy, ProxyType},
    regex_match_config::{RegexMatchConfig, RegexMatchConfigs},
};
use crate::utils::{
    matcher::{apply_matcher, reg_find},
    reg_replace,
    string::{remove_emoji, trim},
};

/// Applies a rename configuration to a node
/// Similar to the C++ nodeRename function
fn node_rename(node: &mut Proxy, rename_array: &RegexMatchConfigs, _extra: &ExtraSettings) {
    let original_remark = node.remark.clone();

    for pattern in rename_array {
        // Skip script-based patterns since we're not implementing JavaScript support here
        if !pattern._match.is_empty() {
            let mut real_rule = String::new();
            if apply_matcher(&pattern._match, &mut real_rule, node) && !real_rule.is_empty() {
                node.remark = reg_replace(&node.remark, &real_rule, &pattern.replace, true, false);
            }
        }
    }

    // If the remark is empty after processing, restore the original
    if node.remark.is_empty() {
        node.remark = original_remark;
    }
}

/// Adds emoji to node remark based on regex matching
fn add_emoji(node: &Proxy, emoji_array: &RegexMatchConfigs, _extra: &ExtraSettings) -> String {
    for pattern in emoji_array {
        // Skip patterns with empty replace
        if pattern.replace.is_empty() {
            continue;
        }

        // Use apply_matcher to handle complex matching rules
        let mut real_rule = String::new();
        if apply_matcher(&pattern._match, &mut real_rule, node) {
            if real_rule.is_empty() || reg_find(&node.remark, &real_rule) {
                return format!("{} {}", pattern.replace, node.remark);
            }
        }
    }

    node.remark.clone()
}

/// Sorts nodes by a specified criterion
fn sort_nodes(nodes: &mut Vec<Proxy>, _sort_script: &str) {
    // Skip script-based sorting since we're not implementing JavaScript support
    // Default sort by remark
    nodes.sort_by(|a, b| {
        if a.proxy_type == ProxyType::Unknown {
            return Ordering::Greater;
        }
        if b.proxy_type == ProxyType::Unknown {
            return Ordering::Less;
        }
        a.remark.cmp(&b.remark)
    });
}

/// Preprocesses nodes before conversion
/// Based on the C++ preprocessNodes function
pub fn preprocess_nodes(
    nodes: &mut Vec<Proxy>,
    extra: &ExtraSettings,
    rename_patterns: &RegexMatchConfigs,
    emoji_patterns: &RegexMatchConfigs,
) {
    // Process each node
    for node in nodes.iter_mut() {
        // Remove emoji if needed
        if extra.remove_emoji {
            node.remark = trim(&remove_emoji(&node.remark)).to_string();
        }

        // Apply rename patterns
        node_rename(node, rename_patterns, extra);

        // Add emoji if needed
        if extra.add_emoji {
            node.remark = add_emoji(node, emoji_patterns, extra);
        }
    }

    // Sort nodes if needed
    if extra.sort_flag {
        info!("Sorting {} nodes", nodes.len());
        sort_nodes(nodes, &extra.sort_script);
    }

    debug!("Node preprocessing completed for {} nodes", nodes.len());
}

/// Appends proxy type to node remark
pub fn append_type_to_remark(nodes: &mut Vec<Proxy>) {
    for node in nodes.iter_mut() {
        match node.proxy_type {
            ProxyType::Unknown => {}
            _ => {
                let type_str = node.proxy_type.to_string();
                node.remark = format!("{} ({})", node.remark, type_str);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a test proxy node
    fn create_test_node() -> Proxy {
        Proxy {
            id: 1,
            group_id: 2,
            group: "TestGroup".to_string(),
            remark: "HK Node 1 [Premium]".to_string(),
            hostname: "server.example.com".to_string(),
            port: 8388,
            proxy_type: ProxyType::Shadowsocks,
            protocol: Some("origin".to_string()),
            udp: Some(true),
            tls_secure: true,
            ..Default::default()
        }
    }

    /// Create test RegexMatchConfig
    fn create_test_match_config(matches: &str, replace: &str) -> RegexMatchConfig {
        RegexMatchConfig {
            _match: matches.to_string(),
            replace: replace.to_string(),
        }
    }

    #[test]
    fn test_node_rename_simple_regex() {
        let mut node = create_test_node();
        let extra = ExtraSettings::default();

        // Simple regex rename
        let rename_patterns = vec![create_test_match_config("HK", "Hong Kong")];

        node_rename(&mut node, &rename_patterns, &extra);
        assert_eq!(node.remark, "Hong Kong Node 1 [Premium]");
    }

    #[test]
    fn test_node_rename_group_match() {
        let mut node = create_test_node();
        let extra = ExtraSettings::default();

        // Group matching rule
        let rename_patterns = vec![create_test_match_config(
            "!!GROUP=TestGroup!!.*Premium.*",
            "[Premium] $0",
        )];

        node_rename(&mut node, &rename_patterns, &extra);
        assert_eq!(node.remark, "[Premium] HK Node 1 [Premium]");
    }

    #[test]
    fn test_node_rename_type_match() {
        let mut node = create_test_node();
        let extra = ExtraSettings::default();

        // Type matching rule
        let rename_patterns = vec![create_test_match_config("!!TYPE=SS!!(.*)", "SS: $1")];

        node_rename(&mut node, &rename_patterns, &extra);
        assert_eq!(node.remark, "SS: HK Node 1 [Premium]");
    }

    #[test]
    fn test_node_rename_multiple_patterns() {
        let mut node = create_test_node();
        let extra = ExtraSettings::default();

        // Multiple patterns
        let rename_patterns = vec![
            create_test_match_config("HK", "Hong Kong"),
            create_test_match_config("Premium", "Pro"),
            create_test_match_config("Node", "Server"),
        ];

        node_rename(&mut node, &rename_patterns, &extra);
        assert_eq!(node.remark, "Hong Kong Server 1 [Pro]");
    }

    #[test]
    fn test_add_emoji_simple() {
        let node = create_test_node();
        let extra = ExtraSettings::default();

        // Simple emoji adding
        let emoji_patterns = vec![create_test_match_config("HK", "🇭🇰")];

        let result = add_emoji(&node, &emoji_patterns, &extra);
        assert_eq!(result, "🇭🇰 HK Node 1 [Premium]");
    }

    #[test]
    fn test_add_emoji_group_match() {
        let node = create_test_node();
        let extra = ExtraSettings::default();

        // Emoji based on group
        let emoji_patterns = vec![create_test_match_config("!!GROUP=TestGroup", "🔒")];

        let result = add_emoji(&node, &emoji_patterns, &extra);
        assert_eq!(result, "🔒 HK Node 1 [Premium]");
    }

    #[test]
    fn test_add_emoji_security_match() {
        let node = create_test_node();
        let extra = ExtraSettings::default();

        // Emoji based on security
        let emoji_patterns = vec![
            create_test_match_config("!!SECURITY=TLS", "🔒"),
            create_test_match_config("!!UDPSUPPORT=yes", "📱"),
        ];

        let result = add_emoji(&node, &emoji_patterns, &extra);
        assert_eq!(result, "🔒 HK Node 1 [Premium]");
    }

    #[test]
    fn test_preprocess_nodes() {
        let mut nodes = vec![
            create_test_node(),
            {
                let mut node = create_test_node();
                node.remark = "JP Node 2".to_string();
                node.group = "TestGroup".to_string();
                node
            },
            {
                let mut node = create_test_node();
                node.remark = "SG Node 3".to_string();
                node.group = "OtherGroup".to_string();
                node
            },
        ];

        let mut extra = ExtraSettings::default();
        extra.add_emoji = true;
        extra.sort_flag = true;

        let rename_patterns = vec![
            create_test_match_config("Node", "Server"),
            create_test_match_config("!!GROUP=TestGroup!!.*Premium.*", "[VIP] $0"),
        ];

        let emoji_patterns = vec![
            create_test_match_config("HK", "🇭🇰"),
            create_test_match_config("JP", "🇯🇵"),
            create_test_match_config("SG", "🇸🇬"),
        ];

        preprocess_nodes(&mut nodes, &extra, &rename_patterns, &emoji_patterns);

        // Check if nodes are renamed and emojis added
        assert_eq!(nodes[0].remark, "🇭🇰 [VIP] HK Server 1 [Premium]");
        assert_eq!(nodes[1].remark, "🇯🇵 JP Server 2");
        assert_eq!(nodes[2].remark, "🇸🇬 SG Server 3");

        // Check if sorted (should be alphabetical by remark)
        assert!(nodes[0].remark.contains("HK"));
        assert!(nodes[1].remark.contains("JP"));
        assert!(nodes[2].remark.contains("SG"));
    }
}
