use crate::log_debug;
use crate::utils::system::safe_system_time;
use crate::vfs::vercel_kv_github::{GitHubConfig, GitHubTreeResponse};
use crate::vfs::vercel_kv_helpers::*;
use crate::vfs::vercel_kv_js_bindings::*;
use crate::vfs::vercel_kv_types::*;
use crate::vfs::vercel_kv_vfs::VercelKvVfs;
use crate::vfs::VfsError;
use crate::vfs::VirtualFileSystem;
use js_sys::Uint8Array;
use log::debug;
use serde_wasm_bindgen;
use std::collections::{HashMap, HashSet};
use std::time::UNIX_EPOCH;
use wasm_bindgen_futures;

impl VercelKvVfs {
    /// Read file attributes
    pub(crate) async fn read_file_attributes_impl(
        &self,
        path: &str,
    ) -> Result<FileAttributes, VfsError> {
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

    /// List directory contents
    pub(crate) async fn list_directory_impl(
        &self,
        path: &str,
    ) -> Result<Vec<DirectoryEntry>, VfsError> {
        log_debug!("Listing directory: '{}'", path);
        let path = normalize_path(path);

        // For root directory, always allow the check
        let dir_exists = if path.is_empty() {
            true
        } else {
            self.exists(&path).await?
        };

        // Check if the directory exists (except for root which always exists)
        if !dir_exists {
            log_debug!("Directory '{}' does not exist", path);
            return Ok(Vec::new());
        }

        // List all files under the directory
        let prefix = format!("{}{}", path, if path.ends_with('/') { "" } else { "/" });
        log_debug!("Using prefix for KV scan: '{}'", prefix);

        let mut files: Vec<DirectoryEntry> = Vec::new();

        // Get the keys from kv_list and deserialize to Vec<String>
        let js_keys = match kv_list(&prefix).await {
            Ok(js_value) => {
                // Convert JsValue to Vec<String>
                let keys: Vec<String> = match serde_wasm_bindgen::from_value(js_value) {
                    Ok(k) => k,
                    Err(e) => {
                        log_debug!("Failed to deserialize keys: {:?}", e);
                        Vec::new()
                    }
                };

                log_debug!(
                    "KV list returned {} keys for prefix '{}'",
                    keys.len(),
                    prefix
                );

                if keys.is_empty() {
                    log_debug!("No keys found for prefix '{}', checking GitHub...", prefix);
                    // Try to load from GitHub if no keys are found
                    match self.load_github_directory_impl(&path, true, false).await {
                        Ok(load_result) => {
                            log_debug!(
                                "GitHub load for '{}' returned {} entries",
                                path,
                                load_result.loaded_files.len()
                            );

                            // Convert LoadDirectoryResult to Vec<DirectoryEntry>
                            let mut entries = Vec::new();

                            // Create a set to track unique directory names at this level
                            let mut direct_subdirs = std::collections::HashSet::new();

                            for file in &load_result.loaded_files {
                                let file_path = &file.path;

                                // Skip if this isn't a direct child of the requested directory
                                let rel_path = if file_path.starts_with(&path) {
                                    // Remove the directory prefix to get the relative path
                                    let prefix_len = if path.ends_with('/') {
                                        path.len()
                                    } else {
                                        path.len() + 1
                                    };
                                    if file_path.len() <= prefix_len {
                                        continue; // Skip entries that are the directory itself
                                    }
                                    &file_path[prefix_len..]
                                } else {
                                    continue; // Skip entries that don't belong to this directory
                                };

                                // Check if this is a direct child or a deeper descendant
                                if let Some(slash_pos) = rel_path.find('/') {
                                    // This is a deeper path - extract the first directory name
                                    let dir_name = &rel_path[0..slash_pos];
                                    if !dir_name.is_empty()
                                        && direct_subdirs.insert(dir_name.to_string())
                                    {
                                        // Add as directory entry
                                        let dir_path = format!(
                                            "{}{}/",
                                            if path.ends_with('/') {
                                                &path
                                            } else {
                                                &format!("{}/", path)
                                            },
                                            dir_name
                                        );

                                        log_debug!(
                                            "Adding direct subdirectory from GitHub: '{}'",
                                            dir_name
                                        );
                                        entries.push(DirectoryEntry {
                                            name: dir_name.to_string(),
                                            path: dir_path,
                                            is_directory: true,
                                            attributes: Some(FileAttributes {
                                                is_directory: true,
                                                ..Default::default()
                                            }),
                                        });
                                    }
                                } else {
                                    // This is a direct file child
                                    if let Ok(attrs) = self.read_file_attributes(file_path).await {
                                        entries.push(DirectoryEntry {
                                            name: get_filename(file_path),
                                            path: file_path.clone(),
                                            is_directory: false,
                                            attributes: Some(attrs),
                                        });
                                    } else {
                                        entries.push(DirectoryEntry {
                                            name: get_filename(file_path),
                                            path: file_path.clone(),
                                            is_directory: false,
                                            attributes: None,
                                        });
                                    }
                                }
                            }

                            // Return the converted entries
                            return Ok(entries);
                        }
                        Err(e) => {
                            log_debug!("GitHub load for '{}' failed: {:?}", path, e);
                        }
                    }
                }

                keys
            }
            Err(e) => {
                log_debug!("Error listing keys with prefix '{}': {:?}", prefix, e);
                Vec::new()
            }
        };

        log_debug!("Processing {} keys from KV store", js_keys.len());
        let prefix_len = prefix.len();

        // Get unique directory names and file paths
        let mut dir_names = HashSet::new();
        for key in &js_keys {
            log_debug!("Processing key: '{}'", key);

            if key.len() <= prefix_len {
                log_debug!(
                    "Key '{}' is shorter than or equal to prefix length ({}), skipping",
                    key,
                    prefix_len
                );
                continue;
            }

            let rel_path = &key[prefix_len..];
            if rel_path.is_empty() {
                log_debug!("Relative path for key '{}' is empty, skipping", key);
                continue;
            }

            // For files directly under this directory
            if !rel_path.contains('/') {
                log_debug!("Found file in directory: '{}'", rel_path);
                let fpath = format!("{}{}", path, rel_path);
                // Get file attributes
                if let Some(attrs) = self.read_file_attributes(&fpath).await.ok() {
                    log_debug!("Found attributes for file: '{}'", fpath);
                    files.push(DirectoryEntry {
                        name: rel_path.to_string(),
                        path: fpath,
                        is_directory: false,
                        attributes: Some(attrs),
                    });
                } else {
                    log_debug!("No attributes found for file: '{}'", fpath);
                    // Add a basic entry if attributes not found
                    files.push(DirectoryEntry {
                        name: rel_path.to_string(),
                        path: fpath,
                        is_directory: false,
                        attributes: None,
                    });
                }
            } else {
                // For subdirectories
                let dir_name = match rel_path.find('/') {
                    Some(pos) => {
                        let dir = &rel_path[0..pos];
                        log_debug!("Found subdirectory: '{}'", dir);
                        dir
                    }
                    None => {
                        log_debug!("Failed to find subdirectory in path: '{}'", rel_path);
                        continue;
                    }
                };

                if !dir_name.is_empty() && dir_names.insert(dir_name) {
                    log_debug!("Adding directory entry: '{}'", dir_name);
                    files.push(DirectoryEntry {
                        name: dir_name.to_string(),
                        path: format!("{}{}/", path, dir_name),
                        is_directory: true,
                        attributes: None,
                    });
                }
            }
        }

        log_debug!("Returning {} entries for directory '{}'", files.len(), path);
        Ok(files)
    }

    /// Create directory
    pub(crate) async fn create_directory_impl(&self, path: &str) -> Result<(), VfsError> {
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
}
