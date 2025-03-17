use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::generator::config::subexport::{
    apply_matcher, match_node, ExtraSettings, RegexMatchConfig,
};
use crate::{Proxy, ProxyType};

/// Settings for parsing proxy configurations
pub struct ParseSettings {
    pub proxy: Option<String>,
    pub exclude_remarks: Vec<String>,
    pub include_remarks: Vec<String>,
    pub stream_rules: Vec<RegexMatchConfig>,
    pub time_rules: Vec<RegexMatchConfig>,
    pub sub_info: Option<String>,
    pub authorized: bool,
    pub request_header: Option<HashMap<String, String>>,
}

impl Default for ParseSettings {
    fn default() -> Self {
        Self {
            proxy: None,
            exclude_remarks: Vec::new(),
            include_remarks: Vec::new(),
            stream_rules: Vec::new(),
            time_rules: Vec::new(),
            sub_info: None,
            authorized: false,
            request_header: None,
        }
    }
}

/// Parse a configuration file and extract proxy nodes
///
/// # Arguments
/// * `filepath` - Path to the configuration file
/// * `nodes` - Vector to store the parsed nodes
///
/// # Returns
/// Number of nodes parsed
pub fn explode_conf(filepath: &str, nodes: &mut Vec<Proxy>) -> usize {
    // Read the file content
    let content = match fs::read_to_string(Path::new(filepath)) {
        Ok(content) => content,
        Err(_) => return 0,
    };

    explode_conf_content(&content, nodes)
}

/// Parse configuration content and extract proxy nodes
///
/// # Arguments
/// * `content` - Configuration content as string
/// * `nodes` - Vector to store the parsed nodes
///
/// # Returns
/// Number of nodes parsed
pub fn explode_conf_content(content: &str, nodes: &mut Vec<Proxy>) -> usize {
    let initial_size = nodes.len();

    // Split the content by lines
    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }

        // Try to parse the line as a proxy
        if let Some(node) = explode(line) {
            nodes.push(node);
        }
    }

    nodes.len() - initial_size
}

/// Parse a single line into a proxy node
///
/// # Arguments
/// * `line` - Line to parse
///
/// # Returns
/// Option containing the parsed proxy node, or None if parsing failed
pub fn explode(line: &str) -> Option<Proxy> {
    // This is a simplified implementation
    // In a real implementation, you would parse different proxy types

    let mut node = Proxy::default();

    // Check for common proxy URI schemes
    if line.starts_with("ss://") {
        // Parse Shadowsocks
        node.proxy_type = ProxyType::Shadowsocks;
        // Parsing logic would go here
        node.remark = line.to_string(); // Placeholder
        Some(node)
    } else if line.starts_with("ssr://") {
        // Parse ShadowsocksR
        node.proxy_type = ProxyType::ShadowsocksR;
        // Parsing logic would go here
        node.remark = line.to_string(); // Placeholder
        Some(node)
    } else if line.starts_with("vmess://") {
        // Parse VMess
        node.proxy_type = ProxyType::VMess;
        // Parsing logic would go here
        node.remark = line.to_string(); // Placeholder
        Some(node)
    } else if line.starts_with("trojan://") {
        // Parse Trojan
        node.proxy_type = ProxyType::Trojan;
        // Parsing logic would go here
        node.remark = line.to_string(); // Placeholder
        Some(node)
    } else {
        // Unknown type
        None
    }
}

