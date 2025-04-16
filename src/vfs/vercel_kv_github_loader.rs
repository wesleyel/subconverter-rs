use crate::utils::http_wasm::{web_get_async, ProxyConfig};
use crate::utils::system::safe_system_time;
use crate::vfs::vercel_kv_github::GitHubTreeResponse;
use crate::vfs::vercel_kv_helpers::*;
use crate::vfs::vercel_kv_js_bindings::*;
use crate::vfs::vercel_kv_types::*;
use crate::vfs::vercel_kv_vfs::VercelKvVfs;
use crate::vfs::VfsError;
use case_insensitive_string::CaseInsensitiveString;
use std::collections::HashMap;
use std::time::UNIX_EPOCH;

impl VercelKvVfs {
    /// Load all files from a GitHub repository directory
    pub(crate) async fn load_github_directory_impl(
        &self,
        shallow: bool,
        recursive: bool,
    ) -> Result<LoadDirectoryResult, VfsError> {
        log::info!(
            "Starting load_github_directory (shallow: {}, recursive: {})",
            shallow,
            recursive
        );

        // Add an env var check to allow triggering a test panic
        if std::option_env!("RUST_TEST_PANIC").is_some() {
            log::warn!("Triggering intentional panic for stack trace testing");
            panic!("This is an intentional test panic to verify stack trace capture");
        }

        log::info!("Loading all files from GitHub configured root path");

        // Generate cache key for this directory lookup
        let cache_key = get_github_tree_cache_key(
            &self.github_config.owner,
            &self.github_config.repo,
            &self.github_config.branch,
            recursive,
        );

        // Check if we have cached data
        let current_time = safe_system_time()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut response_text = None;

        if let Ok(Some(cache)) = self.store.read_github_tree_cache(&cache_key).await {
            if !cache.is_expired(current_time) {
                log::debug!("Using cached GitHub tree data from KV");
                response_text = Some(cache.data);
            } else {
                log::debug!("GitHub tree cache is expired");
            }
        }

        // If no valid cache, fetch from GitHub API
        if response_text.is_none() {
            // When recursive=0, API returns only direct children of the tree
            // When recursive=1, API returns all descendants recursively
            let api_url = format!(
                "https://api.github.com/repos/{}/{}/git/trees/{}?recursive={}",
                self.github_config.owner,
                self.github_config.repo,
                self.github_config.branch,
                if recursive { "1" } else { "0" }
            );

            log::debug!("Fetching GitHub directory tree from: {}", api_url);

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
                "application/vnd.github.v3+json".to_string(),
            );
            headers.insert(
                CaseInsensitiveString::new("User-Agent"),
                "subconverter-rs".to_string(),
            );

            // Make the request
            let proxy_config = ProxyConfig::default();
            let fetch_result = web_get_async(&api_url, &proxy_config, Some(&headers)).await;

            match fetch_result {
                Ok(response) => {
                    // Check if the response is successful (2xx)
                    if (200..300).contains(&response.status) {
                        log::debug!("Successfully fetched GitHub API response");

                        // Check if we got rate limit headers
                        if let Some(rate_limit) = response.headers.get("x-ratelimit-remaining") {
                            log::info!("GitHub API rate limit remaining: {}", rate_limit);
                        }

                        response_text = Some(response.body);

                        // Cache the result
                        let cache = GitHubTreeCache {
                            data: response_text.as_ref().unwrap().clone(),
                            created_at: current_time,
                            ttl: self.github_config.cache_ttl_seconds,
                        };

                        // Store cache in background
                        self.store
                            .write_github_tree_cache_background(cache_key.clone(), cache);
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
                    log::error!("Error fetching GitHub API: {}", e.message);
                    return Err(VfsError::NetworkError(format!(
                        "GitHub API request failed: {}",
                        e.message
                    )));
                }
            }
        }

        // Parse the response to get file information
        let response_text = match response_text {
            Some(text) => text,
            None => {
                return Err(VfsError::NetworkError(
                    "Failed to get GitHub API response".to_string(),
                ))
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
        let mut files_to_process = Vec::new();

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
            // if !relative_path.starts_with(&dir_path) && !dir_path.is_empty() {
            //     continue;
            // }

            if is_directory {
                // For directories, ensure they end with a slash
                let current_dir_path = if relative_path.ends_with('/') {
                    relative_path.clone()
                } else {
                    format!("{}/", relative_path)
                };

                log::trace!("Found directory from GitHub tree: {}", current_dir_path);
                directories.insert(current_dir_path.clone());
            } else {
                // This is a file, add to loading list with reference to original item
                log::trace!("Found file from GitHub tree: {}", relative_path);
                files_to_process.push((relative_path.clone(), item));
            }

            // Track all parent directories for this file or directory
            let mut current_parent_dir = get_parent_directory(&relative_path);
            while !current_parent_dir.is_empty() {
                log::trace!("Tracking parent directory: {}", current_parent_dir);
                directories.insert(current_parent_dir.clone());
                current_parent_dir = get_parent_directory(&current_parent_dir);
            }
        }

        log::info!(
            "Found {} files to load and {} directories to create",
            files_to_process.len(),
            directories.len()
        );

        // Make sure root directory is in the list of directories to create
        directories.insert("".to_string());

        // First create all necessary directories
        for dir in &directories {
            // Skip creating the overall root path itself if it was derived from the argument
            // if dir == &dir_path {
            //     continue;
            // }
            log::debug!("Creating directory: {}", dir);
            if let Err(e) = self.create_directory(dir).await {
                log::warn!("Failed to create directory {}: {:?}", dir, e);
                // Continue anyway
            } else {
                // Set directory attributes
                let current_dir_path = if dir.ends_with('/') {
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
                    source_type: "cloud".to_string(),
                    ..Default::default()
                };

                // Update metadata cache
                self.metadata_cache()
                    .write()
                    .await
                    .insert(current_dir_path.clone(), dir_attributes);
            }
        }

        // In WebAssembly, we can't use tokio::spawn for threading
        // so we'll process the files sequentially
        let mut successes = 0;
        let mut failures = 0;
        let mut loaded_files = Vec::new();

        log::info!(
            "Processing {} files (shallow: {})",
            files_to_process.len(),
            shallow
        );

        for (index, (file_path, item)) in files_to_process.iter().enumerate() {
            log::debug!(
                "Processing file {}/{}: {}",
                index + 1,
                files_to_process.len(),
                file_path
            );

            if shallow {
                // In shallow mode, just create placeholders for files without downloading content
                let normalized_path = normalize_path(file_path);

                // Get size estimate directly from the item
                let size_estimate = item.size.unwrap_or(0);
                if size_estimate == 0 {
                    log::warn!("File size estimate is 0 for: {}", normalized_path);
                }

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
                    source_type: "placeholder".to_string(),
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
                    self.metadata_cache()
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
            // if dir != &dir_path && !dir.is_empty() {
            if !dir.is_empty() {
                // Simplified condition
                // Don't include root directory
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
            total_files: files_to_process.len(),
            successful_files: successes,
            failed_files: failures,
            loaded_files,
        })
    }

    /// Load information about a specific file from GitHub without downloading content
    pub(crate) async fn load_github_file_info_impl(
        &self,
        file_path: &str,
    ) -> Result<LoadedFile, VfsError> {
        log::debug!("Loading GitHub file info for: {}", file_path);

        let normalized_path = normalize_path(file_path);

        // Normalize the path for GitHub API
        let api_path = if normalized_path.starts_with('/') {
            normalized_path[1..].to_string()
        } else {
            normalized_path.clone()
        };

        // Account for root_path from config
        let api_path_with_root = if self.github_config.root_path.is_empty() {
            api_path
        } else {
            let root_path = self.github_config.root_path.trim_matches('/');
            format!("{}/{}", root_path, api_path)
        };

        // Cache key for GitHub tree API
        let cache_key = get_github_tree_cache_key(
            &self.github_config.owner,
            &self.github_config.repo,
            &self.github_config.branch,
            true, // Always use recursive tree for file info
        );

        // Check if we have cached data
        let current_time = safe_system_time()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut response_text = None;

        if let Ok(Some(cache)) = self.store.read_github_tree_cache(&cache_key).await {
            if !cache.is_expired(current_time) {
                log::debug!("Using cached GitHub tree data for file info");
                response_text = Some(cache.data);
            } else {
                log::debug!("GitHub tree cache is expired");
            }
        }

        // If no valid cache, fetch from GitHub API
        if response_text.is_none() {
            // Create GitHub API URL to get file info
            // Use the trees API to get file size without downloading the content
            let url = format!(
                "https://api.github.com/repos/{owner}/{repo}/git/trees/{branch}?recursive=1",
                owner = self.github_config.owner,
                repo = self.github_config.repo,
                branch = self.github_config.branch
            );

            log::debug!("Fetching GitHub tree from: {}", url);

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
                "application/vnd.github.v3+json".to_string(),
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
                        log::debug!("Successfully fetched GitHub API response for file info");

                        // Check if we got rate limit headers
                        if let Some(rate_limit) = response.headers.get("x-ratelimit-remaining") {
                            log::info!("GitHub API rate limit remaining: {}", rate_limit);
                        }

                        response_text = Some(response.body);

                        // Cache the result
                        let cache = GitHubTreeCache {
                            data: response_text.as_ref().unwrap().clone(),
                            created_at: current_time,
                            ttl: self.github_config.cache_ttl_seconds,
                        };

                        // Store cache in background
                        self.store
                            .write_github_tree_cache_background(cache_key.clone(), cache);
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
                    log::error!("Error fetching GitHub API for file info: {}", e.message);
                    return Err(VfsError::NetworkError(format!(
                        "GitHub API request failed: {}",
                        e.message
                    )));
                }
            }
        }

        // Parse the response to get file information
        let response_text = match response_text {
            Some(text) => text,
            None => {
                return Err(VfsError::NetworkError(
                    "Failed to get GitHub API response".to_string(),
                ))
            }
        };

        // Parse the tree response
        let tree_response: GitHubTreeResponse =
            match serde_json::from_str::<GitHubTreeResponse>(&response_text) {
                Ok(tree) => tree,
                Err(e) => {
                    return Err(VfsError::Other(format!(
                        "Failed to parse GitHub tree JSON: {}",
                        e
                    )))
                }
            };

        // Find the file in the tree
        for item in &tree_response.tree {
            // Skip directories
            if item.type_field != "blob" {
                continue;
            }

            // Check if this is the file we're looking for
            if item.path == api_path_with_root {
                // Found the file, get its size
                let size = item.size.unwrap_or(0);

                log::debug!(
                    "Found file in GitHub tree: {} with size {}",
                    item.path,
                    size
                );

                return Ok(LoadedFile {
                    path: normalized_path,
                    size,
                    is_placeholder: false,
                    is_directory: false,
                });
            }
        }

        // File not found in the tree
        log::debug!("File not found in GitHub tree: {}", api_path_with_root);
        Err(VfsError::NotFound(format!(
            "File not found in GitHub repo: {}",
            file_path
        )))
    }
}
