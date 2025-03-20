use serde_json::{self, Value as JsonValue};
use serde_yaml::{self, Mapping, Sequence, Value as YamlValue};
use std::fmt;

/// Error types for YAML operations
#[derive(Debug)]
pub enum YamlError {
    ParseError(serde_yaml::Error),
    SerializeError(serde_yaml::Error),
    PathError(String),
    TypeError(String),
    NotFound,
}

impl fmt::Display for YamlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YamlError::ParseError(e) => write!(f, "YAML parse error: {}", e),
            YamlError::SerializeError(e) => write!(f, "YAML serialize error: {}", e),
            YamlError::PathError(s) => write!(f, "Invalid path: {}", s),
            YamlError::TypeError(s) => write!(f, "Type error: {}", s),
            YamlError::NotFound => write!(f, "Node not found"),
        }
    }
}

impl From<serde_yaml::Error> for YamlError {
    fn from(error: serde_yaml::Error) -> Self {
        YamlError::ParseError(error)
    }
}

/// Wrapper around serde_yaml::Value for easier manipulation, similar to yaml-cpp's Node
#[derive(Debug, Clone)]
pub struct YamlNode {
    pub value: YamlValue,
}

impl YamlNode {
    /// Create a new empty YAML node
    pub fn new() -> Self {
        YamlNode {
            value: YamlValue::Null,
        }
    }

    /// Create a YamlNode from a YAML string
    pub fn from_str(content: &str) -> Result<Self, YamlError> {
        let value = serde_yaml::from_str(content)?;
        Ok(YamlNode { value })
    }

    /// Convert the YAML node to a string
    pub fn to_string(&self) -> Result<String, YamlError> {
        serde_yaml::to_string(&self.value).map_err(YamlError::SerializeError)
    }

    /// Check if the node is null
    pub fn is_null(&self) -> bool {
        matches!(self.value, YamlValue::Null)
    }

    /// Check if the node is a scalar
    pub fn is_scalar(&self) -> bool {
        matches!(
            self.value,
            YamlValue::String(_) | YamlValue::Number(_) | YamlValue::Bool(_) | YamlValue::Null
        )
    }

    /// Check if the node is a sequence
    pub fn is_sequence(&self) -> bool {
        matches!(self.value, YamlValue::Sequence(_))
    }

    /// Check if the node is a mapping
    pub fn is_mapping(&self) -> bool {
        matches!(self.value, YamlValue::Mapping(_))
    }

    /// Get a child node by key from a mapping
    pub fn get(&self, key: &str) -> Option<&YamlValue> {
        if let YamlValue::Mapping(map) = &self.value {
            map.get(&YamlValue::String(key.to_string()))
        } else {
            None
        }
    }

    /// Get a mutable child node by key from a mapping
    pub fn get_mut(&mut self, key: &str) -> Option<&mut YamlValue> {
        if let YamlValue::Mapping(map) = &mut self.value {
            map.get_mut(&YamlValue::String(key.to_string()))
        } else {
            None
        }
    }

    /// Set a value at a specific key in a mapping
    pub fn insert(&mut self, key: &str, value: YamlValue) -> Result<(), YamlError> {
        if let YamlValue::Mapping(map) = &mut self.value {
            map.insert(YamlValue::String(key.to_string()), value);
            Ok(())
        } else {
            Err(YamlError::TypeError(
                "Cannot insert into non-mapping node".to_string(),
            ))
        }
    }

    /// Remove a key from a mapping
    pub fn remove(&mut self, key: &str) -> Result<(), YamlError> {
        if let YamlValue::Mapping(map) = &mut self.value {
            map.remove(&YamlValue::String(key.to_string()));
            Ok(())
        } else {
            Err(YamlError::TypeError(
                "Cannot remove from non-mapping node".to_string(),
            ))
        }
    }

    /// Get a value by path, using dot notation for nested structures
    pub fn get_value(&self, path: &str) -> Option<&YamlValue> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.value;

