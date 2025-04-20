use crate::api::rules::RulesUpdateRequest;
use crate::utils::file_wasm;
use crate::vfs::vercel_kv_helpers::get_directory_marker_key;
use crate::vfs::vercel_kv_js_bindings::{dummy, kv_exists, kv_list};
use crate::vfs::vercel_kv_vfs::VercelKvVfs;
use crate::vfs::{DirectoryEntry, FileAttributes, VfsError, VirtualFileSystem};

use log::{error, info};
use serde_json::{json, Value};
use std::collections::HashMap;

use wasm_bindgen::prelude::*;

// --- Helper Functions ---

// Helper to get the VFS instance
async fn get_vfs() -> Result<VercelKvVfs, VfsError> {
    file_wasm::get_vfs()
        .await
        .map_err(|e| VfsError::Other(format!("Failed to get VFS: {}", e)))
}

// Helper to convert VfsError to JsValue for the FFI boundary
fn vfs_error_to_js(err: VfsError) -> JsValue {
    let error_type = match &err {
        VfsError::NotFound(_) => "NotFound",
        VfsError::ConfigError(_) => "ConfigError",
        VfsError::StorageError(_) => "StorageError",
        VfsError::NetworkError(_) => "NetworkError",
        VfsError::IoError(_) => "IoError",
        VfsError::IsDirectory(_) => "IsDirectory",
        VfsError::NotDirectory(_) => "NotDirectory",
        VfsError::InvalidPath(_) => "InvalidPath",
        VfsError::PermissionDenied(_) => "PermissionDenied",
        VfsError::AlreadyExists(_) => "AlreadyExists",
        VfsError::NotSupported(_) => "NotSupported",
        VfsError::Other(_) => "Other",
    };

    let error_obj = json!({
        "type": error_type,
        "message": format!("{}", err)
    });

    // Serialize the JSON object to a string and then into JsValue
    let error_json = error_obj.to_string();
    JsValue::from_str(&error_json)
}

// --- Wasm Bindgen Exports ---

#[wasm_bindgen]
pub async fn admin_read_file(path: String) -> Result<String, JsValue> {
    info!("admin_read_file called with path: {}", path);
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;
    match vfs.read_file(&path).await {
        Ok(content) => String::from_utf8(content)
            .map_err(|e| JsValue::from_str(&format!("UTF-8 conversion error: {}", e))),
        Err(e) => Err(vfs_error_to_js(e)),
    }
}

#[wasm_bindgen]
pub async fn admin_write_file(path: String, text_content: String) -> Result<(), JsValue> {
    info!("admin_write_file called with path: {}", path);
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;
    let content = text_content.into_bytes();
    vfs.write_file(&path, content)
        .await
        .map_err(vfs_error_to_js)
}

#[wasm_bindgen]
pub async fn admin_delete_file(path: String) -> Result<(), JsValue> {
    info!("admin_delete_file called with path: {}", path);
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;
    vfs.delete_file(&path).await.map_err(vfs_error_to_js)
}

#[wasm_bindgen]
pub async fn admin_file_exists(path: String) -> Result<bool, JsValue> {
    info!("admin_file_exists called with path: {}", path);
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;
    vfs.exists(&path).await.map_err(vfs_error_to_js)
}

/// Get file attributes - admin endpoint
#[wasm_bindgen]
pub async fn admin_get_file_attributes(path: String) -> Result<FileAttributes, JsValue> {
    info!("admin_get_file_attributes called for {}", path);
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;
    vfs.read_file_attributes(&path)
        .await
        .map_err(vfs_error_to_js)
}

/// Create directory - admin endpoint
#[wasm_bindgen]
pub async fn admin_create_directory(path: String) -> Result<(), JsValue> {
    info!("admin_create_directory called for {}", path);
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;
    vfs.create_directory(&path).await.map_err(vfs_error_to_js)
}

/// List directory contents - admin endpoint
#[wasm_bindgen]
pub async fn list_directory(path: String) -> Result<Vec<DirectoryEntry>, JsValue> {
    info!("admin_list_directory called for path: {}", path);
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;
    vfs.list_directory(&path).await.map_err(vfs_error_to_js)
}

