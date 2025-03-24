// use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

use log::error;
use log::info;
use subconverter_rs::interfaces::subconverter::{subconverter, SubconverterConfigBuilder};
use subconverter_rs::models::ProxyGroupConfigs;
use subconverter_rs::settings::update_settings_from_file;
use subconverter_rs::Settings;
use subconverter_rs::SubconverterTarget;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let config_path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("examples/subconverter/config.ini");
    let url = args
        .get(2)
        .map(|s| s.as_str())
        .unwrap_or("https://xn--mesz9ptugxg.com/api/v1/client/subscribe?token=dcb2eb6379a152bbe397572220e8e4a8&flag=meta&types=all");
    let target_format = args.get(3).map(|s| s.as_str()).unwrap_or("clash");

    println!("Subconverter Example");
    println!("-------------------");
    println!("Config path: {}", config_path);
    println!("Subscription URL: {}", url);
    println!("Target format: {}", target_format);

    // Load settings from file if it exists
    if Path::new(config_path).exists() {
        info!("Loading settings from {}", config_path);
        update_settings_from_file(config_path).unwrap_or_else(|e| {
            error!("Failed to update settings from {}: {}", config_path, e);
        });
    }

    let global = Settings::current();
    // Parse proxy groups
    let proxy_groups = ProxyGroupConfigs::new();
    // Proxy groups would be handled separately in a real setup
    // This is a placeholder for how you might create proxy groups

    // Determine target format
    let target = SubconverterTarget::from_str(target_format).unwrap_or(SubconverterTarget::Clash);

    // Read base content for the target format
    let base_path = Path::new("examples/subconverter");
    let base_content = match target {
        SubconverterTarget::Clash => {
            let clash_base_path = base_path.join("clash_base.yml");
            // target.get_base_content_from_external(&settings.external);
            fs::read_to_string(clash_base_path).unwrap_or_default()
        }
        _ => String::new(),
    };
    let ruleset_content = global.rulesets_content.clone();

    // Create config builder
    let builder = SubconverterConfigBuilder::new()
        .target(target.clone())
        .add_url(url)
        // .emoji_patterns(global_settings.)
        .ruleset_content(ruleset_content)
        .proxy_groups(proxy_groups);
    // .extra(extra_settings);

    // Add base content if available
    let builder = if !base_content.is_empty() {
        builder.add_base_content(target.clone(), base_content)
    } else {
        builder
    };

    // Build the configuration
    match builder.build() {
        Ok(config) => {
            // Call subconverter
            match subconverter(config) {
                Ok(result) => {
                    println!("\nConversion successful!");

                    // Output some headers
                    if !result.headers.is_empty() {
                        println!("\nResponse headers:");
                        for (key, value) in &result.headers {
                            println!("  {}: {}", key, value);
                        }
                    }

                    // Output some of the content (first 200 chars)
                    let content_preview = if result.content.len() > 200 {
                        format!("{}...", &result.content[0..200])
                    } else {
                        result.content.clone()
                    };

                    println!("\nContent preview:\n{}", content_preview);

                    // Write output to file
                    let output_path = format!(
                        "examples/subconverter/output.{}",
                        match target {
                            SubconverterTarget::Clash => "yml",
                            SubconverterTarget::Surge(_) => "conf",
                            _ => "txt",
                        }
                    );

                    if let Err(e) = fs::write(&output_path, result.content) {
                        println!("Error writing output file: {}", e);
                    } else {
                        println!("\nFull output written to {}", output_path);
                    }
                }
                Err(e) => println!("Conversion error: {}", e),
            }
        }
        Err(e) => println!("Config build error: {}", e),
    }
}
