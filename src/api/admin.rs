use crate::utils::file_wasm;
use crate::vfs::vercel_kv_helpers::get_directory_marker_key;
use crate::vfs::vercel_kv_js_bindings::*;
use crate::vfs::vercel_kv_vfs::VercelKvVfs;
use crate::vfs::VfsError;
use log::{debug, error, info};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::vfs::vercel_kv_types::{VirtualFileSystem, DirectoryEntry, FileAttributes};

use wasm_bindgen::prelude::*;
use web_sys::console;

// Helper to create VFS instance (simplistic for now)
async fn get_vfs() -> Result<VercelKvVfs, VfsError> {
    file_wasm::get_vfs()
        .await
        .map_err(|e| VfsError::Other(format!("Failed to get VFS: {}", e)))
}

// Helper to convert VfsError to JsValue for FFI boundary with error type information
fn vfs_error_to_js(err: VfsError) -> JsValue {
    let error_type = match &err {
        VfsError::NotFound(_) => "NotFound",
        VfsError::ConfigError(_) => "ConfigError",
        VfsError::StorageError(_) => "StorageError",
        VfsError::NetworkError(_) => "NetworkError",
        VfsError::IoError(_) => "IoError",
        VfsError::IsDirectory(_) => "IsDirectory",
        VfsError::NotDirectory(_) => "NotDirectory",
        VfsError::Other(_) => "Other",
    };

    let error_obj = json!({
        "type": error_type,
        "message": format!("{}", err)
    });

    // Convert to string first since we don't have serde support
    let error_json = error_obj.to_string();
    JsValue::from_str(&error_json)
}

#[wasm_bindgen]
pub async fn admin_read_file(path: String) -> Result<JsValue, JsValue> {
    console::log_1(&format!("admin_read_file called with path: {}", path).into());
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;
    match vfs.read_file(&path).await {
        Ok(content) => {
            // 将字节转换为 UTF-8 字符串直接返回，不再使用 base64
            match String::from_utf8(content) {
                Ok(text_content) => Ok(JsValue::from_str(&text_content)),
                Err(e) => Err(JsValue::from_str(&format!("UTF-8 conversion error: {}", e))),
            }
        }
        Err(e) => Err(vfs_error_to_js(e)),
    }
}

#[wasm_bindgen]
pub async fn admin_write_file(path: String, text_content: String) -> Result<(), JsValue> {
    console::log_1(&format!("admin_write_file called with path: {}", path).into());
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;
    
    // 直接将字符串转换为字节而不是解码 base64
    let content = text_content.into_bytes();
    vfs.write_file(&path, content).await.map_err(vfs_error_to_js)
}

#[wasm_bindgen]
pub async fn admin_delete_file(path: String) -> Result<(), JsValue> {
    console::log_1(&format!("admin_delete_file called with path: {}", path).into());
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;
    vfs.delete_file(&path).await.map_err(vfs_error_to_js)
}

#[wasm_bindgen]
pub async fn admin_file_exists(path: String) -> Result<bool, JsValue> {
    console::log_1(&format!("admin_file_exists called with path: {}", path).into());
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;
    vfs.exists(&path).await.map_err(vfs_error_to_js)
}

/// Get file attributes - admin endpoint
#[wasm_bindgen]
pub async fn admin_get_file_attributes(path: String) -> Result<FileAttributes, JsValue> {
    info!("admin_get_file_attributes called for {}", path);
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;

    // Read file attributes from the VFS
    match vfs.read_file_attributes(&path).await {
        Ok(attributes) => {
            Ok(attributes)
        }
        Err(e) => {
            error!("Error getting file attributes: {}", e);
            Err(vfs_error_to_js(e))
        }
    }
}

/// Create directory - admin endpoint
#[wasm_bindgen]
pub async fn admin_create_directory(path: String) -> Result<(), JsValue> {
    info!("admin_create_directory called for {}", path);
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;

    // Create directory in the VFS
    match vfs.create_directory(&path).await {
        Ok(_) => {
            info!("Directory created successfully: {}", path);
            Ok(())
        }
        Err(e) => {
            error!("Error creating directory: {}", e);
            Err(vfs_error_to_js(e))
        }
    }
}

