use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use serde_yaml;
use toml;

use crate::models::ruleset::RulesetType;
use crate::utils::file::{file_exists, file_get};
use crate::utils::http::web_get;

/// Macro for checking if keys exist in a TOML value and setting targets if they do
///
/// This macro provides similar functionality to the C++ variadic template function
/// Processes a TOML value and a series of key-target pairs
#[macro_export]
macro_rules! find_if_exist {
    // Base case - parse a single value
    ($table:expr, $key:expr, $target:expr) => {
        if $table.contains($key) {
            let value = &$table[$key];
            if let Some(s) = value.as_str() {
                *$target = s.to_string();
            } else if let Some(b) = value.as_bool() {
                if let Some(target_bool) = $target.downcast_mut::<bool>() {
                    *target_bool = b;
                }
            } else if let Some(i) = value.as_integer() {
                if let Some(target_int) = $target.downcast_mut::<i32>() {
                    *target_int = i as i32;
                } else if let Some(target_int) = $target.downcast_mut::<i64>() {
                    *target_int = i;
                } else if let Some(target_int) = $target.downcast_mut::<usize>() {
                    *target_int = i as usize;
                }
            } else if let Some(f) = value.as_float() {
                if let Some(target_float) = $target.downcast_mut::<f64>() {
                    *target_float = f;
                }
            }
        }
    };
    // Multiple key-target pairs
    ($table:expr, $key:expr, $target:expr, $($rest_key:expr, $rest_target:expr),+) => {
        find_if_exist!($table, $key, $target);
        find_if_exist!($table, $($rest_key, $rest_target),+);
    };
}

/// Functions to find values in TOML tables and set targets if they exist
pub mod toml_helpers {
    use toml::value::Table;

    /// Find a string value in a TOML table and set the target if it exists
    pub fn find_string(table: &Table, key: &str, target: &mut String) {
        if table.contains_key(key) {
            if let Some(s) = table[key].as_str() {
                *target = s.to_string();
            }
        }
    }

    /// Find a boolean value in a TOML table and set the target if it exists
    pub fn find_bool(table: &Table, key: &str, target: &mut bool) {
        if table.contains_key(key) {
            if let Some(b) = table[key].as_bool() {
                *target = b;
            } else if let Some(s) = table[key].as_str() {
                // Handle string conversion to bool
                match s.to_lowercase().as_str() {
                    "true" | "yes" | "1" => *target = true,
                    "false" | "no" | "0" => *target = false,
                    _ => {}
                }
            }
        }
    }

    /// Find an optional boolean value in a TOML table and set the target if it exists
    pub fn find_opt_bool(table: &Table, key: &str, target: &mut Option<bool>) {
        if table.contains_key(key) {
            if let Some(b) = table[key].as_bool() {
                *target = Some(b);
            } else if let Some(s) = table[key].as_str() {
                // Handle string conversion to bool
                match s.to_lowercase().as_str() {
                    "true" | "yes" | "1" => *target = Some(true),
                    "false" | "no" | "0" => *target = Some(false),
                    _ => {}
                }
            }
        }
    }

    /// Find an integer value in a TOML table and set the target if it exists
    pub fn find_i32(table: &Table, key: &str, target: &mut i32) {
        if table.contains_key(key) {
            if let Some(i) = table[key].as_integer() {
                *target = i as i32;
            } else if let Some(s) = table[key].as_str() {
                // Try to parse string as integer
                if let Ok(i) = s.parse::<i32>() {
                    *target = i;
                }
            }
        }
    }

    /// Find an integer value in a TOML table and set the target if it exists
    pub fn find_i64(table: &Table, key: &str, target: &mut i64) {
        if table.contains_key(key) {
            if let Some(i) = table[key].as_integer() {
                *target = i;
            } else if let Some(s) = table[key].as_str() {
                // Try to parse string as integer
                if let Ok(i) = s.parse::<i64>() {
                    *target = i;
                }
            }
        }
    }

    /// Find an integer value in a TOML table and set the target if it exists
    pub fn find_usize(table: &Table, key: &str, target: &mut usize) {
        if table.contains_key(key) {
            if let Some(i) = table[key].as_integer() {
                if i >= 0 {
                    *target = i as usize;
                }
            } else if let Some(s) = table[key].as_str() {
                // Try to parse string as integer
                if let Ok(i) = s.parse::<usize>() {
                    *target = i;
                }
            }
        }
    }

