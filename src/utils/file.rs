use std::path::Path;

/// Checks if a file exists at the given path
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}
