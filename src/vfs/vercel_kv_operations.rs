use crate::utils::http_wasm::{web_get_async, ProxyConfig};
use crate::utils::string::normalize_dir_path;
use crate::vfs::vercel_kv_helpers::*;
use crate::vfs::vercel_kv_store::create_file_attributes;
use crate::vfs::vercel_kv_types::*;
use crate::vfs::vercel_kv_vfs::VercelKvVfs;
use crate::vfs::VfsError;
use case_insensitive_string::CaseInsensitiveString;
use std::collections::HashMap;

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
        if let Some(content) = self.store.read_from_memory_cache(&normalized_path).await {
            log::debug!("Cache hit for: {}", normalized_path);
            return Ok(content);
        }

        // 2. Check if this is a placeholder file
        if let Ok(is_placeholder) = self.store.is_placeholder(&normalized_path).await {
            if is_placeholder {
                log::debug!(
                    "File is a placeholder, loading from GitHub: {}",
                    normalized_path
                );

                // Try to load from GitHub
                // Map root '/' to empty string for GitHub API
                let github_path = if normalized_path.starts_with('/') {
                    &normalized_path[1..]
                } else {
                    &normalized_path
                };

                match self.load_github_file(github_path).await {
                    Ok(content) => {
                        // Update the cache and return the content
                        self.store
                            .write_to_memory_cache(&normalized_path, content.clone())
                            .await;
                        return Ok(content);
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to load placeholder file from GitHub: {}, error: {:?}",
                            normalized_path,
                            e
                        );
                        // Continue to try to load from KV
                    }
                }
            }
        }

        // 3. Try to read from KV store
        if let Ok(Some(content)) = self.store.read_from_kv(&normalized_path).await {
            log::debug!("KV hit for: {}", normalized_path);

            // Update memory cache
            self.store
                .write_to_memory_cache(&normalized_path, content.clone())
                .await;

            return Ok(content);
        }

        // 4. Try to load from GitHub if not found locally
        log::debug!("File not found locally, trying GitHub: {}", normalized_path);

        // Map root '/' to empty string for GitHub API
        let github_path = if normalized_path.starts_with('/') {
            &normalized_path[1..]
        } else {
            &normalized_path
        };

        match self.load_github_file(github_path).await {
            Ok(content) => {
                // Update the cache and return the content
                self.store
                    .write_to_memory_cache(&normalized_path, content.clone())
                    .await;

                // Create file attributes
                let attributes = create_file_attributes(&normalized_path, content.len(), "cloud");

                // Update metadata cache
                self.store
                    .write_to_metadata_cache(&normalized_path, attributes.clone())
                    .await;

                // Save to KV in the background
                self.store
                    .write_to_kv_background(normalized_path.clone(), content.clone());
                self.store
                    .write_metadata_to_kv_background(normalized_path, attributes);

                return Ok(content);
            }
            Err(e) => {
                log::error!(
                    "Failed to load file from GitHub: {}, error: {:?}",
                    normalized_path,
                    e
                );
                return Err(VfsError::NotFound(format!(
                    "File not found in any storage: {}",
                    normalized_path
                )));
            }
        }
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
        self.store
            .write_to_memory_cache(&normalized_path, content.clone())
            .await;

        // Create file attributes
        let attributes = create_file_attributes(&normalized_path, content.len(), "user");

        // Update metadata cache
        self.store
            .write_to_metadata_cache(&normalized_path, attributes.clone())
            .await;

        // Store file and its metadata in background
        self.store
            .write_to_kv_background(normalized_path.clone(), content);
        self.store
            .write_metadata_to_kv_background(normalized_path.clone(), attributes);

        // Ensure parent directories exist
        let parent = get_parent_directory(&normalized_path);
        if !parent.is_empty() {
            if let Err(e) = self.create_directory(&parent).await {
                log::error!("Failed to create parent directory {}: {:?}", parent, e);
                // Continue even if parent directory creation fails
            }
        }

        Ok(())
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
        if self.store.exists_in_memory_cache(&normalized_path).await {
            log::trace!("Exists check (memory hit): {}", normalized_path);
            return Ok(true);
        }

        if self.store.exists_in_metadata_cache(&normalized_path).await {
            log::trace!("Exists check (metadata memory hit): {}", normalized_path);
            return Ok(true);
        }

        // First check if this is explicitly a directory path (ends with /)
        if is_directory_path(&normalized_path) {
            if let Ok(exists) = self.store.directory_exists_in_kv(&normalized_path).await {
                if exists {
                    log::trace!("Exists check (directory marker found): {}", normalized_path);
                    return Ok(true);
                }
            }
        }
        // If not explicitly a directory, also check if it might be a directory without trailing slash
        else {
            // Try to check with a trailing slash (as a directory)
            let dir_path = normalize_dir_path(&normalized_path);
            if let Ok(exists) = self.store.directory_exists_in_kv(&dir_path).await {
                if exists {
                    log::trace!(
                        "Exists check (directory without trailing slash): {} -> {}",
                        normalized_path,
                        dir_path
                    );
                    return Ok(true);
                }
            }
        }

        // Check KV storage for file content
        if let Ok(exists) = self.store.exists_in_kv(&normalized_path).await {
            if exists {
                log::trace!("Exists check (KV content found): {}", normalized_path);
                return Ok(true);
            }
        }

        // Check for metadata in KV
        if let Ok(Some(_)) = self.store.read_metadata_from_kv(&normalized_path).await {
            log::trace!("Exists check (KV metadata found): {}", normalized_path);
            return Ok(true);
        }

        // File doesn't exist locally, check GitHub if configuration is available
        if !self.github_config.owner.is_empty() && !self.github_config.repo.is_empty() {
            // Get the parent directory path
            let parent_dir = get_parent_directory(&normalized_path);

            // Check if we have a GitHub cache key for tracking loaded directories
            let dir_cache_key = format!("github_loaded_dir:{}", parent_dir);
            let mut check_github = true;

            // Check if we've already loaded this directory
            if let Ok(Some(loaded_status)) = self.store.read_from_kv(&dir_cache_key).await {
                if loaded_status == b"loaded" {
                    log::trace!("Directory {} was previously loaded from GitHub", parent_dir);
                    check_github = false;
                }
            }

            if check_github {
                log::debug!(
                    "Directory {} not loaded yet, loading from GitHub",
                    parent_dir
                );

                // For single files, check if the specific file exists first
                if !is_directory_path(&normalized_path) {
                    // Try to load file info without downloading content
                    match self.load_github_file_info_impl(&normalized_path).await {
                        Ok(_) => {
                            log::debug!("File exists on GitHub: {}", normalized_path);

                            // Create a placeholder for this file
                            let _file_path = if normalized_path.starts_with('/') {
                                &normalized_path[1..]
                            } else {
                                &normalized_path
                            };

                            match self.load_github_directory_impl(true, false).await {
                                Ok(_) => {
                                    // Mark this directory as loaded
                                    let _ = self
                                        .store
                                        .write_to_kv(&dir_cache_key, b"loaded".to_vec().as_slice())
                                        .await;
                                    return Ok(true);
                                }
                                Err(e) => {
                                    log::warn!(
                                        "Failed to load parent directory from GitHub: {:?}",
                                        e
                                    );
                                }
                            }

                            return Ok(true);
                        }
                        Err(VfsError::NotFound(_)) => {
                            log::debug!("File does not exist on GitHub: {}", normalized_path);
                        }
                        Err(e) => {
                            log::warn!("Error checking file on GitHub: {:?}", e);
                        }
                    }
                }

                // Load the entire directory in shallow mode (create placeholders) and non-recursive mode
                match self.load_github_directory_impl(true, false).await {
                    Ok(result) => {
                        log::debug!(
                            "Loaded {} files from GitHub directory {}",
                            result.successful_files,
                            parent_dir
                        );

                        // Mark this directory as loaded
                        let _ = self
                            .store
                            .write_to_kv(&dir_cache_key, b"loaded".to_vec().as_slice())
                            .await;

                        // Check again if our file exists after loading the directory
                        if self.store.exists_in_metadata_cache(&normalized_path).await {
                            return Ok(true);
                        }

                        for loaded_file in result.loaded_files {
                            if loaded_file.path == normalized_path {
                                return Ok(true);
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to load GitHub directory: {:?}", e);
                    }
                }
            }
        }

        log::trace!("Exists check (not found anywhere): {}", normalized_path);
        Ok(false)
    }

    /// Delete a file from the VFS
    pub(crate) async fn delete_file_impl(&self, path: &str) -> Result<(), VfsError> {
        let normalized_path = normalize_path(path);

        // Don't allow deleting directory paths directly
        if is_directory_path(&normalized_path) {
            return Err(VfsError::IsDirectory(normalized_path));
        }

        // Check if the file exists
        if !self.exists(&normalized_path).await? {
            return Err(VfsError::NotFound(normalized_path));
        }

        // Remove from memory cache
        self.store.remove_from_memory_cache(&normalized_path).await;
        self.store
            .remove_from_metadata_cache(&normalized_path)
            .await;

        // Delete from KV storage
        self.store.delete_from_kv(&normalized_path).await?;
        self.store.delete_metadata_from_kv(&normalized_path).await?;
        self.store
            .clear_placeholder_status(&normalized_path)
            .await?;

        Ok(())
    }

    /// Helper to load a file from GitHub
    async fn load_github_file(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        let url = self.github_config.get_raw_url(path);
        log::debug!("Fetching from GitHub: {}", url);

        // Prepare headers with authorization if token is available
        let mut headers = HashMap::new();
        if let Some(token) = &self.github_config.auth_token {
            headers.insert(
                CaseInsensitiveString::new("Authorization"),
                format!("token {}", token),
            );
        }
        headers.insert(
            CaseInsensitiveString::new("Accept"),
            "application/vnd.github.v3.raw".to_string(),
        );
        headers.insert(
            CaseInsensitiveString::new("User-Agent"),
            "subconverter-rs".to_string(),
        );

        // Make the request
        let proxy_config = ProxyConfig::default();
        let fetch_result = web_get_async(&url, &proxy_config, Some(&headers)).await;

        match fetch_result {
            Ok(response) => {
                // Check if the response is successful (2xx)
                if (200..300).contains(&response.status) {
                    log::debug!("Successfully fetched raw file from GitHub");

                    // Check if we got rate limit headers
                    if let Some(rate_limit) = response.headers.get("x-ratelimit-remaining") {
                        log::info!("GitHub API rate limit remaining: {}", rate_limit);
                    }

                    Ok(response.body.into_bytes())
                } else if response.status == 404 {
                    log::error!("File not found on GitHub (404): {}", path);
                    return Err(VfsError::NotFound(format!(
                        "File not found on GitHub: {}",
                        path
                    )));
                } else {
                    log::error!(
                        "GitHub API returned error status {}: {}",
                        response.status,
                        response.body
                    );
                    return Err(VfsError::NetworkError(format!(
                        "GitHub API returned error status {}: {}",
                        response.status, response.body
                    )));
                }
            }
            Err(e) => {
                log::error!("Error fetching GitHub raw file: {}", e.message);
                return Err(VfsError::NetworkError(format!(
                    "GitHub raw file request failed: {}",
                    e.message
                )));
            }
        }
    }
}
