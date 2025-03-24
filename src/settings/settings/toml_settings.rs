use super::ini_bindings::{FromIni, FromIniWithDelimiter};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::{
    models::{
        cron::CronTaskConfigs, proxy_group_config::ProxyGroupConfigs, ruleset::RulesetConfigs,
        RegexMatchConfigs,
    },
    settings::import_items,
};

// 为toml::Value添加默认值函数
fn default_toml_value() -> toml::Value {
    toml::Value::String(String::new())
}

// 为常用默认值添加函数
fn default_true() -> bool {
    true
}

fn default_empty_string() -> String {
    String::new()
}

fn default_system() -> String {
    "SYSTEM".to_string()
}

fn default_none() -> String {
    "NONE".to_string()
}

fn default_listen_address() -> String {
    "127.0.0.1".to_string()
}

fn default_listen_port() -> i32 {
    25500
}

fn default_max_pending_conns() -> i32 {
    10240
}

fn default_max_concurrent_threads() -> i32 {
    4
}

fn default_info_log_level() -> String {
    "info".to_string()
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

fn default_max_download_size() -> i64 {
    32 * 1024 * 1024 // 32MB
}

/// Stream rule configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RegexMatchRule {
    pub match_pattern: Option<String>,
    #[serde(rename = "match")]
    pub match_str: Option<String>,
    pub replace: Option<String>,
    pub script: Option<String>,
    pub import: Option<String>,
}

/// User info settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct UserInfoSettings {
    pub stream_rule: Vec<RegexMatchRule>,
    pub time_rule: Vec<RegexMatchRule>,
}

/// Common settings section
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CommonSettings {
    pub api_mode: bool,
    pub api_access_token: String,
    #[serde(rename = "default_url")]
    pub default_urls: Vec<String>,
    #[serde(default = "default_true")]
    pub enable_insert: bool,
    #[serde(rename = "insert_url")]
    pub insert_urls: Vec<String>,
    #[serde(default = "default_true")]
    pub prepend_insert_url: bool,
    pub exclude_remarks: Vec<String>,
    pub include_remarks: Vec<String>,
    pub enable_filter: bool,
    pub filter_script: String,
    pub default_external_config: String,
    #[serde(default = "default_empty_string")]
    pub base_path: String,
    pub clash_rule_base: String,
    pub surge_rule_base: String,
    pub surfboard_rule_base: String,
    pub mellow_rule_base: String,
    pub quan_rule_base: String,
    pub quanx_rule_base: String,
    pub loon_rule_base: String,
    pub sssub_rule_base: String,
    pub singbox_rule_base: String,
    #[serde(default = "default_system")]
    pub proxy_config: String,
    #[serde(default = "default_system")]
    pub proxy_ruleset: String,
    #[serde(default = "default_none")]
    pub proxy_subscription: String,
    pub append_proxy_type: bool,
    pub reload_conf_on_request: bool,
}

/// Node preferences
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct NodePreferences {
    pub udp_flag: Option<bool>,
    pub tcp_fast_open_flag: Option<bool>,
    pub skip_cert_verify_flag: Option<bool>,
    pub tls13_flag: Option<bool>,
    pub sort_flag: bool,
    pub sort_script: String,
    pub filter_deprecated_nodes: bool,
    #[serde(default = "default_true")]
    pub append_sub_userinfo: bool,
    #[serde(default = "default_true")]
    pub clash_use_new_field_name: bool,
    #[serde(default = "default_empty_string")]
    pub clash_proxies_style: String,
    #[serde(default = "default_empty_string")]
    pub clash_proxy_groups_style: String,
    pub singbox_add_clash_modes: bool,
    pub rename_node: Vec<RegexMatchRule>,
}

/// Managed config settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ManagedConfigSettings {
    #[serde(default = "default_true")]
    pub write_managed_config: bool,
    #[serde(default = "default_listen_address")]
    pub managed_config_prefix: String,
    #[serde(default = "default_update_interval")]
    pub config_update_interval: i32,
    pub config_update_strict: bool,
    pub quanx_device_id: String,
}