/// Add nodes from a link to the node list
///
/// # Arguments
/// * `link` - Link to parse
/// * `all_nodes` - Vector to add nodes to
/// * `group_id` - Group ID to assign to nodes
/// * `parse_set` - Parse settings
///
/// # Returns
/// `Ok(())` if successful, `Err(String)` with error message otherwise
pub fn add_nodes(
    link: &str,
    all_nodes: &mut Vec<Proxy>,
    group_id: i32,
    parse_set: &ParseSettings,
) -> Result<(), String> {
    // Remove quotes
    let link = link.replace("\"", "");

    // Handle nullnode case
    if link == "nullnode" {
        let mut node = Proxy::default();
        node.group_id = 0;
        all_nodes.push(node);
        return Ok(());
    }

    // Extract custom group if present
    let mut custom_group = String::new();
    let mut link = link.to_string();

    if link.starts_with("tag:") {
        if let Some(pos) = link.find(',') {
            custom_group = link[4..pos].to_string();
            link = link[pos + 1..].to_string();
        }
    }

    // Determine link type and process accordingly
    let mut nodes = Vec::new();

    if link.starts_with("https://t.me/socks") || link.starts_with("tg://socks") {
        // SOCKS link
        if let Some(node) = explode(&link) {
            nodes.push(node);
        }
    } else if link.starts_with("https://t.me/http") || link.starts_with("tg://http") {
        // HTTP link
        if let Some(node) = explode(&link) {
            nodes.push(node);
        }
    } else if is_link(&link) || link.starts_with("surge:///install-config") {
        // Subscription link
        // TODO: Implement web fetching and parsing
        return Err("Subscription links not implemented yet".to_string());
    } else if Path::new(&link).exists() {
        // Local file
        if !parse_set.authorized {
            return Err("Not authorized to access local files".to_string());
        }

        if explode_conf(&link, &mut nodes) == 0 {
            return Err("Invalid configuration file".to_string());
        }

        // Filter and process nodes
        filter_nodes(
            &mut nodes,
            &parse_set.exclude_remarks,
            &parse_set.include_remarks,
            group_id,
        );

        // Set group ID and custom group
        for node in &mut nodes {
            node.group_id = group_id;
            if !custom_group.is_empty() {
                node.group = custom_group.clone();
            }
        }

        // Add nodes to the result
        all_nodes.append(&mut nodes);
        return Ok(());
    } else {
        // Try to parse as a direct node
        if let Some(node) = explode(&link) {
            nodes.push(node);
        } else {
            return Err("No valid link found".to_string());
        }
    }

    // Process direct nodes
    for node in &mut nodes {
        node.group_id = group_id;
        if !custom_group.is_empty() {
            node.group = custom_group.clone();
        }
    }

    // Add nodes to the result
    all_nodes.append(&mut nodes);

    Ok(())
}

