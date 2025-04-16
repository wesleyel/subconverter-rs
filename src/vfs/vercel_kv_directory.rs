use crate::log_debug;
use crate::utils::string::{
    build_dir_entry_path, build_file_entry_path, normalize_dir_path, normalize_file_path,
};
use crate::vfs::vercel_kv_helpers::*;
use crate::vfs::vercel_kv_store::{create_directory_attributes, create_file_attributes};
use crate::vfs::vercel_kv_types::*;
use crate::vfs::vercel_kv_vfs::VercelKvVfs;
use crate::vfs::VfsError;
use crate::vfs::VirtualFileSystem;
use std::collections::HashSet;

impl VercelKvVfs {
    /// Read file attributes
    pub(crate) async fn read_file_attributes_impl(
        &self,
        path: &str,
    ) -> Result<FileAttributes, VfsError> {
        let normalized_path = normalize_path(path);

        // Check memory cache first
        if let Some(attrs) = self.store.read_from_metadata_cache(&normalized_path).await {
            // If it's a placeholder with zero size, try to get actual size from GitHub
            if attrs.source_type == "placeholder" && attrs.size == 0 {
                // Try to load information from GitHub
                if let Ok(github_result) = self.load_github_file_info_impl(&normalized_path).await {
                    let mut updated_attrs = attrs.clone();
                    updated_attrs.size = github_result.size;

                    // Update metadata cache with actual size
                    self.store
                        .write_to_metadata_cache(&normalized_path, updated_attrs.clone())
                        .await;

                    // Update KV store in background
                    self.store.write_metadata_to_kv_background(
                        normalized_path.clone(),
                        updated_attrs.clone(),
                    );

                    return Ok(updated_attrs);
                }
            }
            return Ok(attrs);
        }

        // Check if it's a directory with a marker key
        if is_directory_path(&normalized_path) {
            if let Ok(exists) = self.store.directory_exists_in_kv(&normalized_path).await {
                if exists {
                    // It's a directory, return default directory attributes
                    let attrs = create_directory_attributes("system");
                    return Ok(attrs);
                }
            }
        }

        // Try to get metadata from KV
        match self.store.read_metadata_from_kv(&normalized_path).await {
            Ok(Some(attributes)) => {
                // If it's a placeholder with zero size, try to get actual size from GitHub
                if attributes.source_type == "placeholder" && attributes.size == 0 {
                    // Try to load information from GitHub
                    if let Ok(github_result) =
                        self.load_github_file_info_impl(&normalized_path).await
                    {
                        let mut updated_attrs = attributes.clone();
                        updated_attrs.size = github_result.size;

                        // Update metadata cache with actual size
                        self.store
                            .write_to_metadata_cache(&normalized_path, updated_attrs.clone())
                            .await;

                        // Update KV store in background
                        self.store.write_metadata_to_kv_background(
                            normalized_path.clone(),
                            updated_attrs.clone(),
                        );

                        return Ok(updated_attrs);
                    }
                }

                // Cache the attributes
                self.store
                    .write_to_metadata_cache(&normalized_path, attributes.clone())
                    .await;
                return Ok(attributes);
            }
            Ok(None) => {
                // No metadata found, continue to check if file exists
            }
            Err(e) => {
                log::error!("Failed to read metadata from KV: {:?}", e);
                // Continue to check if file exists
            }
        }

        // If we can't find metadata, but the file exists, create default attributes
        if self.exists(&normalized_path).await? {
            // For files, we need to read the content to get the size
            if !is_directory_path(&normalized_path) {
                // Try to get size information from GitHub first
                if let Ok(github_result) = self.load_github_file_info_impl(&normalized_path).await {
                    let attributes =
                        create_file_attributes(&normalized_path, github_result.size, "cloud");

                    // Cache the attributes
                    self.store
                        .write_to_metadata_cache(&normalized_path, attributes.clone())
                        .await;

                    return Ok(attributes);
                }

                // Fallback to reading the content if GitHub info is not available
                let content = self.read_file(&normalized_path).await?;
                let attributes = create_file_attributes(&normalized_path, content.len(), "user");

                // Cache the attributes
                self.store
                    .write_to_metadata_cache(&normalized_path, attributes.clone())
                    .await;

                return Ok(attributes);
            } else {
                // It's a directory that exists (maybe implicitly)
                let attributes = create_directory_attributes("system");
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
        skip_github_load: bool,
    ) -> Result<Vec<DirectoryEntry>, VfsError> {
        log_debug!("Listing directory: '{}'", path);
        let path = normalize_path(path);
        log_debug!("Normalized path: '{}'", path);

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
        let prefix = if path.is_empty() {
            "".to_string() // For root, use empty prefix, not "/"
        } else {
            normalize_dir_path(&path)
        };
        log_debug!("Using prefix for KV scan: '{}'", prefix);

        let mut files: Vec<DirectoryEntry> = Vec::new();

        // Get the keys from kv_list and deserialize to Vec<String>
        let keys = match self.store.list_keys_with_prefix(&prefix).await {
            Ok(keys) => {
                log_debug!(
                    "KV list returned {} keys for prefix '{}'",
                    keys.len(),
                    prefix
                );

                if keys.is_empty() && !skip_github_load {
                    log_debug!("No keys found for prefix '{}', checking GitHub...", prefix);
                    // Try to load from GitHub if no keys are found
                    match self.load_github_directory_impl(true, false).await {
                        Ok(result) => {
                            log::info!(
                                "Loaded {} files from GitHub directory {}",
                                result.successful_files,
                                path
                            );

                            // Create a set to track unique directory names at this level
                            let mut direct_subdirs = std::collections::HashSet::new();

                            for file in &result.loaded_files {
                                let file_path = &file.path;
                                // Skip if this isn't a direct child of the requested directory
                                let rel_path = if file_path.starts_with(&path) {
                                    // Remove the directory prefix to get the relative path
                                    let prefix_len = if path.ends_with('/') {
                                        path.len()
                                    } else if path.is_empty() && !file_path.starts_with('/') {
                                        0
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
                                    let dir_name = &rel_path[0..slash_pos];
                                    if !dir_name.is_empty()
                                        && direct_subdirs.insert(dir_name.to_string())
                                    {
                                        // Add as directory entry
                                        let dir_path_prefix = if path.ends_with('/') {
                                            path.to_string()
                                        } else {
                                            format!("{}/", path)
                                        };
                                        let dir_path = format!("{}{}/", dir_path_prefix, dir_name);

                                        log_debug!(
                                            "Adding direct subdirectory from GitHub: '{}'",
                                            dir_name
                                        );
                                        files.push(DirectoryEntry {
                                            name: dir_name.to_string(),
                                            path: dir_path,
                                            is_directory: true,
                                            attributes: Some(create_directory_attributes("cloud")),
                                        });
                                    }
                                } else {
                                    // Check if we already have this file in our files list
                                    let file_exists = files
                                        .iter()
                                        .any(|entry| !entry.is_directory && entry.path == rel_path);
                                    // This is a direct file child
                                    log_debug!(
                                        "GitHub file: '{}' from rel_path '{}', file_exists: '{}'",
                                        rel_path,
                                        rel_path,
                                        file_exists
                                    );

                                    if !file_exists {
                                        // Use the file size directly from the GitHub API result
                                        // According to GitHub docs, the trees API provides file sizes
                                        let attrs = create_file_attributes(
                                            &file_path,
                                            file.size, // Use the size from GitHub API
                                            if file.is_placeholder {
                                                "placeholder"
                                            } else {
                                                "cloud"
                                            },
                                        );

                                        files.push(DirectoryEntry {
                                            name: get_filename(file_path),
                                            path: normalize_file_path(file_path),
                                            is_directory: false,
                                            attributes: Some(attrs),
                                        });
                                    }
                                }
                            }

                            // Return the converted entries
                            return Ok(files);
                        }
                        Err(e) => {
                            log_debug!("GitHub load for '{}' failed: {:?}", path, e);
                        }
                    }
                }

                // Filter out internal keys and get unique real paths
                self.store.get_unique_real_paths_filtered(&keys).await
            }
            Err(e) => {
                log_debug!("Error listing keys with prefix '{}': {:?}", prefix, e);
                Vec::new()
            }
        };

        log_debug!("Processing {} unique file paths", keys.len());
        let prefix_len = prefix.len();
        log_debug!("Prefix length: {}, prefix: '{}'", prefix_len, prefix);

        // Get unique directory names and file paths
        let mut dir_names = HashSet::new();
        for key in &keys {
            log_debug!("Processing file path: '{}'", key);

            // Handle the case where path is empty (root directory)
            let rel_path = if prefix.is_empty() {
                key.clone()
            } else {
                // Make sure key starts with prefix before slicing
                if !key.starts_with(&prefix) {
                    log_debug!(
                        "Path '{}' doesn't start with prefix '{}', skipping",
                        key,
                        prefix
                    );
                    continue;
                }
                key[prefix_len..].to_string()
            };

            log_debug!(
                "File: '{}', prefix: '{}', resulting rel_path: '{}'",
                key,
                prefix,
                rel_path
            );

            if rel_path.is_empty() {
                log_debug!("Relative path for file '{}' is empty, skipping", key);
                continue;
            }

            // For files directly under this directory
            if !rel_path.contains('/') {
                log_debug!("Found file in directory: '{}'", rel_path);
                let fpath = build_file_entry_path(&path, &rel_path);
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
                        log_debug!(
                            "Found subdirectory: '{}' at position {} in rel_path '{}'",
                            dir,
                            pos,
                            rel_path
                        );
                        dir.to_string()
                    }
                    None => {
                        log_debug!("Failed to find subdirectory in path: '{}'", rel_path);
                        continue;
                    }
                };

                if !dir_name.is_empty() && dir_names.insert(dir_name.clone()) {
                    log_debug!("Adding directory entry: '{}'", dir_name);
                    let dir_full_path = build_dir_entry_path(&path, &dir_name);
                    log_debug!("Directory full path: '{}'", dir_full_path);
                    files.push(DirectoryEntry {
                        name: dir_name,
                        path: dir_full_path,
                        is_directory: true,
                        attributes: None,
                    });
                }
            }
        }

        // Check if we need to load from GitHub when we have KV entries but might be missing subdirectories
        if !skip_github_load && (files.is_empty() || (files.len() < 10 && path.is_empty())) {
            log_debug!(
                "Attempting to load from GitHub to supplement directory listing for '{}'",
                path
            );
            // Try to supplement with GitHub data
            match self.load_github_directory_impl(true, false).await {
                Ok(result) => {
                    log::info!(
                        "Loaded {} files from GitHub directory {}",
                        result.successful_files,
                        path
                    );

                    // Convert LoadDirectoryResult to Vec<DirectoryEntry>
                    // Create a set to track unique directory names at this level
                    let mut direct_subdirs = std::collections::HashSet::new();

                    for file in &result.loaded_files {
                        let file_path = &file.path;
                        log_debug!("Processing GitHub file: '{}'", file_path);

                        // Skip if this isn't a direct child of the requested directory
                        let rel_path = if file_path.starts_with(&path) {
                            // Remove the directory prefix to get the relative path
                            let prefix_len = if path.ends_with('/') {
                                path.len()
                            } else {
                                path.len() + 1
                            };

                            if file_path.len() <= prefix_len {
                                log_debug!(
                                    "Skipping file '{}' as it's the directory itself",
                                    file_path
                                );
                                continue; // Skip entries that are the directory itself
                            }

                            let rel = &file_path[prefix_len..];
                            log_debug!("GitHub relative path: '{}'", rel);
                            rel
                        } else {
                            log_debug!(
                                "Skipping file '{}' as it doesn't belong to directory '{}'",
                                file_path,
                                path
                            );
                            continue; // Skip entries that don't belong to this directory
                        };

                        // Check if this is a direct child or a deeper descendant
                        if let Some(slash_pos) = rel_path.find('/') {
                            // This is a deeper path - extract the first directory name
                            let dir_name = &rel_path[0..slash_pos];
                            log_debug!(
                                "GitHub directory name: '{}' from rel_path '{}'",
                                dir_name,
                                rel_path
                            );

                            if !dir_name.is_empty() && direct_subdirs.insert(dir_name.to_string()) {
                                // Check if we already have this directory in our files list
                                let dir_exists = files
                                    .iter()
                                    .any(|entry| entry.is_directory && entry.name == dir_name);

                                if !dir_exists {
                                    // Add as directory entry
                                    let dir_path = build_dir_entry_path(&path, dir_name);

                                    log_debug!(
                                        "Adding direct subdirectory from GitHub: '{}'",
                                        dir_name
                                    );

                                    files.push(DirectoryEntry {
                                        name: dir_name.to_string(),
                                        path: dir_path,
                                        is_directory: true,
                                        attributes: Some(create_directory_attributes("cloud")),
                                    });
                                }
                            }
                        } else {
                            // This is a direct file child
                            log_debug!("GitHub file: '{}' from rel_path '{}'", rel_path, rel_path);

                            // Check if we already have this file in our files list
                            let file_exists = files
                                .iter()
                                .any(|entry| !entry.is_directory && entry.name == rel_path);

                            if !file_exists {
                                // Use the file size directly from the GitHub API result
                                // According to GitHub docs, the trees API provides file sizes
                                let attrs = create_file_attributes(
                                    &file_path,
                                    file.size, // Use the size from GitHub API
                                    if file.is_placeholder {
                                        "placeholder"
                                    } else {
                                        "cloud"
                                    },
                                );

                                log_debug!("Adding file from GitHub: '{}'", file_path);

                                files.push(DirectoryEntry {
                                    name: get_filename(file_path),
                                    path: normalize_file_path(file_path),
                                    is_directory: false,
                                    attributes: Some(attrs),
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    log_debug!("GitHub load for '{}' failed: {:?}", path, e);
                }
            }
        }

        log_debug!("Returning {} entries for directory '{}'", files.len(), path);
        Ok(files)
    }

    /// Create directory
    pub(crate) async fn create_directory_impl(&self, path: &str) -> Result<(), VfsError> {
        let normalized_path = normalize_path(path);
        let dir_path = normalize_dir_path(&normalized_path);

        log::debug!("Creating directory: {}", dir_path);

        // Create directory marker
        let dir_attributes = create_directory_attributes("user");

        // Update metadata cache
        self.store
            .write_to_metadata_cache(&dir_path, dir_attributes.clone())
            .await;

        // Store directory marker and metadata
        match self.store.create_directory_in_kv(&dir_path).await {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        match self
            .store
            .write_metadata_to_kv(&dir_path, &dir_attributes)
            .await
        {
            Ok(_) => (),
            Err(e) => {
                log::error!("Failed to store directory metadata: {:?}", e);
                // Continue even if metadata storage fails
            }
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