fn default_update_interval() -> i32 {
    86400 // 24 hours
}

/// Surge external proxy settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SurgeExternalProxySettings {
    pub surge_ssr_path: String,
    pub resolve_hostname: bool,
}

/// Emoji settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EmojiSettings {
    pub add_emoji: bool,
    #[serde(default = "default_true")]
    pub remove_old_emoji: bool,
    pub emoji: Vec<RegexMatchRule>,
}

/// Proxy group configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ProxyGroupConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: String,
    pub rule: Vec<String>,
    #[serde(default = "default_test_url")]
    pub url: Option<String>,
    #[serde(default = "default_interval")]
    pub interval: Option<i32>,
    pub tolerance: Option<i32>,
    pub timeout: Option<i32>,
    pub lazy: Option<bool>,
    pub disable_udp: Option<bool>,
    pub strategy: Option<String>,
    pub import: Option<String>,
}

fn default_test_url() -> Option<String> {
    Some("http://www.gstatic.com/generate_204".to_string())
}

fn default_interval() -> Option<i32> {
    Some(300)
}

/// Ruleset settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RulesetSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub overwrite_original_rules: bool,
    pub update_ruleset_on_request: bool,
}

/// Ruleset configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RulesetConfig {
    pub group: String,
    pub ruleset: Option<String>,
    #[serde(rename = "type")]
    pub ruleset_type: Option<String>,
    pub interval: Option<i32>,
    pub import: Option<String>,
}

/// Template variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub key: String,
    #[serde(default = "default_toml_value")]
    pub value: toml::Value,
}

impl Default for TemplateVariable {
    fn default() -> Self {
        Self {
            key: String::new(),
            value: default_toml_value(),
        }
    }
}

/// Template settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TemplateSettings {
    pub template_path: String,
    pub globals: Vec<TemplateVariable>,
}

/// Alias configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AliasConfig {
    pub uri: String,
    pub target: String,
}

/// Task configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TaskConfig {
    pub name: String,
    pub cronexp: String,
    pub path: String,
    pub timeout: i32,
    pub import: Option<String>,
}

/// Server settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ServerSettings {
    #[serde(default = "default_listen_address")]
    pub listen: String,
    #[serde(default = "default_listen_port")]
    pub port: i32,
    pub serve_file_root: String,
}

/// Advanced settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AdvancedSettings {
    #[serde(default = "default_info_log_level")]
    pub log_level: String,
    pub print_debug_info: bool,
    #[serde(default = "default_max_pending_conns")]
    pub max_pending_connections: i32,
    #[serde(default = "default_max_concurrent_threads")]
    pub max_concurrent_threads: i32,
    #[serde(default = "default_max_rulesets")]
    pub max_allowed_rulesets: usize,
    #[serde(default = "default_max_rules")]
    pub max_allowed_rules: usize,
    #[serde(default = "default_max_download_size")]
    pub max_allowed_download_size: i64,
    pub enable_cache: bool,
    #[serde(default = "default_cache_subscription")]
    pub cache_subscription: i32,
    #[serde(default = "default_cache_config")]
    pub cache_config: i32,
    #[serde(default = "default_cache_ruleset")]
    pub cache_ruleset: i32,
    pub script_clean_context: bool,
    pub async_fetch_ruleset: bool,
    pub skip_failed_links: bool,
}

