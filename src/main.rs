use std::env;
use std::path::Path;
use subconverter_rs::settings::{update_settings, Settings};

fn main() {
    // Check for a config file path from command line
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 { &args[1] } else { "pref.ini" };

    // Load settings from file if it exists
    if Path::new(config_path).exists() {
        println!("Loading settings from {}", config_path);
        match Settings::load_from_file(config_path) {
            Ok(settings) => {
                // Print loaded settings for verification
                println!("Subconverter starting with port: {}", settings.listen_port);
                update_settings(settings);
            }
            Err(e) => {
                eprintln!("Error loading settings: {}", e);
            }
        }
    }

    // TODO: Start the actual application
    println!("Subconverter initialized!");
}
