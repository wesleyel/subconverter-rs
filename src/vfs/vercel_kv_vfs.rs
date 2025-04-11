use super::VirtualFileSystem;
use crate::utils::system::safe_system_time;
use crate::vfs::VfsError;
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures;

// File metadata structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileAttributes {
    /// Size of the file in bytes
    pub size: usize,
    /// Creation timestamp (seconds since UNIX epoch)
    pub created_at: u64,
    /// Last modified timestamp (seconds since UNIX epoch)
    pub modified_at: u64,
    /// File type (mime type or extension)
    pub file_type: String,
    /// Is this a directory marker
    pub is_directory: bool,
}

impl Default for FileAttributes {
    fn default() -> Self {
        let now = safe_system_time()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            size: 0,
            created_at: now,
            modified_at: now,
            file_type: "text/plain".to_string(),
            is_directory: false,
        }
    }
}

// Directory entry for listing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectoryEntry {
    /// Name of the file or directory (not the full path)
    pub name: String,
    /// Full path to the file or directory
    pub path: String,
    /// Is this entry a directory
    pub is_directory: bool,
    /// File attributes
    pub attributes: Option<FileAttributes>,
}

// Configuration for GitHub raw content source
#[derive(Clone, Debug)]
pub struct GitHubConfig {
    owner: String,
    repo: String,
    branch: String,
    root_path: String,
}

impl GitHubConfig {
    pub fn from_env() -> Result<Self, VfsError> {
        Ok(Self {
            owner: std::env::var("VFS_GITHUB_OWNER").unwrap_or_else(|_| "lonelam".to_string()),
            repo: std::env::var("VFS_GITHUB_REPO")
                .unwrap_or_else(|_| "subconverter-rs".to_string()),
            branch: std::env::var("VFS_GITHUB_BRANCH").unwrap_or_else(|_| "main".to_string()),
            root_path: std::env::var("VFS_GITHUB_ROOT_PATH").unwrap_or_else(|_| "base".to_string()),
        })
    }

    fn get_raw_url(&self, file_path: &str) -> String {
        let base = format!(
            "https://raw.githubusercontent.com/{}/{}/{}",
            self.owner, self.repo, self.branch
        );
        let full_path = if self.root_path.is_empty() {
            file_path.to_string()
        } else {
            format!("{}/{}", self.root_path.trim_matches('/'), file_path)
        };
        format!("{}/{}", base, full_path.trim_start_matches('/'))
    }
}

