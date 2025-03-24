//! Settings module for subconverter
//!
//! This module contains all the configuration settings and utilities

pub mod deserializer;
pub mod external;
pub mod import;
pub mod ruleset;
pub mod settings;
pub mod utils;

// Re-export settings struct and functions
pub use external::{load_external_config, ExternalSettings};
pub use import::*;
pub use ruleset::*;
pub use settings::settings_struct::{refresh_configuration, update_settings_from_file, Settings};
