


pub mod conversions;
pub mod external_struct;
pub mod ini_external;
pub mod toml_external;
pub mod yaml_external;

pub use external_struct::ExternalSettings;

/// Load external configuration from file or URL
pub fn load_external_config(path: &str) -> Result<ExternalSettings, Box<dyn std::error::Error>> {
    // TODO: Add any global settings checks here before loading external config
    // In C++, there are checks for proxy settings and other global configuration
    // that might be needed before loading the external config

    ExternalSettings::load_from_file_sync(path)
}

// TODO: Implement validation function for rulesets
// Check for maxAllowedRulesets and other constraints
// In C++: if(global.maxAllowedRulesets && vArray.size() > global.maxAllowedRulesets) {...}
