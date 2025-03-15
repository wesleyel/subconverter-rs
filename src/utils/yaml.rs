use serde_yaml::{self, Value};
use std::collections::HashMap;

/// Wrapper around serde_yaml::Value for easier manipulation
#[derive(Debug, Clone)]
pub struct YamlNode {
    pub value: Value,
}

impl YamlNode {
    /// Create a new empty YAML node
    pub fn new() -> Self {
        YamlNode { value: Value::Null }
    }

    /// Create a YamlNode from a YAML string
    pub fn from_str(content: &str) -> Result<Self, serde_yaml::Error> {
        let value = serde_yaml::from_str(content)?;
        Ok(YamlNode { value })
    }

    /// Convert the YAML node to a string
    pub fn to_string(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(&self.value)
    }

    /// Get a value from a path
    pub fn get_value(&self, path: &str) -> Option<&Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.value;

        for part in parts {
            if part.is_empty() {
                continue;
            }

            match current {
                Value::Mapping(map) => {
                    // Try both string key and non-string key
                    let key_str = Value::String(part.to_string());
                    current = map.get(&key_str)?;
                }
                Value::Sequence(seq) => {
                    if let Ok(index) = part.parse::<usize>() {
                        current = seq.get(index)?;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        Some(current)
    }

    /// Set a value at a path
    pub fn set_value(&mut self, path: &str, value: Value) -> bool {
        // Placeholder for implementation
        false
    }
}

impl Default for YamlNode {
    fn default() -> Self {
        Self::new()
    }
}
