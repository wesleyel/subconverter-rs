// Add the new modules
pub mod vercel_kv_github;
pub mod vercel_kv_helpers;
pub mod vercel_kv_js_bindings;
pub mod vercel_kv_store;
pub mod vercel_kv_types;

// VFS operations modules
pub mod vercel_kv_directory;
pub mod vercel_kv_github_loader;
pub mod vercel_kv_operations;

// Main VFS implementation
pub mod vercel_kv_vfs;

// Re-export core types and the main VFS implementation
pub use vercel_kv_store::{
    create_directory_attributes, create_file_attributes, get_real_path_from_key, is_internal_key,
    VercelKvStore,
};
pub use vercel_kv_types::{DirectoryEntry, FileAttributes, LoadDirectoryResult, LoadedFile};
pub use vercel_kv_vfs::VercelKvVfs;

// Re-export the helper macro
pub use vercel_kv_types::*;

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
