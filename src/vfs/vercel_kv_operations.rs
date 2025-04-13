use crate::vfs::vercel_kv_helpers::*;
use crate::vfs::vercel_kv_js_bindings::*;
use crate::vfs::vercel_kv_store::create_file_attributes;
use crate::vfs::vercel_kv_types::*;
use crate::vfs::vercel_kv_vfs::VercelKvVfs;
use crate::vfs::VfsError;

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

        // For directory paths, check if directory marker exists
        if is_directory_path(&normalized_path) {
            if let Ok(exists) = self.store.directory_exists_in_kv(&normalized_path).await {
                if exists {
                    log::trace!("Exists check (directory marker found): {}", normalized_path);
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

        // File doesn't exist locally, try to check if it exists in GitHub
        // But only if GitHub config is available
        if !self.github_config.owner.is_empty() && !self.github_config.repo.is_empty() {
            // For now, just return false
            // In a real implementation, we'd check if the file exists in GitHub
            log::trace!(
                "Exists check (not found locally, not checking GitHub): {}",
                normalized_path
            );
            return Ok(false);
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
                    "File not found on GitHub: {}",
                    path
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

        Ok(uint8_array.to_vec())
    }
}
