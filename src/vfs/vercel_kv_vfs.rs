use super::VirtualFileSystem;
use crate::vfs::vercel_kv_github::GitHubConfig;
use crate::vfs::vercel_kv_store::VercelKvStore;
use crate::vfs::vercel_kv_types::*;
use crate::vfs::VfsError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct VercelKvVfs {
    pub(crate) store: Arc<VercelKvStore>,
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
            store: Arc::new(VercelKvStore::new()),
            github_config,
        })
    }

    // Internal helper to get memory cache from store
    pub(crate) fn memory_cache(&self) -> Arc<RwLock<HashMap<String, Vec<u8>>>> {
        self.store.get_memory_cache()
    }

    // Internal helper to get metadata cache from store
    pub(crate) fn metadata_cache(&self) -> Arc<RwLock<HashMap<String, FileAttributes>>> {
        self.store.get_metadata_cache()
    }
}

impl VirtualFileSystem for VercelKvVfs {
    fn read_file(
        &self,
        path: &str,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, VfsError>> {
        async move { self.read_file_impl(path).await }
    }

    fn write_file(
        &self,
        path: &str,
        content: Vec<u8>,
    ) -> impl std::future::Future<Output = Result<(), VfsError>> {
        async move { self.write_file_impl(path, content).await }
    }

    fn exists(&self, path: &str) -> impl std::future::Future<Output = Result<bool, VfsError>> {
        async move { self.exists_impl(path).await }
    }

    fn delete_file(&self, path: &str) -> impl std::future::Future<Output = Result<(), VfsError>> {
        async move { self.delete_file_impl(path).await }
    }

    fn read_file_attributes(
        &self,
        path: &str,
    ) -> impl std::future::Future<Output = Result<FileAttributes, VfsError>> {
        async move { self.read_file_attributes_impl(path).await }
    }

    fn list_directory(
        &self,
        path: &str,
    ) -> impl std::future::Future<Output = Result<Vec<DirectoryEntry>, VfsError>> {
        async move { self.list_directory_impl(path).await }
    }

    fn create_directory(
        &self,
        path: &str,
    ) -> impl std::future::Future<Output = Result<(), VfsError>> {
        async move { self.create_directory_impl(path).await }
    }

    fn load_github_directory(
        &self,
        _directory_path: &str,
        shallow: bool,
    ) -> impl std::future::Future<Output = Result<LoadDirectoryResult, VfsError>> {
        async move { self.load_github_directory_impl(shallow, true).await }
    }

    fn load_github_directory_flat(
        &self,
        _directory_path: &str,
        shallow: bool,
    ) -> impl std::future::Future<Output = Result<LoadDirectoryResult, VfsError>> {
        async move { self.load_github_directory_impl(shallow, false).await }
    }
}
