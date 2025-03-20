use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// External configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExternalConfig {
    // Proxy group configurations
    pub custom_proxy_group: Vec<String>,

    // Ruleset configurations
    pub surge_ruleset: Vec<String>,

    // Rule base configurations
    pub clash_rule_base: String,
    pub surge_rule_base: String,
    pub surfboard_rule_base: String,
    pub mellow_rule_base: String,
    pub quan_rule_base: String,
    pub quanx_rule_base: String,
    pub loon_rule_base: String,
    pub sssub_rule_base: String,
    pub singbox_rule_base: String,

    // Node renaming and emoji configurations
    pub rename: Vec<String>,
    pub emoji: Vec<String>,

    // Node filtering
    pub include: Vec<String>,
    pub exclude: Vec<String>,

    // Template arguments
    pub template_args: HashMap<String, String>,

    // Rule generation settings
    pub overwrite_original_rules: bool,
    pub enable_rule_generator: bool,

    // Emoji settings
    pub add_emoji: Option<bool>,
    pub remove_old_emoji: Option<bool>,
}

impl ExternalConfig {
    /// Load external config from file or URL
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        use crate::settings::config::global;
        use crate::utils::http::web_get;

        let mut content = String::new();

        // Get proxy from global config
        let settings = global.read().unwrap();
        let proxy = if settings.proxy_config.is_empty() {
            None
        } else {
            Some(settings.proxy_config.clone())
        };
        drop(settings);

        // Try to load the content from file or URL
        if path.starts_with("http://") || path.starts_with("https://") {
            let (data, _) = web_get(path, proxy.as_deref(), None)?;
            content = data;
        } else {
            content = std::fs::read_to_string(path)?;
        }

        // Try to parse as YAML first
        if content.contains("custom:") {
            return Ok(serde_yaml::from_str(&content)?);
        }

        // Try to parse as TOML
        if let Ok(_) = toml::from_str::<toml::Value>(&content) {
            return Ok(toml::from_str(&content)?);
        }

        // Default to INI
        let mut config = ExternalConfig::default();
        config.load_from_ini(&content)?;
        Ok(config)
    }

    /// Load configuration from INI format
    fn load_from_ini(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut current_section = "";

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
                continue;
            }

            // Check for section header
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = &trimmed[1..trimmed.len() - 1];
                continue;
            }

            // Process key-value pairs
            if let Some(pos) = trimmed.find('=') {
                let key = trimmed[..pos].trim();
                let value = trimmed[pos + 1..].trim();

                if current_section == "custom" {
                    match key {
                        "overwrite_original_rules" => {
                            self.overwrite_original_rules =
                                value.to_lowercase() == "true" || value == "1";
                        }
                        "enable_rule_generator" => {
                            self.enable_rule_generator =
                                value.to_lowercase() == "true" || value == "1";
                        }
                        "add_emoji" => {
                            self.add_emoji = Some(value.to_lowercase() == "true" || value == "1");
                        }
                        "remove_old_emoji" => {
                            self.remove_old_emoji =
                                Some(value.to_lowercase() == "true" || value == "1");
                        }
                        "clash_rule_base" => {
                            self.clash_rule_base = value.to_string();
                        }
                        "surge_rule_base" => {
                            self.surge_rule_base = value.to_string();
                        }
                        "custom_proxy_group" => {
                            self.custom_proxy_group.push(value.to_string());
                        }
                        "ruleset" => {
                            self.surge_ruleset.push(value.to_string());
                        }
                        "rename" => {
                            self.rename.push(value.to_string());
                        }
                        "emoji" => {
                            self.emoji.push(value.to_string());
                        }
                        "include_remarks" => {
                            self.include.push(value.to_string());
                        }
                        "exclude_remarks" => {
                            self.exclude.push(value.to_string());
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}

/// Load external configuration from file or URL
pub fn load_external_config(path: &str) -> Result<ExternalConfig, Box<dyn std::error::Error>> {
    ExternalConfig::load_from_file(path)
}
