use base64::{engine::general_purpose, Engine as _};

/// Encodes a string to Base64 format.
pub fn base64_encode(input: &str) -> String {
    general_purpose::STANDARD.encode(input)
}

/// Decodes a Base64 string to its original form.
///
/// # Arguments
/// * `input` - The Base64 encoded string.
/// * `accept_urlsafe` - A boolean indicating whether to accept URL-safe Base64 encoding.
///
/// # Returns
/// The decoded string, or an empty string if the input is invalid.
pub fn base64_decode(input: &str, accept_urlsafe: bool) -> String {
    let engine = if accept_urlsafe {
        general_purpose::URL_SAFE
    } else {
        general_purpose::STANDARD
    };

    match engine.decode(input) {
        Ok(decoded) => String::from_utf8_lossy(&decoded).to_string(),
        Err(_) => String::new(), // Handle invalid Base64 input
    }
}

/// Reverses a URL-safe Base64 string to standard Base64 format.
pub fn url_safe_base64_reverse(input: &str) -> String {
    input.replace('-', "+").replace('_', "/")
}

/// Converts a Base64 string to URL-safe Base64 format by replacing specific characters.
pub fn url_safe_base64_apply(input: &str) -> String {
    input
        .replace('+', "-")
        .replace('/', "_")
        .replace('=', "") // Remove padding
}

/// Decodes a URL-safe Base64 string to its original form.
pub fn url_safe_base64_decode(input: &str) -> String {
    base64_decode(&url_safe_base64_reverse(input), true)
}

/// Encodes a string to URL-safe Base64 format.
pub fn url_safe_base64_encode(input: &str) -> String {
    url_safe_base64_apply(&base64_encode(input))
}
