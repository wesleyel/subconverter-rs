use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use serde_yml;
use toml;

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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    // Common settings
    pub pref_path: String,
    pub default_ext_config: String,
    #[serde(default)]
    pub exclude_remarks: Vec<String>,
    #[serde(default)]
    pub include_remarks: Vec<String>,
    #[serde(default = "default_listen_address")]
    pub listen_address: String,
    #[serde(default = "default_listen_port")]
    pub listen_port: i32,
    pub default_urls: String,
    pub insert_urls: String,
    pub managed_config_prefix: String,
    #[serde(default = "default_max_pending_conns")]
    pub max_pending_conns: i32,
    #[serde(default = "default_max_concur_threads")]
    pub max_concur_threads: i32,
    #[serde(default)]
    pub prepend_insert: bool,
    #[serde(default)]
    pub skip_failed_links: bool,
    #[serde(default)]
    pub api_mode: bool,
    #[serde(default)]
    pub write_managed_config: bool,
    #[serde(default = "default_true")]
    pub enable_rule_gen: bool,
    #[serde(default)]
    pub update_ruleset_on_request: bool,
    #[serde(default)]
    pub overwrite_original_rules: bool,
    #[serde(default)]
    pub print_dbg_info: bool,
    #[serde(default = "default_true")]
    pub append_userinfo: bool,
    #[serde(default)]
    pub async_fetch_ruleset: bool,
    #[serde(default)]
    pub surge_resolve_hostname: bool,
    pub api_access_token: String,
    pub base_path: String,
    pub custom_group: String,
    #[serde(default = "default_log_level")]
    pub log_level: i32,
    #[serde(default = "default_max_download_size")]
    pub max_allowed_download_size: i64,
    pub template_path: String,
    #[serde(default)]
    pub template_vars: HashMap<String, String>,

    // Generator settings
    #[serde(default)]
    pub generator_mode: bool,
    pub generate_profiles: String,

    // Preferences
    #[serde(default)]
    pub reload_conf_on_request: bool,
    #[serde(default)]
    pub add_emoji: bool,
    #[serde(default)]
    pub remove_emoji: bool,
    #[serde(default)]
    pub append_type: bool,
    #[serde(default = "default_true")]
    pub filter_deprecated: bool,
    pub udp_flag: Option<bool>,
    pub tfo_flag: Option<bool>,
    pub skip_cert_verify: Option<bool>,
    pub tls13_flag: Option<bool>,
    pub enable_insert: Option<bool>,
    #[serde(default)]
    pub enable_sort: bool,
    #[serde(default)]
    pub update_strict: bool,
    #[serde(default = "default_true")]
    pub clash_use_new_field: bool,
    #[serde(default)]
    pub singbox_add_clash_modes: bool,
    pub clash_proxies_style: String,
    pub clash_proxy_groups_style: String,
    pub proxy_config: String,
    pub proxy_ruleset: String,
    pub proxy_subscription: String,
    #[serde(default)]
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
    #[serde(default)]
    pub serve_cache_on_fetch_fail: bool,
    #[serde(default = "default_cache_subscription")]
    pub cache_subscription: i32,
    #[serde(default = "default_cache_config")]
    pub cache_config: i32,
    #[serde(default = "default_cache_ruleset")]
    pub cache_ruleset: i32,

    // Limits
    #[serde(default = "default_max_rulesets")]
    pub max_allowed_rulesets: usize,
    #[serde(default = "default_max_rules")]
    pub max_allowed_rules: usize,
    #[serde(default)]
    pub script_clean_context: bool,

    // Cron system
    #[serde(default)]
    pub enable_cron: bool,

    // Custom rulesets
    #[serde(default)]
    pub custom_rulesets: Vec<String>,
}

// Default value functions for serde
fn default_listen_address() -> String {
    "127.0.0.1".to_string()
}

fn default_listen_port() -> i32 {
    25500
}

fn default_max_pending_conns() -> i32 {
    10240
}

fn default_max_concur_threads() -> i32 {
    4
}

fn default_true() -> bool {
    true
}

fn default_log_level() -> i32 {
    1
}

fn default_max_download_size() -> i64 {
    32 * 1024 * 1024 // 32MB
}

fn default_cache_subscription() -> i32 {
    60
}

fn default_cache_config() -> i32 {
    300
}

fn default_cache_ruleset() -> i32 {
    21600
}

fn default_max_rulesets() -> usize {
    64
}

