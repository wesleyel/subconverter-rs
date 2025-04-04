use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use clap::{FromArgMatches as _, Parser};
use env_logger::Env;
use log::{error, info};

use subconverter::models::AppState;
use subconverter::settings::settings::settings_struct::init_settings;
use subconverter::web_handlers::interfaces;
use subconverter::Settings;

/// A more powerful utility to convert between proxy subscription format
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    /// Listen address (e.g., 127.0.0.1 or 0.0.0.0)
    #[arg(short, long, value_name = "ADDRESS")]
    address: Option<String>,

    /// Listen port
    #[arg(short, long, value_name = "PORT")]
    port: Option<u32>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // Parse command line arguments
    let args = Args::parse();

    // Initialize settings with config file path if provided
    init_settings(args.config.as_deref().unwrap_or("")).unwrap();

    // Ensure we have a valid listen address
    let listen_address = {
        // Get a mutable reference to the current settings
        let mut settings_guard = Settings::current_mut();
        let settings = Arc::make_mut(&mut *settings_guard);

        // Override settings with command line arguments if provided
        if let Some(address) = args.address {
            settings.listen_address = address;
        }
        if let Some(port) = args.port {
            settings.listen_port = port;
        }
        if settings.listen_address.trim().is_empty() {
            error!("Empty listen_address in settings, defaulting to 127.0.0.1");
            format!("127.0.0.1:{}", settings.listen_port)
        } else {
            // Check if the address contains a port
            if settings.listen_address.contains(':') {
                // Already has a port, use as is
                settings.listen_address.clone()
            } else {
                // No port specified, use the one from settings
                format!("{}:{}", settings.listen_address, settings.listen_port)
            }
        }
    };

    let max_concur_threads = Settings::current().max_concur_threads;

    info!("Subconverter starting on {}", listen_address);

    // Create app state with settings
    let app_state = Arc::new(AppState::new());

    // Load base configurations
    app_state.load_base_configs();

    // Start web server
    HttpServer::new(move || {
        App::new()
            // Add app state
            .app_data(web::Data::new(Arc::clone(&app_state)))
            // Register web handlers
            .configure(interfaces::config)
            // For health check
            .route("/", web::get().to(|| async { "Subconverter is running!" }))
    })
    .bind(listen_address)?
    .workers(max_concur_threads as usize)
    .run()
    .await
}
