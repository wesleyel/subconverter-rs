use log::{error, info};

use crate::settings::config::global;
use crate::utils::http::web_get;
use crate::utils::{file_exists, file_get, is_link};

/// Import items from a file or URL into a target vector
///
/// This function imports configuration items from either a local file or remote URL
/// and adds them to the provided target vector.
///
/// # Arguments
/// * `target` - Vector to add the imported items to
/// * `scope_limit` - Whether to limit the scope of imports for security
///
/// # Returns
/// * `i32` - Number of items imported, or -1 on failure
pub fn import_items(target: &mut Vec<String>, _scope_limit: bool) -> i32 {
    let mut result: Vec<String> = Vec::new();
    let mut count = 0;

    for url in target.iter() {
        // Skip comment lines
        if url.starts_with("//") || url.starts_with('#') {
            continue;
        }

        let mut _content = String::new();
        if file_exists(url) {
            match file_get(url) {
                Ok(data) => _content = data,
                Err(e) => {
                    error!("Error reading file '{}': {}", url, e);
                    continue;
                }
            }
        } else if is_link(url) {
            // Parse proxy from config
            let settings = global.read().unwrap();
            let proxy = if settings.proxy_config.is_empty() {
                None
            } else {
                Some(settings.proxy_config.clone())
            };
            drop(settings);

            // Fetch from URL
            match web_get(url, proxy.as_deref(), None) {
                Ok((data, _)) => _content = data,
                Err(e) => {
                    error!("Error fetching URL '{}': {}", url, e);
                    continue;
                }
            }
        } else {
            // Not a file or URL, add directly to result
            result.push(url.clone());
            count += 1;
            continue;
        }

        // Process content line by line
        for line in _content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with('#') {
                result.push(trimmed.to_string());
                count += 1;
            }
        }
    }

    // Replace target with result
    *target = result;

    info!("Imported {} item(s)", count);
    count as i32
}
