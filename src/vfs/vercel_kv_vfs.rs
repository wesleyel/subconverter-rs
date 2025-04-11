use crate::vfs::VfsError;
use reqwest::Client as ReqwestClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock; // Use RwLock for better read concurrency
use vercel_kv::{KVClient, BatchCommand};

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
            self.owner,
            self.repo,
            self.branch
        );
        let full_path = if self.root_path.is_empty() {
            file_path.to_string()
        } else {
            format!("{}/{}", self.root_path.trim_matches('/'), file_path)
        };
        format!("{}/{}", base, full_path.trim_start_matches('/'))
    }
}

// Wrapper for content to handle serialization/deserialization if needed
// Using Vec<u8> directly with vercel_kv might require base64 or similar if it expects strings.
// Let's assume the crate handles raw bytes or JSON representation appropriately.
// For simplicity, we'll store Vec<u8> in memory and try to store it directly in KV.
#[derive(Serialize, Deserialize, Clone, Debug)]
struct FileContent(#[serde(with = "serde_bytes")] Vec<u8>);


#[derive(Clone)]
pub struct VercelKvVfs {
    memory_cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    kv_client: KVClient,
    http_client: ReqwestClient,
    github_config: GitHubConfig,
}

impl VercelKvVfs {
    pub async fn new() -> Result<Self, VfsError> {
        let kv_client = KVClient::new().map_err(|e| VfsError::StorageError(format!("Failed to create KV client: {}", e)))?;
        let github_config = GitHubConfig::from_env()?;

        Ok(VercelKvVfs {
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
            kv_client,
            http_client: ReqwestClient::builder()
                            .timeout(Duration::from_secs(10)) // Add timeout
                            .build()?, 
            github_config,
        })
    }

    // Normalize path: ensure consistency, e.g., remove leading '/'
    fn normalize_path(path: &str) -> String {
        path.trim_start_matches('/').to_string()
    }

    pub async fn read_file(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        let normalized_path = Self::normalize_path(path);

        // 1. Check memory cache
        if let Some(content) = self.memory_cache.read().await.get(&normalized_path) {
            log::debug!("Cache hit for: {}", normalized_path);
            return Ok(content.clone());
        }

        // 2. Check Vercel KV
        log::debug!("Cache miss, checking KV for: {}", normalized_path);
        match self.kv_client.get::<FileContent>(&normalized_path).await {
            Ok(Some(file_content)) => {
                log::debug!("KV hit for: {}", normalized_path);
                let content = file_content.0;
                // Store in memory cache
                self.memory_cache
                    .write().await
                    .insert(normalized_path.clone(), content.clone());
                return Ok(content);
            }
            Ok(None) => {
                log::debug!("KV miss for: {}", normalized_path);
                // Not found in KV, proceed to fetch from GitHub
            }
            Err(e) => {
                 log::error!("Vercel KV read error for {}: {:?}", normalized_path, e);
                 // Decide if we should try GitHub or return error
                 // Let's try GitHub, but log the error
            }
        }

        // 3. Fetch from GitHub
        let url = self.github_config.get_raw_url(&normalized_path);
        log::debug!("Fetching from GitHub: {}", url);
        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
             log::warn!("GitHub fetch failed for {}: Status {}", url, response.status());
             if response.status() == reqwest::StatusCode::NOT_FOUND {
                 return Err(VfsError::NotFound(format!("File not found locally or on GitHub: {}", normalized_path)));
             } else {
                 // Convert response error without consuming body
                 let status = response.status();
                 let text = response.text().await.unwrap_or_else(|_| "Failed to read error body".to_string());
                 return Err(VfsError::NetworkError(reqwest::Error::from(status.as_u16().try_into().unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR))));
                // return Err(VfsError::Other(format!("GitHub fetch failed: {} - {}", status, text)));
             }
        }

        let content = response.bytes().await?.to_vec();
        log::debug!("Successfully fetched {} bytes from GitHub for: {}", content.len(), normalized_path);

        // 4. Store in memory cache AND Vercel KV (async, fire-and-forget for KV write?)
        self.memory_cache
            .write().await
            .insert(normalized_path.clone(), content.clone());

        // Asynchronously store in KV without blocking the read operation completion
        let kv_client_clone = self.kv_client.clone();
        let path_clone = normalized_path.clone();
        let content_clone = content.clone();
        tokio::spawn(async move {
            match kv_client_clone.set(&path_clone, &FileContent(content_clone)).await {
                 Ok(_) => { log::debug!("Successfully stored {} in KV background.", path_clone); },
                 Err(e) => {
                     log::error!("Background Vercel KV write error after GitHub fetch for {}: {:?}", path_clone, e);
                 }
             }
        });

        Ok(content)
    }

    pub async fn write_file(&self, path: &str, content: Vec<u8>) -> Result<(), VfsError> {
         let normalized_path = Self::normalize_path(path);
         log::debug!("Writing file: {}", normalized_path);

         // 1. Write to memory cache
         self.memory_cache
            .write().await
            .insert(normalized_path.clone(), content.clone());

         // 2. Write to Vercel KV
         match self.kv_client.set(&normalized_path, &FileContent(content)).await {
             Ok(_) => {
                log::debug!("Successfully wrote {} to KV.", normalized_path);
                Ok(())
             },
             Err(e) => {
                 log::error!("Vercel KV write error for {}: {:?}", normalized_path, e);
                 Err(VfsError::StorageError(format!("KV set failed: {}", e)))
             }
         }
    }

     pub async fn exists(&self, path: &str) -> Result<bool, VfsError> {
         let normalized_path = Self::normalize_path(path);

         // 1. Check memory cache
         if self.memory_cache.read().await.contains_key(&normalized_path) {
             log::trace!("Exists check (memory hit): {}", normalized_path);
             return Ok(true);
         }

         // 2. Check Vercel KV
         // Use `exists` command if the `vercel_kv` crate supports it efficiently.
         // Otherwise, fall back to `get` and check for Some.
         // The crate seems to prefer get/set/del, let's try `get` without deserializing fully?
         // A simple `get <key>` might be efficient enough.
         // Let's try `kv_client.exists()` which seems available.
         log::trace!("Exists check (checking KV): {}", normalized_path);
         match self.kv_client.exists(&normalized_path).await {
            Ok(count) => {
                if count > 0 {
                    log::trace!("Exists check (KV hit): {}", normalized_path);
                    // Optional: Pre-warm memory cache? No, let read_file handle it.
                    return Ok(true);
                }
            }
            Err(e) => {
                log::error!("Vercel KV exists check error for {}: {:?}", normalized_path, e);
                // Don't return error yet, try GitHub
            }
         }

         // 3. Check GitHub (HEAD request)
         let url = self.github_config.get_raw_url(&normalized_path);
         log::trace!("Exists check (checking GitHub HEAD): {}", url);
         let response = self.http_client.head(&url).send().await?;

         let found = response.status().is_success();
         log::trace!("Exists check (GitHub result {}): {}", found, normalized_path);
         Ok(found)
     }

     // Optional: Add delete, list_dir (might be complex/expensive) later if needed.
     pub async fn delete_file(&self, path: &str) -> Result<(), VfsError> {
        let normalized_path = Self::normalize_path(path);
        log::debug!("Deleting file: {}", normalized_path);

        // 1. Remove from memory cache
        self.memory_cache.write().await.remove(&normalized_path);

        // 2. Remove from Vercel KV
        match self.kv_client.del(&normalized_path).await {
            Ok(_) => {
                log::debug!("Successfully deleted {} from KV.", normalized_path);
                Ok(())
            },
            Err(e) => {
                 log::error!("Vercel KV delete error for {}: {:?}", normalized_path, e);
                 Err(VfsError::StorageError(format!("KV del failed: {}", e)))
            }
        }
     }
} 