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
use std::collections::{HashMap, HashSet};

impl VercelKvVfs {
    /// Read file or directory attributes
    pub(crate) async fn read_file_attributes_impl(
        &self,
        path: &str,
    ) -> Result<FileAttributes, VfsError> {
        let normalized_path = normalize_path(path);
        let is_dir = is_directory_path(&normalized_path); // Check if it looks like a directory path

        // --- Cache Check ---
        // Check memory metadata cache first
        if let Some(attrs) = self.store.read_from_metadata_cache(&normalized_path).await {
            // If it's a placeholder file with zero size, try to get actual size from GitHub
            if !attrs.is_directory && attrs.source_type == "placeholder" && attrs.size == 0 {
                if let Ok(github_result) = self.load_github_file_info_impl(&normalized_path).await {
                    let mut updated_attrs = attrs.clone();
                    updated_attrs.size = github_result.size;
                    updated_attrs.source_type = "cloud".to_string(); // Update status

                    // Update metadata cache
                    self.store
                        .write_to_metadata_cache(&normalized_path, updated_attrs.clone())
                        .await;
                    // Update KV store in background (write to parent dir metadata)
                    self.store.write_file_attributes_to_dir_kv_background(
                        normalized_path.clone(),
                        updated_attrs.clone(),
                    );
                    return Ok(updated_attrs);
                } else {
                    // If GitHub load fails, return the cached placeholder attributes
                    log::debug!(
                        "GitHub info lookup failed for placeholder '{}', returning cached attrs.",
                        normalized_path
                    );
                }
            }
            // Return cached attributes (could be file or directory)
            return Ok(attrs);
        }

        // --- KV Check (Directory Metadata) ---
        if !is_dir {
            // It's potentially a file, try reading its attributes from the parent directory's metadata
            match self
                .store
                .read_file_attributes_from_dir_kv(&normalized_path)
                .await
            {
                Ok(Some(attributes)) => {
                    // If it's a placeholder file with zero size, try to get actual size from GitHub
                    if attributes.source_type == "placeholder" && attributes.size == 0 {
                        if let Ok(github_result) =
                            self.load_github_file_info_impl(&normalized_path).await
                        {
                            let mut updated_attrs = attributes.clone();
                            updated_attrs.size = github_result.size;
                            updated_attrs.source_type = "cloud".to_string(); // Update status

                            // Update metadata cache
                            self.store
                                .write_to_metadata_cache(&normalized_path, updated_attrs.clone())
                                .await;
                            // Update KV store in background (write to parent dir metadata)
                            self.store.write_file_attributes_to_dir_kv_background(
                                normalized_path.clone(),
                                updated_attrs.clone(),
                            );
                            return Ok(updated_attrs);
                        } else {
                            log::debug!("GitHub info lookup failed for placeholder '{}', returning KV attrs.", normalized_path);
                        }
                    }
                    // Cache the attributes found in directory metadata
                    self.store
                        .write_to_metadata_cache(&normalized_path, attributes.clone())
                        .await;
                    return Ok(attributes);
                }
                Ok(None) => {
                    log::debug!(
                        "No attributes found for file '{}' in parent directory metadata.",
                        normalized_path
                    );
                    // No attributes found in parent dir metadata, continue checks...
                }
                Err(e) => {
                    log::error!(
                        "Failed to read attributes from directory KV for '{}': {:?}",
                        normalized_path,
                        e
                    );
                    // Continue checks, maybe it's a directory or exists implicitly
                }
            }
        }

        // --- Existence Check (KV Content/Marker) ---
        // Check if the path exists (either as content or directory marker)
        let exists = self.exists_impl(&normalized_path).await?;

        if exists {
            if is_dir {
                // It's a directory that exists (via marker). Return default directory attributes.
                log::debug!(
                    "Directory marker exists for '{}', returning default directory attributes.",
                    normalized_path
                );
                let attributes = create_directory_attributes(&normalized_path, "system"); // Pass path
                                                                                          // Cache these default directory attributes
                self.store
                    .write_to_metadata_cache(&normalized_path, attributes.clone())
                    .await;
                return Ok(attributes);
            } else {
                // It's a file that exists (has @@content key), but had no attributes in parent dir metadata.
                // This might happen if created outside the standard VFS write process or metadata failed.
                log::warn!("File content exists for '{}' but no attributes found in parent directory metadata. Creating default attributes.", normalized_path);

                // Try to get size information from GitHub first as it might be a cloud file without metadata entry
                if let Ok(github_result) = self.load_github_file_info_impl(&normalized_path).await {
                    let attributes =
                        create_file_attributes(&normalized_path, github_result.size, "cloud");
                    // Cache the attributes and write back to parent dir metadata
                    self.store
                        .write_to_metadata_cache(&normalized_path, attributes.clone())
                        .await;
                    self.store.write_file_attributes_to_dir_kv_background(
                        normalized_path.clone(),
                        attributes.clone(),
                    );
                    return Ok(attributes);
                }

                // Fallback: read content to get size (assume it's 'user' or 'unknown' source)
                return match self.store.read_from_kv(&normalized_path).await {
                    Ok(Some(content)) => {
                        let attributes =
                            create_file_attributes(&normalized_path, content.len(), "user");
                        // Cache the attributes and write back to parent dir metadata
                        self.store
                            .write_to_metadata_cache(&normalized_path, attributes.clone())
                            .await;
                        self.store.write_file_attributes_to_dir_kv_background(
                            normalized_path.clone(),
                            attributes.clone(),
                        );
                        Ok(attributes)
                    }
                    Ok(None) => {
                        // Content key exists but reading returns None? Should be rare.
                        log::error!(
                            "File content key exists for '{}' but read returned None.",
                            normalized_path
                        );
                        Err(VfsError::NotFound(format!(
                            "File content inconsistent for: {}",
                            normalized_path
                        )))
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to read content for existing file '{}' to determine size: {:?}",
                            normalized_path,
                            e
                        );
                        Err(e) // Propagate the read error
                    }
                };
            }
        }

