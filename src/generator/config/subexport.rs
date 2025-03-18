use crate::{
    utils::{ini_reader::IniReader, yaml::YamlNode},
    Proxy,
};
use regex::Regex;
use std::collections::HashMap;

/// Configuration for regex-based matching operations
#[derive(Debug, Clone)]
pub struct RegexMatchConfig {
    pub regex: String,
    pub replacement: String,
}

/// Collection of regex match configurations
pub type RegexMatchConfigs = Vec<RegexMatchConfig>;

/// Settings for subscription export operations
#[derive(Debug, Clone)]
pub struct ExtraSettings {
    /// Whether to enable the rule generator
    pub enable_rule_generator: bool,
    /// Whether to overwrite original rules
    pub overwrite_original_rules: bool,
    /// Rename operations to apply
    pub rename_array: RegexMatchConfigs,
    /// Emoji operations to apply
    pub emoji_array: RegexMatchConfigs,
    /// Whether to add emoji
    pub add_emoji: bool,
    /// Whether to remove emoji
    pub remove_emoji: bool,
    /// Whether to append proxy type
    pub append_proxy_type: bool,
    /// Whether to output as node list
    pub nodelist: bool,
    /// Whether to sort nodes
    pub sort_flag: bool,
    /// Whether to filter deprecated nodes
    pub filter_deprecated: bool,
    /// Whether to use new field names in Clash
    pub clash_new_field_name: bool,
    /// Whether to use scripts in Clash
    pub clash_script: bool,
    /// Path to Surge SSR binary
    pub surge_ssr_path: String,
    /// Prefix for managed configs
    pub managed_config_prefix: String,
    /// QuantumultX device ID
    pub quanx_dev_id: String,
    /// UDP support flag
    pub udp: Option<bool>,
    /// TCP Fast Open support flag
    pub tfo: Option<bool>,
    /// Skip certificate verification flag
    pub skip_cert_verify: Option<bool>,
    /// TLS 1.3 support flag
    pub tls13: Option<bool>,
    /// Whether to use classical ruleset in Clash
    pub clash_classical_ruleset: bool,
    /// Script for sorting nodes
    pub sort_script: String,
    /// Style for Clash proxies output
    pub clash_proxies_style: String,
    /// Style for Clash proxy groups output
    pub clash_proxy_groups_style: String,
    /// Whether the export is authorized
    pub authorized: bool,
    /// JavaScript runtime context (not implemented in Rust version)
    pub js_context: Option<()>,
}

impl Default for ExtraSettings {
    fn default() -> Self {
        ExtraSettings {
            enable_rule_generator: true,
            overwrite_original_rules: true,
            rename_array: Vec::new(),
            emoji_array: Vec::new(),
            add_emoji: false,
            remove_emoji: false,
            append_proxy_type: false,
            nodelist: false,
            sort_flag: false,
            filter_deprecated: false,
            clash_new_field_name: false,
            clash_script: false,
            surge_ssr_path: String::new(),
            managed_config_prefix: String::new(),
            quanx_dev_id: String::new(),
            udp: None,
            tfo: None,
            skip_cert_verify: None,
            tls13: None,
            clash_classical_ruleset: false,
            sort_script: String::new(),
            clash_proxies_style: "flow".to_string(),
            clash_proxy_groups_style: "flow".to_string(),
            authorized: false,
            js_context: None,
        }
    }
}

/// Configuration for proxy groups
#[derive(Debug, Clone)]
pub struct ProxyGroupConfig {
    pub name: String,
    pub type_field: String,
    pub url: String,
    pub interval: u32,
    pub tolerance: u32,
    pub proxies: Vec<String>,
    pub use_provider: bool,
}

/// Collection of proxy group configurations
pub type ProxyGroupConfigs = Vec<ProxyGroupConfig>;

/// Match a range against a target integer value
///
/// This function checks if a target value is within a specified range.
/// The range can be defined in different formats like "1", "1-100", ">100", etc.
///
/// # Arguments
/// * `range` - Range specification string
/// * `target` - Target integer value to check
///
/// # Returns
/// `true` if target is within the specified range, `false` otherwise
pub fn match_range(range: &str, target: i32) -> bool {
    // Empty range matches everything
    if range.is_empty() {
        return true;
    }

    // Direct equality check
    if let Ok(value) = range.parse::<i32>() {
        return target == value;
    }

    // Range with dash: "1-100"
    if range.contains('-') {
        let parts: Vec<&str> = range.split('-').collect();
        if parts.len() == 2 {
            let start = parts[0].parse::<i32>().unwrap_or(i32::MIN);
            let end = parts[1].parse::<i32>().unwrap_or(i32::MAX);
            return target >= start && target <= end;
        }
    }

    // Greater than: ">100"
    if range.starts_with('>') {
        if let Ok(value) = range[1..].parse::<i32>() {
            return target > value;
        }
    }

    // Greater than or equal: ">=100"
    if range.starts_with(">=") {
        if let Ok(value) = range[2..].parse::<i32>() {
            return target >= value;
        }
    }

    // Less than: "<100"
    if range.starts_with('<') {
        if let Ok(value) = range[1..].parse::<i32>() {
            return target < value;
        }
    }

    // Less than or equal: "<=100"
    if range.starts_with("<=") {
        if let Ok(value) = range[2..].parse::<i32>() {
            return target <= value;
        }
    }

    false
}

/// Process remarks for display
///
/// This function processes the remark string according to various settings,
/// such as adding/removing emoji, applying rename rules, etc.
///
/// # Arguments
/// * `remark` - Remark to process
/// * `ext` - Extra settings for processing
/// * `proc_comma` - Whether to process commas in the remark
pub fn process_remark(remark: &mut String, ext: &ExtraSettings, proc_comma: bool) {
    if proc_comma && remark.contains(',') {
        *remark = remark.replace(',', "_");
    }

    // Apply rename regex rules
    for rule in &ext.rename_array {
        if let Ok(re) = Regex::new(&rule.regex) {
            *remark = re.replace_all(remark, &rule.replacement).to_string();
        }
    }

    // Add or remove emoji based on settings
    if ext.remove_emoji {
        if let Ok(re) = Regex::new(r"[\x{1F300}-\x{1F6FF}]") {
            *remark = re.replace_all(remark, "").to_string();
        }
    }

    // Other emoji processing would go here
}
