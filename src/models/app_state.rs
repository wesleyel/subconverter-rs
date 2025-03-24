use std::collections::HashMap;
use std::sync::RwLock;

use crate::settings::Settings;

use crate::models::SubconverterTarget;
use crate::utils::file_get;

/// Application state structure for the web server
#[derive(Debug)]
pub struct AppState {
    /// Base configuration content for different targets
    base_configs: RwLock<HashMap<SubconverterTarget, String>>,

    /// Emoji mapping for node remarks
    pub emoji_map: Option<HashMap<String, String>>,

    /// Runtime variables
    pub runtime_vars: RwLock<HashMap<String, String>>,
}

impl AppState {
    /// Create a new AppState instance
    pub fn new() -> Self {
        Self {
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

        let global = Settings::current();
        // Read base path from settings
        let base_path = global.base_path.clone();

        // Load Clash base config
        if !global.clash_base.is_empty() {
            if let Ok(content) = file_get(&global.clash_base, Some(&base_path)) {
                configs.insert(SubconverterTarget::Clash, content.clone());
                configs.insert(SubconverterTarget::ClashR, content);
            }
        }

        // Load Surge base configs
        if !global.surge_base.is_empty() {
            if let Ok(content) = file_get(&global.surge_base, Some(&base_path)) {
                configs.insert(SubconverterTarget::Surge(3), content.clone());
                configs.insert(SubconverterTarget::Surge(4), content);
            }
        }

        // Load other base configs
        if !global.surfboard_base.is_empty() {
            if let Ok(content) = file_get(&global.surfboard_base, Some(&base_path)) {
                configs.insert(SubconverterTarget::Surfboard, content);
            }
        }

        if !global.mellow_base.is_empty() {
            if let Ok(content) = file_get(&global.mellow_base, Some(&base_path)) {
                configs.insert(SubconverterTarget::Mellow, content);
            }
        }

        if !global.quan_base.is_empty() {
            if let Ok(content) = file_get(&global.quan_base, Some(&base_path)) {
                configs.insert(SubconverterTarget::Quantumult, content);
            }
        }

        if !global.quanx_base.is_empty() {
            if let Ok(content) = file_get(&global.quanx_base, Some(&base_path)) {
                configs.insert(SubconverterTarget::QuantumultX, content);
            }
        }

        if !global.loon_base.is_empty() {
            if let Ok(content) = file_get(&global.loon_base, Some(&base_path)) {
                configs.insert(SubconverterTarget::Loon, content);
            }
        }

        if !global.ssub_base.is_empty() {
            if let Ok(content) = file_get(&global.ssub_base, Some(&base_path)) {
                configs.insert(SubconverterTarget::SSSub, content);
            }
        }

        if !global.singbox_base.is_empty() {
            if let Ok(content) = file_get(&global.singbox_base, Some(&base_path)) {
                configs.insert(SubconverterTarget::SingBox, content);
            }
        }
    }

    /// Load emoji mapping from file
    pub fn load_emoji_map(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = file_get(path, None)?;
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
