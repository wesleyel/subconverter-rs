use serde::Deserialize;
use std::collections::HashMap;

use crate::models::RegexMatchConfig;

/// INI external settings structure
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct IniExternalSettings {
    // Rule bases
    pub clash_rule_base: String,
    pub surge_rule_base: String,
    pub surfboard_rule_base: String,
    pub mellow_rule_base: String,
    pub quan_rule_base: String,
    pub quanx_rule_base: String,
    pub loon_rule_base: String,
    pub sssub_rule_base: String,
    pub singbox_rule_base: String,

    // Rule generation options
    pub enable_rule_generator: bool,
    pub overwrite_original_rules: bool,

    // Emoji options
    pub add_emoji: bool,
    pub remove_old_emoji: bool,

    // Filtering options
    pub include_remarks: Vec<String>,
    pub exclude_remarks: Vec<String>,

    // Rulesets and proxy groups (stored as raw strings)
    pub rulesets: Vec<String>,
    pub custom_proxy_groups: Vec<String>,

    // Rename rules
    pub rename: Vec<RegexMatchConfig>,

    // Template arguments
    pub tpl_args: Option<HashMap<String, String>>,
}

impl IniExternalSettings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load settings from INI format
    pub fn load_from_ini(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut current_section = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
                continue;
            }

            // Check for section header
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len() - 1].to_string();
                continue;
            }

            // Process key-value pairs
            if let Some(pos) = trimmed.find('=') {
                let key = trimmed[..pos].trim();
                let value = trimmed[pos + 1..].trim();

                match current_section.as_str() {
                    "custom" => self.process_custom_section(key, value),
                    "rename" => self.process_rename_section(key, value),
                    "template" => self.process_template_section(key, value),
                    _ => {} // Ignore unknown sections
                }
            }
        }
        Ok(())
    }

    fn process_custom_section(&mut self, key: &str, value: &str) {
        match key {
            "clash_rule_base" => self.clash_rule_base = value.to_string(),
            "surge_rule_base" => self.surge_rule_base = value.to_string(),
            "surfboard_rule_base" => self.surfboard_rule_base = value.to_string(),
            "mellow_rule_base" => self.mellow_rule_base = value.to_string(),
            "quan_rule_base" => self.quan_rule_base = value.to_string(),
            "quanx_rule_base" => self.quanx_rule_base = value.to_string(),
            "loon_rule_base" => self.loon_rule_base = value.to_string(),
            "sssub_rule_base" => self.sssub_rule_base = value.to_string(),
            "singbox_rule_base" => self.singbox_rule_base = value.to_string(),
            "enable_rule_generator" => self.enable_rule_generator = parse_bool(value),
            "overwrite_original_rules" => self.overwrite_original_rules = parse_bool(value),
            "add_emoji" => self.add_emoji = parse_bool(value),
            "remove_old_emoji" => self.remove_old_emoji = parse_bool(value),
            "include_remarks" => {
                self.include_remarks = value.split(',').map(|s| s.trim().to_string()).collect();
            }
            "exclude_remarks" => {
                self.exclude_remarks = value.split(',').map(|s| s.trim().to_string()).collect();
            }
            "ruleset" | "surge_ruleset" => {
                self.rulesets.push(value.to_string());
            }
            "custom_proxy_group" => {
                self.custom_proxy_groups.push(value.to_string());
            }
            _ => {}
        }
    }

    fn process_rename_section(&mut self, key: &str, value: &str) {
        // Handle rename rules
        if key.starts_with("rename_") {
            // Create a RegexMatchConfig from key/value
            let config = RegexMatchConfig {
                _match: key.trim_start_matches("rename_").to_string(),
                replace: value.to_string(),
            };
            self.rename.push(config);
        }
    }

    fn process_template_section(&mut self, key: &str, value: &str) {
        // Initialize tpl_args if it's None
        if self.tpl_args.is_none() {
            self.tpl_args = Some(HashMap::new());
        }

        // Add the key-value pair to the template arguments
        if let Some(ref mut args) = self.tpl_args {
            args.insert(key.to_string(), value.to_string());
        }
    }
}

/// Parse a string as boolean
fn parse_bool(value: &str) -> bool {
    value.to_lowercase() == "true" || value == "1"
}
