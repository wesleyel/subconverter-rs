use serde::Deserialize;
use std::collections::HashMap;

use crate::models::RegexMatchConfig;

// Default value functions
fn default_true() -> bool {
    true
}

/// Rule bases settings
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct RuleBasesSettings {
    pub clash_rule_base: String,
    pub surge_rule_base: String,
    pub surfboard_rule_base: String,
    pub mellow_rule_base: String,
    pub quan_rule_base: String,
    pub quanx_rule_base: String,
    pub loon_rule_base: String,
    pub sssub_rule_base: String,
    pub singbox_rule_base: String,
}

/// Rule generation options
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct RuleGenerationSettings {
    #[serde(default = "default_true")]
    pub enable_rule_generator: bool,
    pub overwrite_original_rules: bool,
}

/// Emoji settings
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct EmojiSettings {
    pub add_emoji: bool,
    #[serde(default = "default_true")]
    pub remove_old_emoji: bool,
    pub emoji: Vec<RegexMatchConfig>,
}

/// Filtering settings
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct FilteringSettings {
    pub include_remarks: Vec<String>,
    pub exclude_remarks: Vec<String>,
}

/// Ruleset configuration
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct RulesetConfig {
    pub group: String,
    pub ruleset: String,
    pub interval: Option<i32>,
    pub url: Option<String>,
}

/// Proxy group configuration
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct ProxyGroupConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: String,
    pub rule: Vec<String>,
    pub url: Option<String>,
    pub interval: Option<i32>,
    pub tolerance: Option<i32>,
    pub timeout: Option<i32>,
    pub lazy: Option<bool>,
    pub disable_udp: Option<bool>,
    pub strategy: Option<String>,
}

/// Custom settings
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct CustomSettings {
    // Include the custom settings from RuleBasesSettings
    #[serde(flatten)]
    pub rule_bases: RuleBasesSettings,

    // Rule generation options
    #[serde(flatten)]
    pub rule_generation: RuleGenerationSettings,

    // Emoji settings
    #[serde(flatten)]
    pub emoji_settings: EmojiSettings,

    // Filtering settings
    #[serde(flatten)]
    pub filtering: FilteringSettings,

    // Rulesets and proxy groups
    pub rulesets: Vec<RulesetConfig>,
    pub custom_proxy_group: Vec<ProxyGroupConfig>,
}

/// Main YAML external settings structure
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct YamlExternalSettings {
    pub custom: CustomSettings,
    pub rename: Vec<RegexMatchConfig>,
    pub tpl_args: Option<HashMap<String, String>>,
}
