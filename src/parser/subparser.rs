use crate::models::Proxy;
use crate::parser::explodes::*;
use crate::parser::infoparser::{get_sub_info_from_nodes, get_sub_info_from_ssd};
use crate::parser::settings::{CaseInsensitiveString, ParseSettings, RegexMatchConfigs};
use crate::utils::base64::{base64_decode, base64_encode};
use crate::utils::http::{get_sub_info_from_response, web_get};
use crate::utils::matcher::{apply_matcher, match_range};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use url::Url;

/// Equivalent to ConfType enum in C++
#[derive(Debug, PartialEq, Eq)]
pub enum ConfType {
    SOCKS,
    HTTP,
    SUB,
    Netch,
    Local,
    Unknown,
}

/// Transform of C++ addNodes function
/// Adds nodes from a link to the provided vector
///
/// # Arguments
/// * `link` - Link to parse for proxies
/// * `all_nodes` - Vector to add nodes to
/// * `group_id` - Group ID to assign to nodes
/// * `parse_settings` - Settings for parsing
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(String)` with error message on failure
pub fn add_nodes(
    mut link: String,
    all_nodes: &mut Vec<Proxy>,
    group_id: u32,
    parse_settings: &ParseSettings,
) -> Result<(), String> {
    // Extract references to settings for easier access
    let proxy = parse_settings.proxy.as_deref();
    let sub_info = parse_settings.sub_info.as_deref().unwrap_or("");
    let exclude_remarks = parse_settings.exclude_remarks.as_ref();
    let include_remarks = parse_settings.include_remarks.as_ref();
    let stream_rules = parse_settings.stream_rules.as_ref();
    let time_rules = parse_settings.time_rules.as_ref();
    let request_header = parse_settings.request_header.as_ref();
    let authorized = parse_settings.authorized;

    // Variables to store data during processing
    let mut link_type = ConfType::Unknown;
    let mut nodes: Vec<Proxy> = Vec::new();
    let mut node = Proxy::default();
    let mut custom_group = String::new();

    // Clean up the link string
    link = link.replace("\"", "");

    // Handle JavaScript scripts (Not implementing JS support here)
    #[cfg(feature = "js_runtime")]
    if authorized && link.starts_with("script:") {
        // Script processing would go here
        return Err("Script processing not implemented".to_string());
    }

    // Handle tag: prefix for custom group
    if link.starts_with("tag:") {
        if let Some(pos) = link.find(',') {
            custom_group = link[4..pos].to_string();
            link = link[pos + 1..].to_string();
        }
    }

    // Handle null node
    if link == "nullnode" {
        let mut null_node = Proxy::default();
        null_node.group_id = 0;
        all_nodes.push(null_node);
        return Ok(());
    }

    // Determine link type
    if link.starts_with("https://t.me/socks") || link.starts_with("tg://socks") {
        link_type = ConfType::SOCKS;
    } else if link.starts_with("https://t.me/http") || link.starts_with("tg://http") {
        link_type = ConfType::HTTP;
    } else if is_link(&link) || link.starts_with("surge:///install-config") {
        link_type = ConfType::SUB;
    } else if link.starts_with("Netch://") {
        link_type = ConfType::Netch;
    } else if file_exists(&link) {
        link_type = ConfType::Local;
    }

    match link_type {
        ConfType::SUB => {
            // Handle subscription links
            if link.starts_with("surge:///install-config") {
                // Extract URL from Surge config link
                if let Some(url_arg) = get_url_arg(&link, "url") {
                    link = url_decode(&url_arg);
                }
            }

            // Download subscription content
            let sub_content = match web_get(&link, proxy, request_header) {
                Ok(content) => content,
                Err(err) => return Err(format!("Cannot download subscription data: {}", err)),
            };

            if !sub_content.is_empty() {
                // Parse the subscription content
                let result = explode_conf_content(&sub_content, &mut nodes);
                if result > 0 {
                    // Get subscription info
                    let mut _sub_info = String::new();

                    if sub_content.starts_with("ssd://") {
                        // Extract info from SSD subscription
                        if let Some(info) = get_sub_info_from_ssd(&sub_content) {
                            _sub_info = info;
                            // If needed, store or use _sub_info elsewhere
                        }
                    } else {
                        // Try to get info from header or nodes
                        // No headers available from the web_get call, so only try from nodes
                        if let (Some(stream_rules_unwrapped), Some(time_rules_unwrapped)) =
                            (stream_rules, time_rules)
                        {
                            if let Some(info) = get_sub_info_from_nodes(
                                &nodes,
                                stream_rules_unwrapped,
                                time_rules_unwrapped,
                            ) {
                                _sub_info = info;
                                // If needed, store or use _sub_info elsewhere
                            }
                        }
                    }

                    // Filter nodes and set group info
                    filter_nodes(&mut nodes, exclude_remarks, include_remarks, group_id);
                    for node in &mut nodes {
                        node.group_id = group_id;
                        if !custom_group.is_empty() {
                            node.group = custom_group.clone();
                        }
                    }

                    // Add nodes to result vector
                    all_nodes.append(&mut nodes);
                    Ok(())
                } else {
                    Err(format!("Invalid subscription: '{}'", link))
                }
            } else {
                Err("Cannot download subscription data".to_string())
            }
        }
        ConfType::Local => {
            if !authorized {
                return Err("Not authorized to access local files".to_string());
            }

            // Read and parse local file
            let result = explode_conf(&link, &mut nodes);
            if result > 0 {
                // The rest is similar to SUB case
                // Get subscription info
                let mut _sub_info = String::new();

                if link.starts_with("ssd://") {
                    // Extract info from SSD subscription
                    if let Some(info) = get_sub_info_from_ssd(&link) {
                        _sub_info = info;
                        // If needed, store or use _sub_info elsewhere
                    }
                } else {
                    // Try to get info from nodes
                    if let (Some(stream_rules_unwrapped), Some(time_rules_unwrapped)) =
                        (stream_rules, time_rules)
                    {
                        if let Some(info) = get_sub_info_from_nodes(
                            &nodes,
                            stream_rules_unwrapped,
                            time_rules_unwrapped,
                        ) {
                            _sub_info = info;
                            // If needed, store or use _sub_info elsewhere
                        }
                    }
                }

                filter_nodes(&mut nodes, exclude_remarks, include_remarks, group_id);
                for node in &mut nodes {
                    node.group_id = group_id;
                    if !custom_group.is_empty() {
                        node.group = custom_group.clone();
                    }
                }
                all_nodes.append(&mut nodes);
                Ok(())
            } else {
                Err("Invalid configuration file".to_string())
            }
        }
        _ => {
            // Handle direct link to a single proxy
            if explode(&link, &mut node) {
                if node.proxy_type == crate::models::ProxyType::Unknown {
                    return Err("No valid link found".to_string());
                }
                node.group_id = group_id;
                if !custom_group.is_empty() {
                    node.group = custom_group;
                }
                all_nodes.push(node);
                Ok(())
            } else {
                Err("No valid link found".to_string())
            }
        }
    }
}

/// Checks if a string is a valid URL
fn is_link(link: &str) -> bool {
    link.starts_with("http://")
        || link.starts_with("https://")
        || link.starts_with("data:")
        || link.starts_with("content://")
}

/// Checks if a file exists at the given path
fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Decodes URL-encoded strings
fn url_decode(input: &str) -> String {
    let mut result = input.to_string();

    // Common URL decodings
    let replacements = [
        ("%20", " "),
        ("%21", "!"),
        ("%22", "\""),
        ("%23", "#"),
        ("%24", "$"),
        ("%25", "%"),
        ("%26", "&"),
        ("%27", "'"),
        ("%28", "("),
        ("%29", ")"),
        ("%2A", "*"),
        ("%2B", "+"),
        ("%2C", ","),
        ("%2D", "-"),
        ("%2E", "."),
        ("%2F", "/"),
        ("%3A", ":"),
        ("%3B", ";"),
        ("%3C", "<"),
        ("%3D", "="),
        ("%3E", ">"),
        ("%3F", "?"),
        ("%40", "@"),
        ("%5B", "["),
        ("%5C", "\\"),
        ("%5D", "]"),
        ("%5E", "^"),
        ("%5F", "_"),
        ("%60", "`"),
        ("%7B", "{"),
        ("%7C", "|"),
        ("%7D", "}"),
        ("%7E", "~"),
        ("+", " "),
    ];

    for (encoded, decoded) in replacements.iter() {
        result = result.replace(encoded, decoded);
    }

    result
}

