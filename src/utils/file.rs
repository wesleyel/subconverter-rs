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
pub fn file_get<P: AsRef<Path>>(path: P, base_path: Option<&str>) -> io::Result<String> {
    if let Some(base_path) = base_path {
        match path.as_ref().to_str() {
            Some(path_str) => {
                if !path_str.starts_with(base_path) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "File path is not within the base path",
                    ));
                }
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "File path is not a valid UTF-8 string",
                ));
            }
        }
    }
    fs::read_to_string(path)
}
