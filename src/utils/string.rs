//! String utility functions for text processing

/// Replace all occurrences of a string with another
///
/// # Arguments
///
/// * `s` - The input string
/// * `from` - The string to replace
/// * `to` - The replacement string
///
/// # Returns
///
/// The string with all occurrences replaced
pub fn replace_all_distinct(s: &str, from: &str, to: &str) -> String {
    let mut result = s.to_string();
    let mut position = 0;

    while let Some(found_pos) = result[position..].find(from) {
        let absolute_pos = position + found_pos;
        result.replace_range(absolute_pos..absolute_pos + from.len(), to);
        position = absolute_pos + to.len();
    }

    result
}

/// Check if a string starts with a specific prefix
///
/// # Arguments
///
/// * `s` - The string to check
/// * `prefix` - The prefix to look for
///
/// # Returns
///
/// True if the string starts with the prefix, false otherwise
pub fn starts_with(s: &str, prefix: &str) -> bool {
    s.starts_with(prefix)
}

/// Check if a string ends with a specific suffix
///
/// # Arguments
///
/// * `s` - The string to check
/// * `suffix` - The suffix to look for
///
/// # Returns
///
/// True if the string ends with the suffix, false otherwise
pub fn ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

/// Convert a string to lowercase
///
/// # Arguments
///
/// * `s` - The string to convert
///
/// # Returns
///
/// The lowercase version of the string
pub fn to_lower(s: &str) -> String {
    s.to_lowercase()
}

/// Trim whitespace from the beginning and end of a string
///
/// # Arguments
///
/// * `s` - The string to trim
///
/// # Returns
///
/// The trimmed string
pub fn trim(s: &str) -> &str {
    s.trim()
}

pub fn trim_whitespace(s: &str, before: bool, after: bool) -> String {
    if before {
        s.trim_start().to_string()
    } else if after {
        s.trim_end().to_string()
    } else {
        s.trim().to_string()
    }
}

/// Find the position of a substring in a string
///
/// # Arguments
///
/// * `s` - The string to search in
/// * `search` - The substring to find
///
/// # Returns
///
/// The position of the substring if found, None otherwise
pub fn find_str(s: &str, search: &str) -> Option<usize> {
    s.find(search)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_all_distinct() {
        assert_eq!(replace_all_distinct("hello world", "o", "0"), "hell0 w0rld");
        assert_eq!(replace_all_distinct("test-test", "-", "_"), "test_test");
        assert_eq!(replace_all_distinct("abcabc", "a", "x"), "xbcxbc");
    }

    #[test]
    fn test_starts_with() {
        assert!(starts_with("hello world", "hello"));
        assert!(!starts_with("hello world", "world"));
    }

    #[test]
    fn test_ends_with() {
        assert!(ends_with("hello world", "world"));
        assert!(!ends_with("hello world", "hello"));
    }

    #[test]
    fn test_to_lower() {
        assert_eq!(to_lower("HELLO"), "hello");
        assert_eq!(to_lower("Hello World"), "hello world");
    }

    #[test]
    fn test_trim() {
        assert_eq!(trim("  hello  "), "hello");
        assert_eq!(trim("\t\nhello\r\n"), "hello");
    }
}