    /// Find a string array value in a TOML table and set the target if it exists
    pub fn find_string_array(table: &Table, key: &str, target: &mut Vec<String>) {
        if table.contains_key(key) {
            if let Some(arr) = table[key].as_array() {
                *target = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            } else if let Some(s) = table[key].as_str() {
                // Handle comma-separated string
                *target = s.split(',').map(|part| part.trim().to_string()).collect();
            }
        }
    }
}

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
    pub api_access_token: String,
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
    /// Create a new settings instance with default values
    pub fn new() -> Self {
        Settings::default()
    }

    /// Load settings from file or URL
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut content = String::new();

        // Try to load the content from file or URL
        if path.starts_with("http://") || path.starts_with("https://") {
            let (data, _) = web_get(path, None, None)?;
            content = data;
        } else {
            content = std::fs::read_to_string(path)?;
        }

        // Try to parse as YAML first
        if content.contains("common:") {
            return Ok(serde_yaml::from_str(&content)?);
        }

        // Try to parse as TOML
        if let Ok(_) = toml::from_str::<toml::Value>(&content) {
            return Ok(toml::from_str(&content)?);
        }

        // Default to INI
        let mut settings = Settings::default();
        settings.load_from_ini(&content)?;
        Ok(settings)
    }

    /// Load settings from INI format
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

                match current_section {
                    "common" => match key {
                        "api_mode" => {
                            self.api_mode = value.to_lowercase() == "true" || value == "1"
                        }
                        "api_access_token" => self.api_access_token = value.to_string(),
                        "default_url" => self.default_urls = value.to_string(),
                        "insert_url" => self.insert_urls = value.to_string(),
                        "exclude_remarks" => {
                            self.exclude_remarks =
                                value.split(',').map(|s| s.trim().to_string()).collect();
                        }
                        "include_remarks" => {
                            self.include_remarks =
                                value.split(',').map(|s| s.trim().to_string()).collect();
                        }
                        "base_path" => self.base_path = value.to_string(),
                        "clash_rule_base" => self.clash_base = value.to_string(),
                        "surge_rule_base" => self.surge_base = value.to_string(),
                        "proxy_config" => self.proxy_config = value.to_string(),
                        "proxy_ruleset" => self.proxy_ruleset = value.to_string(),
                        "proxy_subscription" => self.proxy_subscription = value.to_string(),
                        _ => {}
                    },
                    "node_pref" => match key {
                        "sort_flag" => {
                            self.enable_sort = value.to_lowercase() == "true" || value == "1"
                        }
                        "filter_deprecated" => {
                            self.filter_deprecated = value.to_lowercase() == "true" || value == "1"
                        }
                        "append_sub_userinfo" => {
                            self.append_userinfo = value.to_lowercase() == "true" || value == "1"
                        }
                        "clash_use_new_field" => {
                            self.clash_use_new_field =
                                value.to_lowercase() == "true" || value == "1"
                        }
                        "clash_proxies_style" => self.clash_proxies_style = value.to_string(),
                        "clash_proxy_groups_style" => {
                            self.clash_proxy_groups_style = value.to_string()
                        }
                        "udp_flag" => {
                            self.udp_flag = Some(value.to_lowercase() == "true" || value == "1")
                        }
                        "tcp_fast_open_flag" => {
                            self.tfo_flag = Some(value.to_lowercase() == "true" || value == "1")
                        }
                        "skip_cert_verify_flag" => {
                            self.skip_cert_verify =
                                Some(value.to_lowercase() == "true" || value == "1")
                        }
                        "tls13_flag" => {
                            self.tls13_flag = Some(value.to_lowercase() == "true" || value == "1")
                        }
                        _ => {}
                    },
                    "managed_config" => match key {
                        "write_managed_config" => {
                            self.write_managed_config =
                                value.to_lowercase() == "true" || value == "1"
                        }
                        "managed_config_prefix" => self.managed_config_prefix = value.to_string(),
                        "config_update_interval" => {
                            self.update_interval = value.parse().unwrap_or(0)
                        }
                        "config_update_strict" => {
                            self.update_strict = value.to_lowercase() == "true" || value == "1"
                        }
                        "quanx_device_id" => self.quanx_dev_id = value.to_string(),
                        _ => {}
                    },
                    "advanced" => match key {
                        "log_level" => {
                            self.log_level = match value.to_lowercase().as_str() {
                                "debug" => 0,
                                "info" => 1,
                                "warn" => 2,
                                "error" => 3,
                                "fatal" => 4,
                                _ => value.parse().unwrap_or(1),
                            };
                        }
                        "max_pending_connections" => {
                            self.max_pending_conns = value.parse().unwrap_or(10240)
                        }
                        "max_concurrent_threads" => {
                            self.max_concur_threads = value.parse().unwrap_or(2)
                        }
                        "max_allowed_rulesets" => {
                            self.max_allowed_rulesets = value.parse().unwrap_or(64)
                        }
                        "max_allowed_rules" => {
                            self.max_allowed_rules = value.parse().unwrap_or(32768)
                        }
                        "max_allowed_download_size" => {
                            self.max_allowed_download_size = value.parse().unwrap_or(0)
                        }
                        "enable_cache" => {
                            self.serve_cache_on_fetch_fail =
                                value.to_lowercase() == "true" || value == "1"
                        }
                        "cache_subscription" => {
                            self.cache_subscription = value.parse().unwrap_or(60)
                        }
                        "cache_config" => self.cache_config = value.parse().unwrap_or(300),
                        "cache_ruleset" => self.cache_ruleset = value.parse().unwrap_or(21600),
                        "script_clean_context" => {
                            self.script_clean_context =
                                value.to_lowercase() == "true" || value == "1"
                        }
                        "async_fetch_ruleset" => {
                            self.async_fetch_ruleset =
                                value.to_lowercase() == "true" || value == "1"
                        }
                        "skip_failed_links" => {
                            self.skip_failed_links = value.to_lowercase() == "true" || value == "1"
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        Ok(())
    }
}

// Global settings instance
lazy_static::lazy_static! {
    pub static ref global: Arc<RwLock<Settings>> = Arc::new(RwLock::new(Settings::new()));
}

/// Get the current settings
pub fn get_settings() -> Settings {
    global.read().unwrap().clone()
}

/// Update the global settings
pub fn update_settings(settings: Settings) {
    *global.write().unwrap() = settings;
}

/// Parse TOML content
pub fn parse_toml(content: &str, _fname: &str) -> Result<toml::Value, toml::de::Error> {
    toml::from_str(content)
}

/// Refresh the configuration
pub fn refresh_configuration() {
    let settings = global.read().unwrap();
    if let Ok(new_settings) = Settings::load_from_file(&settings.pref_path) {
        update_settings(new_settings);
    }
}
