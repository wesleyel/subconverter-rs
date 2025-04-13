use crate::vfs::vercel_kv_types::*;

//------------------------------------------------------------------------------
// LOGGING
//------------------------------------------------------------------------------

/// Provides consistent debug logging across the VFS module
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*)
    };
}

//------------------------------------------------------------------------------
// PATH MANIPULATION
//------------------------------------------------------------------------------

/// Normalizes a path by removing leading slashes for consistent key generation
///
/// # Arguments
/// * `path` - The path to normalize
///
/// # Returns
/// A normalized path string without leading slashes
pub fn normalize_path(path: &str) -> String {
    let normalized = path.trim_start_matches('/').to_string();
    // log_debug!("Normalized path: '{}' → '{}'", path, normalized);
    normalized
}

/// Checks if a path represents a directory (ends with '/' or is empty)
///
/// # Arguments
/// * `path` - The path to check
///
/// # Returns
/// `true` if the path is a directory path, `false` otherwise
pub fn is_directory_path(path: &str) -> bool {
    let is_dir = path.ends_with('/') || path.is_empty();
    log_debug!("Checking if path '{}' is directory: {}", path, is_dir);
    is_dir
}

/// Extracts the parent directory path from a given path
///
/// # Arguments
/// * `path` - The path to get the parent directory from
///
/// # Returns
/// The parent directory path, ending with '/'. Returns empty string for top-level items.
pub fn get_parent_directory(path: &str) -> String {
    let path = path.trim_end_matches('/');
    let parent = match path.rfind('/') {
        Some(idx) => path[..=idx].to_string(),
        None => "".to_string(),
    };
    log_debug!("Parent directory of '{}': '{}'", path, parent);
    parent
}

/// Extracts the filename from a path
///
/// # Arguments
/// * `path` - The path to extract the filename from
///
/// # Returns
/// The filename (last path component)
pub fn get_filename(path: &str) -> String {
    let path = path.trim_end_matches('/');
    let filename = match path.rfind('/') {
        Some(idx) => path[idx + 1..].to_string(),
        None => path.to_string(),
    };
    log_debug!("Filename from path '{}': '{}'", path, filename);
    filename
}

//------------------------------------------------------------------------------
// KV KEY GENERATION
//------------------------------------------------------------------------------

/// Generates a complete key for KV storage by appending a suffix to a path
///
/// # Arguments
/// * `path` - The normalized path
/// * `suffix` - The suffix to append
///
/// # Returns
/// A key string for KV storage
pub fn get_key_with_suffix(path: &str, suffix: &str) -> String {
    let key = format!("{}{}", path, suffix);
    log_debug!(
        "Generated key with suffix '{}' for path '{}': '{}'",
        suffix,
        path,
        key
    );
    key
}

/// Generates the content key for a file path
///
/// # Arguments
/// * `path` - The normalized path
///
/// # Returns
/// A content key string for KV storage
pub fn get_content_key(path: &str) -> String {
    get_key_with_suffix(path, FILE_CONTENT_SUFFIX)
}

/// Generates the metadata key for a file path
///
/// # Arguments
/// * `path` - The normalized path
///
/// # Returns
/// A metadata key string for KV storage
pub fn get_metadata_key(path: &str) -> String {
    get_key_with_suffix(path, FILE_METADATA_SUFFIX)
}

/// Generates the status key for a file path
///
/// # Arguments
/// * `path` - The normalized path
///
/// # Returns
/// A status key string for KV storage
pub fn get_status_key(path: &str) -> String {
    get_key_with_suffix(path, FILE_STATUS_SUFFIX)
}

/// Generates the directory marker key for a directory path
///
/// # Arguments
/// * `path` - The normalized path
///
/// # Returns
/// A directory marker key string for KV storage
pub fn get_directory_marker_key(path: &str) -> String {
    // Fix: Properly handle directory marker suffix for empty or directory paths
    if path.is_empty() || path.ends_with('/') {
        // For directories, use the suffix without the leading slash
        get_key_with_suffix(path, DIRECTORY_MARKER_SUFFIX.trim_start_matches('/'))
    } else {
        // For regular files, use the full suffix
        get_key_with_suffix(path, DIRECTORY_MARKER_SUFFIX)
    }
}

//------------------------------------------------------------------------------
// FILE TYPE HANDLING
//------------------------------------------------------------------------------

/// Guesses the MIME type of a file based on its extension
///
/// # Arguments
/// * `path` - The file path
///
/// # Returns
/// A MIME type string
pub fn guess_file_type(path: &str) -> String {
    // Extract the file extension and convert to lowercase
    let extension = path
        .rsplit('.')
        .next()
        .filter(|&ext| !ext.contains('/'))
        .map(|ext| ext.to_lowercase());

    // Map the extension to a MIME type
    let file_type = match extension {
        Some(ext) => match ext.as_str() {
            "txt" => "text/plain",
            "html" | "htm" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "json" => "application/json",
            "xml" => "application/xml",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "pdf" => "application/pdf",
            "md" => "text/markdown",
            "ini" => "text/plain",
            "yaml" | "yml" => "application/yaml",
            "conf" => "text/plain",
            "rs" => "text/rust",          // Added Rust files
            "toml" => "application/toml", // Added TOML files
            "wasm" => "application/wasm", // Added WebAssembly files
            _ => "application/octet-stream",
        }
        .to_string(),
        None => "application/octet-stream".to_string(),
    };

    // log_debug!("Guessed file type for '{}': '{}'", path, file_type);
    file_type
}
