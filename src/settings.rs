use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use serde_yaml;
use toml;

/// Settings structure to hold global configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    // Common settings
    pub pref_path: String,
    pub default_ext_config: String,
    pub exclude_remarks: Vec<String>,
    pub include_remarks: Vec<String>,
    pub listen_address: String,
    pub listen_port: i32,
    pub default_urls: String,
    pub insert_urls: String,
    pub managed_config_prefix: String,
    pub max_pending_conns: i32,
    pub max_concur_threads: i32,
    pub prepend_insert: bool,
    pub skip_failed_links: bool,
    pub api_mode: bool,
    pub write_managed_config: bool,
    pub enable_rule_gen: bool,
    pub update_ruleset_on_request: bool,
    pub overwrite_original_rules: bool,
    pub print_dbg_info: bool,
    pub append_userinfo: bool,
    pub async_fetch_ruleset: bool,
    pub surge_resolve_hostname: bool,
    pub access_token: String,
    pub base_path: String,
    pub custom_group: String,
    pub log_level: i32,
    pub max_allowed_download_size: i64,
    pub template_path: String,
    pub template_vars: HashMap<String, String>,

    // Generator settings
    pub generator_mode: bool,
    pub generate_profiles: String,

    // Preferences
    pub reload_conf_on_request: bool,
    pub add_emoji: bool,
    pub remove_emoji: bool,
    pub append_type: bool,
    pub filter_deprecated: bool,
    pub udp_flag: Option<bool>,
    pub tfo_flag: Option<bool>,
    pub skip_cert_verify: Option<bool>,
    pub tls13_flag: Option<bool>,
    pub enable_insert: Option<bool>,
    pub enable_sort: bool,
    pub update_strict: bool,
    pub clash_use_new_field: bool,
    pub singbox_add_clash_modes: bool,
    pub clash_proxies_style: String,
    pub clash_proxy_groups_style: String,
    pub proxy_config: String,
    pub proxy_ruleset: String,
    pub proxy_subscription: String,
    pub update_interval: i32,
    pub sort_script: String,
    pub filter_script: String,

    // Base configs
    pub clash_base: String,
    pub surge_base: String,
    pub surfboard_base: String,
    pub mellow_base: String,
    pub quan_base: String,
    pub quanx_base: String,
    pub loon_base: String,
    pub ssub_base: String,
    pub singbox_base: String,
    pub surge_ssr_path: String,
    pub quanx_dev_id: String,

    // Cache system
    pub serve_cache_on_fetch_fail: bool,
    pub cache_subscription: i32,
    pub cache_config: i32,
    pub cache_ruleset: i32,

    // Limits
    pub max_allowed_rulesets: usize,
    pub max_allowed_rules: usize,
    pub script_clean_context: bool,

    // Cron system
    pub enable_cron: bool,
}

impl Settings {
    /// Create new settings with default values
    pub fn new() -> Self {
        Self {
            pref_path: "pref.ini".to_string(),
            listen_address: "127.0.0.1".to_string(),
            listen_port: 25500,
            max_pending_conns: 10,
            max_concur_threads: 4,
            prepend_insert: true,
            skip_failed_links: false,
            api_mode: true,
            write_managed_config: false,
            enable_rule_gen: true,
            update_ruleset_on_request: false,
            overwrite_original_rules: true,
            print_dbg_info: false,
            append_userinfo: true,
            async_fetch_ruleset: false,
            surge_resolve_hostname: true,
            base_path: "base".to_string(),
            log_level: 0, // LOG_LEVEL_VERBOSE
            max_allowed_download_size: 1048576,
            template_path: "templates".to_string(),

            // Preferences
            filter_deprecated: true,
            enable_sort: false,
            update_strict: false,
            clash_use_new_field: false,
            singbox_add_clash_modes: true,
            clash_proxies_style: "flow".to_string(),
            clash_proxy_groups_style: "block".to_string(),
            update_interval: 0,

            // Cache settings
            serve_cache_on_fetch_fail: false,
            cache_subscription: 60,
            cache_config: 300,
            cache_ruleset: 21600,

            // Limits
            max_allowed_rulesets: 64,
            max_allowed_rules: 32768,
            script_clean_context: false,

            // Cron settings
            enable_cron: false,

            // Fill in defaults for the rest
            ..Default::default()
        }
    }

    /// Load settings from file
    ///
    /// Supports YAML, TOML and INI formats
    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Read the file content
        let content = fs::read_to_string(path)?;

        // Try to parse as YAML first
        if content.contains("common:") {
            return self.load_from_yaml(&content, path);
        }

        // Try to parse as TOML
        if let Ok(_) = toml::from_str::<toml::Value>(&content) {
            return self.load_from_toml(&content, path);
        }