fn default_max_rules() -> usize {
    32768
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            pref_path: String::new(),
            default_ext_config: String::new(),
            exclude_remarks: Vec::new(),
            include_remarks: Vec::new(),
            listen_address: default_listen_address(),
            listen_port: default_listen_port(),
            default_urls: String::new(),
            insert_urls: String::new(),
            managed_config_prefix: String::new(),
            max_pending_conns: default_max_pending_conns(),
            max_concur_threads: default_max_concur_threads(),
            prepend_insert: false,
            skip_failed_links: false,
            api_mode: false,
            write_managed_config: false,
            enable_rule_gen: default_true(),
            update_ruleset_on_request: false,
            overwrite_original_rules: false,
            print_dbg_info: false,
            append_userinfo: default_true(),
            async_fetch_ruleset: false,
            surge_resolve_hostname: false,
            api_access_token: String::new(),
            base_path: String::new(),
            custom_group: String::new(),
            log_level: default_log_level(),
            max_allowed_download_size: default_max_download_size(),
            template_path: String::new(),
            template_vars: HashMap::new(),

            // Generator settings
            generator_mode: false,
            generate_profiles: String::new(),

            // Preferences
            reload_conf_on_request: false,
            add_emoji: false,
            remove_emoji: false,
            append_type: false,
            filter_deprecated: default_true(),
            udp_flag: None,
            tfo_flag: None,
            skip_cert_verify: None,
            tls13_flag: None,
            enable_insert: None,
            enable_sort: false,
            update_strict: false,
            clash_use_new_field: default_true(),
            singbox_add_clash_modes: false,
            clash_proxies_style: String::new(),
            clash_proxy_groups_style: String::new(),
            proxy_config: String::new(),
            proxy_ruleset: String::new(),
            proxy_subscription: String::new(),
            update_interval: 0,
            sort_script: String::new(),
            filter_script: String::new(),

            // Base configs
            clash_base: String::new(),
            surge_base: String::new(),
            surfboard_base: String::new(),
            mellow_base: String::new(),
            quan_base: String::new(),
            quanx_base: String::new(),
            loon_base: String::new(),
            ssub_base: String::new(),
            singbox_base: String::new(),
            surge_ssr_path: String::new(),
            quanx_dev_id: String::new(),

            // Cache system
            serve_cache_on_fetch_fail: false,
            cache_subscription: default_cache_subscription(),
            cache_config: default_cache_config(),
            cache_ruleset: default_cache_ruleset(),

            // Limits
            max_allowed_rulesets: default_max_rulesets(),
            max_allowed_rules: default_max_rules(),
            script_clean_context: false,

            // Cron system
            enable_cron: false,

            // Custom rulesets
            custom_rulesets: Vec::new(),
        }
    }
}

impl Settings {
    /// Create a new settings instance with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Load settings from file or URL
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut _content = String::new();

        // Try to load the content from file or URL
        if path.starts_with("http://") || path.starts_with("https://") {
            let (data, _) = web_get(path, None, None)?;
            _content = data;
        } else {
            _content = std::fs::read_to_string(path)?;
        }

        // Try to parse as YAML first
        if _content.contains("common:") {
            let mut settings: Settings = serde_yml::from_str(&_content)?;
            settings.pref_path = path.to_string();

            // Ensure listen_address is not empty
            if settings.listen_address.trim().is_empty() {
                settings.listen_address = default_listen_address();
            }

            return Ok(settings);
        }

        // Try to parse as TOML
        if let Ok(_) = toml::from_str::<toml::Value>(&_content) {
            let mut settings: Settings = toml::from_str(&_content)?;
            settings.pref_path = path.to_string();

            // Ensure listen_address is not empty
            if settings.listen_address.trim().is_empty() {
                settings.listen_address = default_listen_address();
            }

            return Ok(settings);
        }

        // Default to INI
        let mut settings = Settings::default();
        settings.load_from_ini(&_content)?;
        settings.pref_path = path.to_string();

        // Ensure listen_address is not empty
        if settings.listen_address.trim().is_empty() {
            settings.listen_address = default_listen_address();
        }

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
                        "listen" => self.listen_address = value.to_string(),
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
                        // Also handle ruleset entries in common section
                        "ruleset" | "surge_ruleset" => {
                            self.custom_rulesets.push(value.to_string());
                        }
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
                            if let Ok(val) = value.parse() {
                                self.update_interval = val;
                            }
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
                            if let Ok(val) = value.parse() {
                                self.max_pending_conns = val;
                            }
                        }
                        "max_concurrent_threads" => {
                            if let Ok(val) = value.parse() {
                                self.max_concur_threads = val;
                            }
                        }
                        "max_allowed_rulesets" => {
                            if let Ok(val) = value.parse() {
                                self.max_allowed_rulesets = val;
                            }
                        }
                        "max_allowed_rules" => {
                            if let Ok(val) = value.parse() {
                                self.max_allowed_rules = val;
                            }
                        }
                        "max_allowed_download_size" => {
                            if let Ok(val) = value.parse() {
                                self.max_allowed_download_size = val;
                            }
                        }
                        "enable_cache" => {
                            self.serve_cache_on_fetch_fail =
                                value.to_lowercase() == "true" || value == "1"
                        }
                        "cache_subscription" => {
                            if let Ok(val) = value.parse() {
                                self.cache_subscription = val;
                            }
                        }
                        "cache_config" => {
                            if let Ok(val) = value.parse() {
                                self.cache_config = val;
                            }
                        }
                        "cache_ruleset" => {
                            if let Ok(val) = value.parse() {
                                self.cache_ruleset = val;
                            }
                        }
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
                    "rulesets" | "ruleset" => match key {
                        "enabled" => {
                            self.enable_rule_gen = value.to_lowercase() == "true" || value == "1"
                        }
                        "overwrite_original_rules" => {
                            self.overwrite_original_rules =
                                value.to_lowercase() == "true" || value == "1"
                        }
                        "update_ruleset_on_request" => {
                            self.update_ruleset_on_request =
                                value.to_lowercase() == "true" || value == "1"
                        }
                        "ruleset" | "surge_ruleset" => {
                            self.custom_rulesets.push(value.to_string());
                        }
                        _ => {}
                    },
                    "custom" => match key {
                        "ruleset" | "surge_ruleset" => {
                            self.custom_rulesets.push(value.to_string());
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
pub fn refresh_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let settings = global.read().unwrap();
    let path = settings.pref_path.clone();
    drop(settings); // Release the lock before potential long operation

    match Settings::load_from_file(&path) {
        Ok(new_settings) => {
            update_settings(new_settings);
            Ok(())
        }
        Err(err) => {
            eprintln!("Failed to refresh configuration from '{}': {}", path, err);
            Err(err)
        }
    }
}