/// List directory contents - admin endpoint
#[wasm_bindgen]
pub async fn list_directory(path: String) -> Result<Vec<DirectoryEntry>, JsValue> {
    info!("list_directory called for path: {}", path);
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;

    match vfs.list_directory(&path).await {
        Ok(entries) => {
            Ok(entries)
        }
        Err(e) => {
            error!("Error listing directory: {}", e);
            Err(vfs_error_to_js(e))
        }
    }
}

/// Load all files from a GitHub repository directory at once.
/// If shallow=true, only creates placeholder entries without downloading content.
#[wasm_bindgen]
pub async fn admin_load_github_directory(path: String, shallow: bool) -> Result<JsValue, JsValue> {
    info!("admin_load_github_directory called for path: {} (shallow: {})", path, shallow);
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;

    match vfs.load_github_directory(&path, shallow).await {
        Ok(result) => {
            // Convert the result to JavaScript
            match serde_wasm_bindgen::to_value(&result) {
                Ok(js_result) => {
                    info!(
                        "GitHub directory load complete: {} files loaded, {} failed, {} placeholders",
                        result.successful_files, 
                        result.failed_files,
                        result.loaded_files.iter().filter(|f| f.is_placeholder).count()
                    );
                    Ok(js_result)
                }
                Err(e) => Err(JsValue::from_str(&format!(
                    "Error serializing load result: {}",
                    e
                ))),
            }
        }
        Err(e) => {
            error!("Error loading directory from GitHub: {}", e);
            Err(vfs_error_to_js(e))
        }
    }
}

/// Load only direct children of a GitHub repository directory (non-recursive).
/// If shallow=true, only creates placeholder entries without downloading content.
#[wasm_bindgen]
pub async fn admin_load_github_directory_flat(path: String, shallow: bool) -> Result<JsValue, JsValue> {
    info!("admin_load_github_directory_flat called for path: {} (shallow: {})", path, shallow);
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;

    match vfs.load_github_directory_impl(shallow, false).await {
        Ok(result) => {
            // Convert the result to JavaScript
            match serde_wasm_bindgen::to_value(&result) {
                Ok(js_result) => {
                    info!(
                        "GitHub directory flat load complete: {} files loaded, {} failed, {} placeholders",
                        result.successful_files, 
                        result.failed_files,
                        result.loaded_files.iter().filter(|f| f.is_placeholder).count()
                    );
                    Ok(js_result)
                }
                Err(e) => Err(JsValue::from_str(&format!(
                    "Error serializing load result: {}",
                    e
                ))),
            }
        }
        Err(e) => {
            error!("Error loading directory from GitHub: {}", e);
            Err(vfs_error_to_js(e))
        }
    }
}