/// Main TOML settings structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TomlSettings {
    pub version: i32,
    pub common: CommonSettings,
    pub userinfo: UserInfoSettings,
    #[serde(rename = "node_pref")]
    pub node_pref: NodePreferences,
    #[serde(rename = "managed_config")]
    pub managed_config: ManagedConfigSettings,
    #[serde(rename = "surge_external_proxy")]
    pub surge_external_proxy: SurgeExternalProxySettings,
    pub emojis: EmojiSettings,
    pub ruleset: RulesetSettings,
    pub rulesets: Vec<RulesetConfig>,
    #[serde(rename = "custom_groups")]
    pub custom_proxy_groups: Vec<ProxyGroupConfig>,
    pub template: TemplateSettings,
    pub aliases: Vec<AliasConfig>,
    pub tasks: Vec<TaskConfig>,
    pub server: ServerSettings,
    pub advanced: AdvancedSettings,
    // Internal fields not present in TOML file
    #[serde(skip)]
    pub parsed_rename: RegexMatchConfigs,
    #[serde(skip)]
    pub parsed_stream_rule: RegexMatchConfigs,
    #[serde(skip)]
    pub parsed_time_rule: RegexMatchConfigs,
    #[serde(skip)]
    pub parsed_emoji_rules: RegexMatchConfigs,
    #[serde(skip)]
    pub parsed_proxy_group: ProxyGroupConfigs,
    #[serde(skip)]
    pub parsed_ruleset: RulesetConfigs,
    #[serde(skip)]
    pub parsed_tasks: CronTaskConfigs,
}

