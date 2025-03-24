use serde::Deserialize;

use crate::utils::file_get;

pub fn deserialize_rulesets<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize, Debug, Default)]
    #[serde(default)]
    struct ImportableRuleSet {
        import: Option<String>,
        ruleset: String,
        group: String,
        rule: String,
        interval: String,
    }
    let mut source_vec: Vec<ImportableRuleSet> = Vec::deserialize(deserializer)?;
    let mut result_vec = Vec::new();
    let mut i = 0;
    while i < source_vec.len() {
        let item = &source_vec[i];
        if let Some(import) = item.import.clone() {
            if import.ends_with(".toml") {
                // TODO: 安全隐患，需要改进
                let content = file_get(&import, None).unwrap_or_default();
                let rulesets =
                    toml::from_str::<Vec<ImportableRuleSet>>(&content).unwrap_or_default();
                source_vec.extend(rulesets);
            }
            if import.ends_with(".yaml") || import.ends_with(".yml") {
                let content = file_get(&import, None).unwrap_or_default();
                let rulesets =
                    serde_yml::from_str::<Vec<ImportableRuleSet>>(&content).unwrap_or_default();
                source_vec.extend(rulesets);
            }
            result_vec.push(format!("!!import:{}", import));
        } else {
            let mut line;
            if !item.ruleset.is_empty() {
                line = format!("{},{}", item.group, item.ruleset);
                if !item.interval.is_empty() {
                    line.push_str(&format!(",{}", item.interval));
                }
            } else if !item.rule.is_empty() {
                line = format!("{},[]{}", item.group, item.rule);
            } else {
                continue;
            }
            result_vec.push(line);
        }
        i += 1;
    }
    Ok(result_vec)
}

pub fn deserialize_proxy_groups<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize, Debug, Clone, Default)]
    #[serde(default)]
    struct ImportableProxyGroup {
        import: Option<String>,
        name: String,
        #[serde(rename = "type")]
        _type: String,
        url: String,
        interval: String,
        tolerance: String,
        timeout: String,
        rule: Vec<String>,
    }
    let source_vec: Vec<ImportableProxyGroup> = Vec::deserialize(deserializer)?;
    let mut result_vec = Vec::new();
    for item in source_vec {
        if let Some(import) = item.import {
            result_vec.push(import);
        } else {
            let mut temp_arr = item.rule;
            temp_arr.insert(0, item.name);
            temp_arr.insert(1, item._type.clone());
            if item._type == "select" {
                if temp_arr.len() < 3 {
                    continue;
                }
            } else if item._type == "ssid" {
                if temp_arr.len() > 4 {
                    continue;
                }
            } else {
                if temp_arr.len() < 3 {
                    continue;
                }
                temp_arr.push(item.url);
                temp_arr.push(format!(
                    "{},{},{}",
                    item.interval, item.timeout, item.tolerance
                ));
            }
            result_vec.push(temp_arr.join("`"));
        }
    }
    Ok(result_vec)
}
