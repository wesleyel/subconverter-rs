use crate::vfs::vercel_kv_types::*;
use crate::vfs::VfsError;
use log::debug;
use wasm_bindgen::JsValue;

// Define the log_debug macro with public visibility
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*)
    };
}
// Normalize path: ensure consistency, e.g., remove leading '/'
pub fn normalize_path(path: &str) -> String {
    path.trim_start_matches('/').to_string()
}

// Get content key from path
pub fn get_content_key(path: &str) -> String {
    format!("{}{}", path, FILE_CONTENT_SUFFIX)
}

// Get metadata key from path
pub fn get_metadata_key(path: &str) -> String {
    format!("{}{}", path, FILE_METADATA_SUFFIX)
}

// Get status key from path
pub fn get_status_key(path: &str) -> String {
    format!("{}{}", path, FILE_STATUS_SUFFIX)
}

// Get directory marker key for a path
pub fn get_directory_marker_key(path: &str) -> String {
    if path.is_empty() || path.ends_with('/') {
        format!(
            "{}{}",
            path,
            DIRECTORY_MARKER_SUFFIX.trim_start_matches('/')
        )
    } else {
        format!("{}{}", path, DIRECTORY_MARKER_SUFFIX)
    }
}

// Check if path is a directory path (ends with /)
pub fn is_directory_path(path: &str) -> bool {
    path.ends_with('/') || path.is_empty()
}

// Extract parent directory path
pub fn get_parent_directory(path: &str) -> String {
    let path = path.trim_end_matches('/');
    match path.rfind('/') {
        Some(idx) => path[..=idx].to_string(),
        None => "".to_string(),
    }
}

// Extract filename from a path
pub fn get_filename(path: &str) -> String {
    let path = path.trim_end_matches('/');
    match path.rfind('/') {
        Some(idx) => path[idx + 1..].to_string(),
        None => path.to_string(),
    }
}

// Helper to guess file type from path (extension)
pub fn guess_file_type(path: &str) -> String {
    if let Some(ext) = path.split('.').last() {
        match ext.to_lowercase().as_str() {
            "txt" => "text/plain".to_string(),
            "html" | "htm" => "text/html".to_string(),
            "css" => "text/css".to_string(),
            "js" => "application/javascript".to_string(),
            "json" => "application/json".to_string(),
            "xml" => "application/xml".to_string(),
            "png" => "image/png".to_string(),
            "jpg" | "jpeg" => "image/jpeg".to_string(),
            "gif" => "image/gif".to_string(),
            "svg" => "image/svg+xml".to_string(),
            "pdf" => "application/pdf".to_string(),
            "md" => "text/markdown".to_string(),
            "ini" => "text/plain".to_string(),
            "yaml" | "yml" => "application/yaml".to_string(),
            "conf" => "text/plain".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    } else {
        "application/octet-stream".to_string()
    }
}
