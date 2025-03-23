//! INI file reader implementation
//!
//! This module provides functionality for reading and parsing INI files,
//! similar to the C++ INIReader class in the original subconverter.

use configparser::ini::Ini;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Error types for the INI reader
#[derive(Debug)]
pub enum IniReaderError {
    Empty,
    Duplicate,
    OutOfBound,
    NotExist,
    NotParsed,
    IoError(io::Error),
    None,
}

impl std::fmt::Display for IniReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IniReaderError::Empty => write!(f, "Empty document"),
            IniReaderError::Duplicate => write!(f, "Duplicate section"),
            IniReaderError::OutOfBound => write!(f, "Item exists outside of any section"),
            IniReaderError::NotExist => write!(f, "Target does not exist"),
            IniReaderError::NotParsed => write!(f, "Parse error"),
            IniReaderError::IoError(e) => write!(f, "IO error: {}", e),
            IniReaderError::None => write!(f, "No error"),
        }
    }
}

impl From<io::Error> for IniReaderError {
    fn from(error: io::Error) -> Self {
        IniReaderError::IoError(error)
    }
}

/// INI file reader with similar functionality to the C++ INIReader class
pub struct IniReader {
    /// The parsed INI content
    ini: Ini,
    /// Whether the INI has been successfully parsed
    parsed: bool,
    /// The current section being operated on
    current_section: String,
    /// List of sections to exclude when parsing
    // exclude_sections: HashSet<String>,
    /// List of sections to include when parsing (if empty, all sections are included)
    // include_sections: HashSet<String>,
    /// List of sections to save directly without processing
    direct_save_sections: HashSet<String>,
    /// Ordered list of sections as they appear in the original file
    section_order: Vec<String>,
    /// Mapping of sections to key-value pairs
    content: HashMap<String, HashMap<String, String>>,
    /// Last error that occurred
    last_error: IniReaderError,
    /// Save any line within a section even if it doesn't follow the key=value format
    pub store_any_line: bool,
    /// Allow section titles to appear multiple times
    pub allow_dup_section_titles: bool,
    /// Keep empty sections while parsing
    pub keep_empty_section: bool,
}

impl Default for IniReader {
    fn default() -> Self {
        Self::new()
    }
}

impl IniReader {
    /// Create a new INI reader
    pub fn new() -> Self {
        IniReader {
            ini: Ini::new(),
            parsed: false,
            current_section: String::new(),
            direct_save_sections: HashSet::new(),
            section_order: Vec::new(),
            content: HashMap::new(),
            last_error: IniReaderError::None,
            store_any_line: false,
            allow_dup_section_titles: false,
            keep_empty_section: true,
        }
    }

    /// Add a section to be saved directly without processing
    pub fn add_direct_save_section(&mut self, section: &str) {
        self.direct_save_sections.insert(section.to_string());
    }

    /// Erase all contents of the current section
    pub fn erase_section(&mut self) {
        if self.current_section.is_empty() {
            return;
        }

        // Remove the section from the ini
        self.ini.remove_section(&self.current_section);

        // Add it back as an empty section
        if let Some(section_map) = self.content.get_mut(&self.current_section) {
            section_map.clear();
        }

        // Make sure the section exists in the ini
        self.ini.set(&self.current_section, "", None);
    }