        // --- Not Found ---
        log::debug!(
            "Attributes not found and path does not exist: '{}'",
            normalized_path
        );
        Err(VfsError::NotFound(format!(
            "File or directory not found: {}",
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
        let normalized_dir_path = normalize_path(path); // Path for the directory itself
        log_debug!("Normalized directory path: '{}'", normalized_dir_path);

        // --- Check if Directory Exists ---
        // Root always exists implicitly
        let dir_exists = if normalized_dir_path.is_empty() {
            true
        } else {
            // Check for the directory marker key
            self.store
                .directory_exists_in_kv(&normalized_dir_path)
                .await?
        };

        if !dir_exists {
            log_debug!(
                "Directory '{}' does not exist (no marker found)",
                normalized_dir_path
            );
            // Before returning empty, quickly check if it's implicitly defined by files inside it via kv_list
            // This handles cases where directories might not have been explicitly created but contain files.
            let kv_prefix = if normalized_dir_path.is_empty() {
                "".to_string()
            } else {
                // Use trailing slash for prefix listing
                format!("{}/", normalized_dir_path.trim_end_matches('/'))
            };
            match self.store.list_keys_with_prefix(&kv_prefix).await {
                Ok(keys) if !keys.is_empty() => {
                    log::debug!("Directory '{}' marker missing, but found {} keys inside. Proceeding with listing.", normalized_dir_path, keys.len());
                    // Proceed as if directory exists implicitly
                }
                _ => {
                    log::debug!("Directory '{}' marker missing and no keys found inside. Returning empty list.", normalized_dir_path);
                    return Ok(Vec::new()); // Truly doesn't exist or is empty
                }
            }
        }

        // --- Get Entries from Directory Metadata ---
        let mut entries_map: HashMap<String, DirectoryEntry> = HashMap::new();
        match self
            .store
            .read_directory_metadata_from_kv(&normalized_dir_path)
            .await
        {
            Ok(dir_metadata) => {
                log_debug!(
                    "Read directory metadata for '{}', found {} file entries.",
                    normalized_dir_path,
                    dir_metadata.files.len()
                );
                for (filename, attrs) in dir_metadata.files {
                    let entry_path = build_file_entry_path(&normalized_dir_path, &filename);
                    entries_map.insert(
                        filename.clone(),
                        DirectoryEntry {
                            name: filename,
                            path: entry_path,
                            is_directory: false, // Files stored in metadata are files
                            attributes: Some(attrs),
                        },
                    );
                }
            }
            Err(e) => {
                log::warn!(
                    "Failed to read directory metadata for '{}': {:?}. Listing may be incomplete.",
                    normalized_dir_path,
                    e
                );
                // Continue without metadata, rely on kv_list
            }
        }

        // --- Get Entries from KV List (Files and Subdirectories) ---
        let kv_list_prefix = if normalized_dir_path.is_empty() {
            "".to_string() // List everything from root
        } else {
            format!("{}/", normalized_dir_path.trim_end_matches('/')) // Ensure trailing slash for prefix scan
        };
        log_debug!("Using prefix for KV scan: '{}'", kv_list_prefix);

        let keys = match self.store.list_keys_with_prefix(&kv_list_prefix).await {
            Ok(keys) => {
                log_debug!(
                    "KV list returned {} keys for prefix '{}'",
                    keys.len(),
                    kv_list_prefix
                );
                keys
            }
            Err(e) => {
                log::error!(
                    "Error listing keys with prefix '{}': {:?}",
                    kv_list_prefix,
                    e
                );
                // If KV list fails but we have metadata, return metadata entries? Or error out?
                // For now, return what we have from metadata if any.
                return Ok(entries_map.values().cloned().collect());
            }
        };

        // --- Process KV Keys ---
        let prefix_len = kv_list_prefix.len();
        let mut subdirs_found = HashSet::new(); // Track unique subdirectory names found via KV list

        for key in &keys {
            log::trace!("Processing KV key: '{}'", key);

            // Extract path relative to the directory being listed
            let relative_path = if key.starts_with(&kv_list_prefix) && key.len() > prefix_len {
                &key[prefix_len..]
            } else if kv_list_prefix.is_empty() {
                // Handle root listing case
                key
            } else {
                log::trace!(
                    "Key '{}' doesn't match prefix '{}' or is too short, skipping",
                    key,
                    kv_list_prefix
                );
                continue; // Skip keys not directly under the prefix
            };

            log::trace!("Relative path: '{}'", relative_path);

            // Check if it's a direct child or nested further
            if let Some(slash_pos) = relative_path.find('/') {
                // It's nested. Extract the first component (subdirectory name)
                let subdir_name = &relative_path[..slash_pos];
                if !subdir_name.is_empty() && subdirs_found.insert(subdir_name.to_string()) {
                    log::trace!("Found potential subdirectory: '{}'", subdir_name);
                    // Add directory entry if not already present from metadata
                    if !entries_map.contains_key(subdir_name) {
                        let subdir_full_path =
                            build_dir_entry_path(&normalized_dir_path, subdir_name);
                        log::debug!(
                            "Adding subdirectory entry from KV list: '{}'",
                            subdir_full_path
                        );
                        entries_map.insert(
                            subdir_name.to_string(),
                            DirectoryEntry {
                                name: subdir_name.to_string(),
                                path: subdir_full_path,
                                is_directory: true,
                                // Fetching attributes here could be slow; maybe get later if needed?
                                // Or rely on the fact that a @@dir key exists.
                                attributes: None, // Initially None, could fetch if needed by caller
                            },
                        );
                    }
                }
            } else {
                // It's a direct child (file or maybe an empty dir marker?)
                let filename = relative_path;
                if filename.is_empty() {
                    continue;
                } // Should not happen with correct prefix logic

                // We only care about files (`@@content`) here, as metadata blob handles file attributes.
                // Directory markers (`@@dir`) at this level were handled above.
                if key.ends_with(FILE_CONTENT_SUFFIX) {
                    let real_filename = get_filename(key); // Get filename without suffix
                    if !entries_map.contains_key(&real_filename) {
                        // File content exists, but wasn't in metadata. Add a basic entry.
                        log::warn!("File content key '{}' found but no metadata entry in parent dir '{}'. Adding basic entry.", key, normalized_dir_path);
                        let file_full_path =
                            build_file_entry_path(&normalized_dir_path, &real_filename);
                        entries_map.insert(
                            real_filename.clone(),
                            DirectoryEntry {
                                name: real_filename,
                                path: file_full_path,
                                is_directory: false,
                                attributes: None, // Attributes should have been in metadata; mark as missing
                            },
                        );
                    }
                }
            }
        }

        // --- GitHub Supplement (Optional) ---
        // Decide if we need to load from GitHub
        let should_load_github = !skip_github_load && {
            let kv_found_something = !keys.is_empty();
            let metadata_found_something = !entries_map.is_empty();
            // Load if KV/Metadata found nothing, OR if listing root and found less than 10 items (heuristic)
            (!kv_found_something && !metadata_found_something)
                || (normalized_dir_path.is_empty() && entries_map.len() < 10)
        };

        if should_load_github {
            log_debug!(
                "Attempting to load from GitHub to supplement listing for '{}'",
                normalized_dir_path
            );
            match self
                .load_github_directory_flat(&normalized_dir_path, true)
                .await
            {
                // Use flat load, shallow=true
                Ok(result) => {
                    log::info!(
                        "Loaded {} potential entries from GitHub directory {}",
                        result.loaded_files.len(),
                        normalized_dir_path
                    );

                    for github_file in result.loaded_files {
                        let file_path = github_file.path; // This path is relative to VFS root
                        let filename = get_filename(&file_path);
                        if filename.is_empty() {
                            continue;
                        }

                        // Ensure it's a direct child of the directory we are listing
                        let parent_dir = get_parent_directory(&file_path);
                        // Normalize parent_dir for comparison (e.g. "" vs "/")
                        let normalized_parent = normalize_dir_path(&parent_dir);
                        let expected_parent = normalize_dir_path(&normalized_dir_path);

                        if normalized_parent == expected_parent {
                            // It's a direct child. Add or update entry.
                            if !entries_map.contains_key(&filename) {
                                log::debug!(
                                    "Adding entry from GitHub: '{}' (is_dir: {})",
                                    file_path,
                                    github_file.is_directory
                                );
                                let attributes = if github_file.is_directory {
                                    create_directory_attributes(&file_path, "cloud")
                                    // Pass path
                                } else {
                                    create_file_attributes(
                                        &file_path,
                                        github_file.size,
                                        if github_file.is_placeholder {
                                            "placeholder"
                                        } else {
                                            "cloud"
                                        },
                                    )
                                };
                                entries_map.insert(
                                    filename.clone(),
                                    DirectoryEntry {
                                        name: filename,
                                        path: if github_file.is_directory {
                                            normalize_dir_path(&file_path)
                                        } else {
                                            normalize_file_path(&file_path)
                                        },
                                        is_directory: github_file.is_directory,
                                        attributes: Some(attributes),
                                    },
                                );
                            } else {
                                // Entry exists from KV/Metadata. Maybe update attributes if source is different?
                                // For simplicity now, we prioritize KV/Metadata entries if they exist.
                                log::trace!("Entry '{}' already exists from KV/Metadata, skipping GitHub version.", filename);
                            }
                        } else {
                            log::trace!("Skipping GitHub entry '{}' as its parent '{}' doesn't match listed dir '{}'", file_path, normalized_parent, expected_parent);
                        }
                    }
                }
                Err(e) => {
                    log::warn!(
                        "GitHub flat load for supplementing '{}' failed: {:?}",
                        normalized_dir_path,
                        e
                    );
                }
            }
        }

        // --- Final Result ---
        let final_entries: Vec<DirectoryEntry> = entries_map.values().cloned().collect();
        log_debug!(
            "Returning {} entries for directory '{}'",
            final_entries.len(),
            normalized_dir_path
        );
        Ok(final_entries)
    }

    /// Create directory
    pub(crate) async fn create_directory_impl(&self, path: &str) -> Result<(), VfsError> {
        let normalized_path = normalize_path(path);
        // Use normalize_dir_path to ensure it represents a directory consistently
        let dir_path = normalize_dir_path(&normalized_path);

        if dir_path == "/" || dir_path.is_empty() {
            log::debug!("Attempted to create root directory, which implicitly exists.");
            return Ok(()); // Root implicitly exists
        }

        log::debug!("Attempting to create directory: {}", dir_path);

        // --- Ensure Parent Exists ---
        // Get parent path (must end with /)
        let parent = get_parent_directory(&dir_path);
        log::trace!("Parent directory for '{}' is '{}'", dir_path, parent);

        // Check parent existence recursively only if parent is not root
        if !parent.is_empty() && parent != "/" {
            // Check cache first for parent attributes
            let parent_attrs = self.store.read_from_metadata_cache(&parent).await;
            if parent_attrs.is_none() || !parent_attrs.unwrap().is_directory {
                // If not cached or not a directory, check KV marker
                if !self.store.directory_exists_in_kv(&parent).await? {
                    log::debug!(
                        "Parent directory '{}' does not exist. Creating recursively.",
                        parent
                    );
                    // Box the recursive future to avoid infinitely sized types
                    Box::pin(self.create_directory_impl(&parent)).await?;
                } else {
                    log::trace!("Parent directory '{}' exists (KV marker found).", parent);
                }
            } else {
                log::trace!(
                    "Parent directory '{}' exists (found in metadata cache).",
                    parent
                );
            }
        } else {
            log::trace!(
                "Parent directory is root ('{}'), skipping existence check.",
                parent
            );
        }

        // --- Create Directory Marker and Initial Metadata in KV ---
        // This function now handles creating the @@dir key with empty DirectoryMetadata JSON
        match self.store.create_directory_in_kv(&dir_path).await {
            Ok(_) => {
                log::debug!(
                    "Successfully ensured directory marker exists for: {}",
                    dir_path
                );
                // Optionally, cache the directory attributes immediately
                let dir_attributes = create_directory_attributes(&dir_path, "user"); // Pass path
                self.store
                    .write_to_metadata_cache(&dir_path, dir_attributes)
                    .await;
                Ok(())
            }
            Err(e) => {
                log::error!(
                    "Failed to create directory marker/metadata in KV for '{}': {:?}",
                    dir_path,
                    e
                );
                Err(e)
            }
        }
    }
}