        // Default to INI
        self.load_from_ini(&content, path)
    }

    /// Load settings from YAML content
    fn load_from_yaml(
        &mut self,
        content: &str,
        _path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let yaml_settings: serde_yaml::Value = serde_yaml::from_str(content)?;

        // Parse common section
        if let Some(common) = yaml_settings.get("common") {
            if let Some(api_mode) = common.get("api_mode") {
                self.api_mode = api_mode.as_bool().unwrap_or(self.api_mode);
            }

            if let Some(access_token) = common.get("api_access_token") {
                self.access_token = access_token.as_str().unwrap_or("").to_string();
            }

            // Parse other common settings...
            // For brevity, I'm only showing a few examples

            if let Some(base_path) = common.get("base_path") {
                self.base_path = base_path.as_str().unwrap_or(&self.base_path).to_string();
            }

            if let Some(clash_rule_base) = common.get("clash_rule_base") {
                self.clash_base = clash_rule_base
                    .as_str()
                    .unwrap_or(&self.clash_base)
                    .to_string();
            }

            // Parse default_url array
            if let Some(default_url) = common.get("default_url") {
                if let Some(urls) = default_url.as_sequence() {
                    let url_strings: Vec<String> = urls
                        .iter()
                        .filter_map(|u| u.as_str().map(|s| s.to_string()))
                        .collect();
                    self.default_urls = url_strings.join("|");
                }
            }
        }

        // TODO: Parse more sections like node_pref, userinfo, etc.

        println!("Loaded settings from YAML");
        Ok(())
    }

    /// Load settings from TOML content
    fn load_from_toml(
        &mut self,
        content: &str,
        _path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let toml_settings: toml::Value = toml::from_str(content)?;

        // Parse common section
        if let Some(common) = toml_settings.get("common") {
            if let Some(api_mode) = common.get("api_mode") {
                self.api_mode = api_mode.as_bool().unwrap_or(self.api_mode);
            }

            if let Some(access_token) = common.get("api_access_token") {
                if let Some(token) = access_token.as_str() {
                    self.access_token = token.to_string();
                }
            }

            // Parse default_url array
            if let Some(default_url) = common.get("default_url") {
                if let Some(urls) = default_url.as_array() {
                    let url_strings: Vec<String> = urls
                        .iter()
                        .filter_map(|u| u.as_str().map(|s| s.to_string()))
                        .collect();
                    self.default_urls = url_strings.join("|");
                }
            }

            // Parse other common settings...
        }

        // TODO: Parse more sections

        println!("Loaded settings from TOML");
        Ok(())
    }

    /// Load settings from INI content
    fn load_from_ini(
        &mut self,
        content: &str,
        _path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Note: In a real implementation, you would use a proper INI parser
        // This is just a simplified example

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
                    "common" => {
                        match key {
                            "api_mode" => {
                                self.api_mode = value.to_lowercase() == "true" || value == "1"
                            }
                            "api_access_token" => self.access_token = value.to_string(),
                            "base_path" => self.base_path = value.to_string(),
                            "clash_rule_base" => self.clash_base = value.to_string(),
                            "default_url" => self.default_urls = value.to_string(),
                            // Add more keys as needed
                            _ => {} // Ignore unknown keys
                        }
                    }
                    "server" => {
                        match key {
                            "listen" => self.listen_address = value.to_string(),
                            "port" => self.listen_port = value.parse().unwrap_or(self.listen_port),
                            _ => {} // Ignore unknown keys
                        }
                    }
                    // Add more sections as needed
                    _ => {} // Ignore unknown sections
                }
            }
        }

        println!("Loaded settings from INI");
        Ok(())
    }
}

/// External configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExternalConfig {
    pub clash_rule_base: String,
    pub surge_rule_base: String,
    pub surfboard_rule_base: String,
    pub mellow_rule_base: String,
    pub quan_rule_base: String,
    pub quanx_rule_base: String,
    pub loon_rule_base: String,
    pub sssub_rule_base: String,
    pub singbox_rule_base: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub overwrite_original_rules: bool,
    pub enable_rule_generator: bool,
    pub add_emoji: Option<bool>,
    pub remove_old_emoji: Option<bool>,
}

impl ExternalConfig {
    /// Load external config from file
    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Read the file content
        let content = fs::read_to_string(path)?;

        // Try to parse as YAML first
        if content.contains("custom:") {
            return self.load_from_yaml(&content);
        }

        // Try to parse as TOML
        if let Ok(_) = toml::from_str::<toml::Value>(&content) {
            return self.load_from_toml(&content);
        }

        // Default to INI
        self.load_from_ini(&content)
    }

    // Implementations for specific formats
    fn load_from_yaml(&mut self, _content: &str) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement YAML parsing for external config
        Ok(())
    }

    fn load_from_toml(&mut self, _content: &str) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement TOML parsing for external config
        Ok(())
    }

    fn load_from_ini(&mut self, _content: &str) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement INI parsing for external config
        Ok(())
    }
}

// Global settings instance protected by a read-write lock
lazy_static! {
    pub static ref SETTINGS: RwLock<Settings> = RwLock::new(Settings::new());
}

/// Get a reference to the global settings
pub fn get_settings() -> Settings {
    SETTINGS.read().unwrap().clone()
}

/// Update the global settings
pub fn update_settings(settings: Settings) {
    let mut global = SETTINGS.write().unwrap();
    *global = settings;
}

/// Load external configuration
pub fn load_external_config(
    path: &str,
    ext: &mut ExternalConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    ext.load_from_file(path)
}

/// Ruleset type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulesetType {
    SurgeRuleset,
    ClashDomain,
    ClashIpcidr,
    ClashClassical,
    Quanx,
}

// Define a map of ruleset prefixes to types
lazy_static! {
    pub static ref RULESET_TYPES: HashMap<&'static str, RulesetType> = {
        let mut map = HashMap::new();
        map.insert("clash-domain:", RulesetType::ClashDomain);
        map.insert("clash-ipcidr:", RulesetType::ClashIpcidr);
        map.insert("clash-classic:", RulesetType::ClashClassical);
        map.insert("quanx:", RulesetType::Quanx);
        map.insert("surge:", RulesetType::SurgeRuleset);
        map
    };
}