/// Extracts a specific argument from a URL
fn get_url_arg(url: &str, arg_name: &str) -> Option<String> {
    if let Some(query_start) = url.find('?') {
        let query = &url[query_start + 1..];
        for pair in query.split('&') {
            let parts: Vec<&str> = pair.split('=').collect();
            if parts.len() == 2 && parts[0] == arg_name {
                return Some(parts[1].to_string());
            }
        }
    }
    None
}

/// Parses a configuration file into a vector of Proxy objects
/// Returns the number of proxies parsed
fn explode_conf(path: &str, nodes: &mut Vec<Proxy>) -> i32 {
    match fs::read_to_string(path) {
        Ok(content) => explode_conf_content(&content, nodes),
        Err(_) => 0,
    }
}

/// Filters nodes based on include/exclude rules
fn filter_nodes(
    nodes: &mut Vec<Proxy>,
    exclude_remarks: Option<&Vec<String>>,
    include_remarks: Option<&Vec<String>>,
    group_id: u32,
) {
    let mut node_index = 0;
    let mut i = 0;

    while i < nodes.len() {
        if should_ignore(&nodes[i], exclude_remarks, include_remarks) {
            // If this node should be ignored, remove it
            nodes.remove(i);
        } else {
            // Otherwise update its ID and groupID and keep it
            nodes[i].id = node_index;
            nodes[i].group_id = group_id;
            node_index += 1;
            i += 1;
        }
    }
}

/// Determines if a node should be ignored based on its remarks and the filtering rules
fn should_ignore(
    node: &Proxy,
    exclude_remarks: Option<&Vec<String>>,
    include_remarks: Option<&Vec<String>>,
) -> bool {
    let mut excluded = false;
    let mut included = true; // Default to true if no include rules

    // Check exclude rules
    if let Some(excludes) = exclude_remarks {
        for pattern in excludes {
            let mut real_rule = String::new();
            if apply_matcher(pattern, &mut real_rule, node) {
                if !real_rule.is_empty() && real_rule.contains(&node.remark) {
                    excluded = true;
                    break;
                } else if real_rule.is_empty() && pattern == &node.remark {
                    excluded = true;
                    break;
                }
            }
        }
    }

    // Check include rules if they exist
    if let Some(includes) = include_remarks {
        if !includes.is_empty() {
            included = false; // Start with false when we have include rules
            for pattern in includes {
                let mut real_rule = String::new();
                if apply_matcher(pattern, &mut real_rule, node) {
                    if !real_rule.is_empty() && real_rule.contains(&node.remark) {
                        included = true;
                        break;
                    } else if real_rule.is_empty() && pattern == &node.remark {
                        included = true;
                        break;
                    }
                }
            }
        }
    }

    // A node is ignored if it's excluded OR not included
    excluded || !included
}