/// Debug function to provide detailed info about list_directory operation
#[wasm_bindgen]
pub async fn debug_list_directory(path: String, shallow: bool) -> Result<JsValue, JsValue> {
    info!("debug_list_directory called for path: '{}' (shallow: {})", path, shallow);

    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;
    let result = vfs.list_directory(&path).await;

    // Collect diagnostic information
    let mut debug_info = HashMap::new();
    debug_info.insert("path".to_string(), json!(path));

    // Check if directory exists first
    let exists = if path.is_empty() {
        // Root directory always exists
        true
    } else {
        vfs.exists(&path).await.unwrap_or(false)
    };
    debug_info.insert("directory_exists".to_string(), json!(exists));

    // Get prefix for KV operations
    let prefix = if path.ends_with('/') || path.is_empty() {
        path.clone()
    } else {
        format!("{}/", path)
    };
    debug_info.insert("prefix_for_kv".to_string(), json!(prefix));

    // Check if there's a directory marker
    let marker_key = get_directory_marker_key(&path);
    let has_marker = match kv_exists(&marker_key).await {
        Ok(js_value) => js_value.as_bool().unwrap_or(false),
        Err(_) => false,
    };
    debug_info.insert("has_directory_marker".to_string(), json!(has_marker));
    debug_info.insert("marker_key".to_string(), json!(marker_key));

    // Get raw KV keys for this prefix
    let raw_keys = match kv_list(&prefix).await {
        Ok(js_value) => {
            let keys: Vec<String> =
                serde_wasm_bindgen::from_value(js_value).unwrap_or_else(|_| Vec::new());

            debug!(
                "Raw KV list returned {} keys for prefix '{}'",
                keys.len(),
                prefix
            );
            keys
        }
        Err(e) => {
            debug!("Error listing keys with prefix '{}': {:?}", prefix, e);
            Vec::new()
        }
    };
    debug_info.insert("raw_kv_keys".to_string(), json!(raw_keys));

    // Process and categorize the keys
    let mut content_keys = Vec::new();
    let mut metadata_keys = Vec::new();
    let mut directory_marker_keys = Vec::new();

    for key in &raw_keys {
        if key.ends_with(crate::vfs::vercel_kv_types::FILE_CONTENT_SUFFIX) {
            content_keys.push(key.clone());
        } else if key.ends_with(crate::vfs::vercel_kv_types::FILE_METADATA_SUFFIX) {
            metadata_keys.push(key.clone());
        } else if key.ends_with(crate::vfs::vercel_kv_types::DIRECTORY_MARKER_SUFFIX) {
            directory_marker_keys.push(key.clone());
        }
    }

    debug_info.insert("content_keys".to_string(), json!(content_keys));
    debug_info.insert("metadata_keys".to_string(), json!(metadata_keys));
    debug_info.insert(
        "directory_marker_keys".to_string(),
        json!(directory_marker_keys),
    );

    // Try the GitHub load
    let github_result = vfs.load_github_directory(&path, shallow).await;
    match &github_result {
        Ok(load_result) => {
            debug_info.insert("github_load_success".to_string(), json!(true));
            debug_info.insert(
                "github_loaded_files".to_string(),
                json!(load_result
                    .loaded_files
                    .iter()
                    .map(|f| f.path.clone())
                    .collect::<Vec<String>>()),
            );
            debug_info.insert(
                "github_placeholder_count".to_string(),
                json!(load_result.loaded_files.iter().filter(|f| f.is_placeholder).count()),
            );
        }
        Err(e) => {
            debug_info.insert("github_load_success".to_string(), json!(false));
            debug_info.insert("github_load_error".to_string(), json!(format!("{:?}", e)));
        }
    }

    // Include the actual result
    match result {
        Ok(entries) => {
            debug_info.insert("success".to_string(), json!(true));
            debug_info.insert("entry_count".to_string(), json!(entries.len()));

            // Include summarized entries
            let entry_summaries: Vec<Value> = entries
                .iter()
                .map(|entry| {
                    json!({
                        "name": entry.name,
                        "path": entry.path,
                        "is_directory": entry.is_directory,
                        "has_attributes": entry.attributes.is_some(),
                        "size": entry.attributes.as_ref().map(|a| a.size).unwrap_or(0)
                    })
                })
                .collect();

            debug_info.insert("entries".to_string(), json!(entry_summaries));
        }
        Err(e) => {
            debug_info.insert("success".to_string(), json!(false));
            debug_info.insert("error".to_string(), json!(format!("{:?}", e)));
        }
    }

    // Convert to JsValue and return
    match serde_wasm_bindgen::to_value(&debug_info) {
        Ok(js_value) => Ok(js_value),
        Err(e) => Err(JsValue::from_str(&format!(
            "Error serializing debug info: {}",
            e
        ))),
    }
}

#[wasm_bindgen]
pub fn admin_debug_test_panic() -> Result<(), JsValue> {
    log::warn!("admin_debug_test_panic called - triggering intentional panic");
    // This function is only for debugging panic capture
    panic!("This is an intentional test panic from admin_debug_test_panic()");
}

#[wasm_bindgen]
pub fn admin_init_kv_bindings_js() -> Result<JsValue, JsValue> {
    dummy().map_err(|_| JsValue::from_str("Failed to initialize KV bindings"))
}