    /// Create a new INI reader and parse a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, IniReaderError> {
        let mut reader = IniReader::new();
        reader.parse_file(path)?;
        Ok(reader)
    }

    /// Get the last error as a string
    pub fn get_last_error(&self) -> String {
        self.last_error.to_string()
    }

    /// Parse INI content into the internal data structure
    pub fn parse(&mut self, content: &str) -> Result<(), IniReaderError> {
        if content.is_empty() {
            self.last_error = IniReaderError::Empty;
            return Err(IniReaderError::Empty);
        }

        // Configure the parser
        let mut ini = Ini::new();
        // Parse the content
        match ini.read(content.to_string()) {
            Ok(_) => {
                self.ini = ini;
                self.parsed = true;

                // Extract sections and their contents
                self.section_order.clear();
                self.content.clear();

                // Get all sections
                self.direct_save_sections = self.ini.sections().iter().map(|s| s.clone()).collect();
                // TODO: Process each section

                self.last_error = IniReaderError::None;
                Ok(())
            }
            Err(_) => {
                self.last_error = IniReaderError::NotParsed;
                Err(IniReaderError::NotParsed)
            }
        }
    }

    /// Parse an INI file
    pub fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), IniReaderError> {
        // Check if file exists
        if !path.as_ref().exists() {
            self.last_error = IniReaderError::NotExist;
            return Err(IniReaderError::NotExist);
        }

        // Read the file
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        // Parse the content
        self.parse(&content)
    }

    /// Check if a section exists
    pub fn section_exist(&self, section: &str) -> bool {
        self.content.contains_key(section)
    }

    /// Get the count of sections
    pub fn section_count(&self) -> usize {
        self.content.len()
    }

    /// Get all section names
    pub fn get_section_names(&self) -> &[String] {
        &self.section_order
    }

    /// Set the current section
    pub fn set_current_section(&mut self, section: &str) {
        self.current_section = section.to_string();
    }

    /// Enter a section with the given name
    pub fn enter_section(&mut self, section: &str) -> Result<(), IniReaderError> {
        if !self.section_exist(section) {
            self.last_error = IniReaderError::NotExist;
            return Err(IniReaderError::NotExist);
        }

        self.current_section = section.to_string();
        self.last_error = IniReaderError::None;
        Ok(())
    }

    /// Check if an item exists in the given section
    pub fn item_exist(&self, section: &str, item_name: &str) -> bool {
        if !self.section_exist(section) {
            return false;
        }

        self.content
            .get(section)
            .map(|items| items.contains_key(item_name))
            .unwrap_or(false)
    }

    /// Check if an item exists in the current section
    pub fn item_exist_current(&self, item_name: &str) -> bool {
        if self.current_section.is_empty() {
            return false;
        }

        self.item_exist(&self.current_section, item_name)
    }

    /// Get all items in a section
    pub fn get_items(&self, section: &str) -> Result<HashMap<String, String>, IniReaderError> {
        if !self.parsed {
            return Err(IniReaderError::NotParsed);
        }

        if !self.section_exist(section) {
            return Err(IniReaderError::NotExist);
        }

        Ok(self.content.get(section).cloned().unwrap_or_default())
    }

    /// Get all items with the same name prefix in a section
    pub fn get_all(&self, section: &str, item_name: &str) -> Result<Vec<String>, IniReaderError> {
        if !self.parsed {
            return Err(IniReaderError::NotParsed);
        }

        if !self.section_exist(section) {
            return Err(IniReaderError::NotExist);
        }

        let mut results = Vec::new();

        if let Some(items) = self.content.get(section) {
            for (key, value) in items {
                if key.starts_with(item_name) {
                    results.push(value.clone());
                }
            }
        }

        Ok(results)
    }

    /// Get all items with the same name prefix in the current section
    pub fn get_all_current(&self, item_name: &str) -> Result<Vec<String>, IniReaderError> {
        if self.current_section.is_empty() {
            return Err(IniReaderError::NotExist);
        }

        self.get_all(&self.current_section, item_name)
    }

    /// Get an item with the exact same name in the given section
    pub fn get(&self, section: &str, item_name: &str) -> String {
        if !self.parsed || !self.section_exist(section) {
            return String::new();
        }

        self.content
            .get(section)
            .and_then(|items| items.get(item_name))
            .cloned()
            .unwrap_or_default()
    }

    /// Get an item with the exact same name in the current section
    pub fn get_current(&self, item_name: &str) -> String {
        if self.current_section.is_empty() {
            return String::new();
        }

        self.get(&self.current_section, item_name)
    }

    /// Get a boolean value from the given section
    pub fn get_bool(&self, section: &str, item_name: &str) -> bool {
        self.get(section, item_name) == "true"
    }

    /// Get a boolean value from the current section
    pub fn get_bool_current(&self, item_name: &str) -> bool {
        self.get_current(item_name) == "true"
    }

    /// Get an integer value from the given section
    pub fn get_int(&self, section: &str, item_name: &str) -> i32 {
        self.get(section, item_name).parse::<i32>().unwrap_or(0)
    }

    /// Get an integer value from the current section
    pub fn get_int_current(&self, item_name: &str) -> i32 {
        self.get_current(item_name).parse::<i32>().unwrap_or(0)
    }

    /// Set a value in the given section
    pub fn set(
        &mut self,
        section: &str,
        item_name: &str,
        item_val: &str,
    ) -> Result<(), IniReaderError> {
        if section.is_empty() {
            self.last_error = IniReaderError::NotExist;
            return Err(IniReaderError::NotExist);
        }

        if !self.parsed {
            self.parsed = true;
        }

        // If section is {NONAME}, we're setting key directly to the current section
        let real_section = if section == "{NONAME}" {
            if self.current_section.is_empty() {
                self.last_error = IniReaderError::NotExist;
                return Err(IniReaderError::NotExist);
            }
            &self.current_section
        } else {
            section
        };

        // Add section if it doesn't exist
        if !self.section_exist(real_section) {
            self.section_order.push(real_section.to_string());
            self.content
                .insert(real_section.to_string(), HashMap::new());
        }

        // Update both the ini parser and our content HashMap
        self.ini
            .set(real_section, item_name, Some(item_val.to_string()));

        if let Some(section_map) = self.content.get_mut(real_section) {
            section_map.insert(item_name.to_string(), item_val.to_string());
        }

        self.last_error = IniReaderError::None;
        Ok(())
    }

    /// Set a value in the current section
    pub fn set_current(&mut self, item_name: &str, item_val: &str) -> Result<(), IniReaderError> {
        let current = self.current_section.clone();
        if current.is_empty() {
            self.last_error = IniReaderError::NotExist;
            return Err(IniReaderError::NotExist);
        }

        self.set(&current, item_name, item_val)
    }

    /// Set a boolean value in the given section
    pub fn set_bool(
        &mut self,
        section: &str,
        item_name: &str,
        item_val: bool,
    ) -> Result<(), IniReaderError> {
        self.set(section, item_name, if item_val { "true" } else { "false" })
    }

    /// Set a boolean value in the current section
    pub fn set_bool_current(
        &mut self,
        item_name: &str,
        item_val: bool,
    ) -> Result<(), IniReaderError> {
        let current = self.current_section.clone();
        self.set(&current, item_name, if item_val { "true" } else { "false" })
    }

    /// Set an integer value in the given section
    pub fn set_int(
        &mut self,
        section: &str,
        item_name: &str,
        item_val: i32,
    ) -> Result<(), IniReaderError> {
        self.set(section, item_name, &item_val.to_string())
    }

    /// Set an integer value in the current section
    pub fn set_int_current(
        &mut self,
        item_name: &str,
        item_val: i32,
    ) -> Result<(), IniReaderError> {
        let current = self.current_section.clone();
        self.set(&current, item_name, &item_val.to_string())
    }

    /// Remove a section
    pub fn remove_section(&mut self, section: &str) {
        if !self.section_exist(section) {
            return;
        }

        // Remove from the ini parser
        self.ini.remove_section(section);

        // Remove from our content HashMap
        self.content.remove(section);

        // Remove from section order
        if let Some(pos) = self.section_order.iter().position(|s| s == section) {
            self.section_order.remove(pos);
        }

        // Clear current section if it was the removed one
        if self.current_section == section {
            self.current_section.clear();
        }
    }

    /// Remove the current section
    pub fn remove_current_section(&mut self) {
        let current = self.current_section.clone();
        if current.is_empty() {
            return;
        }

        self.remove_section(&current);
    }

    /// Export the INI to a string
    pub fn to_string(&self) -> String {
        if !self.parsed {
            return String::new();
        }

        // Use the ini parser to write the config
        self.ini.writes()
    }

    /// Export the INI to a file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        if !self.parsed {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "INI not parsed"));
        }

        let content = self.to_string();
        std::fs::write(path, content)?;
        Ok(())
    }
}