/// Check if a string is a valid URL
///
/// # Arguments
/// * `url` - URL to check
///
/// # Returns
/// `true` if the URL is valid, `false` otherwise
fn is_link(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

/// Check if a node should be ignored based on include/exclude rules
///
/// # Arguments
/// * `node` - Proxy node to check
/// * `exclude_remarks` - List of exclusion patterns
/// * `include_remarks` - List of inclusion patterns
///
/// # Returns
/// `true` if the node should be ignored, `false` otherwise
pub fn check_ignore(node: &Proxy, exclude_remarks: &[String], include_remarks: &[String]) -> bool {
    // Check exclusion rules
    let excluded = exclude_remarks.iter().any(|x| {
        let mut real_rule = String::new();
        if apply_matcher(x, &mut real_rule, node) {
            if real_rule.is_empty() {
                return true;
            }
            if let Ok(re) = Regex::new(&real_rule) {
                return re.is_match(&node.remark);
            }
        }
        false
    });

    // Check inclusion rules
    let included = if include_remarks.is_empty() {
        true
    } else {
        include_remarks.iter().any(|x| {
            let mut real_rule = String::new();
            if apply_matcher(x, &mut real_rule, node) {
                if real_rule.is_empty() {
                    return true;
                }
                if let Ok(re) = Regex::new(&real_rule) {
                    return re.is_match(&node.remark);
                }
            }
            false
        })
    };

    excluded || !included
}

/// Filter nodes based on include/exclude rules
///
/// # Arguments
/// * `nodes` - Vector of nodes to filter
/// * `exclude_remarks` - List of exclusion patterns
/// * `include_remarks` - List of inclusion patterns
/// * `group_id` - Group ID to assign to nodes
pub fn filter_nodes(
    nodes: &mut Vec<Proxy>,
    exclude_remarks: &[String],
    include_remarks: &[String],
    group_id: i32,
) {
    let mut node_index = 0;
    let mut i = 0;

    while i < nodes.len() {
        if check_ignore(&nodes[i], exclude_remarks, include_remarks) {
            println!(
                "Node {} - {} has been ignored and will not be added.",
                nodes[i].group, nodes[i].remark
            );
            nodes.remove(i);
        } else {
            println!(
                "Node {} - {} has been added.",
                nodes[i].group, nodes[i].remark
            );
            nodes[i].id = node_index;
            nodes[i].group_id = group_id;
            node_index += 1;
            i += 1;
        }
    }
}

/// Remove emoji characters from a remark
///
/// # Arguments
/// * `orig_remark` - Original remark string
///
/// # Returns
/// String with emoji characters removed
pub fn remove_emoji(orig_remark: &str) -> String {
    // In Rust, we can use a regex to remove emoji characters
    if let Ok(re) = Regex::new(r"[\x{1F300}-\x{1F6FF}]") {
        re.replace_all(orig_remark, "").to_string()
    } else {
        orig_remark.to_string()
    }
}

/// Add emoji to a node's remark based on matching rules
///
/// # Arguments
/// * `node` - Proxy node
/// * `emoji_array` - Array of emoji matching rules
/// * `ext` - Extra settings
///
/// # Returns
/// Updated remark string with emoji
pub fn add_emoji(node: &Proxy, emoji_array: &[RegexMatchConfig], _ext: &ExtraSettings) -> String {
    for rule in emoji_array {
        let mut real_rule = String::new();

        // Skip empty replacements
        if rule.replacement.is_empty() {
            continue;
        }

        // Apply matcher and check if node remark matches the rule
        if apply_matcher(&rule.regex, &mut real_rule, node) && !real_rule.is_empty() {
            if let Ok(re) = Regex::new(&real_rule) {
                if re.is_match(&node.remark) {
                    return format!("{} {}", rule.replacement, node.remark);
                }
            }
        }
    }

    // Return original remark if no match
    node.remark.clone()
}

/// Rename a node based on matching rules
///
/// # Arguments
/// * `node` - Proxy node to rename
/// * `rename_array` - Array of rename matching rules
/// * `ext` - Extra settings
pub fn node_rename(node: &mut Proxy, rename_array: &[RegexMatchConfig], _ext: &ExtraSettings) {
    let original_remark = node.remark.clone();

    for rule in rename_array {
        let mut real_rule = String::new();

        if apply_matcher(&rule.regex, &mut real_rule, node) && !real_rule.is_empty() {
            if let Ok(re) = Regex::new(&real_rule) {
                node.remark = re.replace_all(&node.remark, &rule.replacement).to_string();
            }
        }
    }

    // If remark became empty, restore original
    if node.remark.is_empty() {
        node.remark = original_remark;
    }
}

/// Preprocess nodes - apply emoji, rename, and sort operations
///
/// # Arguments
/// * `nodes` - Vector of nodes to process
/// * `ext` - Extra settings
pub fn preprocess_nodes(nodes: &mut Vec<Proxy>, ext: &ExtraSettings) {
    // Process each node
    for node in nodes.iter_mut() {
        // Remove emoji if needed
        if ext.remove_emoji {
            node.remark = trim(&remove_emoji(&node.remark));
        }

        // Apply rename rules
        node_rename(node, &ext.rename_array, ext);

        // Add emoji if needed
        if ext.add_emoji {
            node.remark = add_emoji(node, &ext.emoji_array, ext);
        }
    }

    // Sort nodes if needed
    if ext.sort_flag {
        nodes.sort_by(|a, b| {
            if a.proxy_type == ProxyType::Unknown {
                return std::cmp::Ordering::Greater;
            }
            if b.proxy_type == ProxyType::Unknown {
                return std::cmp::Ordering::Less;
            }
            a.remark.cmp(&b.remark)
        });
    }
}
