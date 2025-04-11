use crate::utils::system::safe_system_time;
use crate::vfs::VfsError;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::time::UNIX_EPOCH;

// File metadata structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileAttributes {
    /// Size of the file in bytes
    pub size: usize,
    /// Creation timestamp (seconds since UNIX epoch)
    pub created_at: u64,
    /// Last modified timestamp (seconds since UNIX epoch)
    pub modified_at: u64,
    /// File type (mime type or extension)
    pub file_type: String,
    /// Is this a directory marker
    pub is_directory: bool,
}

impl Default for FileAttributes {
    fn default() -> Self {
        let now = safe_system_time()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            size: 0,
            created_at: now,
            modified_at: now,
            file_type: "text/plain".to_string(),
            is_directory: false,
        }
    }
}

// Directory entry for listing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectoryEntry {
    /// Name of the file or directory (not the full path)
    pub name: String,
    /// Full path to the file or directory
    pub path: String,
    /// Is this entry a directory
    pub is_directory: bool,
    /// File attributes
    pub attributes: Option<FileAttributes>,
}

/// Represents a file that was loaded from GitHub
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoadedFile {
    /// Path to the file that was loaded
    pub path: String,
    /// Size of the file in bytes
    pub size: usize,
    /// Whether this is a placeholder entry (content not loaded)
    pub is_placeholder: bool,
    /// Whether this is a directory
    pub is_directory: bool,
}

/// Result of loading a directory from GitHub
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoadDirectoryResult {
    /// Total number of files attempted to load
    pub total_files: usize,
    /// Number of files successfully loaded
    pub successful_files: usize,
    /// Number of files that failed to load
    pub failed_files: usize,
    /// Information about each successfully loaded file
    pub loaded_files: Vec<LoadedFile>,
}

// Constants
pub const FILE_CONTENT_SUFFIX: &str = ".content";
pub const FILE_METADATA_SUFFIX: &str = ".metadata";
pub const DIRECTORY_MARKER_SUFFIX: &str = "/.dir";
pub const FILE_STATUS_PLACEHOLDER: &str = "placeholder";
pub const FILE_STATUS_SUFFIX: &str = ".status";

// VFS trait definition
pub trait VirtualFileSystem {
    fn read_file(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, VfsError>>;
    fn write_file(
        &self,
        path: &str,
        content: Vec<u8>,
    ) -> impl Future<Output = Result<(), VfsError>>;
    fn exists(&self, path: &str) -> impl Future<Output = Result<bool, VfsError>>;
    fn delete_file(&self, path: &str) -> impl Future<Output = Result<(), VfsError>>;

    // Directory operations and file attributes

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

    /// Load all files from a GitHub repository directory at once
    fn load_github_directory(
        &self,
        directory_path: &str,
        shallow: bool,
    ) -> impl Future<Output = Result<LoadDirectoryResult, VfsError>>;
}
