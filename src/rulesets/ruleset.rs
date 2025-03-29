use std::fs::read_to_string as read_file;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use log::{debug, error, info, warn};

use crate::models::ruleset::{get_ruleset_type_from_url, RulesetContent, RulesetType};
use crate::models::RulesetConfig;
use crate::utils::http::{parse_proxy, web_get, ProxyConfig};
use crate::utils::md5;
use crate::Settings;

// Create cache directory if it doesn't exist
fn ensure_cache_dir() -> std::io::Result<String> {
    let cache_dir = std::env::current_dir()?.join("cache").join("rulesets");
    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir.to_string_lossy().to_string())
}

// Generate a cache filename from URL
fn get_cache_filename(url: &str) -> String {
    format!("{}.rules", md5(url))
}

/// Fetch ruleset content from file or URL
pub fn fetch_ruleset(
    url: &str,
    proxy: &ProxyConfig,
    cache_timeout: u32,
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

    // Check cache if cache_timeout > 0
    if cache_timeout > 0 {
        // Try to get cache directory
        let cache_dir = match ensure_cache_dir() {
            Ok(dir) => dir,
            Err(e) => {
                warn!("Failed to create cache directory: {}", e);
                // Continue without cache
                return fetch_from_url(url, proxy);
            }
        };

        let cache_file = format!("{}/{}", cache_dir, get_cache_filename(url));
        let cache_path = Path::new(&cache_file);

        // Check if cache file exists and is not expired
        if cache_path.exists() {
            if let Ok(metadata) = cache_path.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(duration) = modified.elapsed() {
                        // Check if cache is still valid
                        if duration.as_secs() < cache_timeout as u64 {
                            debug!("Using cached ruleset for URL: {}", url);
                            match read_file(cache_path) {
                                Ok(content) => return Ok(content),
                                Err(e) => warn!("Failed to read cache file: {}", e),
                                // Continue to fetch from URL
                            }
                        } else {
                            debug!("Cache expired for URL: {}", url);
                        }
                    }
                }
            }
        }

        // Cache doesn't exist, is expired, or couldn't be read - fetch from URL and update cache
        match fetch_from_url(url, proxy) {
            Ok(content) => {
                // Update cache
                if let Err(e) = save_to_cache(&cache_file, &content) {
                    warn!("Failed to update cache for URL {}: {}", url, e);
                }
                Ok(content)
            }
            Err(e) => Err(e),
        }
    } else {
        // No caching, directly fetch from URL
        fetch_from_url(url, proxy)
    }
}

// Helper function to fetch content from URL
fn fetch_from_url(url: &str, proxy: &ProxyConfig) -> Result<String, String> {
    debug!("Fetching ruleset from URL: {}", url);
    match web_get(url, proxy, None) {
        Ok((content, _)) => Ok(content),
        Err(e) => Err(format!("Failed to fetch ruleset from URL: {}", e)),
    }
}

// Helper function to save content to cache
fn save_to_cache(cache_file: &str, content: &str) -> std::io::Result<()> {
    let mut file = File::create(cache_file)?;
    file.write_all(content.as_bytes())?;
    Ok(())
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
                        "Updating {} ruleset URL '{}' with group '{}'",
                        prefix, rule_url_without_prefix, rule_group
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::file_exists;
    use crate::utils::http::parse_proxy;
    use std::fs;
    use std::time::Duration;

    // 使用工厂函数创建测试用 ProxyConfig
    fn create_test_proxy() -> ProxyConfig {
        parse_proxy("NONE")
    }

    #[test]
    fn test_fetch_ruleset_cache() {
        // Setup test
        let test_url = "https://example.com/test_ruleset.conf";
        let proxy = &create_test_proxy();
        let cache_dir = std::env::current_dir()
            .unwrap()
            .join("cache")
            .join("rulesets");
        let cache_file = format!("{}/{}.rules", cache_dir.to_string_lossy(), md5(test_url));

        // Make sure the cache directory exists
        fs::create_dir_all(&cache_dir).unwrap();

        // Delete the cache file if it exists
        if file_exists(&cache_file) {
            fs::remove_file(&cache_file).unwrap();
        }

        // Create a mock function for web_get that will be used by fetch_from_url
        // We can't directly replace it, but we can test the cache logic
        let cache_content = "# Test ruleset\nRULE-SET,https://example.com/ruleset2.conf,DIRECT";

        // Manually create a cache file
        let mut file = File::create(&cache_file).unwrap();
        file.write_all(cache_content.as_bytes()).unwrap();

        // Test cache hit
        let result1 = fetch_ruleset(test_url, proxy, 3600, false);
        assert!(result1.is_ok());
        if let Ok(content) = result1 {
            assert_eq!(content, cache_content);
        }

        // Allow some time to pass
        std::thread::sleep(Duration::from_secs(1));

        // Modify the cache file to test if it's re-read when still valid
        let updated_content =
            "# Updated ruleset\nRULE-SET,https://example.com/ruleset3.conf,REJECT";
        let mut file = File::create(&cache_file).unwrap();
        file.write_all(updated_content.as_bytes()).unwrap();

        // Test cache hit with updated content
        let result2 = fetch_ruleset(test_url, proxy, 3600, false);
        assert!(result2.is_ok());
        if let Ok(content) = result2 {
            assert_eq!(content, updated_content);
        }

        // Clean up
        fs::remove_file(&cache_file).unwrap();
    }

    #[test]
    fn test_fetch_ruleset_cache_expiration() {
        // This test would simulate cache expiration by using a small cache timeout
        // In a real test, you would use a mock time provider, but for simplicity
        // we'll just check the file operations

        let test_url = "https://example.com/expiring_ruleset.conf";
        let proxy = &create_test_proxy();
        let cache_dir = std::env::current_dir()
            .unwrap()
            .join("cache")
            .join("rulesets");
        let cache_file = format!("{}/{}.rules", cache_dir.to_string_lossy(), md5(test_url));

        // Make sure the cache directory exists
        fs::create_dir_all(&cache_dir).unwrap();

        // Delete the cache file if it exists
        if file_exists(&cache_file) {
            fs::remove_file(&cache_file).unwrap();
        }

        // Create a cache file
        let cache_content = "# Expiring ruleset\nRULE-SET,https://example.com/expired.conf,DIRECT";
        let mut file = File::create(&cache_file).unwrap();
        file.write_all(cache_content.as_bytes()).unwrap();

        // Set file time to 1 hour ago (this would be better with a mock time provider)
        // Instead, we'll rely on the behavior of fetch_ruleset when cache_timeout is 0

        // Test cache miss due to zero cache_timeout
        let result_no_cache = fetch_ruleset(test_url, proxy, 0, false);
        // This will fail since we can't actually make HTTP requests in tests
        assert!(result_no_cache.is_err());
        assert!(result_no_cache
            .unwrap_err()
            .contains("Failed to fetch ruleset from URL"));

        // Clean up
        if file_exists(&cache_file) {
            fs::remove_file(&cache_file).unwrap();
        }
    }
}
