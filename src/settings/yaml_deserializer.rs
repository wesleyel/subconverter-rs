use serde::Deserialize;

/// Stream rule configuration
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct RegexMatchRuleInYaml {
    #[serde(rename = "match")]
    pub match_str: Option<String>,
    pub replace: Option<String>,
    pub script: Option<String>,
    pub import: Option<String>,
}

/// Trait for converting to INI format with a specified delimiter
pub trait ToIniWithDelimiter {
    fn to_ini_with_delimiter(&self, delimiter: &str) -> String;
}

impl ToIniWithDelimiter for RegexMatchRuleInYaml {
    fn to_ini_with_delimiter(&self, delimiter: &str) -> String {
        // Check for script first
        if let Some(script) = &self.script {
            if !script.is_empty() {
                return format!("!!script:{}", script);
            }
        }

        // Then check for import
        if let Some(import) = &self.import {
            if !import.is_empty() {
                return format!("!!import:{}", import);
            }
        }

        // Finally check for match and replace
        if let (Some(match_str), Some(replace)) = (&self.match_str, &self.replace) {
            if !match_str.is_empty() && !replace.is_empty() {
                return format!("{}{}{}", match_str, delimiter, replace);
            }
        }

        // Default to empty string if nothing matches
        String::new()
    }
}

pub trait ToIni {
    fn to_ini(&self) -> String;
}

impl ToIni for RulesetConfigInYaml {
    fn to_ini(&self) -> String {
        // Check for import first
        if let Some(import) = &self.import {
            if !import.is_empty() {
                return format!("!!import:{}", import);
            }
        }

        // Then check for ruleset URL
        if let Some(ruleset) = &self.ruleset {
            if !ruleset.is_empty() {
                let mut result = format!("{},{}", self.group, ruleset);
                // Add interval if provided
                if let Some(interval) = self.interval {
                    result = format!("{},{}", result, interval);
                }
                return result;
            }
        }

        // Finally check for rule
        if let Some(rule) = &self.rule {
            if !rule.is_empty() {
                return format!("{},[]{}", self.group, rule);
            }
        }

        // Default to empty string if nothing matches
        String::new()
    }
}

impl ToIni for TaskConfigInYaml {
    fn to_ini(&self) -> String {
        // Check for import first
        if let Some(import) = &self.import {
            if !import.is_empty() {
                return format!("!!import:{}", import);
            }
        }

        // Otherwise join fields with backticks
        format!(
            "{}`{}`{}`{}",
            self.name, self.cronexp, self.path, self.timeout
        )
    }
}

/// Proxy group configuration
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct ProxyGroupConfigInYaml {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: String,
    pub rule: Vec<String>,
    #[serde(default = "default_test_url")]
    pub url: Option<String>,
    #[serde(default = "default_interval")]
    pub interval: Option<u32>,
    pub tolerance: Option<u32>,
    pub timeout: Option<u32>,
    pub import: Option<String>,
}

fn default_test_url() -> Option<String> {
    Some("http://www.gstatic.com/generate_204".to_string())
}

fn default_interval() -> Option<u32> {
    Some(300)
}

/// Task configuration
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct TaskConfigInYaml {
    pub name: String,
    pub cronexp: String,
    pub path: String,
    pub timeout: u32,
    pub import: Option<String>,
}

/// Ruleset configuration
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct RulesetConfigInYaml {
    pub rule: Option<String>,
    pub ruleset: Option<String>,
    pub group: String,
    pub interval: Option<u32>,
    pub import: Option<String>,
}
