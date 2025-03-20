pub mod constants;
pub mod generator;
pub mod interfaces;
pub mod models;
pub mod parser;
pub mod settings;
pub mod utils;
pub mod web_handlers;

// Re-export the main proxy types for easier access
pub use models::{Proxy, ProxyType};

// Re-export configuration types
pub use parser::types::ConfType;

// Re-export settings
pub use settings::{get_settings, global, import_items, update_settings, ExternalConfig, Settings};

// Re-export ruleset types
pub use models::ruleset::RulesetType;

// Re-export interfaces
pub use interfaces::*;
