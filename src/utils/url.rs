//! URL encoding/decoding utilities

use std::path::Path;

/// Encodes a string using URL encoding
///
/// # Arguments
/// * `input` - The string to encode
///
/// # Returns
/// * String containing the URL-encoded input
///
/// # Examples
/// ```
/// use subconverter_rs::utils::url::url_encode;
///
/// let encoded = url_encode("Hello World!");
/// assert_eq!(encoded, "Hello%20World%21");
/// ```
pub fn url_encode(input: &str) -> String {
    urlencoding::encode(input).into_owned()
}

/// Decodes a URL-encoded string
///
/// # Arguments
/// * `input` - The URL-encoded string to decode
///
/// # Returns
/// * String containing the decoded input
/// * Returns the original string if decoding fails
///
/// # Examples
/// ```
/// use subconverter_rs::utils::url::url_decode;
///
/// let decoded = url_decode("Hello%20World%21");
/// assert_eq!(decoded, "Hello World!");
/// ```
pub fn url_decode(input: &str) -> String {
    urlencoding::decode(input)
        .map(|cow| cow.into_owned())
        .unwrap_or_else(|_| input.to_string())
}
