//! Settings module for subconverter
//!
//! This module contains all the configuration settings and utilities

pub mod config;
pub mod external;
pub mod import;
pub mod ruleset;
pub mod unified;

// Re-export settings struct and functions
pub use config::{get_settings, refresh_configuration, update_settings, Settings};
pub use external::{load_external_config, ExternalConfig};
pub use import::*;
pub use ruleset::*;
pub use unified::{MergedSettings, UnifiedSettings};
