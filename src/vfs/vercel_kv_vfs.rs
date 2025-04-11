use super::VirtualFileSystem;
use crate::vfs::vercel_kv_directory::*;
use crate::vfs::vercel_kv_github::GitHubConfig;
use crate::vfs::vercel_kv_github_loader::*;
use crate::vfs::vercel_kv_operations::*;
use crate::vfs::vercel_kv_types::*;
use crate::vfs::VfsError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct VercelKvVfs {
    pub(crate) memory_cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    pub(crate) metadata_cache: Arc<RwLock<HashMap<String, FileAttributes>>>,
    pub(crate) github_config: GitHubConfig,
}

impl VercelKvVfs {
    pub fn new() -> Result<Self, VfsError> {
        // Initialize GitHub config from environment
        let github_config = match GitHubConfig::from_env() {
            Ok(config) => {
                log::info!(
                    "Initialized GitHub config: repo={}/{}, branch={}, root_path={}",
                    config.owner,
                    config.repo,
                    config.branch,
                    config.root_path
                );
                config
            }
            Err(e) => {
                log::warn!("Failed to initialize GitHub config: {}", e);
                return Err(e);
            }
        };

        Ok(Self {
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            github_config,
        })
    }
}

impl VirtualFileSystem for VercelKvVfs {
    async fn read_file(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        self.read_file_impl(path).await
    }

    async fn write_file(&self, path: &str, content: Vec<u8>) -> Result<(), VfsError> {
        self.write_file_impl(path, content).await
    }

    async fn exists(&self, path: &str) -> Result<bool, VfsError> {
        self.exists_impl(path).await
    }

    async fn delete_file(&self, path: &str) -> Result<(), VfsError> {
        self.delete_file_impl(path).await
    }

    async fn read_file_attributes(&self, path: &str) -> Result<FileAttributes, VfsError> {
        self.read_file_attributes_impl(path).await
    }

    async fn list_directory(&self, path: &str) -> Result<Vec<DirectoryEntry>, VfsError> {
        self.list_directory_impl(path).await
    }

    async fn create_directory(&self, path: &str) -> Result<(), VfsError> {
        self.create_directory_impl(path).await
    }

    async fn load_github_directory(
        &self,
        directory_path: &str,
        shallow: bool,
    ) -> Result<LoadDirectoryResult, VfsError> {
        self.load_github_directory_impl(directory_path, shallow)
            .await
    }
}
