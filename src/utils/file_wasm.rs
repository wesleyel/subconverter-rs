use std::io;
use std::path::Path;

use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;
use web_sys::Storage;

/// Read a file into a string
pub fn read_file(path: &str) -> Result<String, io::Error> {
    // In WASM environment, we use localStorage to simulate file system
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(content)) = storage.get_item(path) {
                return Ok(content);
            }
        }
    }
    
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("File not found in localStorage: {}", path),
    ))
}

/// Async version of read_file that reads a file into a string asynchronously
///
/// # Arguments
/// * `path` - Path to the file to read
///
/// # Returns
/// * `Ok(String)` - The file contents
/// * `Err(io::Error)` - If the file can't be read
pub async fn read_file_async(path: &str) -> Result<String, io::Error> {
    // In WASM environment, we use localStorage which is synchronous
    // We just wrap the synchronous call in an async function
    read_file(path)
}

/// Check if a file exists
pub fn file_exists(path: &str) -> bool {
    // In WASM environment, we check if the key exists in localStorage
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(_)) = storage.get_item(path) {
                return true;
            }
        }
    }
    false
}

/// Read the contents of a file as a string
///
/// # Arguments
/// * `path` - Path to the file to read
/// * `base_path` - Optional base path for security checking
///
/// # Returns
/// * `Ok(String)` - The file contents
/// * `Err(io::Error)` - If the file can't be read
pub fn file_get<P: AsRef<Path>>(path: P, base_path: Option<&str>) -> io::Result<String> {
    if let Some(base_path) = base_path {
        match path.as_ref().to_str() {
            Some(path_str) => {
                if !path_str.starts_with(base_path) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "File path is not within the base path",
                    ));
                }
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "File path is not a valid UTF-8 string",
                ));
            }
        }
    }
    
    // Use read_file for WASM implementation
    if let Some(path_str) = path.as_ref().to_str() {
        read_file(path_str)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "File path is not a valid UTF-8 string",
        ))
    }
}

/// Copy a file from source to destination
pub fn copy_file(src: &str, dst: &str) -> io::Result<()> {
    // Check if source exists
    if !file_exists(src) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source file {} not found", src),
        ));
    }

    // Read source content
    let content = read_file(src)?;
    
    // Write to destination (in localStorage)
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Err(e) = storage.set_item(dst, &content) {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to write to localStorage: {:?}", e),
                ));
            }
            return Ok(());
        }
    }
    
    Err(io::Error::new(
        io::ErrorKind::Other,
        "Failed to access localStorage",
    ))
}

/// Async version of file_get that reads file contents asynchronously
///
/// # Arguments
/// * `path` - Path to the file to read
/// * `base_path` - Optional base path for security checking
///
/// # Returns
/// * `Ok(String)` - The file contents
/// * `Err(io::Error)` - If the file can't be read
pub async fn file_get_async<P: AsRef<Path>>(
    path: P,
    base_path: Option<&str>,
) -> io::Result<String> {
    // In WASM, we just call the synchronous version
    file_get(path, base_path)
}