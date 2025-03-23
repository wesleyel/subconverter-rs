use std::env;
use std::path::Path;
use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use env_logger::Env;
use log::{error, info};

use subconverter_rs::models::AppState;
use subconverter_rs::settings::unified::{get_instance, init_settings};
use subconverter_rs::web_handlers::interfaces;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // Check for a config file path from command line
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 { &args[1] } else { "pref.ini" };

    // Load settings from file if it exists
    if Path::new(config_path).exists() {
        info!("Loading settings from {}", config_path);
        if let Err(e) = init_settings(config_path) {
            error!("Error loading settings: {}", e);
        }
    }

    // Get the current settings
    let settings = get_instance().get_global();

    // Ensure we have a valid listen address
    let listen_address = if settings.listen_address.trim().is_empty() {
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
    };

    info!("Subconverter starting on {}", listen_address);

    // Create app state with settings
    let app_state = Arc::new(AppState::new(settings.clone()));

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
    .workers(settings.max_concur_threads as usize)
    .run()
    .await
}
