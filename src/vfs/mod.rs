pub mod vercel_kv_vfs;

use std::future::Future;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VfsError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

// Uncomment the VFS trait definition
pub trait VirtualFileSystem {
    fn read_file(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, VfsError>>;
    fn write_file(
        &self,
        path: &str,
        content: Vec<u8>,
    ) -> impl Future<Output = Result<(), VfsError>>;
    fn exists(&self, path: &str) -> impl Future<Output = Result<bool, VfsError>>;
    fn delete_file(&self, path: &str) -> impl Future<Output = Result<(), VfsError>>;
    // ... other methods like list_dir, etc. (add later if needed)
}
