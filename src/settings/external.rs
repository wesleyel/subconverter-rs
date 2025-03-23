use std::collections::HashMap;
use std::path::Path;

use log::debug;
use serde_yml;
use toml;

use crate::models::extra_settings::RegexMatchConfig;
use crate::settings::config::get_settings;
use crate::settings::ruleset::RulesetConfig;
use crate::utils::file::read_file;
use crate::utils::http::web_get;

/// External configuration structure
#[derive(Debug, Clone, Default)]
pub struct ExternalConfig {
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
    pub enable_rule_generator: Option<bool>,
    pub overwrite_original_rules: Option<bool>,

    // Custom rules and groups
    pub custom_proxy_group: Vec<String>,
    pub surge_ruleset: Vec<RulesetConfig>,

    // Node operations
    pub rename: Vec<RegexMatchConfig>,
    pub emoji: Vec<RegexMatchConfig>,
    pub add_emoji: Option<bool>,
    pub remove_old_emoji: Option<bool>,

    // Filtering
    pub include: Vec<String>,
    pub exclude: Vec<String>,

    // Template arguments
    pub tpl_args: Option<HashMap<String, String>>,
}

impl ExternalConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load external configuration from file or URL
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut _content = String::new();

        // Try to load content from file or URL
        if path.starts_with("http://") || path.starts_with("https://") {
            let settings = get_settings();
            let proxy = if settings.proxy_config.is_empty() {
                None
            } else {
                Some(settings.proxy_config.as_str())
            };

            match web_get(path, proxy, None) {
                Ok((data, _)) => _content = data,
                Err(e) => return Err(format!("Failed to fetch external config: {}", e).into()),
            }
        } else if Path::new(path).exists() {
            match read_file(path) {
                Ok(data) => _content = data,
                Err(e) => return Err(format!("Failed to read external config file: {}", e).into()),
            }
        } else {
            return Err(format!("External config path not found: {}", path).into());
        }

        // Try to parse the content
        // let external_config = Self::new();

        // Try YAML format first
        if _content.contains("custom:") {
            match Self::parse_yaml(&_content) {
                Ok(config) => return Ok(config),
                Err(e) => debug!("Failed to parse external config as YAML: {}", e),
            }
        }

        // Try TOML format
        match Self::parse_toml(&_content) {
            Ok(config) => return Ok(config),
            Err(e) => debug!("Failed to parse external config as TOML: {}", e),
        }

        // Fall back to INI format
        match Self::parse_ini(&_content) {
            Ok(config) => Ok(config),
            Err(e) => Err(format!("Failed to parse external config as INI: {}", e).into()),
        }
    }

    /// Parse external config from YAML format
    fn parse_yaml(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let yaml_value: serde_yml::Value = serde_yml::from_str(content)?;
        let mut config = Self::new();

        if let Some(custom) = yaml_value.get("custom") {
            // Parse rule bases
            if let Some(val) = custom.get("clash_rule_base").and_then(|v| v.as_str()) {
                config.clash_rule_base = val.to_string();
            }
            if let Some(val) = custom.get("surge_rule_base").and_then(|v| v.as_str()) {
                config.surge_rule_base = val.to_string();
            }
            if let Some(val) = custom.get("surfboard_rule_base").and_then(|v| v.as_str()) {
                config.surfboard_rule_base = val.to_string();
            }
            if let Some(val) = custom.get("mellow_rule_base").and_then(|v| v.as_str()) {
                config.mellow_rule_base = val.to_string();
            }
            if let Some(val) = custom.get("quan_rule_base").and_then(|v| v.as_str()) {
                config.quan_rule_base = val.to_string();
            }
            if let Some(val) = custom.get("quanx_rule_base").and_then(|v| v.as_str()) {
                config.quanx_rule_base = val.to_string();
            }
            if let Some(val) = custom.get("loon_rule_base").and_then(|v| v.as_str()) {
                config.loon_rule_base = val.to_string();
            }
            if let Some(val) = custom.get("sssub_rule_base").and_then(|v| v.as_str()) {
                config.sssub_rule_base = val.to_string();
            }
            if let Some(val) = custom.get("singbox_rule_base").and_then(|v| v.as_str()) {
                config.singbox_rule_base = val.to_string();
            }

            // Parse rule generation options
            if let Some(val) = custom
                .get("enable_rule_generator")
                .and_then(|v| v.as_bool())
            {
                config.enable_rule_generator = Some(val);
            }
            if let Some(val) = custom
                .get("overwrite_original_rules")
                .and_then(|v| v.as_bool())
            {
                config.overwrite_original_rules = Some(val);
            }

            // Parse emoji options
            if let Some(val) = custom.get("add_emoji").and_then(|v| v.as_bool()) {
                config.add_emoji = Some(val);
            }
            if let Some(val) = custom.get("remove_old_emoji").and_then(|v| v.as_bool()) {
                config.remove_old_emoji = Some(val);
            }

            // Parse include/exclude remarks
            if let Some(includes) = custom.get("include_remarks").and_then(|v| v.as_sequence()) {
                for item in includes {
                    if let Some(val) = item.as_str() {
                        config.include.push(val.to_string());
                    }
                }
            }
            if let Some(excludes) = custom.get("exclude_remarks").and_then(|v| v.as_sequence()) {
                for item in excludes {
                    if let Some(val) = item.as_str() {
                        config.exclude.push(val.to_string());
                    }
                }
            }

            // Parse rulesets
            let ruleset_key = if custom.get("rulesets").is_some() {
                "rulesets"
            } else {
                "surge_ruleset"
            };

            if let Some(rulesets) = custom.get(ruleset_key).and_then(|v| v.as_sequence()) {
                for ruleset in rulesets {
                    if let (Some(group), Some(url)) = (
                        ruleset.get("group").and_then(|v| v.as_str()),
                        ruleset
                            .get("rule")
                            .and_then(|v| v.as_str())
                            .or_else(|| ruleset.get("ruleset").and_then(|v| v.as_str())),
                    ) {
                        let interval = ruleset
                            .get("interval")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u32;
                        config
                            .surge_ruleset
                            .push(RulesetConfig::new(group, url, interval));
                    }
                }
            }

            // Parse proxy groups (placeholder for now)
            // TODO: Implement proxy group parsing
        }

        Ok(config)
    }

    /// Parse external config from TOML format
    fn parse_toml(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let toml_value: toml::Value = toml::from_str(content)?;
        let mut config = Self::new();

        if let Some(custom) = toml_value.get("custom").and_then(|v| v.as_table()) {
            // Parse rule bases
            if let Some(val) = custom.get("clash_rule_base").and_then(|v| v.as_str()) {
                config.clash_rule_base = val.to_string();
            }
            if let Some(val) = custom.get("surge_rule_base").and_then(|v| v.as_str()) {
                config.surge_rule_base = val.to_string();
            }
            if let Some(val) = custom.get("surfboard_rule_base").and_then(|v| v.as_str()) {
                config.surfboard_rule_base = val.to_string();
            }
            if let Some(val) = custom.get("mellow_rule_base").and_then(|v| v.as_str()) {
                config.mellow_rule_base = val.to_string();
            }
            if let Some(val) = custom.get("quan_rule_base").and_then(|v| v.as_str()) {
                config.quan_rule_base = val.to_string();
            }
            if let Some(val) = custom.get("quanx_rule_base").and_then(|v| v.as_str()) {
                config.quanx_rule_base = val.to_string();
            }
            if let Some(val) = custom.get("loon_rule_base").and_then(|v| v.as_str()) {
                config.loon_rule_base = val.to_string();
            }
            if let Some(val) = custom.get("sssub_rule_base").and_then(|v| v.as_str()) {
                config.sssub_rule_base = val.to_string();
            }
            if let Some(val) = custom.get("singbox_rule_base").and_then(|v| v.as_str()) {
                config.singbox_rule_base = val.to_string();
            }

            // Parse rule generation options
            if let Some(val) = custom
                .get("enable_rule_generator")
                .and_then(|v| v.as_bool())
            {
                config.enable_rule_generator = Some(val);
            }
            if let Some(val) = custom
                .get("overwrite_original_rules")
                .and_then(|v| v.as_bool())
            {
                config.overwrite_original_rules = Some(val);
            }

            // Parse emoji options
            if let Some(val) = custom.get("add_emoji").and_then(|v| v.as_bool()) {
                config.add_emoji = Some(val);
            }
            if let Some(val) = custom.get("remove_old_emoji").and_then(|v| v.as_bool()) {
                config.remove_old_emoji = Some(val);
            }

            // Parse include/exclude remarks (as array)
            if let Some(includes) = custom.get("include_remarks").and_then(|v| v.as_array()) {
                for item in includes {
                    if let Some(val) = item.as_str() {
                        config.include.push(val.to_string());
                    }
                }
            }
            if let Some(excludes) = custom.get("exclude_remarks").and_then(|v| v.as_array()) {
                for item in excludes {
                    if let Some(val) = item.as_str() {
                        config.exclude.push(val.to_string());
                    }
                }
            }
        }

        // Parse rulesets from the root level
        if let Some(rulesets) = toml_value.get("rulesets").and_then(|v| v.as_array()) {
            for ruleset in rulesets {
                if let Some(table) = ruleset.as_table() {
                    if let (Some(group), Some(url)) = (
                        table.get("group").and_then(|v| v.as_str()),
                        table.get("ruleset").and_then(|v| v.as_str()),
                    ) {
                        let interval = table
                            .get("interval")
                            .and_then(|v| v.as_integer())
                            .unwrap_or(0) as u32;
                        config
                            .surge_ruleset
                            .push(RulesetConfig::new(group, url, interval));
                    }
                }
            }
        }

        Ok(config)
    }

    /// Parse external config from INI format
    fn parse_ini(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = Self::new();
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

                match current_section {
                    "custom" => match key {
                        "clash_rule_base" => config.clash_rule_base = value.to_string(),
                        "surge_rule_base" => config.surge_rule_base = value.to_string(),
                        "surfboard_rule_base" => config.surfboard_rule_base = value.to_string(),
                        "mellow_rule_base" => config.mellow_rule_base = value.to_string(),
                        "quan_rule_base" => config.quan_rule_base = value.to_string(),
                        "quanx_rule_base" => config.quanx_rule_base = value.to_string(),
                        "loon_rule_base" => config.loon_rule_base = value.to_string(),
                        "sssub_rule_base" => config.sssub_rule_base = value.to_string(),
                        "singbox_rule_base" => config.singbox_rule_base = value.to_string(),
                        "enable_rule_generator" => {
                            config.enable_rule_generator =
                                Some(value.to_lowercase() == "true" || value == "1")
                        }
                        "overwrite_original_rules" => {
                            config.overwrite_original_rules =
                                Some(value.to_lowercase() == "true" || value == "1")
                        }
                        "add_emoji" => {
                            config.add_emoji = Some(value.to_lowercase() == "true" || value == "1")
                        }
                        "remove_old_emoji" => {
                            config.remove_old_emoji =
                                Some(value.to_lowercase() == "true" || value == "1")
                        }
                        "ruleset" | "surge_ruleset" => {
                            if let Some(ruleset) = RulesetConfig::from_str(value) {
                                config.surge_ruleset.push(ruleset);
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        Ok(config)
    }

    /// Apply external configuration settings on top of global settings
    pub fn apply_to_settings(&self, settings: &mut crate::settings::config::Settings) {
        // Apply rule bases
        if !self.clash_rule_base.is_empty() {
            settings.clash_base = self.clash_rule_base.clone();
        }
        if !self.surge_rule_base.is_empty() {
            settings.surge_base = self.surge_rule_base.clone();
        }
        if !self.surfboard_rule_base.is_empty() {
            settings.surfboard_base = self.surfboard_rule_base.clone();
        }
        if !self.mellow_rule_base.is_empty() {
            settings.mellow_base = self.mellow_rule_base.clone();
        }
        if !self.quan_rule_base.is_empty() {
            settings.quan_base = self.quan_rule_base.clone();
        }
        if !self.quanx_rule_base.is_empty() {
            settings.quanx_base = self.quanx_rule_base.clone();
        }
        if !self.loon_rule_base.is_empty() {
            settings.loon_base = self.loon_rule_base.clone();
        }
        if !self.sssub_rule_base.is_empty() {
            settings.ssub_base = self.sssub_rule_base.clone();
        }
        if !self.singbox_rule_base.is_empty() {
            settings.singbox_base = self.singbox_rule_base.clone();
        }

        // Apply rule generation options
        if let Some(val) = self.enable_rule_generator {
            settings.enable_rule_gen = val;
        }
        if let Some(val) = self.overwrite_original_rules {
            settings.overwrite_original_rules = val;
        }

        // Apply emoji options
        if let Some(val) = self.add_emoji {
            settings.add_emoji = val;
        }
        if let Some(val) = self.remove_old_emoji {
            settings.remove_emoji = val;
        }

        // Apply include/exclude remarks
        if !self.include.is_empty() {
            settings.include_remarks = self.include.clone();
        }
        if !self.exclude.is_empty() {
            settings.exclude_remarks = self.exclude.clone();
        }
    }
}

/// Load external configuration from file or URL
pub fn load_external_config(path: &str) -> Result<ExternalConfig, Box<dyn std::error::Error>> {
    ExternalConfig::load_from_file(path)
}
