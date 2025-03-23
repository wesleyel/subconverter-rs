use std::fs::read_to_string as read_file;
use std::path::Path;

use log::{error, info, warn};

use crate::models::ruleset::{get_ruleset_type_from_url, RulesetContent, RulesetType};
use crate::settings::config::get_settings;
use crate::utils::http::web_get;

/// Ruleset configuration
#[derive(Debug, Clone)]
pub struct RulesetConfig {
    pub group: String,
    pub url: String,
    pub interval: u32,
}

impl RulesetConfig {
    /// Create a new ruleset configuration
    pub fn new(group: &str, url: &str, interval: u32) -> Self {
        Self {
            group: group.to_string(),
            url: url.to_string(),
            interval: interval,
        }
    }

    /// Parse from string in format "group,url[,interval]"
    pub fn from_str(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 2 {
            return None;
        }

        let group = parts[0];
        let url = parts[1];
        let interval = if parts.len() > 2 {
            parts[2].parse::<u32>().unwrap_or(0)
        } else {
            0
        };

        Some(Self::new(group, url, interval))
    }
}

/// Fetch ruleset content from file or URL
pub fn fetch_ruleset(
    url: &str,
    proxy: Option<&str>,
    _cache_timeout: i32,
    _async_fetch: bool,
) -> Result<String, String> {
    // First check if it's a local file
    if Path::new(url).exists() {
        match read_file(url) {
            Ok(content) => return Ok(content),
            Err(e) => return Err(format!("Failed to read ruleset file: {}", e)),
        }
    }

    // If not a file, treat as URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!("Invalid ruleset URL: {}", url));
    }

    // Perform web request
    match web_get(url, proxy, None) {
        Ok((content, _)) => Ok(content),
        Err(e) => Err(format!("Failed to fetch ruleset from URL: {}", e)),
    }
}

/// Refresh rulesets based on configuration
pub fn refresh_rulesets(
    ruleset_list: &[RulesetConfig],
    ruleset_content_array: &mut Vec<RulesetContent>,
) {
    // Clear existing ruleset content
    ruleset_content_array.clear();

    // Get global settings
    let settings = get_settings();
    let proxy = if settings.proxy_ruleset.is_empty() {
        None
    } else {
        Some(settings.proxy_ruleset.as_str())
    };

    for ruleset_config in ruleset_list {
        let rule_group = &ruleset_config.group;
        let rule_url = &ruleset_config.url;

        // Check if it's an inline rule (with [] prefix)
        if let Some(pos) = rule_url.find("[]") {
            info!(
                "Adding rule '{}' with group '{}'",
                &rule_url[pos + 2..],
                rule_group
            );

            let mut ruleset = RulesetContent::new("", rule_group);
            ruleset.set_rule_content(&rule_url[pos..]);
            ruleset_content_array.push(ruleset);
            continue;
        }

        // Determine ruleset type from URL
        let _rule_type = RulesetType::default();
        let rule_url_typed = rule_url.clone();

        if let Some(detected_type) = get_ruleset_type_from_url(rule_url) {
            // Find prefix length and trim it from the URL
            for (prefix, prefix_type) in crate::models::ruleset::RULESET_TYPES.iter() {
                if rule_url.starts_with(prefix) && *prefix_type == detected_type {
                    let rule_url_without_prefix = &rule_url[prefix.len()..];

                    info!(
                        "Updating ruleset URL '{}' with group '{}'",
                        rule_url_without_prefix, rule_group
                    );

                    // Set ruleset properties
                    let mut ruleset = RulesetContent::new(rule_url_without_prefix, rule_group);
                    ruleset.rule_path_typed = rule_url_typed;
                    ruleset.rule_type = detected_type;
                    ruleset.update_interval = ruleset_config.interval;

                    // Fetch the content
                    match fetch_ruleset(
                        rule_url_without_prefix,
                        proxy,
                        settings.cache_ruleset,
                        settings.async_fetch_ruleset,
                    ) {
                        Ok(content) => {
                            ruleset.set_rule_content(&content);
                            ruleset_content_array.push(ruleset);
                        }
                        Err(e) => {
                            error!(
                                "Failed to fetch ruleset from '{}': {}",
                                rule_url_without_prefix, e
                            );
                        }
                    }

                    break;
                }
            }
        } else {
            // No special prefix, use default type
            info!(
                "Updating ruleset URL '{}' with group '{}'",
                rule_url, rule_group
            );

            let mut ruleset = RulesetContent::new(rule_url, rule_group);
            ruleset.rule_path_typed = rule_url.clone();
            ruleset.update_interval = ruleset_config.interval;

            // Fetch the content
            match fetch_ruleset(
                rule_url,
                proxy,
                settings.cache_ruleset,
                settings.async_fetch_ruleset,
            ) {
                Ok(content) => {
                    ruleset.set_rule_content(&content);
                    ruleset_content_array.push(ruleset);
                }
                Err(e) => {
                    error!("Failed to fetch ruleset from '{}': {}", rule_url, e);
                }
            }
        }
    }
}

/// Read ruleset configurations from INI format
pub fn parse_rulesets_from_ini(lines: &[String]) -> Vec<RulesetConfig> {
    let mut rulesets = Vec::new();

    for line in lines {
        if let Some(ruleset) = RulesetConfig::from_str(line) {
            rulesets.push(ruleset);
        } else {
            warn!("Invalid ruleset format: {}", line);
        }
    }

    rulesets
}

/// Parse rulesets from external config format
pub fn parse_rulesets_from_external(config: &str) -> Vec<RulesetConfig> {
    let mut rulesets = Vec::new();

    for line in config.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }

        // Check for section headers
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            continue;
        }

        // Process key-value pairs
        if let Some(pos) = trimmed.find('=') {
            let key = trimmed[..pos].trim();
            let value = trimmed[pos + 1..].trim();

            if key == "ruleset" || key == "surge_ruleset" {
                if let Some(ruleset) = RulesetConfig::from_str(value) {
                    rulesets.push(ruleset);
                }
            }
        }
    }

    rulesets
}

/// Parse a ruleset section from an INI file
pub fn parse_ruleset_from_ini(content: &str) -> Vec<RulesetConfig> {
    let mut result = Vec::new();
    let mut _in_ruleset_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }

        // Check for section headers
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            _in_ruleset_section = false;
            continue;
        }

        // Process key-value pairs
        if let Some(pos) = trimmed.find('=') {
            let key = trimmed[..pos].trim();
            let value = trimmed[pos + 1..].trim();

            if key == "ruleset" || key == "surge_ruleset" {
                if let Some(ruleset) = RulesetConfig::from_str(value) {
                    result.push(ruleset);
                }
            }
        }
    }

    result
}