// Define placeholder JS functions (replace with actual bindings later)
#[wasm_bindgen(module = "/js/kv_bindings.js")] // Use snippet name
extern "C" {
    #[wasm_bindgen(catch)]
    async fn kv_get(key: &str) -> Result<JsValue, JsValue>; // Returns Option<Vec<u8>> encoded as JsValue?
    #[wasm_bindgen(catch)]
    async fn kv_set(key: &str, value: &[u8]) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    async fn kv_exists(key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    async fn kv_del(key: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    async fn kv_list(prefix: &str) -> Result<JsValue, JsValue>; // New function to list keys with a prefix
    #[wasm_bindgen(catch)]
    async fn fetch_url(url: &str) -> Result<JsValue, JsValue>; // Returns Response object or similar?
                                                               // Need helpers to extract status and bytes from the fetch result
    #[wasm_bindgen(catch)]
    async fn response_status(response: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    async fn response_bytes(response: &JsValue) -> Result<Uint8Array, JsValue>; // Use imported Uint8Array
}

// Constants and helpers for file storage
const FILE_CONTENT_SUFFIX: &str = ".content";
const FILE_METADATA_SUFFIX: &str = ".metadata";
const DIRECTORY_MARKER_SUFFIX: &str = "/.dir";

// Normalize path: ensure consistency, e.g., remove leading '/'
fn normalize_path(path: &str) -> String {
    path.trim_start_matches('/').to_string()
}

// Get content key from path
fn get_content_key(path: &str) -> String {
    format!("{}{}", path, FILE_CONTENT_SUFFIX)
}

// Get metadata key from path
fn get_metadata_key(path: &str) -> String {
    format!("{}{}", path, FILE_METADATA_SUFFIX)
}

// Get directory marker key for a path
fn get_directory_marker_key(path: &str) -> String {
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
fn is_directory_path(path: &str) -> bool {
    path.ends_with('/') || path.is_empty()
}

// Extract parent directory path
fn get_parent_directory(path: &str) -> String {
    let path = path.trim_end_matches('/');
    match path.rfind('/') {
        Some(idx) => path[..=idx].to_string(),
        None => "".to_string(),
    }
}

// Extract filename from a path
fn get_filename(path: &str) -> String {
    let path = path.trim_end_matches('/');
    match path.rfind('/') {
        Some(idx) => path[idx + 1..].to_string(),
        None => path.to_string(),
    }
}

// Helper to convert JsValue error to VfsError
fn js_error_to_vfs(err: JsValue, context: &str) -> VfsError {
    let msg = format!("{}: {:?}", context, err);
    log::error!("{}", msg);
    if context.contains("KV") {
        VfsError::StorageError(msg)
    } else if context.contains("Fetch") || context.contains("GitHub") {
        // Match GitHub context too
        VfsError::NetworkError(msg)
    } else {
        VfsError::Other(msg)
    }
}

#[derive(Clone)]
pub struct VercelKvVfs {
    memory_cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    metadata_cache: Arc<RwLock<HashMap<String, FileAttributes>>>,
    github_config: GitHubConfig,
}

impl VirtualFileSystem for VercelKvVfs {
    async fn read_file(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        let normalized_path = normalize_path(path);

        // Don't allow reading directory markers
        if is_directory_path(&normalized_path) {
            return Err(VfsError::IsDirectory(normalized_path));
        }

        // 1. Check memory cache
        if let Some(content) = self.memory_cache.read().await.get(&normalized_path) {
            log::debug!("Cache hit for: {}", normalized_path);
            return Ok(content.clone());
        }

        // 2. Check Vercel KV via JS binding
        log::debug!("Cache miss, checking KV for: {}", normalized_path);
        let content_key = get_content_key(&normalized_path);
        match kv_get(&content_key).await {
            Ok(js_value) => {
                if !js_value.is_null() && !js_value.is_undefined() {
                    log::debug!("KV hit for: {}", normalized_path);
                    let content: Vec<u8> =
                        serde_wasm_bindgen::from_value(js_value).map_err(|e| {
                            VfsError::Other(format!("Failed to deserialize KV value: {}", e))
                        })?;

                    self.memory_cache
                        .write()
                        .await
                        .insert(normalized_path.clone(), content.clone());
                    return Ok(content);
                } else {
                    log::debug!("KV miss (null/undefined) for: {}", normalized_path);
                }
            }
            Err(e) => {
                log::error!("Vercel KV read error (JS) for {}: {:?}", normalized_path, e);
            }
        }

        // 3. Fetch from GitHub via JS binding
        let url = self.github_config.get_raw_url(&normalized_path);
        log::debug!("Fetching from GitHub via JS: {}", url);

        let fetch_response = fetch_url(&url)
            .await
            .map_err(|e| js_error_to_vfs(e, "Fetch GitHub URL failed"))?;

        let status_js = response_status(&fetch_response)
            .await
            .map_err(|e| js_error_to_vfs(e, "Get fetch status failed"))?;
        let status = status_js
            .as_f64()
            .map(|f| f as u16)
            .ok_or_else(|| js_error_to_vfs(status_js, "GitHub fetch status was not a number"))?;

        if !(200..300).contains(&status) {
            log::warn!("GitHub fetch failed for {}: Status {}", url, status);
            if status == 404 {
                return Err(VfsError::NotFound(format!(
                    "File not found locally or on GitHub: {}",
                    normalized_path
                )));
            } else {
                return Err(VfsError::NetworkError(format!(
                    "GitHub fetch failed with status: {}",
                    status
                )));
            }
        }

        let uint8_array = response_bytes(&fetch_response)
            .await
            .map_err(|e| js_error_to_vfs(e, "Get fetch response bytes failed"))?;
        let content = uint8_array.to_vec();
        log::debug!(
            "Successfully fetched {} bytes from GitHub for: {}",
            content.len(),
            normalized_path
        );

        // 4. Store in memory cache AND Vercel KV
        self.memory_cache
            .write()
            .await
            .insert(normalized_path.clone(), content.clone());

        // Create file attributes for this new file
        let attributes = FileAttributes {
            size: content.len(),
            created_at: safe_system_time()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            modified_at: safe_system_time()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            file_type: guess_file_type(&normalized_path),
            is_directory: false,
        };

        // Store file and its metadata
        let path_clone = normalized_path.clone();
        let content_clone = content.clone();
        let attributes_clone = attributes.clone();

        // Update metadata cache
        self.metadata_cache
            .write()
            .await
            .insert(normalized_path.clone(), attributes);

        wasm_bindgen_futures::spawn_local(async move {
            // Save file content
            let content_key = get_content_key(&path_clone);
            match kv_set(&content_key, &content_clone).await {
                Ok(_) => {
                    log::debug!(
                        "Successfully stored {} in KV background via JS.",
                        path_clone
                    );
                }
                Err(e) => {
                    log::error!(
                        "Background Vercel KV write error (JS) for {}: {:?}",
                        path_clone,
                        e
                    );
                }
            }

            // Save file metadata
            let metadata_key = get_metadata_key(&path_clone);
            let metadata_json = serde_json::to_vec(&attributes_clone).unwrap_or_default();
            match kv_set(&metadata_key, &metadata_json).await {
                Ok(_) => {
                    log::debug!(
                        "Successfully stored metadata for {} in KV background via JS.",
                        path_clone
                    );
                }
                Err(e) => {
                    log::error!(
                        "Background Vercel KV metadata write error (JS) for {}: {:?}",
                        path_clone,
                        e
                    );
                }
            }

            // Ensure parent directories exist
            let parent = get_parent_directory(&path_clone);
            if !parent.is_empty() {
                let mut current_path = parent.clone();
                while !current_path.is_empty() {
                    let dir_marker_key = get_directory_marker_key(&current_path);
                    let dir_attributes = FileAttributes {
                        is_directory: true,
                        ..Default::default()
                    };
                    let dir_metadata_json = serde_json::to_vec(&dir_attributes).unwrap_or_default();

                    let marker_result = kv_set(&dir_marker_key, &[]).await;
                    let metadata_result =
                        kv_set(&get_metadata_key(&current_path), &dir_metadata_json).await;

                    if let Err(e) = marker_result {
                        log::error!(
                            "Failed to create directory marker for {}: {:?}",
                            current_path,
                            e
                        );
                    }

                    if let Err(e) = metadata_result {
                        log::error!(
                            "Failed to store directory metadata for {}: {:?}",
                            current_path,
                            e
                        );
                    }

                    // Move up to parent directory
                    current_path = get_parent_directory(&current_path);
                }
            }
        });

        Ok(content)
    }

    async fn write_file(&self, path: &str, content: Vec<u8>) -> Result<(), VfsError> {
        let normalized_path = normalize_path(path);

        // Don't allow writing to directory paths
        if is_directory_path(&normalized_path) {
            return Err(VfsError::IsDirectory(normalized_path));
        }

        log::debug!("Writing file via JS: {}", normalized_path);

        // Update memory cache
        self.memory_cache
            .write()
            .await
            .insert(normalized_path.clone(), content.clone());

        // Create file attributes
        let attributes = FileAttributes {
            size: content.len(),
            created_at: safe_system_time()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            modified_at: safe_system_time()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            file_type: guess_file_type(&normalized_path),
            is_directory: false,
        };

        // Update metadata cache
        self.metadata_cache
            .write()
            .await
            .insert(normalized_path.clone(), attributes.clone());

        // Get keys for storage
        let content_key = get_content_key(&normalized_path);

        // Store file content
        match kv_set(&content_key, &content).await {
            Ok(_) => {
                // Store file metadata
                let metadata_key = get_metadata_key(&normalized_path);
                let metadata_json = serde_json::to_vec(&attributes).map_err(|e| {
                    VfsError::Other(format!("Failed to serialize file attributes: {}", e))
                })?;

                if let Err(e) = kv_set(&metadata_key, &metadata_json).await {
                    log::error!("Failed to store metadata for {}: {:?}", normalized_path, e);
                    // Continue even if metadata storage fails
                }

                // Ensure parent directories exist
                let parent = get_parent_directory(&normalized_path);
                if !parent.is_empty() {
                    wasm_bindgen_futures::spawn_local(async move {
                        let mut current_path = parent.clone();
                        while !current_path.is_empty() {
                            let dir_marker_key = get_directory_marker_key(&current_path);
                            let dir_attributes = FileAttributes {
                                is_directory: true,
                                ..Default::default()
                            };
                            let dir_metadata_json =
                                serde_json::to_vec(&dir_attributes).unwrap_or_default();

                            let marker_result = kv_set(&dir_marker_key, &[]).await;
                            let metadata_result =
                                kv_set(&get_metadata_key(&current_path), &dir_metadata_json).await;

                            if let Err(e) = marker_result {
                                log::error!(
                                    "Failed to create directory marker for {}: {:?}",
                                    current_path,
                                    e
                                );
                            }

                            if let Err(e) = metadata_result {
                                log::error!(
                                    "Failed to store directory metadata for {}: {:?}",
                                    current_path,
                                    e
                                );
                            }

                            // Move up to parent directory
                            current_path = get_parent_directory(&current_path);
                        }
                    });
                }

                Ok(())
            }
            Err(e) => Err(js_error_to_vfs(e, "KV set failed")),
        }
    }

    async fn exists(&self, path: &str) -> Result<bool, VfsError> {
        let normalized_path = normalize_path(path);

        // Check memory cache for both content and metadata
        if self
            .memory_cache
            .read()
            .await
            .contains_key(&normalized_path)
        {
            log::trace!("Exists check (memory hit): {}", normalized_path);
            return Ok(true);
        }

        if self
            .metadata_cache
            .read()
            .await
            .contains_key(&normalized_path)
        {
            log::trace!("Exists check (metadata memory hit): {}", normalized_path);
            return Ok(true);
        }

        // For directory paths, check if directory marker exists
        if is_directory_path(&normalized_path) {
            let dir_marker_key = get_directory_marker_key(&normalized_path);
            log::trace!(
                "Exists check (checking directory marker): {}",
                dir_marker_key
            );

            match kv_exists(&dir_marker_key).await {
                Ok(js_value) => {
                    let exists = js_value.as_bool().unwrap_or(false);
                    if exists {
                        log::trace!("Exists check (directory marker found): {}", normalized_path);
                        return Ok(true);
                    }
                }
                Err(e) => {
                    log::error!(
                        "Vercel KV directory marker check error (JS) for {}: {:?}",
                        normalized_path,
                        e
                    );
                }
            }
        }

        // Check KV storage for file content
        let content_key = get_content_key(&normalized_path);
        log::trace!("Exists check (checking KV via JS): {}", content_key);
        match kv_exists(&content_key).await {
            Ok(js_value) => {
                let exists = js_value.as_bool().unwrap_or(false);
                if exists {
                    log::trace!("Exists check (KV hit via JS): {}", normalized_path);
                    return Ok(true);
                }
                log::trace!("Exists check (KV miss via JS): {}", normalized_path);
            }
            Err(e) => {
                log::error!(
                    "Vercel KV exists check error (JS) for {}: {:?}",
                    normalized_path,
                    e
                );
            }
        }

        // If it's a directory, we only check GitHub for directory marker if it's not a directory path
        if !is_directory_path(&normalized_path) {
            let url = self.github_config.get_raw_url(&normalized_path);
            log::trace!("Exists check (checking GitHub GET via JS): {}", url);
            match fetch_url(&url).await {
                Ok(fetch_response) => match response_status(&fetch_response).await {
                    Ok(status_js) => {
                        let status = status_js.as_f64().map(|f| f as u16).ok_or_else(|| {
                            js_error_to_vfs(
                                status_js,
                                "GitHub exists check status was not a number",
                            )
                        })?;
                        let found = (200..300).contains(&status);
                        log::trace!(
                            "Exists check (GitHub result {}): {}",
                            found,
                            normalized_path
                        );
                        Ok(found)
                    }
                    Err(e) => {
                        log::error!(
                            "Exists check (GitHub get status failed via JS) for {}: {:?}",
                            normalized_path,
                            e
                        );
                        Err(js_error_to_vfs(
                            e,
                            "GitHub exists check (get status) failed",
                        ))
                    }
                },
                Err(e) => {
                    log::warn!(
                        "Exists check (GitHub fetch failed via JS) for {}: {:?}",
                        normalized_path,
                        e
                    );
                    Err(js_error_to_vfs(e, "GitHub exists check (fetch) failed"))
                }
            }
        } else {
            // For directory paths that don't have a marker, return false
            Ok(false)
        }
    }

    async fn delete_file(&self, path: &str) -> Result<(), VfsError> {
        let normalized_path = normalize_path(path);
        log::debug!("Deleting file via JS: {}", normalized_path);

        // Remove from memory cache
        self.memory_cache.write().await.remove(&normalized_path);
        self.metadata_cache.write().await.remove(&normalized_path);

        // Delete content and metadata
        let content_key = get_content_key(&normalized_path);
        let metadata_key = get_metadata_key(&normalized_path);

        // Delete directory marker if it's a directory
        if is_directory_path(&normalized_path) {
            let dir_marker_key = get_directory_marker_key(&normalized_path);
            if let Err(e) = kv_del(&dir_marker_key).await {
                log::error!(
                    "Failed to delete directory marker for {}: {:?}",
                    normalized_path,
                    e
                );
                // Continue even if marker delete fails
            }
        }

        // Delete content and metadata in parallel
        let content_result = kv_del(&content_key).await;
        let metadata_result = kv_del(&metadata_key).await;

        // Check for errors on content deletion (primary operation)
        if let Err(e) = content_result {
            return Err(js_error_to_vfs(e, "KV content del failed"));
        }

        // Log but don't fail if metadata deletion fails
        if let Err(e) = metadata_result {
            log::error!("Failed to delete metadata for {}: {:?}", normalized_path, e);
        }

        Ok(())
    }

    // New methods for directory listing and attributes

    async fn read_file_attributes(&self, path: &str) -> Result<FileAttributes, VfsError> {
        let normalized_path = normalize_path(path);

        // Check memory cache first
        if let Some(attrs) = self.metadata_cache.read().await.get(&normalized_path) {
            return Ok(attrs.clone());
        }

        // Check if it's a directory with a marker key
        if is_directory_path(&normalized_path) {
            let dir_marker_key = get_directory_marker_key(&normalized_path);
            match kv_exists(&dir_marker_key).await {
                Ok(js_value) => {
                    let exists = js_value.as_bool().unwrap_or(false);
                    if exists {
                        // It's a directory, return default directory attributes
                        let attrs = FileAttributes {
                            is_directory: true,
                            ..Default::default()
                        };
                        return Ok(attrs);
                    }
                }
                Err(e) => {
                    log::error!("KV directory marker check error: {:?}", e);
                }
            }
        }

        // Try to get metadata from KV
        let metadata_key = get_metadata_key(&normalized_path);
        match kv_get(&metadata_key).await {
            Ok(js_value) => {
                if !js_value.is_null() && !js_value.is_undefined() {
                    // Try to deserialize metadata
                    let metadata_bytes: Vec<u8> = serde_wasm_bindgen::from_value(js_value)
                        .map_err(|e| {
                            VfsError::Other(format!("Failed to deserialize metadata bytes: {}", e))
                        })?;

                    let attributes: FileAttributes = serde_json::from_slice(&metadata_bytes)
                        .map_err(|e| {
                            VfsError::Other(format!("Failed to parse file attributes: {}", e))
                        })?;

                    // Cache the attributes
                    self.metadata_cache
                        .write()
                        .await
                        .insert(normalized_path, attributes.clone());

                    return Ok(attributes);
                }
            }
            Err(e) => {
                log::error!("KV metadata get error: {:?}", e);
            }
        }

        // If we can't find metadata, but the file exists, create default attributes
        if self.exists(&normalized_path).await? {
            // For files, we need to read the content to get the size
            if !is_directory_path(&normalized_path) {
                let content = self.read_file(&normalized_path).await?;
                let attributes = FileAttributes {
                    size: content.len(),
                    file_type: guess_file_type(&normalized_path),
                    is_directory: false,
                    ..Default::default()
                };

                // Cache the attributes
                self.metadata_cache
                    .write()
                    .await
                    .insert(normalized_path, attributes.clone());

                return Ok(attributes);
            } else {
                // It's a directory that exists (maybe implicitly)
                let attributes = FileAttributes {
                    is_directory: true,
                    ..Default::default()
                };
                return Ok(attributes);
            }
        }

        // File doesn't exist
        Err(VfsError::NotFound(format!(
            "File not found: {}",
            normalized_path
        )))
    }

    async fn list_directory(&self, path: &str) -> Result<Vec<DirectoryEntry>, VfsError> {
        let normalized_path = normalize_path(path);
        let dir_path = if normalized_path.is_empty() || normalized_path.ends_with('/') {
            normalized_path
        } else {
            format!("{}/", normalized_path)
        };

        log::debug!("Listing directory: {}", dir_path);

        // Check if directory exists
        let exists = self.exists(&dir_path).await?;
        if !exists && !dir_path.is_empty() {
            return Err(VfsError::NotFound(format!(
                "Directory not found: {}",
                dir_path
            )));
        }

        // Query KV for all keys with the directory prefix
        let prefix = dir_path.clone();
        match kv_list(&prefix).await {
            Ok(js_value) => {
                if js_value.is_null() || js_value.is_undefined() {
                    log::debug!("No keys found for prefix: {}", prefix);
                    return Ok(Vec::new());
                }

                // Parse the returned keys (should be an array of strings)
                let keys: Vec<String> = serde_wasm_bindgen::from_value(js_value).map_err(|e| {
                    VfsError::Other(format!("Failed to deserialize key list: {}", e))
                })?;

                log::debug!("Found {} keys with prefix {}", keys.len(), prefix);

                // Process the keys to extract unique directory entries
                let mut entries = HashMap::new();

                for key in keys {
                    // Skip metadata and directory marker keys
                    if key.ends_with(FILE_METADATA_SUFFIX) || key.ends_with(DIRECTORY_MARKER_SUFFIX)
                    {
                        continue;
                    }

                    // Remove content suffix if present
                    let path = if key.ends_with(FILE_CONTENT_SUFFIX) {
                        key[..key.len() - FILE_CONTENT_SUFFIX.len()].to_string()
                    } else {
                        key
                    };

                    // If path is outside the requested directory, skip it
                    if !path.starts_with(&dir_path) {
                        continue;
                    }

                    // Extract the relative path within this directory
                    let rel_path = &path[dir_path.len()..];

                    // For nested paths, we only want the top-level entries
                    let parts: Vec<&str> = rel_path.split('/').filter(|s| !s.is_empty()).collect();
                    if parts.is_empty() {
                        continue;
                    }

                    let name = parts[0].to_string();
                    let is_directory = parts.len() > 1 || rel_path.ends_with('/');
                    let entry_path = if is_directory {
                        format!("{}{}/", dir_path, name)
                    } else {
                        format!("{}{}", dir_path, name)
                    };

                    // Add to entries if not already present
                    if !entries.contains_key(&name) {
                        let attributes = self.read_file_attributes(&entry_path).await.ok();

                        entries.insert(
                            name.clone(),
                            DirectoryEntry {
                                name,
                                path: entry_path,
                                is_directory,
                                attributes,
                            },
                        );
                    }
                }

                // Convert HashMap to Vec
                let mut result: Vec<DirectoryEntry> = entries.into_values().collect();

                // Sort by name
                result.sort_by(|a, b| a.name.cmp(&b.name));

                Ok(result)
            }
            Err(e) => {
                log::error!("KV list error: {:?}", e);
                Err(js_error_to_vfs(e, "KV list failed"))
            }
        }
    }

    async fn create_directory(&self, path: &str) -> Result<(), VfsError> {
        let normalized_path = normalize_path(path);
        let dir_path = if normalized_path.is_empty() || normalized_path.ends_with('/') {
            normalized_path
        } else {
            format!("{}/", normalized_path)
        };

        log::debug!("Creating directory: {}", dir_path);

        // Create directory marker
        let dir_marker_key = get_directory_marker_key(&dir_path);
        let dir_attributes = FileAttributes {
            is_directory: true,
            ..Default::default()
        };

        // Update metadata cache
        self.metadata_cache
            .write()
            .await
            .insert(dir_path.clone(), dir_attributes.clone());

        // Store directory marker and metadata
        let metadata_key = get_metadata_key(&dir_path);
        let metadata_json = serde_json::to_vec(&dir_attributes).map_err(|e| {
            VfsError::Other(format!("Failed to serialize directory attributes: {}", e))
        })?;

        let marker_result = kv_set(&dir_marker_key, &[]).await;
        let metadata_result = kv_set(&metadata_key, &metadata_json).await;

        if let Err(e) = marker_result {
            return Err(js_error_to_vfs(e, "Failed to create directory marker"));
        }

        if let Err(e) = metadata_result {
            log::error!("Failed to store directory metadata: {:?}", e);
            // Continue even if metadata storage fails
        }

        // Ensure parent directories exist (if any)
        let parent = get_parent_directory(&dir_path);
        if !parent.is_empty() {
            // Box the recursive future to avoid infinitely sized types
            Box::pin(self.create_directory(&parent)).await?;
        }

        Ok(())
    }

    /// This function fetches all files from a specific directory in the GitHub repository
    /// and stores them in the VFS (both memory cache and Vercel KV).
    ///
    /// # Arguments
    ///
    /// * `directory_path` - The path to the directory to load files from (empty string for root)
    ///
    /// # Returns
    ///
    /// A Result containing information about the loaded files or an error
    async fn load_github_directory(
        &self,
        directory_path: &str,
    ) -> Result<LoadDirectoryResult, VfsError> {
        log::info!(
            "Starting load_github_directory with path: {}",
            directory_path
        );

        // Add an env var check to allow triggering a test panic
        if std::option_env!("RUST_TEST_PANIC").is_some() {
            log::warn!("Triggering intentional panic for stack trace testing");
            panic!("This is an intentional test panic to verify stack trace capture");
        }

        let normalized_path = normalize_path(directory_path);
        log::debug!("Normalized path: {}", normalized_path);

        let dir_path = if normalized_path.is_empty() || normalized_path.ends_with('/') {
            normalized_path
        } else {
            format!("{}/", normalized_path)
        };

        log::info!("Loading all files from GitHub directory: {}", dir_path);

        // Use GitHub API to fetch tree information for this directory
        let api_url = format!(
            "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
            self.github_config.owner, self.github_config.repo, self.github_config.branch
        );

        log::debug!("Fetching GitHub directory tree from: {}", api_url);

        let fetch_response = match fetch_url(&api_url).await {
            Ok(response) => {
                log::debug!("Successfully fetched GitHub API response");
                response
            }
            Err(e) => {
                log::error!("Error fetching GitHub API: {:?}", e);
                return Err(js_error_to_vfs(e, "Fetch GitHub API failed"));
            }
        };

        let status_js = match response_status(&fetch_response).await {
            Ok(status) => {
                log::debug!("Successfully got response status");
                status
            }
            Err(e) => {
                log::error!("Error getting response status: {:?}", e);
                return Err(js_error_to_vfs(e, "Get GitHub API status failed"));
            }
        };

        let status = match status_js.as_f64().map(|f| f as u16) {
            Some(s) => {
                log::debug!("Response status code: {}", s);
                s
            }
            None => {
                log::error!("Status is not a number: {:?}", status_js);
                return Err(js_error_to_vfs(
                    status_js,
                    "GitHub API status was not a number",
                ));
            }
        };

        if !(200..300).contains(&status) {
            log::warn!("GitHub API call failed: Status {}", status);
            return Err(VfsError::NetworkError(format!(
                "GitHub API call failed with status: {}",
                status
            )));
        }

        // Parse the response to get file information
        let response_bytes = match response_bytes(&fetch_response).await {
            Ok(bytes) => {
                log::debug!(
                    "Successfully got response bytes, length: {}",
                    bytes.length()
                );
                bytes
            }
            Err(e) => {
                log::error!("Error getting response bytes: {:?}", e);
                return Err(js_error_to_vfs(e, "Get GitHub API response bytes failed"));
            }
        };

        let response_text = match String::from_utf8(response_bytes.to_vec()) {
            Ok(text) => {
                log::debug!(
                    "Successfully converted bytes to text, length: {}",
                    text.len()
                );
                if text.len() < 1000 {
                    log::debug!("Response text: {}", text);
                } else {
                    log::debug!("Response text too long to log in full");
                }
                text
            }
            Err(e) => {
                log::error!("Error converting bytes to text: {:?}", e);
                return Err(VfsError::Other(format!(
                    "Failed to parse GitHub API response: {}",
                    e
                )));
            }
        };

        let tree_response: GitHubTreeResponse =
            match serde_json::from_str::<GitHubTreeResponse>(&response_text) {
                Ok(tree) => {
                    log::debug!(
                        "Successfully parsed GitHub tree JSON with {} items",
                        tree.tree.len()
                    );
                    tree
                }
                Err(e) => {
                    log::error!("Error parsing GitHub tree JSON: {:?}", e);
                    return Err(VfsError::Other(format!(
                        "Failed to parse GitHub tree JSON: {}",
                        e
                    )));
                }
            };

        // Check if the tree was truncated (too large)
        if tree_response.truncated {
            log::warn!("GitHub tree response was truncated. Some files might be missing.");
        }

        let root_path_prefix = if self.github_config.root_path.is_empty() {
            "".to_string()
        } else {
            format!("{}/", self.github_config.root_path.trim_matches('/'))
        };
        log::debug!("Root path prefix: '{}'", root_path_prefix);

        // Track directories to create
        let mut directories = std::collections::HashSet::new();

        // Filter files to only include those in the requested directory
        let mut files_to_load = Vec::new();

        for item in tree_response.tree {
            // Skip if not a blob (file)
            if item.type_field != "blob" {
                continue;
            }

            // Account for root_path from config
            let relative_path = if item.path.starts_with(&root_path_prefix) {
                item.path[root_path_prefix.len()..].to_string()
            } else {
                // Skip if not under the configured root path
                continue;
            };

            // Skip if not under the requested directory
            if !relative_path.starts_with(&dir_path) && !dir_path.is_empty() {
                continue;
            }

            log::debug!("Adding file to load queue: {}", relative_path);

            // Add file to loading list
            files_to_load.push(relative_path.clone());

            // Track all parent directories for this file
            let mut current_dir = get_parent_directory(&relative_path);
            while !current_dir.is_empty() {
                directories.insert(current_dir.clone());
                current_dir = get_parent_directory(&current_dir);
            }
        }

        log::info!(
            "Found {} files to load and {} directories to create",
            files_to_load.len(),
            directories.len()
        );

        // First create all necessary directories
        for dir in &directories {
            log::debug!("Creating directory: {}", dir);
            if let Err(e) = self.create_directory(dir).await {
                log::warn!("Failed to create directory {}: {:?}", dir, e);
                // Continue anyway
            }
        }

        // In WebAssembly, we can't use tokio::spawn for threading
        // so we'll load the files sequentially
        let mut successes = 0;
        let mut failures = 0;
        let mut loaded_files = Vec::new();

        log::info!("Starting to load {} files", files_to_load.len());

        for (index, file_path) in files_to_load.iter().enumerate() {
            log::debug!(
                "Loading file {}/{}: {}",
                index + 1,
                files_to_load.len(),
                file_path
            );

            match self.read_file(file_path).await {
                Ok(content) => {
                    log::debug!(
                        "Successfully loaded file: {} ({} bytes)",
                        file_path,
                        content.len()
                    );
                    successes += 1;
                    loaded_files.push(LoadedFile {
                        path: file_path.to_string(),
                        size: content.len(),
                    });
                }
                Err(e) => {
                    log::warn!("Failed to load file {}: {:?}", file_path, e);
                    failures += 1;
                }
            }
        }

        log::info!(
            "Directory load complete. Loaded {} files ({} bytes), {} failures.",
            successes,
            loaded_files.iter().map(|f| f.size).sum::<usize>(),
            failures
        );

        Ok(LoadDirectoryResult {
            total_files: successes + failures,
            successful_files: successes,
            failed_files: failures,
            loaded_files,
        })
    }
}

/// Represents a file that was loaded from GitHub
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoadedFile {
    /// Path to the file that was loaded
    pub path: String,
    /// Size of the file in bytes
    pub size: usize,
}

/// Result of loading a directory from GitHub
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoadDirectoryResult {
    /// Total number of files attempted to load
    pub total_files: usize,
    /// Number of files successfully loaded
    pub successful_files: usize,
    /// Number of files that failed to load
    pub failed_files: usize,
    /// Information about each successfully loaded file
    pub loaded_files: Vec<LoadedFile>,
}

/// GitHub API tree response structure
#[derive(Debug, Deserialize)]
struct GitHubTreeResponse {
    tree: Vec<GitHubTreeItem>,
    truncated: bool,
}

/// GitHub API tree item structure
#[derive(Debug, Deserialize)]
struct GitHubTreeItem {
    path: String,
    #[serde(rename = "type")]
    type_field: String,
    size: Option<usize>,
}

// Helper to guess file type from path (extension)
fn guess_file_type(path: &str) -> String {
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

impl VercelKvVfs {
    pub fn new() -> Result<Self, VfsError> {
        let github_config = GitHubConfig::from_env()?;

        Ok(VercelKvVfs {
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            github_config,
        })
    }
}
