use std::io;
use std::path::Path;

use crate::settings::Settings;
use crate::utils::http::{parse_proxy, web_get_async};

// Import platform-specific implementations
#[cfg(not(target_arch = "wasm32"))]
mod platform {
    pub use crate::utils::file_std::*;
}

#[cfg(target_arch = "wasm32")]
mod platform {
    pub use crate::utils::file_wasm::*;
}

// Re-export platform-specific implementations
pub use platform::*;

// These functions are re-exported from platform-specific implementations

/// Async version of load_content
///
/// # Arguments
/// * `path` - Path to the file or URL to load
///
/// # Returns
/// * `Ok(String)` - The content
/// * `Err(String)` - Error message if loading failed
pub async fn load_content_async(path: &str) -> Result<String, String> {
    if path.starts_with("http://") || path.starts_with("https://") {
        // It's a URL, use HTTP client
        match web_get_async(path, &parse_proxy(&Settings::current().proxy_config), None).await {
            Ok((data, _)) => Ok(data),
            Err(e) => Err(format!("Failed to fetch content: {}", e)),
        }
    } else if Path::new(path).exists() {
        // It's a file, read it asynchronously
        match read_file_async(path).await {
            Ok(data) => Ok(data),
            Err(e) => Err(format!("Failed to read file: {}", e)),
        }
    } else {
        Err(format!("Path not found: {}", path))
    }
}
