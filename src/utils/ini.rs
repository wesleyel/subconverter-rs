use std::collections::HashMap;

/// Simple INI reader implementation
#[derive(Debug, Clone)]
pub struct IniReader {
    sections: HashMap<String, HashMap<String, String>>,
    anonymous_section: HashMap<String, String>,
}

impl IniReader {
    /// Create a new empty INI reader
    pub fn new() -> Self {
        IniReader {
            sections: HashMap::new(),
            anonymous_section: HashMap::new(),
        }
    }

    /// Parse an INI string
    pub fn parse(&mut self, content: &str) -> bool {
        // Placeholder for implementation
        true
    }

    /// Get a value from a section
    pub fn get_value(&self, section: &str, key: &str) -> Option<&String> {
        if section.is_empty() {
            self.anonymous_section.get(key)
        } else if let Some(sect) = self.sections.get(section) {
            sect.get(key)
        } else {
            None
        }
    }

    /// Set a value in a section
    pub fn set_value(&mut self, section: &str, key: &str, value: &str) {
        if section.is_empty() {
            self.anonymous_section
                .insert(key.to_string(), value.to_string());
        } else {
            let sect = self.sections.entry(section.to_string()).or_default();
            sect.insert(key.to_string(), value.to_string());
        }
    }

    /// Get all sections
    pub fn get_sections(&self) -> Vec<&String> {
        self.sections.keys().collect()
    }

    /// Get all keys in a section
    pub fn get_keys(&self, section: &str) -> Vec<&String> {
        if section.is_empty() {
            self.anonymous_section.keys().collect()
        } else if let Some(sect) = self.sections.get(section) {
            sect.keys().collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for IniReader {
    fn default() -> Self {
        Self::new()
    }
}
