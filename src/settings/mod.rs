//! Settings module for subconverter
//!
//! This module contains all the configuration settings and utilities

pub mod config;
pub mod external;
pub mod import;

// Re-export settings struct and functions
pub use config::{get_settings, global, refresh_configuration, update_settings, Settings};
pub use external::ExternalConfig;
pub use import::import_items;
