use crate::utils::system::safe_system_time;
use crate::vfs::vercel_kv_helpers::*;
use crate::vfs::vercel_kv_js_bindings::*;
use crate::vfs::vercel_kv_types::*;
use crate::vfs::vercel_kv_vfs::VercelKvVfs;
use crate::vfs::VfsError;
use serde_wasm_bindgen;
use std::time::UNIX_EPOCH;
use wasm_bindgen_futures;

// Define helper methods for VercelKvVfs to keep the implementation
// manageable and well-organized
impl VercelKvVfs {
    // File operations

    /// Read a file from the VFS
    pub(crate) async fn read_file_impl(&self, path: &str) -> Result<Vec<u8>, VfsError> {
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

        // 2. Check if this is a placeholder file
        let status_key = get_status_key(&normalized_path);
        let is_placeholder = match kv_get(&status_key).await {
            Ok(js_value) => {
                if js_value.is_null() || js_value.is_undefined() {
                    false
                } else {
                    let status: Vec<u8> = serde_wasm_bindgen::from_value(js_value)
                        .map_err(|_| false)
                        .unwrap_or(Vec::new());
                    if let Ok(status_str) = String::from_utf8(status.clone()) {
                        status_str == FILE_STATUS_PLACEHOLDER
                    } else {
                        false
                    }
                }
            }
            Err(_) => false,
        };

        if is_placeholder {
            log::debug!(
                "Found placeholder for: {}, fetching from GitHub",
                normalized_path
            );
            // Clear the placeholder status since we're about to load the real content
            let _ = kv_del(&status_key).await;
        }

        // 3. Check Vercel KV via JS binding if not a placeholder
        if !is_placeholder {
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
        }

        // 4. Fetch from GitHub via JS binding
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

        // 5. Store in memory cache AND Vercel KV
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
            source_type: "cloud".to_string(),
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
                        source_type: "user".to_string(),
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

    /// Write a file to the VFS
    pub(crate) async fn write_file_impl(
        &self,
        path: &str,
        content: Vec<u8>,
    ) -> Result<(), VfsError> {
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
            source_type: "user".to_string(),
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
                                source_type: "user".to_string(),
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

    /// Check if a file or directory exists
    pub(crate) async fn exists_impl(&self, path: &str) -> Result<bool, VfsError> {
        let normalized_path = normalize_path(path);

        // Special case: Root directory always exists
        if normalized_path.is_empty() {
            log::trace!("Exists check (root directory): true");
            return Ok(true);
        }

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

    /// Delete a file or directory
    pub(crate) async fn delete_file_impl(&self, path: &str) -> Result<(), VfsError> {
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
}
