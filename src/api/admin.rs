use crate::utils::file_wasm;
use crate::vfs::{vercel_kv_vfs::VercelKvVfs, VfsError, VirtualFileSystem};
use base64;
use log::{error, info};
use wasm_bindgen::prelude::*;
use web_sys::console;

// Helper to create VFS instance (simplistic for now)
async fn get_vfs() -> Result<VercelKvVfs, VfsError> {
    file_wasm::get_vfs()
        .await
        .map_err(|e| VfsError::Other(format!("Failed to get VFS: {}", e)))
}

// Helper to convert VfsError to JsValue for FFI boundary
fn vfs_error_to_js(err: VfsError) -> JsValue {
    JsValue::from_str(&format!("VFS Error: {}", err))
}

#[wasm_bindgen]
pub async fn admin_read_file(path: String) -> Result<JsValue, JsValue> {
    console::log_1(&format!("admin_read_file called with path: {}", path).into());
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;
    match vfs.read_file(&path).await {
        Ok(content) => {
            // Return content as Base64 encoded string for safe JS transfer
            let base64_content = base64::encode(&content);
            Ok(JsValue::from_str(&base64_content))
        }
        Err(e) => Err(vfs_error_to_js(e)),
    }
}

#[wasm_bindgen]
pub async fn admin_write_file(path: String, base64_content: String) -> Result<(), JsValue> {
    console::log_1(&format!("admin_write_file called with path: {}", path).into());
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;
    match base64::decode(&base64_content) {
        Ok(content) => vfs
            .write_file(&path, content)
            .await
            .map_err(vfs_error_to_js),
        Err(e) => Err(JsValue::from_str(&format!("Base64 decode error: {}", e))),
    }
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
pub async fn admin_get_file_attributes(path: String) -> Result<JsValue, JsValue> {
    info!("admin_get_file_attributes called for {}", path);
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;

    // Read file attributes from the VFS
    match vfs.read_file_attributes(&path).await {
        Ok(attributes) => {
            // Convert the FileAttributes struct to a JavaScript object
            match serde_wasm_bindgen::to_value(&attributes) {
                Ok(js_attributes) => Ok(js_attributes),
                Err(e) => Err(JsValue::from_str(&format!(
                    "Error serializing attributes: {}",
                    e
                ))),
            }
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
pub async fn list_directory(path: String) -> Result<JsValue, JsValue> {
    info!("list_directory called for path: {}", path);
    let vfs = get_vfs().await.map_err(vfs_error_to_js)?;

    match vfs.list_directory(&path).await {
        Ok(entries) => {
            // Convert the entries to a JavaScript Array
            match serde_wasm_bindgen::to_value(&entries) {
                Ok(js_entries) => Ok(js_entries),
                Err(e) => Err(JsValue::from_str(&format!(
                    "Error serializing directory entries: {}",
                    e
                ))),
            }
        }
        Err(e) => {
            error!("Error listing directory: {}", e);
            Err(vfs_error_to_js(e))
        }
    }
}
