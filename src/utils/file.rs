use std::fs;
use std::io;
use std::path::Path;

/// Check if a file exists
pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    Path::new(path.as_ref()).exists()
}

/// Read the contents of a file as a string
///
/// # Arguments
/// * `path` - Path to the file to read
///
/// # Returns
/// * `Ok(String)` - The file contents
/// * `Err(io::Error)` - If the file can't be read
pub fn file_get<P: AsRef<Path>>(path: P) -> io::Result<String> {
    fs::read_to_string(path)
}
