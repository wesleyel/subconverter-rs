use crate::utils::system::safe_system_time;
use crate::vfs::vercel_kv_github::{GitHubConfig, GitHubTreeResponse};
use crate::vfs::vercel_kv_helpers::*;
use crate::vfs::vercel_kv_js_bindings::*;
use crate::vfs::vercel_kv_types::*;
use crate::vfs::vercel_kv_vfs::VercelKvVfs;
use crate::vfs::VfsError;
use js_sys::Uint8Array;
use log::debug;
use serde_wasm_bindgen;
use std::collections::{HashMap, HashSet};
use std::time::UNIX_EPOCH;
use wasm_bindgen_futures;

impl VercelKvVfs {
    /// Load all files from a GitHub repository directory
    pub(crate) async fn load_github_directory_impl(
        &self,
        directory_path: &str,
        shallow: bool,
    ) -> Result<LoadDirectoryResult, VfsError> {
        log::info!(
            "Starting load_github_directory with path: {} (shallow: {})",
            directory_path,
            shallow
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

        for item in &tree_response.tree {
            // Handle both blob (file) and tree (directory) items
            let is_directory = item.type_field == "tree";
            if item.type_field != "blob" && !is_directory {
                continue; // Skip other item types
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

            if is_directory {
                // For directories, ensure they end with a slash
                let dir_path = if relative_path.ends_with('/') {
                    relative_path.clone()
                } else {
                    format!("{}/", relative_path)
                };

                log::debug!("Adding directory to create: {}", dir_path);
                directories.insert(dir_path.clone());
            } else {
                // This is a file, add to loading list
                log::debug!("Adding file to load queue: {}", relative_path);
                files_to_load.push(relative_path.clone());
            }

            // Track all parent directories for this file or directory
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

        // Make sure root directory is in the list of directories to create
        directories.insert("".to_string());

        // First create all necessary directories
        for dir in &directories {
            log::debug!("Creating directory: {}", dir);
            if let Err(e) = self.create_directory(dir).await {
                log::warn!("Failed to create directory {}: {:?}", dir, e);
                // Continue anyway
            } else {
                // Set directory attributes
                let dir_path = if dir.ends_with('/') {
                    dir.clone()
                } else {
                    format!("{}/", dir)
                };

                // Create directory attributes
                let dir_attributes = FileAttributes {
                    is_directory: true,
                    created_at: safe_system_time()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    modified_at: safe_system_time()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    ..Default::default()
                };

                // Update metadata cache
                self.metadata_cache
                    .write()
                    .await
                    .insert(dir_path.clone(), dir_attributes);
            }
        }

        // In WebAssembly, we can't use tokio::spawn for threading
        // so we'll process the files sequentially
        let mut successes = 0;
        let mut failures = 0;
        let mut loaded_files = Vec::new();

        log::info!(
            "Processing {} files (shallow: {})",
            files_to_load.len(),
            shallow
        );

        for (index, file_path) in files_to_load.iter().enumerate() {
            log::debug!(
                "Processing file {}/{}: {}",
                index + 1,
                files_to_load.len(),
                file_path
            );

            if shallow {
                // In shallow mode, just create placeholders for files without downloading content
                let normalized_path = normalize_path(file_path);

                // Create file attributes with estimated size if available
                let size_estimate = tree_response
                    .tree
                    .iter()
                    .find(|item| item.path == *file_path)
                    .and_then(|item| item.size)
                    .unwrap_or(0);

                let attributes = FileAttributes {
                    size: size_estimate,
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

                // Store file metadata
                let metadata_key = get_metadata_key(&normalized_path);
                let metadata_json = match serde_json::to_vec(&attributes) {
                    Ok(json) => json,
                    Err(e) => {
                        log::warn!("Failed to serialize metadata for {}: {:?}", file_path, e);
                        continue;
                    }
                };

                // Store placeholder status
                let status_key = get_status_key(&normalized_path);
                let status_value = FILE_STATUS_PLACEHOLDER.as_bytes().to_vec();

                // Store in KV
                let metadata_result = kv_set(&metadata_key, &metadata_json).await;
                let status_result = kv_set(&status_key, &status_value).await;

                if let Err(e) = metadata_result {
                    log::warn!("Failed to store metadata for {}: {:?}", file_path, e);
                    failures += 1;
                } else if let Err(e) = status_result {
                    log::warn!(
                        "Failed to store placeholder status for {}: {:?}",
                        file_path,
                        e
                    );
                    failures += 1;
                } else {
                    // Update metadata cache
                    self.metadata_cache
                        .write()
                        .await
                        .insert(normalized_path.clone(), attributes);

                    successes += 1;
                    loaded_files.push(LoadedFile {
                        path: normalized_path,
                        size: size_estimate,
                        is_placeholder: true,
                        is_directory: false,
                    });
                }
            } else {
                // In deep mode, actually download file content
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
                            is_placeholder: false,
                            is_directory: false,
                        });
                    }
                    Err(e) => {
                        log::warn!("Failed to load file {}: {:?}", file_path, e);
                        failures += 1;
                    }
                }
            }
        }

        log::info!(
            "Finished loading: {} successes, {} failures",
            successes,
            failures
        );

        // Add created directories to the result
        for dir in &directories {
            if dir != &dir_path && !dir.is_empty() {
                // Don't include current or root directory
                let dir_path_with_slash = if dir.ends_with('/') {
                    dir.clone()
                } else {
                    format!("{}/", dir)
                };

                loaded_files.push(LoadedFile {
                    path: dir_path_with_slash,
                    size: 0,
                    is_placeholder: false,
                    is_directory: true,
                });
            }
        }

        Ok(LoadDirectoryResult {
            total_files: files_to_load.len(),
            successful_files: successes,
            failed_files: failures,
            loaded_files,
        })
    }
}
