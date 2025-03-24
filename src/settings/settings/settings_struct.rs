use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use serde_yml;
use toml;

use crate::models::cron::CronTaskConfigs;
use crate::models::proxy_group_config::ProxyGroupConfig;
use crate::models::ruleset::RulesetContent;
use crate::models::RegexMatchConfig;
use crate::models::RegexMatchConfigs;
use crate::models::RulesetConfig;
use crate::settings::import_items;
use crate::utils::file_get;
use crate::utils::http::web_get;

/// Settings structure to hold global configuration
#[derive(Debug, Clone)]
pub struct Settings {
    // Common settings
    pub pref_path: String,
    pub default_ext_config: String,
    pub exclude_remarks: Vec<String>,
    pub include_remarks: Vec<String>,
    // Custom ruleset and proxy groups
    pub custom_rulesets: Vec<RulesetConfig>,
    pub custom_proxy_groups: Vec<ProxyGroupConfig>,
    // Runtime ruleset content cache (non-serialized field)
    pub rulesets_content: Vec<RulesetContent>,

    // Stream/time rules, for ParseSettings initialize
    pub stream_rules: Vec<RegexMatchConfig>,
    pub time_rules: Vec<RegexMatchConfig>,

    // Rename and emoji rules
    pub renames: RegexMatchConfigs,
    pub emojis: RegexMatchConfigs,

    pub default_urls: Vec<String>,
    pub insert_urls: Vec<String>,
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

    pub aliases: HashMap<String, String>,

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
    pub enable_insert: bool,
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

    // Server
    pub listen_address: String,
    pub listen_port: i32,
    pub serve_file: bool,
    pub serve_file_root: String,

    // Limits
    pub max_allowed_rulesets: usize,
    pub max_allowed_rules: usize,
    pub script_clean_context: bool,

    // Cron system
    pub enable_cron: bool,
    pub cron_tasks: CronTaskConfigs,
}

// Default value functions for serde
pub fn default_listen_address() -> String {
    "127.0.0.1".to_string()
}

pub fn default_listen_port() -> i32 {
    25500
}

pub fn default_max_pending_conns() -> i32 {
    10240
}

pub fn default_max_concur_threads() -> i32 {
    4
}

pub fn default_true() -> bool {
    true
}

pub fn default_log_level() -> i32 {
    1
}

pub fn default_max_download_size() -> i64 {
    32 * 1024 * 1024 // 32MB
}

pub fn default_cache_subscription() -> i32 {
    60
}

pub fn default_cache_config() -> i32 {
    300
}

pub fn default_cache_ruleset() -> i32 {
    21600
}

pub fn default_max_rulesets() -> usize {
    64
}

pub fn default_max_rules() -> usize {
    32768
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            pref_path: String::new(),
            default_ext_config: String::new(),
            exclude_remarks: Vec::new(),
            include_remarks: Vec::new(),
            custom_rulesets: Vec::new(),
            custom_proxy_groups: Vec::new(),
            rulesets_content: Vec::new(),
            stream_rules: Vec::new(),
            time_rules: Vec::new(),
            renames: RegexMatchConfigs::new(),
            emojis: RegexMatchConfigs::new(),
            aliases: HashMap::new(),
            default_urls: Vec::new(),
            insert_urls: Vec::new(),
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
            enable_insert: false,
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

            // Server
            listen_address: default_listen_address(),
            listen_port: default_listen_port(),
            serve_file: false,
            serve_file_root: String::new(),

            // Limits
            max_allowed_rulesets: default_max_rulesets(),
            max_allowed_rules: default_max_rules(),
            script_clean_context: false,

            // Cron system
            enable_cron: false,
            cron_tasks: CronTaskConfigs::new(),
        }
    }
}

impl Settings {
    /// Create a new settings instance with default values
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current() -> Arc<Settings> {
        global.read().unwrap().clone()
    }

    fn load_from_content(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Try to parse as YAML first
        if content.contains("common:") {
            let mut yaml_settings: crate::settings::settings::yaml_settings::YamlSettings =
                serde_yml::from_str(&content)?;
            yaml_settings.process_imports_and_inis()?;

            let mut settings = Settings::from(yaml_settings);

            return Ok(settings);
        }

        // Try to parse as TOML
        if toml::from_str::<toml::Value>(&content).is_ok() {
            let mut toml_settings: crate::settings::settings::toml_settings::TomlSettings =
                toml::from_str(&content)?;

            toml_settings.process_imports()?;

            let mut settings = Settings::from(toml_settings);

            // Ensure listen_address is not empty
            if settings.listen_address.trim().is_empty() {
                settings.listen_address = default_listen_address();
            }

            return Ok(settings);
        }

        // Default to INI
        let mut ini_settings = crate::settings::settings::ini_settings::IniSettings::new();
        ini_settings.load_from_ini(&content)?;
        ini_settings.process_imports()?;

        let mut settings = Settings::from(ini_settings);

        // Ensure listen_address is not empty
        if settings.listen_address.trim().is_empty() {
            settings.listen_address = default_listen_address();
        }

        Ok(settings)
    }

    /// Load settings from file or URL
    fn load_from_file_sync(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut content = String::new();

        // Try to load the content from file or URL
        if path.starts_with("http://") || path.starts_with("https://") {
            let (data, _) = web_get(path, None, None)?;
            content = data;
        } else {
            content = file_get(path, None)?;
        }
        let mut settings = Settings::load_from_content(&content)?;
        settings.pref_path = path.to_owned();
        Ok(settings)
    }
}

// Global settings instance
pub static global: LazyLock<RwLock<Arc<Settings>>> =
    LazyLock::new(|| RwLock::new(Arc::new(Settings::new())));

/// Refresh the configuration
pub fn refresh_configuration() {
    let settings = global.read().unwrap();
    let path = settings.pref_path.clone();
    drop(settings); // Release the lock before potential long operation

    std::thread::spawn(move || match Settings::load_from_file_sync(&path) {
        Ok(new_settings) => {
            *global.write().unwrap() = Arc::new(new_settings);
        }
        Err(err) => {
            eprintln!("Failed to refresh configuration from '{}': {}", path, err);
        }
    })
    .join()
    .unwrap()
}

/// Update settings directly from file path with proper locking
pub fn update_settings_from_file(
    path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path = path.to_owned();
    std::thread::spawn(move || match Settings::load_from_file_sync(&path) {
        Ok(new_settings) => {
            *global.write().unwrap() = Arc::new(new_settings);
            Ok(())
        }
        Err(err) => {
            eprintln!("Failed to refresh configuration from '{}': {}", path, err);
            Err(format!("Failed to refresh configuration: {}", err).into())
        }
    })
    .join()
    .unwrap_or(Err("Failed to join thread".into()))
}

pub fn update_settings_from_content(
    content: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let content = content.to_owned();
    let handle = std::thread::spawn(move || {
        let settings = Settings::load_from_content(&content).unwrap();
        *global.write().unwrap() = Arc::new(settings);
    });
    handle.join().map_err(|_| "Failed to join thread".into())
}