impl TomlSettings {
    pub fn process_imports(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Process rename nodes
        let mut rename_nodes = Vec::new();
        for rule in &self.node_pref.rename_node {
            if let Some(import) = &rule.import {
                rename_nodes.push(format!("!!import:{}", import));
            } else if let (Some(match_pattern), Some(replace)) =
                (&rule.match_pattern, &rule.replace)
            {
                rename_nodes.push(format!("{}@{}", match_pattern, replace));
            } else if let (Some(match_str), Some(replace)) = (&rule.match_str, &rule.replace) {
                rename_nodes.push(format!("{}@{}", match_str, replace));
            } else if let Some(script) = &rule.script {
                rename_nodes.push(format!("!!script:{}", script));
            }
        }
        import_items(
            &mut rename_nodes,
            false,
            &self.common.proxy_config,
            &self.common.base_path,
        )?;
        self.parsed_rename = RegexMatchConfigs::from_ini_with_delimiter(&rename_nodes, "@");

        // Process stream rules
        let mut stream_rules = Vec::new();
        for rule in &self.userinfo.stream_rule {
            if let Some(import) = &rule.import {
                stream_rules.push(format!("!!import:{}", import));
            } else if let (Some(match_pattern), Some(replace)) =
                (&rule.match_pattern, &rule.replace)
            {
                stream_rules.push(format!("{}|{}", match_pattern, replace));
            } else if let (Some(match_str), Some(replace)) = (&rule.match_str, &rule.replace) {
                stream_rules.push(format!("{}|{}", match_str, replace));
            } else if let Some(script) = &rule.script {
                stream_rules.push(format!("!!script:{}", script));
            }
        }
        import_items(
            &mut stream_rules,
            false,
            &self.common.proxy_config,
            &self.common.base_path,
        )?;
        self.parsed_stream_rule = RegexMatchConfigs::from_ini_with_delimiter(&stream_rules, "|");

        // Process time rules
        let mut time_rules = Vec::new();
        for rule in &self.userinfo.time_rule {
            if let Some(import) = &rule.import {
                time_rules.push(format!("!!import:{}", import));
            } else if let (Some(match_pattern), Some(replace)) =
                (&rule.match_pattern, &rule.replace)
            {
                time_rules.push(format!("{}|{}", match_pattern, replace));
            } else if let (Some(match_str), Some(replace)) = (&rule.match_str, &rule.replace) {
                time_rules.push(format!("{}|{}", match_str, replace));
            } else if let Some(script) = &rule.script {
                time_rules.push(format!("!!script:{}", script));
            }
        }
        import_items(
            &mut time_rules,
            false,
            &self.common.proxy_config,
            &self.common.base_path,
        )?;
        self.parsed_time_rule = RegexMatchConfigs::from_ini_with_delimiter(&time_rules, "|");

        // Process emoji rules
        let mut emoji_rules = Vec::new();
        for rule in &self.emojis.emoji {
            if let Some(import) = &rule.import {
                emoji_rules.push(format!("!!import:{}", import));
            } else if let (Some(match_pattern), Some(replace)) =
                (&rule.match_pattern, &rule.replace)
            {
                emoji_rules.push(format!("{},{}", match_pattern, replace));
            } else if let (Some(match_str), Some(replace)) = (&rule.match_str, &rule.replace) {
                emoji_rules.push(format!("{},{}", match_str, replace));
            } else if let Some(script) = &rule.script {
                emoji_rules.push(format!("!!script:{}", script));
            }
        }
        import_items(
            &mut emoji_rules,
            false,
            &self.common.proxy_config,
            &self.common.base_path,
        )?;
        self.parsed_emoji_rules = RegexMatchConfigs::from_ini_with_delimiter(&emoji_rules, ",");

        // Process rulesets
        let mut rulesets = Vec::new();
        for ruleset in &self.rulesets {
            if let Some(import) = &ruleset.import {
                rulesets.push(format!("!!import:{}", import));
            } else {
                let mut ruleset_str = ruleset.group.clone();

                if !ruleset.ruleset.as_ref().map_or(true, |s| s.is_empty()) {
                    ruleset_str.push_str(&format!(",{}", ruleset.ruleset.as_ref().unwrap()));

                    // Add interval if provided
                    if let Some(interval) = ruleset.interval {
                        ruleset_str.push_str(&format!(",{}", interval));
                    }
                } else {
                    ruleset_str.push_str(",[]");
                }

                if !ruleset_str.eq(&ruleset.group) {
                    rulesets.push(ruleset_str);
                }
            }
        }
        import_items(
            &mut rulesets,
            false,
            &self.common.proxy_config,
            &self.common.base_path,
        )?;
        self.parsed_ruleset = RulesetConfigs::from_ini(&rulesets);

        // Process proxy groups
        let mut proxy_groups = Vec::new();
        for group in &self.custom_proxy_groups {
            if let Some(import) = &group.import {
                proxy_groups.push(format!("!!import:{}", import));
            } else {
                let mut group_str = format!("{}`{}", group.name, group.group_type);

                // Add all rules
                for rule in &group.rule {
                    group_str.push_str(&format!("`{}", rule));
                }

                // Add URL and interval information for appropriate group types
                if group.group_type == "url-test"
                    || group.group_type == "fallback"
                    || group.group_type == "load-balance"
                    || group.group_type == "smart"
                {
                    if let Some(url) = &group.url {
                        group_str.push_str(&format!("`{}", url));

                        // Format: interval,timeout,tolerance
                        let interval = group
                            .interval
                            .unwrap_or_else(|| default_interval().unwrap());
                        let timeout = group.timeout.unwrap_or(5);
                        let tolerance = group.tolerance.unwrap_or(0);
                        group_str.push_str(&format!("`{},{},{}", interval, timeout, tolerance));
                    }
                }

                proxy_groups.push(group_str);
            }
        }
        import_items(
            &mut proxy_groups,
            false,
            &self.common.proxy_config,
            &self.common.base_path,
        )?;
        self.parsed_proxy_group = ProxyGroupConfigs::from_ini(&proxy_groups);

        // Process tasks
        let mut tasks = Vec::new();
        for task in &self.tasks {
            if let Some(import) = &task.import {
                tasks.push(format!("!!import:{}", import));
            } else {
                tasks.push(format!(
                    "{}`{}`{}`{}",
                    task.name, task.cronexp, task.path, task.timeout
                ));
            }
        }
        import_items(
            &mut tasks,
            false,
            &self.common.proxy_config,
            &self.common.base_path,
        )?;
        self.parsed_tasks = CronTaskConfigs::from_ini(&tasks);

        Ok(())
    }
}
