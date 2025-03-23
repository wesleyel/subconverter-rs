// use subconverter_rs::models::ruleset::RulesetContent;
// use subconverter_rs::settings::config::{update_settings, Settings};
use subconverter_rs::settings::external::load_external_config;
// use subconverter_rs::settings::ruleset::{refresh_rulesets, RulesetConfig};
use subconverter_rs::settings::unified::{get_instance, init_settings};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the subconverter settings from the provided config file
    let config_path = "config/example_external_config.ini";
    println!("Loading settings from {}", config_path);

    // Initialize settings system
    init_settings(config_path)?;

    // Get the unified settings instance and current ruleset content
    let unified = get_instance();
    let ruleset_content = unified.get_ruleset_content();

    // Print loaded rulesets
    println!(
        "Loaded {} rulesets from global configuration",
        ruleset_content.len()
    );
    for (i, ruleset) in ruleset_content.iter().enumerate() {
        println!(
            "  Ruleset {}: Group: {}, Path: {}",
            i + 1,
            ruleset.group,
            ruleset.rule_path
        );
    }

    // Load an external config file
    let ext_config_path = "config/example_external_config.ini";
    println!("\nLoading external config from {}", ext_config_path);
    let external_config = load_external_config(ext_config_path)?;

    // Print external rulesets
    println!(
        "External config has {} rulesets",
        external_config.surge_ruleset.len()
    );
    for (i, ruleset) in external_config.surge_ruleset.iter().enumerate() {
        println!(
            "  Ruleset {}: Group: {}, URL: {}",
            i + 1,
            ruleset.group,
            ruleset.url
        );
    }

    // Create merged settings
    let merged = unified.create_merged(Some(external_config));

    // Print merged rulesets
    println!(
        "\nMerged configuration has {} rulesets",
        merged.ruleset_content.len()
    );
    for (i, ruleset) in merged.ruleset_content.iter().enumerate() {
        println!(
            "  Ruleset {}: Group: {}, Path: {}",
            i + 1,
            ruleset.group,
            ruleset.rule_path
        );
    }

    Ok(())
}
