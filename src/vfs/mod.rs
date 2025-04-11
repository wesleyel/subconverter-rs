pub mod vercel_kv_vfs;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VfsError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

// Potentially define a common VFS trait here later if needed
// pub trait VirtualFileSystem {
//     async fn read_file(&self, path: &str) -> Result<Vec<u8>, VfsError>;
//     async fn write_file(&self, path: &str, content: Vec<u8>) -> Result<(), VfsError>;
//     async fn exists(&self, path: &str) -> Result<bool, VfsError>;
//     // ... other methods like list_dir, delete, etc.
// } 