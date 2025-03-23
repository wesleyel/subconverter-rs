use std::fs;
use std::io::{self, Read};
use std::path::Path;

/// Read a file into a string
pub fn read_file(path: &str) -> Result<String, io::Error> {
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Check if a file exists
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
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