        for part in parts {
            if part.is_empty() {
                continue;
            }

            match current {
                YamlValue::Mapping(map) => {
                    // Try both string key and non-string key
                    let key_str = YamlValue::String(part.to_string());
                    current = map.get(&key_str)?;
                }
                YamlValue::Sequence(seq) => {
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

    /// Set a value at a path using dot notation
    pub fn set_value(&mut self, path: &str, value: YamlValue) -> Result<(), YamlError> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return Err(YamlError::PathError("Empty path".to_string()));
        }

        // Use recursive approach
        self.set_value_recursive(&parts, 0, value)
    }

    /// Recursive helper for set_value
    fn set_value_recursive(
        &mut self,
        parts: &[&str],
        index: usize,
        value: YamlValue,
    ) -> Result<(), YamlError> {
        if index >= parts.len() {
            return Ok(());
        }

        let part = parts[index];
        if part.is_empty() {
            // Skip empty parts
            return self.set_value_recursive(parts, index + 1, value);
        }

        // Last part - set the value directly
        if index == parts.len() - 1 {
            match &mut self.value {
                YamlValue::Mapping(map) => {
                    map.insert(YamlValue::String(part.to_string()), value);
                    Ok(())
                }
                YamlValue::Sequence(seq) => {
                    if let Ok(idx) = part.parse::<usize>() {
                        // Ensure sequence is big enough
                        while seq.len() <= idx {
                            seq.push(YamlValue::Null);
                        }
                        seq[idx] = value;
                        Ok(())
                    } else {
                        Err(YamlError::PathError(format!(
                            "Invalid sequence index: {}",
                            part
                        )))
                    }
                }
                _ => {
                    // Convert to mapping if it's not a container
                    self.value = YamlValue::Mapping(Mapping::new());
                    if let YamlValue::Mapping(map) = &mut self.value {
                        map.insert(YamlValue::String(part.to_string()), value);
                        Ok(())
                    } else {
                        // This should never happen
                        Err(YamlError::TypeError(
                            "Failed to convert to mapping".to_string(),
                        ))
                    }
                }
            }
        } else {
            // Intermediate part - ensure the path exists and recurse
            match &mut self.value {
                YamlValue::Mapping(map) => {
                    let key = YamlValue::String(part.to_string());
                    if !map.contains_key(&key) {
                        // Create intermediate node
                        map.insert(key.clone(), YamlValue::Mapping(Mapping::new()));
                    }

                    if let Some(next_value) = map.get_mut(&key) {
                        let mut next_node = YamlNode {
                            value: std::mem::take(next_value),
                        };
                        let result = next_node.set_value_recursive(parts, index + 1, value);
                        *next_value = next_node.value;
                        result
                    } else {
                        Err(YamlError::PathError(format!(
                            "Failed to navigate to {}",
                            part
                        )))
                    }
                }
                YamlValue::Sequence(seq) => {
                    if let Ok(idx) = part.parse::<usize>() {
                        // Ensure sequence is big enough
                        while seq.len() <= idx {
                            seq.push(YamlValue::Mapping(Mapping::new()));
                        }

                        let mut next_node = YamlNode {
                            value: std::mem::take(&mut seq[idx]),
                        };
                        let result = next_node.set_value_recursive(parts, index + 1, value);
                        seq[idx] = next_node.value;
                        result
                    } else {
                        Err(YamlError::PathError(format!(
                            "Invalid sequence index: {}",
                            part
                        )))
                    }
                }
                _ => {
                    // If this isn't the last part, we need a container here
                    // Convert to mapping and continue
                    self.value = YamlValue::Mapping(Mapping::new());
                    self.set_value_recursive(parts, index, value) // Try again with new mapping
                }
            }
        }
    }

    /// Get a value as a string
    pub fn as_str(&self) -> Option<&str> {
        if let YamlValue::String(s) = &self.value {
            Some(s)
        } else {
            None
        }
    }

    /// Get a value as an integer
    pub fn as_i64(&self) -> Option<i64> {
        if let YamlValue::Number(n) = &self.value {
            n.as_i64()
        } else {
            None
        }
    }

    /// Get a value as a float
    pub fn as_f64(&self) -> Option<f64> {
        if let YamlValue::Number(n) = &self.value {
            n.as_f64()
        } else {
            None
        }
    }

    /// Get a value as a boolean
    pub fn as_bool(&self) -> Option<bool> {
        if let YamlValue::Bool(b) = &self.value {
            Some(*b)
        } else {
            None
        }
    }

    /// Get a value as a sequence
    pub fn as_sequence(&self) -> Option<&Sequence> {
        if let YamlValue::Sequence(seq) = &self.value {
            Some(seq)
        } else {
            None
        }
    }

    /// Get a value as a mutable sequence
    pub fn as_sequence_mut(&mut self) -> Option<&mut Sequence> {
        if let YamlValue::Sequence(seq) = &mut self.value {
            Some(seq)
        } else {
            None
        }
    }

    /// Get a value as a mapping
    pub fn as_mapping(&self) -> Option<&Mapping> {
        if let YamlValue::Mapping(map) = &self.value {
            Some(map)
        } else {
            None
        }
    }

    /// Get a value as a mutable mapping
    pub fn as_mapping_mut(&mut self) -> Option<&mut Mapping> {
        if let YamlValue::Mapping(map) = &mut self.value {
            Some(map)
        } else {
            None
        }
    }

    /// Convert YAML node to a JSON value
    pub fn to_json(&self) -> Result<JsonValue, YamlError> {
        self.yaml_to_json(&self.value)
    }

    /// Convert a YAML value to a JSON value (internal helper)
    fn yaml_to_json(&self, yaml: &YamlValue) -> Result<JsonValue, YamlError> {
        match yaml {
            YamlValue::Null => Ok(JsonValue::Null),
            YamlValue::Bool(b) => Ok(JsonValue::Bool(*b)),
            YamlValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(JsonValue::Number(serde_json::Number::from(i)))
                } else if let Some(f) = n.as_f64() {
                    // This might fail for NaN or infinity
                    match serde_json::Number::from_f64(f) {
                        Some(num) => Ok(JsonValue::Number(num)),
                        None => Err(YamlError::TypeError(format!("Invalid JSON number: {}", f))),
                    }
                } else {
                    Err(YamlError::TypeError("Invalid number format".to_string()))
                }
            }
            YamlValue::String(s) => Ok(JsonValue::String(s.clone())),
            YamlValue::Sequence(seq) => {
                let mut json_array = Vec::with_capacity(seq.len());
                for item in seq {
                    json_array.push(self.yaml_to_json(item)?);
                }
                Ok(JsonValue::Array(json_array))
            }
            YamlValue::Mapping(map) => {
                let mut json_obj = serde_json::Map::new();
                for (k, v) in map {
                    let key = if let YamlValue::String(s) = k {
                        s.clone()
                    } else {
                        // Convert non-string keys to string
                        format!("{:?}", k)
                    };
                    json_obj.insert(key, self.yaml_to_json(v)?);
                }
                Ok(JsonValue::Object(json_obj))
            }
            // Handle any other types here if needed
            _ => Err(YamlError::TypeError(format!(
                "Unsupported YAML type: {:?}",
                yaml
            ))),
        }
    }

    /// Convert JSON value to a YAML node
    pub fn from_json(json: &JsonValue) -> Result<Self, YamlError> {
        let yaml_value = Self::json_to_yaml(json)?;
        Ok(YamlNode { value: yaml_value })
    }

    /// Convert a JSON value to a YAML value (internal helper)
    fn json_to_yaml(json: &JsonValue) -> Result<YamlValue, YamlError> {
        match json {
            JsonValue::Null => Ok(YamlValue::Null),
            JsonValue::Bool(b) => Ok(YamlValue::Bool(*b)),
            JsonValue::Number(n) => {
                if n.is_i64() {
                    Ok(YamlValue::Number(n.as_i64().unwrap().into()))
                } else if n.is_f64() {
                    Ok(YamlValue::Number(n.as_f64().unwrap().into()))
                } else {
                    Err(YamlError::TypeError(
                        "Invalid JSON number format".to_string(),
                    ))
                }
            }
            JsonValue::String(s) => Ok(YamlValue::String(s.clone())),
            JsonValue::Array(arr) => {
                let mut yaml_seq = Sequence::new();
                for item in arr {
                    yaml_seq.push(Self::json_to_yaml(item)?);
                }
                Ok(YamlValue::Sequence(yaml_seq))
            }
            JsonValue::Object(obj) => {
                let mut yaml_map = Mapping::new();
                for (k, v) in obj {
                    yaml_map.insert(YamlValue::String(k.clone()), Self::json_to_yaml(v)?);
                }
                Ok(YamlValue::Mapping(yaml_map))
            }
        }
    }

    /// Remove all null values recursively from the YAML structure
    pub fn remove_null(&mut self) {
        match &mut self.value {
            YamlValue::Mapping(map) => {
                // Collect keys to remove first to avoid borrow checker issues
                let keys_to_remove: Vec<YamlValue> = map
                    .iter()
                    .filter(|(_, v)| matches!(v, YamlValue::Null))
                    .map(|(k, _)| k.clone())
                    .collect();

                // Remove null values
                for key in keys_to_remove {
                    map.remove(&key);
                }

                // Recursively process remaining values
                for (_, value) in map.iter_mut() {
                    if let YamlValue::Mapping(_) | YamlValue::Sequence(_) = value {
                        // Create a temporary empty value to swap with value
                        let mut temp_value = YamlValue::Null;
                        std::mem::swap(value, &mut temp_value);

                        // Process the value
                        let mut node = YamlNode { value: temp_value };
                        node.remove_null();

                        // Put the result back
                        *value = node.value;
                    }
                }
            }
            YamlValue::Sequence(seq) => {
                // Remove null values
                seq.retain(|v| !matches!(v, YamlValue::Null));

                // Recursively process remaining values
                for value in seq.iter_mut() {
                    if let YamlValue::Mapping(_) | YamlValue::Sequence(_) = value {
                        // Create a temporary empty value to swap with value
                        let mut temp_value = YamlValue::Null;
                        std::mem::swap(value, &mut temp_value);

                        // Process the value
                        let mut node = YamlNode { value: temp_value };
                        node.remove_null();

                        // Put the result back
                        *value = node.value;
                    }
                }
            }
            _ => {}
        }
    }
}

impl Default for YamlNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a Rust value to a YAML value
impl<T> From<T> for YamlNode
where
    T: serde::Serialize,
{
    fn from(value: T) -> Self {
        match serde_yaml::to_value(value) {
            Ok(yaml) => YamlNode { value: yaml },
            Err(_) => YamlNode::new(),
        }
    }
}
