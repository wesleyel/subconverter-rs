use crate::utils::system::safe_system_time;
use crate::vfs::vercel_kv_helpers::*;
use crate::vfs::vercel_kv_js_bindings::*;
use crate::vfs::vercel_kv_types::*;
use crate::vfs::VfsError;
use serde_wasm_bindgen;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use tokio::sync::RwLock;
use wasm_bindgen_futures;

/// Represents the storage layer for Vercel KV VFS
/// Handles all interactions with KV store and memory caches
pub struct VercelKvStore {
    memory_cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    metadata_cache: Arc<RwLock<HashMap<String, FileAttributes>>>,
}

impl VercelKvStore {
    /// Create a new KV store instance
    pub fn new() -> Self {
        Self {
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get memory cache reference
    pub fn get_memory_cache(&self) -> Arc<RwLock<HashMap<String, Vec<u8>>>> {
        self.memory_cache.clone()
    }

    /// Get metadata cache reference
    pub fn get_metadata_cache(&self) -> Arc<RwLock<HashMap<String, FileAttributes>>> {
        self.metadata_cache.clone()
    }

    //------------------------------------------------------------------------------
    // Memory Cache Operations
    //------------------------------------------------------------------------------

    /// Read file content from memory cache
    pub async fn read_from_memory_cache(&self, path: &str) -> Option<Vec<u8>> {
        self.memory_cache.read().await.get(path).cloned()
    }

    /// Write file content to memory cache
    pub async fn write_to_memory_cache(&self, path: &str, content: Vec<u8>) {
        self.memory_cache
            .write()
            .await
            .insert(path.to_string(), content);
    }

    /// Check if file exists in memory cache
    pub async fn exists_in_memory_cache(&self, path: &str) -> bool {
        self.memory_cache.read().await.contains_key(path)
    }

    /// Remove file from memory cache
    pub async fn remove_from_memory_cache(&self, path: &str) {
        self.memory_cache.write().await.remove(path);
    }

    //------------------------------------------------------------------------------
    // Metadata Cache Operations
    //------------------------------------------------------------------------------

    /// Read file attributes from metadata cache
    pub async fn read_from_metadata_cache(&self, path: &str) -> Option<FileAttributes> {
        self.metadata_cache.read().await.get(path).cloned()
    }

    /// Write file attributes to metadata cache
    pub async fn write_to_metadata_cache(&self, path: &str, attributes: FileAttributes) {
        self.metadata_cache
            .write()
            .await
            .insert(path.to_string(), attributes);
    }

    /// Check if metadata exists in cache
    pub async fn exists_in_metadata_cache(&self, path: &str) -> bool {
        self.metadata_cache.read().await.contains_key(path)
    }

    /// Remove metadata from cache
    pub async fn remove_from_metadata_cache(&self, path: &str) {
        self.metadata_cache.write().await.remove(path);
    }

    //------------------------------------------------------------------------------
    // KV Store Content Operations
    //------------------------------------------------------------------------------

    /// Read file content from KV store
    pub async fn read_from_kv(&self, path: &str) -> Result<Option<Vec<u8>>, VfsError> {
        let content_key = get_content_key(path);
        match kv_get(&content_key).await {
            Ok(js_value) => {
                if js_value.is_null() || js_value.is_undefined() {
                    Ok(None)
                } else {
                    // Convert JsValue to Vec<u8>
                    let content: Vec<u8> =
                        serde_wasm_bindgen::from_value(js_value).map_err(|e| {
                            VfsError::Other(format!("Failed to deserialize file content: {}", e))
                        })?;
                    Ok(Some(content))
                }
            }
            Err(e) => Err(js_error_to_vfs(e, "Failed to read file from KV")),
        }
    }

    /// Write file content to KV store
    pub async fn write_to_kv(&self, path: &str, content: &[u8]) -> Result<(), VfsError> {
        let content_key = get_content_key(path);
        match kv_set(&content_key, content).await {
            Ok(_) => Ok(()),
            Err(e) => Err(js_error_to_vfs(e, "Failed to write file to KV")),
        }
    }

    /// Write file content to KV store in background (non-blocking)
    pub fn write_to_kv_background(&self, path: String, content: Vec<u8>) {
        let content_key = get_content_key(&path);
        let content_clone = content.clone();

        wasm_bindgen_futures::spawn_local(async move {
            match kv_set(&content_key, &content_clone).await {
                Ok(_) => {
                    log::debug!("Successfully stored {} in KV background.", path);
                }
                Err(e) => {
                    log::error!("Background KV write error for {}: {:?}", path, e);
                }
            }
        });
    }

    /// Check if file exists in KV store
    pub async fn exists_in_kv(&self, path: &str) -> Result<bool, VfsError> {
        let content_key = get_content_key(path);
        match kv_exists(&content_key).await {
            Ok(js_value) => {
                let exists = js_value.as_bool().unwrap_or(false);
                Ok(exists)
            }
            Err(e) => Err(js_error_to_vfs(e, "Failed to check if file exists in KV")),
        }
    }

    /// Delete file from KV store
    pub async fn delete_from_kv(&self, path: &str) -> Result<(), VfsError> {
        let content_key = get_content_key(path);
        match kv_del(&content_key).await {
            Ok(_) => Ok(()),
            Err(e) => Err(js_error_to_vfs(e, "Failed to delete file from KV")),
        }
    }

    //------------------------------------------------------------------------------
    // KV Store Metadata Operations
    //------------------------------------------------------------------------------

    /// Read file metadata from KV store
    pub async fn read_metadata_from_kv(
        &self,
        path: &str,
    ) -> Result<Option<FileAttributes>, VfsError> {
        let metadata_key = get_metadata_key(path);
        match kv_get(&metadata_key).await {
            Ok(js_value) => {
                if js_value.is_null() || js_value.is_undefined() {
                    Ok(None)
                } else {
                    // Convert JsValue to FileAttributes
                    let metadata_bytes: Vec<u8> = serde_wasm_bindgen::from_value(js_value)
                        .map_err(|e| {
                            VfsError::Other(format!("Failed to deserialize metadata bytes: {}", e))
                        })?;

                    let attributes: FileAttributes = serde_json::from_slice(&metadata_bytes)
                        .map_err(|e| {
                            VfsError::Other(format!("Failed to parse file attributes: {}", e))
                        })?;

                    Ok(Some(attributes))
                }
            }
            Err(e) => Err(js_error_to_vfs(e, "Failed to read metadata from KV")),
        }
    }

    /// Write file metadata to KV store
    pub async fn write_metadata_to_kv(
        &self,
        path: &str,
        attributes: &FileAttributes,
    ) -> Result<(), VfsError> {
        let metadata_key = get_metadata_key(path);
        let metadata_json = serde_json::to_vec(attributes)
            .map_err(|e| VfsError::Other(format!("Failed to serialize file attributes: {}", e)))?;

        match kv_set(&metadata_key, &metadata_json).await {
            Ok(_) => Ok(()),
            Err(e) => Err(js_error_to_vfs(e, "Failed to write metadata to KV")),
        }
    }

    /// Write file metadata to KV store in background (non-blocking)
    pub fn write_metadata_to_kv_background(&self, path: String, attributes: FileAttributes) {
        let metadata_key = get_metadata_key(&path);
        let metadata_json = match serde_json::to_vec(&attributes) {
            Ok(json) => json,
            Err(e) => {
                log::error!("Failed to serialize metadata for {}: {}", path, e);
                return;
            }
        };

        wasm_bindgen_futures::spawn_local(async move {
            match kv_set(&metadata_key, &metadata_json).await {
                Ok(_) => {
                    log::debug!(
                        "Successfully stored metadata for {} in KV background.",
                        path
                    );
                }
                Err(e) => {
                    log::error!("Background KV metadata write error for {}: {:?}", path, e);
                }
            }
        });
    }

    /// Delete file metadata from KV store
    pub async fn delete_metadata_from_kv(&self, path: &str) -> Result<(), VfsError> {
        let metadata_key = get_metadata_key(path);
        match kv_del(&metadata_key).await {
            Ok(_) => Ok(()),
            Err(e) => Err(js_error_to_vfs(e, "Failed to delete metadata from KV")),
        }
    }

    //------------------------------------------------------------------------------
    // Directory Operations
    //------------------------------------------------------------------------------

    /// Check if directory exists in KV store
    pub async fn directory_exists_in_kv(&self, path: &str) -> Result<bool, VfsError> {
        let dir_marker_key = get_directory_marker_key(path);
        match kv_exists(&dir_marker_key).await {
            Ok(js_value) => {
                let exists = js_value.as_bool().unwrap_or(false);
                Ok(exists)
            }
            Err(e) => Err(js_error_to_vfs(
                e,
                "Failed to check if directory exists in KV",
            )),
        }
    }

    /// Create directory marker in KV store
    pub async fn create_directory_in_kv(&self, path: &str) -> Result<(), VfsError> {
        let dir_marker_key = get_directory_marker_key(path);
        match kv_set(&dir_marker_key, &[]).await {
            Ok(_) => Ok(()),
            Err(e) => Err(js_error_to_vfs(
                e,
                "Failed to create directory marker in KV",
            )),
        }
    }

    /// Delete directory marker from KV store
    pub async fn delete_directory_from_kv(&self, path: &str) -> Result<(), VfsError> {
        let dir_marker_key = get_directory_marker_key(path);
        match kv_del(&dir_marker_key).await {
            Ok(_) => Ok(()),
            Err(e) => Err(js_error_to_vfs(
                e,
                "Failed to delete directory marker from KV",
            )),
        }
    }

    /// List keys with prefix from KV store (for directory listing)
    pub async fn list_keys_with_prefix(&self, prefix: &str) -> Result<Vec<String>, VfsError> {
        match kv_list(prefix).await {
            Ok(js_value) => {
                // Convert JsValue to Vec<String>
                let keys: Vec<String> = serde_wasm_bindgen::from_value(js_value)
                    .map_err(|e| VfsError::Other(format!("Failed to deserialize keys: {}", e)))?;
                Ok(keys)
            }
            Err(e) => Err(js_error_to_vfs(e, "Failed to list keys from KV")),
        }
    }

    //------------------------------------------------------------------------------
    // Status Operations
    //------------------------------------------------------------------------------

    /// Check if file is a placeholder
    pub async fn is_placeholder(&self, path: &str) -> Result<bool, VfsError> {
        let status_key = get_status_key(path);
        match kv_get(&status_key).await {
            Ok(js_value) => {
                if js_value.is_null() || js_value.is_undefined() {
                    Ok(false)
                } else {
                    let status: Vec<u8> = serde_wasm_bindgen::from_value(js_value)
                        .map_err(|_| VfsError::Other("Failed to deserialize status".to_string()))?;
                    if let Ok(status_str) = String::from_utf8(status.clone()) {
                        Ok(status_str == FILE_STATUS_PLACEHOLDER)
                    } else {
                        Ok(false)
                    }
                }
            }
            Err(e) => Err(js_error_to_vfs(e, "Failed to check if file is placeholder")),
        }
    }

    /// Set file as placeholder
    pub async fn set_as_placeholder(&self, path: &str) -> Result<(), VfsError> {
        let status_key = get_status_key(path);
        match kv_set(&status_key, FILE_STATUS_PLACEHOLDER.as_bytes()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(js_error_to_vfs(e, "Failed to set file as placeholder")),
        }
    }

    /// Clear placeholder status
    pub async fn clear_placeholder_status(&self, path: &str) -> Result<(), VfsError> {
        let status_key = get_status_key(path);
        match kv_del(&status_key).await {
            Ok(_) => Ok(()),
            Err(e) => Err(js_error_to_vfs(e, "Failed to clear placeholder status")),
        }
    }

    /// Filter internal keys and deduplicate to get unique real paths
    pub async fn get_unique_real_paths(&self, keys: &[String]) -> Vec<String> {
        let mut unique_paths = HashSet::new();

        for key in keys {
            if let Some(real_path) = get_real_path_from_key(key) {
                unique_paths.insert(real_path);
            } else if !is_internal_key(key) {
                // Not an internal key, so it's a real path already
                unique_paths.insert(key.clone());
            }
        }

        unique_paths.into_iter().collect()
    }

    /// Filter internal keys and deduplicate to get unique real file paths
    /// This version also filters out internal directory markers and their base paths
    pub async fn get_unique_real_paths_filtered(&self, keys: &[String]) -> Vec<String> {
        let mut unique_paths = HashSet::new();
        let mut exclude_paths = HashSet::new();

        // First pass: identify directory marker paths to exclude their non-directory versions
        for key in keys {
            if key.ends_with(DIRECTORY_MARKER_SUFFIX) {
                if let Some(real_path) = get_real_path_from_key(key) {
                    // Add the base path (without trailing slash) to exclude list
                    if real_path.ends_with('/') {
                        exclude_paths.insert(real_path[..real_path.len() - 1].to_string());
                    } else {
                        exclude_paths.insert(real_path);
                    }
                }
            }
        }

        // Second pass: add all non-excluded paths
        for key in keys {
            // Skip internal directory marker keys
            if key.ends_with(DIRECTORY_MARKER_SUFFIX) {
                continue;
            }

            if let Some(real_path) = get_real_path_from_key(key) {
                if !exclude_paths.iter().any(|p| p == &real_path) {
                    unique_paths.insert(real_path);
                }
            } else if !is_internal_key(key) && !exclude_paths.iter().any(|p| p == key) {
                // Not an internal key, so it's a real path already
                unique_paths.insert(key.clone());
            }
        }

        unique_paths.into_iter().collect()
    }
}

// Helper function to create default FileAttributes for a file
pub fn create_file_attributes(
    path: &str,
    content_size: usize,
    source_type: &str,
) -> FileAttributes {
    FileAttributes {
        size: content_size,
        created_at: safe_system_time()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        modified_at: safe_system_time()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        file_type: guess_file_type(path),
        is_directory: false,
        source_type: source_type.to_string(),
    }
}

// Helper function to create default directory FileAttributes
pub fn create_directory_attributes(source_type: &str) -> FileAttributes {
    FileAttributes {
        is_directory: true,
        source_type: source_type.to_string(),
        ..Default::default()
    }
}

/// Helper function to check if a key is an internal key (metadata, status, etc.)
pub fn is_internal_key(key: &str) -> bool {
    key.ends_with(FILE_METADATA_SUFFIX)
        || key.ends_with(FILE_STATUS_SUFFIX)
        || key.ends_with(FILE_CONTENT_SUFFIX)
        || key.ends_with(DIRECTORY_MARKER_SUFFIX)
}

/// Helper function to extract the real path from a key by removing internal suffixes
pub fn get_real_path_from_key(key: &str) -> Option<String> {
    if key.ends_with(FILE_METADATA_SUFFIX) {
        Some(key[..key.len() - FILE_METADATA_SUFFIX.len()].to_string())
    } else if key.ends_with(FILE_STATUS_SUFFIX) {
        Some(key[..key.len() - FILE_STATUS_SUFFIX.len()].to_string())
    } else if key.ends_with(FILE_CONTENT_SUFFIX) {
        Some(key[..key.len() - FILE_CONTENT_SUFFIX.len()].to_string())
    } else if key.ends_with(DIRECTORY_MARKER_SUFFIX) {
        Some(key[..key.len() - DIRECTORY_MARKER_SUFFIX.len()].to_string())
    } else {
        // Not an internal key
        None
    }
}
