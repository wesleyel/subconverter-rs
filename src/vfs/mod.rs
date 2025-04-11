pub mod vercel_kv_vfs;
pub use vercel_kv_vfs::*;

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

    #[error("Is a directory: {0}")]
    IsDirectory(String),

    #[error("Is not a directory: {0}")]
    NotDirectory(String),

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

    // New methods for directory operations and file attributes

    /// List the contents of a directory
    fn list_directory(
        &self,
        path: &str,
    ) -> impl Future<Output = Result<Vec<DirectoryEntry>, VfsError>>;

    /// Read the attributes of a file or directory
    fn read_file_attributes(
        &self,
        path: &str,
    ) -> impl Future<Output = Result<FileAttributes, VfsError>>;

    /// Create a directory (and any necessary parent directories)
    fn create_directory(&self, path: &str) -> impl Future<Output = Result<(), VfsError>>;
}
