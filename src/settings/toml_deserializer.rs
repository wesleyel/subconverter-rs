use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::models::{
    cron::CronTaskConfig, BalanceStrategy, ProxyGroupConfig, ProxyGroupType, RegexMatchConfig,
    RulesetConfig,
};
use crate::settings::settings::toml_settings::TemplateSettings;

pub trait ImportableInToml: serde::de::DeserializeOwned + Clone {
    fn is_import_node(&self) -> bool;
    fn get_import_path(&self) -> Option<String>;
    fn try_from_toml_value(value: &toml::Value) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(value.clone().try_into()?)
    }
}

/// Stream rule configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RegexMatchRuleInToml {
    #[serde(rename = "match")]
    pub match_str: Option<String>,

    #[serde(alias = "emoji")]
    pub replace: Option<String>,
    pub script: Option<String>,
    pub import: Option<String>,
}

impl Into<RegexMatchConfig> for RegexMatchRuleInToml {
    fn into(self) -> RegexMatchConfig {
        RegexMatchConfig {
            _match: self.match_str.unwrap_or_default(),
            replace: self.replace.unwrap_or_default(),
        }
    }
}

impl ImportableInToml for RegexMatchRuleInToml {
    fn is_import_node(&self) -> bool {
        self.import.is_some()
    }

    fn get_import_path(&self) -> Option<String> {
        self.import.clone()
    }
}

/// Ruleset configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RulesetConfigInToml {
    pub group: String,
    pub ruleset: Option<String>,
    #[serde(rename = "type")]
    pub ruleset_type: Option<String>,
    pub interval: Option<u32>,
    pub import: Option<String>,
}

impl ImportableInToml for RulesetConfigInToml {
    fn is_import_node(&self) -> bool {
        self.import.is_some()
    }

    fn get_import_path(&self) -> Option<String> {
        self.import.clone()
    }
}

impl Into<RulesetConfig> for RulesetConfigInToml {
    fn into(self) -> RulesetConfig {
        RulesetConfig {
            url: self.ruleset.unwrap_or_default(),
            group: self.group,
            interval: self.interval.unwrap_or(300),
        }
    }
}

fn default_test_url() -> Option<String> {
    Some("http://www.gstatic.com/generate_204".to_string())
}

fn default_interval() -> Option<u32> {
    Some(300)
}

/// Proxy group configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ProxyGroupConfigInToml {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: String,
    pub strategy: Option<String>,
    pub rule: Vec<String>,
    #[serde(default = "default_test_url")]
    pub url: Option<String>,
    #[serde(default = "default_interval")]
    pub interval: Option<u32>,
    pub lazy: Option<bool>,
    pub tolerance: Option<u32>,
    pub timeout: Option<u32>,
    pub disable_udp: Option<bool>,
    pub import: Option<String>,
}

impl ImportableInToml for ProxyGroupConfigInToml {
    fn is_import_node(&self) -> bool {
        self.import.is_some()
    }

    fn get_import_path(&self) -> Option<String> {
        self.import.clone()
    }
}

impl Into<ProxyGroupConfig> for ProxyGroupConfigInToml {
    fn into(self) -> ProxyGroupConfig {
        let group_type = match self.group_type.as_str() {
            "select" => ProxyGroupType::Select,
            "url-test" => ProxyGroupType::URLTest,
            "load-balance" => ProxyGroupType::LoadBalance,
            "fallback" => ProxyGroupType::Fallback,
            "relay" => ProxyGroupType::Relay,
            "ssid" => ProxyGroupType::SSID,
            "smart" => ProxyGroupType::Smart,
            _ => ProxyGroupType::Select, // 默认为 Select
        };

        // 处理 strategy 字段
        let strategy = match self.strategy.as_deref() {
            Some("consistent-hashing") => BalanceStrategy::ConsistentHashing,
            Some("round-robin") => BalanceStrategy::RoundRobin,
            _ => BalanceStrategy::ConsistentHashing,
        };

        // 创建基本的 ProxyGroupConfig
        let mut config = ProxyGroupConfig {
            name: self.name,
            group_type,
            proxies: self.rule,
            url: self.url.unwrap_or_default(),
            interval: self.interval.unwrap_or(300),
            tolerance: self.tolerance.unwrap_or(0),
            timeout: self.timeout.unwrap_or(5),
            lazy: self.lazy.unwrap_or(false),
            disable_udp: self.disable_udp.unwrap_or(false),
            strategy,
            // 添加缺失的字段
            persistent: false,
            evaluate_before_use: false,
            using_provider: Vec::new(),
        };

        // 根据不同的代理组类型设置特定属性
        match config.group_type {
            ProxyGroupType::URLTest | ProxyGroupType::Smart => {
                // 这些类型需要 URL 和 interval
                if config.url.is_empty() {
                    config.url = "http://www.gstatic.com/generate_204".to_string();
                }
            }
            ProxyGroupType::LoadBalance => {
                // 负载均衡需要 URL、interval 和 strategy
                if config.url.is_empty() {
                    config.url = "http://www.gstatic.com/generate_204".to_string();
                }
            }
            ProxyGroupType::Fallback => {
                // 故障转移需要 URL 和 interval
                if config.url.is_empty() {
                    config.url = "http://www.gstatic.com/generate_204".to_string();
                }
            }
            _ => {}
        }

        config
    }
}

/// Task configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TaskConfigInToml {
    pub name: String,
    pub cronexp: String,
    pub path: String,
    pub timeout: u32,
    pub import: Option<String>,
}

impl ImportableInToml for TaskConfigInToml {
    fn is_import_node(&self) -> bool {
        self.import.is_some()
    }

    fn get_import_path(&self) -> Option<String> {
        self.import.clone()
    }
}

impl Into<CronTaskConfig> for TaskConfigInToml {
    fn into(self) -> CronTaskConfig {
        CronTaskConfig {
            name: self.name,
            cron_exp: self.cronexp,
            path: self.path,
            timeout: self.timeout,
        }
    }
}

pub fn deserialize_template_settings<'de, D>(deserializer: D) -> Result<TemplateSettings, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct TemplateSettingsVisitor;

    impl<'de> Visitor<'de> for TemplateSettingsVisitor {
        type Value = TemplateSettings;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a TemplateSettings struct")
        }

        fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
        where
            V: MapAccess<'de>,
        {
            let mut template_settings = TemplateSettings::default();
            while let Some(key) = map.next_key::<String>()? {
                let value = map.next_value::<String>()?;
                if key == "template_path" {
                    template_settings.template_path = value.clone();
                } else {
                    template_settings.globals.insert(key, value);
                }
            }
            Ok(template_settings)
        }
    }

    deserializer.deserialize_any(TemplateSettingsVisitor)
}
