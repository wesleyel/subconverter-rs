use crate::Settings;
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn initialize_settings_from_content(content: &str) -> Result<(), JsValue> {
    let content = content.to_string();
    match Settings::load_from_content(&content, "").await {
        Ok(settings) => {
            *Settings::current_mut() = Arc::new(settings);
            Ok(())
        }
        Err(err) => {
            web_sys::console::error_1(&format!("Failed to initialize settings: {}", err).into());
            Err(JsValue::from_str(&format!(
                "Failed to initialize settings: {}",
                err
            )))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn initialize_settings_from_content(
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::load_from_content(content, "").await?;
    *Settings::current_mut() = Arc::new(settings);
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init_wasm_logging(level: Option<String>) -> Result<(), JsValue> {
    use log::Level;
    use log::LevelFilter;
    use std::sync::atomic::{AtomicBool, Ordering};

    // Static flag to track if logger has been initialized
    static LOGGER_INITIALIZED: AtomicBool = AtomicBool::new(false);

    // Set up the panic hook for better stack traces
    crate::utils::set_panic_hook();

    // Check if logger is already initialized
    if LOGGER_INITIALIZED.load(Ordering::SeqCst) {
        log::debug!("Logger already initialized, skipping initialization");
        return Ok(());
    }

    let log_level = match level.as_deref() {
        Some("error") => Level::Error,
        Some("warn") => Level::Warn,
        Some("info") => Level::Info,
        Some("debug") => Level::Debug,
        Some("trace") => Level::Trace,
        _ => Level::Info, // Default to Info level
    };

    match console_log::init_with_level(log_level) {
        Ok(_) => {
            // Mark logger as initialized
            LOGGER_INITIALIZED.store(true, Ordering::SeqCst);
            log::info!(
                "WASM logging initialized at level: {} with stack trace support",
                log_level
            );
            Ok(())
        }
        Err(e) => {
            // If error is about logger already being set, consider it a success
            if e.to_string().contains("already initialized") {
                LOGGER_INITIALIZED.store(true, Ordering::SeqCst);
                log::debug!("Logger was already initialized by another part of the application");
                Ok(())
            } else {
                Err(JsValue::from_str(&format!(
                    "Failed to initialize logger: {}",
                    e
                )))
            }
        }
    }
}
