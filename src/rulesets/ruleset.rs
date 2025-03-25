use std::fs::read_to_string as read_file;
use std::path::Path;

use log::{error, info, warn};

use crate::models::ruleset::{get_ruleset_type_from_url, RulesetContent, RulesetType};
use crate::models::RulesetConfig;
use crate::utils::http::{parse_proxy, web_get, ProxyConfig};
use crate::Settings;

/// Fetch ruleset content from file or URL
pub fn fetch_ruleset(
    url: &str,
    proxy: &ProxyConfig,
    _cache_timeout: u32,
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
    let settings = Settings::current();
    let proxy = parse_proxy(&settings.proxy_ruleset);

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
                        &proxy,
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
                &proxy,
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
