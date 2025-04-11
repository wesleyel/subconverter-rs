use super::VirtualFileSystem;
use crate::vfs::VfsError;
use js_sys::Uint8Array;
use serde_wasm_bindgen;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures;

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
            owner: std::env::var("VFS_GITHUB_OWNER")
                .map_err(|_| VfsError::ConfigError("Missing VFS_GITHUB_OWNER".into()))?,
            repo: std::env::var("VFS_GITHUB_REPO")
                .map_err(|_| VfsError::ConfigError("Missing VFS_GITHUB_REPO".into()))?,
            branch: std::env::var("VFS_GITHUB_BRANCH")
                .map_err(|_| VfsError::ConfigError("Missing VFS_GITHUB_BRANCH".into()))?,
            root_path: std::env::var("VFS_GITHUB_ROOT_PATH").unwrap_or_else(|_| "".to_string()),
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
#[wasm_bindgen(module = "/js/kv_bindings.js")] // Example path
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
    async fn fetch_url(url: &str) -> Result<JsValue, JsValue>; // Returns Response object or similar?
                                                               // Need helpers to extract status and bytes from the fetch result
    #[wasm_bindgen(catch)]
    async fn response_status(response: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    async fn response_bytes(response: &JsValue) -> Result<Uint8Array, JsValue>; // Use imported Uint8Array
}

// Normalize path: ensure consistency, e.g., remove leading '/'
fn normalize_path(path: &str) -> String {
    path.trim_start_matches('/').to_string()
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
    github_config: GitHubConfig,
}

impl VirtualFileSystem for VercelKvVfs {
    async fn read_file(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        let normalized_path = normalize_path(path);

        // 1. Check memory cache
        if let Some(content) = self.memory_cache.read().await.get(&normalized_path) {
            log::debug!("Cache hit for: {}", normalized_path);
            return Ok(content.clone());
        }

        // 2. Check Vercel KV via JS binding
        log::debug!("Cache miss, checking KV for: {}", normalized_path);
        match kv_get(&normalized_path).await {
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

        let path_clone = normalized_path.clone();
        let content_clone = content.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match kv_set(&path_clone, &content_clone).await {
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
        });

        Ok(content)
    }

    async fn write_file(&self, path: &str, content: Vec<u8>) -> Result<(), VfsError> {
        let normalized_path = normalize_path(path);
        log::debug!("Writing file via JS: {}", normalized_path);

        self.memory_cache
            .write()
            .await
            .insert(normalized_path.clone(), content.clone());

        kv_set(&normalized_path, &content)
            .await
            .map_err(|e| js_error_to_vfs(e, "KV set failed"))
    }

    async fn exists(&self, path: &str) -> Result<bool, VfsError> {
        let normalized_path = normalize_path(path);

        if self
            .memory_cache
            .read()
            .await
            .contains_key(&normalized_path)
        {
            log::trace!("Exists check (memory hit): {}", normalized_path);
            return Ok(true);
        }

        log::trace!("Exists check (checking KV via JS): {}", normalized_path);
        match kv_exists(&normalized_path).await {
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

        let url = self.github_config.get_raw_url(&normalized_path);
        log::trace!("Exists check (checking GitHub GET via JS): {}", url);
        match fetch_url(&url).await {
            Ok(fetch_response) => match response_status(&fetch_response).await {
                Ok(status_js) => {
                    let status = status_js.as_f64().map(|f| f as u16).ok_or_else(|| {
                        js_error_to_vfs(status_js, "GitHub exists check status was not a number")
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
    }

    async fn delete_file(&self, path: &str) -> Result<(), VfsError> {
        let normalized_path = normalize_path(path);
        log::debug!("Deleting file via JS: {}", normalized_path);

        self.memory_cache.write().await.remove(&normalized_path);

        kv_del(&normalized_path)
            .await
            .map_err(|e| js_error_to_vfs(e, "KV del failed"))
    }
}

impl VercelKvVfs {
    pub fn new() -> Result<Self, VfsError> {
        let github_config = GitHubConfig::from_env()?;

        Ok(VercelKvVfs {
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
            github_config,
        })
    }
}
