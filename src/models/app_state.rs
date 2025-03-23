use std::collections::HashMap;
use std::sync::RwLock;

use crate::settings::Settings;

use crate::models::SubconverterTarget;

/// Application state structure for the web server
#[derive(Debug)]
pub struct AppState {
    /// Global application settings
    pub config: Settings,

    /// Base configuration content for different targets
    base_configs: RwLock<HashMap<SubconverterTarget, String>>,

    /// Emoji mapping for node remarks
    pub emoji_map: Option<HashMap<String, String>>,

    /// Runtime variables
    pub runtime_vars: RwLock<HashMap<String, String>>,
}

impl AppState {
    /// Create a new AppState instance
    pub fn new(config: Settings) -> Self {
        Self {
            config,
            base_configs: RwLock::new(HashMap::new()),
            emoji_map: None,
            runtime_vars: RwLock::new(HashMap::new()),
        }
    }

    /// Get base configuration for a specific target
    pub fn get_base_config(&self, target: &SubconverterTarget) -> Option<String> {
        self.base_configs.read().unwrap().get(target).cloned()
    }

    /// Set base configuration for a specific target
    pub fn set_base_config(&self, target: SubconverterTarget, content: String) {
        self.base_configs.write().unwrap().insert(target, content);
    }

    /// Load base configurations from file system
    pub fn load_base_configs(&self) {
        let mut configs = self.base_configs.write().unwrap();

        // Clear existing configs
        configs.clear();

        // Read base path from settings
        // let base_path = &self.config.base_path;

        // Load Clash base config
        if !self.config.clash_base.is_empty() {
            if let Ok(content) = std::fs::read_to_string(&self.config.clash_base) {
                configs.insert(SubconverterTarget::Clash, content.clone());
                configs.insert(SubconverterTarget::ClashR, content);
            }
        }

        // Load Surge base configs
        if !self.config.surge_base.is_empty() {
            if let Ok(content) = std::fs::read_to_string(&self.config.surge_base) {
                configs.insert(SubconverterTarget::Surge(3), content.clone());
                configs.insert(SubconverterTarget::Surge(4), content);
            }
        }

        // Load other base configs
        if !self.config.surfboard_base.is_empty() {
            if let Ok(content) = std::fs::read_to_string(&self.config.surfboard_base) {
                configs.insert(SubconverterTarget::Surfboard, content);
            }
        }

        if !self.config.mellow_base.is_empty() {
            if let Ok(content) = std::fs::read_to_string(&self.config.mellow_base) {
                configs.insert(SubconverterTarget::Mellow, content);
            }
        }

        if !self.config.quan_base.is_empty() {
            if let Ok(content) = std::fs::read_to_string(&self.config.quan_base) {
                configs.insert(SubconverterTarget::Quantumult, content);
            }
        }

        if !self.config.quanx_base.is_empty() {
            if let Ok(content) = std::fs::read_to_string(&self.config.quanx_base) {
                configs.insert(SubconverterTarget::QuantumultX, content);
            }
        }

        if !self.config.loon_base.is_empty() {
            if let Ok(content) = std::fs::read_to_string(&self.config.loon_base) {
                configs.insert(SubconverterTarget::Loon, content);
            }
        }

        if !self.config.ssub_base.is_empty() {
            if let Ok(content) = std::fs::read_to_string(&self.config.ssub_base) {
                configs.insert(SubconverterTarget::SSSub, content);
            }
        }

        if !self.config.singbox_base.is_empty() {
            if let Ok(content) = std::fs::read_to_string(&self.config.singbox_base) {
                configs.insert(SubconverterTarget::SingBox, content);
            }
        }
    }

    /// Load emoji mapping from file
    pub fn load_emoji_map(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let mut emoji_map = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let keyword = parts[0].trim();
                let emoji = parts[1].trim();

                if !keyword.is_empty() && !emoji.is_empty() {
                    emoji_map.insert(keyword.to_string(), emoji.to_string());
                }
            }
        }

        if !emoji_map.is_empty() {
            self.emoji_map = Some(emoji_map);
        }

        Ok(())
    }
}
