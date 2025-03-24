use log::debug;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::path::Path;
use toml;

use crate::models::RegexMatchConfig;
use crate::settings::deserializer::*;
use crate::settings::ruleset::RulesetConfig;
use crate::settings::{import_items, Settings};
use crate::utils::file::read_file;
use crate::utils::http::{parse_proxy, web_get};
// TODO: Implement template rendering module similar to C++ render_template function

use super::ini_external::IniExternalSettings;
use super::toml_external::TomlExternalSettings;
use super::yaml_external::YamlExternalSettings;

/// External configuration structure with flattened fields
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ExternalSettings {
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

    pub add_emoji: Option<bool>,
    pub remove_old_emoji: Option<bool>,

    // Filtering
    pub include_remarks: Vec<String>,
    pub exclude_remarks: Vec<String>,

    #[serde(default, deserialize_with = "deserialize_rulesets")]
    pub rulesets: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_proxy_groups")]
    pub custom_proxy_groups: Vec<String>,

    // Node operations
    pub rename: Vec<RegexMatchConfig>,

    // Template arguments
    pub tpl_args: Option<HashMap<String, String>>,
}

impl ExternalSettings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process imports in the configuration
    pub fn process_imports(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let settings = Settings::current();
        let proxy_config = parse_proxy(&settings.proxy_config);
        // Process imports for rulesets
        import_items(
            &mut self.rulesets,
            settings.api_mode,
            &proxy_config,
            &settings.base_path,
        )?;

        // Process imports for custom proxy groups
        import_items(
            &mut self.custom_proxy_groups,
            settings.api_mode,
            &proxy_config,
            &settings.base_path,
        )?;

        Ok(())
    }

    /// Load external configuration from file or URL
    pub fn load_from_file_sync(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut _content = String::new();

        // Try to load content from file or URL
        if path.starts_with("http://") || path.starts_with("https://") {
            match web_get(path, &parse_proxy(&Settings::current().proxy_config), None) {
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

        // TODO: Implement template rendering here
        // In C++: if(render_template(config, *ext.tpl_args, base_content, global.templatePath) != 0)
        //           base_content = config;

        // Try YAML format first
        if _content.contains("custom:") {
            match serde_yaml::from_str::<YamlExternalSettings>(&_content) {
                Ok(yaml_settings) => {
                    // Convert to ExternalSettings
                    let mut config = Self::from(yaml_settings);
                    // Process any imports
                    config.process_imports()?;
                    return Ok(config);
                }
                Err(e) => debug!("Failed to parse external config as YAML: {}", e),
            }
        }

        // Try TOML format
        match toml::from_str::<TomlExternalSettings>(&_content) {
            Ok(toml_settings) => {
                // Convert to ExternalSettings
                let mut config = Self::from(toml_settings);
                // Process any imports
                config.process_imports()?;
                return Ok(config);
            }
            Err(e) => debug!("Failed to parse external config as TOML: {}", e),
        }

        // Fall back to INI format
        let mut ini_settings = IniExternalSettings::new();
        match ini_settings.load_from_ini(&_content) {
            Ok(_) => {
                // Convert to ExternalSettings
                let mut config = Self::from(ini_settings);
                // Process any imports
                config.process_imports()?;
                Ok(config)
            }
            Err(e) => Err(format!("Failed to parse external config as INI: {}", e).into()),
        }
    }

    // TODO: Implement validate_rulesets method - in C++ there's a check for maxAllowedRulesets
    // In C++: if(global.maxAllowedRulesets && vArray.size() > global.maxAllowedRulesets) { ... }
}
