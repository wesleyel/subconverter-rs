use std::env;
use std::path::Path;
use subconverter_rs::settings::{get_settings, update_settings, Settings};

fn main() {
    // Initialize default settings
    let mut settings = Settings::new();

    // Check for a config file path from command line
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 { &args[1] } else { "pref.ini" };

    // Load settings from file if it exists
    if Path::new(config_path).exists() {
        println!("Loading settings from {}", config_path);
        if let Err(e) = settings.load_from_file(config_path) {
            eprintln!("Error loading settings: {}", e);
        }
    }

    // Update global settings
    update_settings(settings);

    // Print loaded settings for verification
    let global = get_settings();
    println!("Subconverter starting with port: {}", global.listen_port);

    // TODO: Start the actual application
    println!("Subconverter initialized!");
}