/// Load all files from a GitHub repository directory recursively.
/// If shallow=true, only creates placeholder entries without downloading content.
#[wasm_bindgen]
pub async fn admin_load_github_directory(path: String, shallow: bool) -> Result<JsValue, JsValue> {
    info!(
        "admin_load_github_directory called for path: {} (shallow: {})",
        path, shallow
    );
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;

    match vfs.load_github_directory(&path, shallow).await {
        Ok(result) => match serde_wasm_bindgen::to_value(&result) {
            Ok(js_result) => Ok(js_result),
            Err(e) => Err(JsValue::from_str(&format!(
                "Error serializing load result: {}",
                e
            ))),
        },
        Err(e) => Err(vfs_error_to_js(e)),
    }
}

/// Load only direct children of a GitHub repository directory (non-recursive).
/// If shallow=true, only creates placeholder entries without downloading content.
#[wasm_bindgen]
pub async fn admin_load_github_directory_flat(
    path: String,
    shallow: bool,
) -> Result<JsValue, JsValue> {
    info!(
        "admin_load_github_directory_flat called for path: {} (shallow: {})",
        path, shallow
    );
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;

    match vfs.load_github_directory_flat(&path, shallow).await {
        Ok(result) => match serde_wasm_bindgen::to_value(&result) {
            Ok(js_result) => Ok(js_result),
            Err(e) => Err(JsValue::from_str(&format!(
                "Error serializing load result: {}",
                e
            ))),
        },
        Err(e) => Err(vfs_error_to_js(e)),
    }
}

/// Debug function to provide detailed info about list_directory operation
#[wasm_bindgen]
pub async fn debug_list_directory(path: String) -> Result<JsValue, JsValue> {
    info!("debug_list_directory called for path: '{}'", path);

    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;
    let result = vfs.list_directory(&path).await;

    // Collect diagnostic information
    let mut debug_info = HashMap::new();
    debug_info.insert("path".to_string(), json!(path));

    // Check if directory exists first
    let exists = if path.is_empty() {
        true
    } else {
        vfs.exists(&path).await.unwrap_or(false)
    };
    debug_info.insert("directory_exists".to_string(), json!(exists));

    let prefix = if path.ends_with('/') || path.is_empty() {
        path.clone()
    } else {
        format!("{}/", path)
    };
    debug_info.insert("prefix_for_kv".to_string(), json!(prefix));

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
            serde_wasm_bindgen::from_value(js_value).unwrap_or_else(|_| Vec::<String>::new())
        }
        Err(_) => Vec::new(),
    };
    debug_info.insert("raw_kv_keys".to_string(), json!(raw_keys));

    // Include the actual result
    match result {
        Ok(entries) => {
            debug_info.insert("success".to_string(), json!(true));
            let entry_summaries: Vec<Value> = entries
                .iter()
                .map(|entry| {
                    json!({
                        "name": entry.name,
                        "path": entry.path,
                        "is_directory": entry.is_directory,
                        "attributes_present": entry.attributes.is_some(),
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
    panic!("Intentional test panic from admin_debug_test_panic()");
}

#[wasm_bindgen]
pub fn admin_init_kv_bindings_js() -> Result<JsValue, JsValue> {
    // Assuming dummy() initializes or returns something needed for JS bindings
    dummy().map_err(|_| JsValue::from_str("Failed to initialize KV bindings"))
}

/// Update rules from GitHub repos based on a configuration file
#[wasm_bindgen]
pub async fn admin_update_rules(config_path: Option<String>) -> Result<JsValue, JsValue> {
    info!(
        "admin_update_rules called with config path: {:?}",
        config_path
    );

    let request = RulesUpdateRequest { config_path };

    match crate::api::rules::update_rules(Some(request)).await {
        Ok(response) => {
            // Assuming the response body is directly convertible/useful as JsValue
            // This might need adjustment based on the actual return type of update_rules
            let body = response.body;
            Ok(JsValue::from_str(&body)) // Adjust if body is not String
        }
        Err(e) => {
            error!("Error updating rules: {}", e);
            Err(JsValue::from_str(&format!("Error updating rules: {}", e)))
        }
    }
}
