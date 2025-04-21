use base64::{
    alphabet::{STANDARD as STANDARD_ALPHABET, URL_SAFE as URL_SAFE_ALPHABET},
    engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig},
    engine::DecodePaddingMode,
    Engine as _,
};

const NO_PAD: GeneralPurposeConfig = GeneralPurposeConfig::new()
    .with_encode_padding(false)
    .with_decode_padding_mode(DecodePaddingMode::Indifferent);
const STANDARD_NO_PAD: GeneralPurpose = GeneralPurpose::new(&STANDARD_ALPHABET, NO_PAD);
const URL_SAFE_NO_PAD: GeneralPurpose = GeneralPurpose::new(&URL_SAFE_ALPHABET, NO_PAD);

/// Encodes a string to Base64 format. Auto detect if the input is URL-safe.
///
/// # Arguments
/// * `input` - The string to encode.
///
/// # Returns
/// The Base64 encoded string.
pub fn base64_encode(input: &str) -> String {
    let accept_urlsafe = input.contains("+") || input.contains("/") || input.contains("=");

    let encoded = match accept_urlsafe {
        true => URL_SAFE_NO_PAD.encode(input),
        false => STANDARD_NO_PAD.encode(input),
    };
    encoded
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
    let decoded = match accept_urlsafe {
        true => URL_SAFE_NO_PAD.decode(input).unwrap_or_default(),
        false => STANDARD_NO_PAD.decode(input).unwrap_or_default(),
    };
    String::from_utf8_lossy(&decoded).to_string()
}

/// Decodes a URL-safe Base64 string to its original form.
pub fn url_safe_base64_decode(input: &str) -> String {
    base64_decode(input, true)
}

/// Encodes a string to URL-safe Base64 format.
pub fn url_safe_base64_encode(input: &str) -> String {
    base64_encode(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode() {
        let input = "64.137.228.35:5760:auth_sha1_v4:chacha20:tls1.2_ticket_auth:ZG91Yi5pby9zc3poZngvKjU3NjA/?remarks=5pys5YWN6LS56LSm5Y-35p2l6IeqOmRvdWIuaW8vc3N6aGZ4Lw";
        let encoded = base64_encode(input);
        assert_eq!(encoded, "NjQuMTM3LjIyOC4zNTo1NzYwOmF1dGhfc2hhMV92NDpjaGFjaGEyMDp0bHMxLjJfdGlja2V0X2F1dGg6Wkc5MVlpNXBieTl6YzNwb1puZ3ZLalUzTmpBLz9yZW1hcmtzPTVweXM1WVdONkxTNTZMU201WS0zNXAybDZJZXFPbVJ2ZFdJdWFXOHZjM042YUdaNEx3");
    }

    #[test]
    fn test_base64_decode() {
        let input = "NjQuMTM3LjIyOC4zNTo1NzYwOmF1dGhfc2hhMV92NDpjaGFjaGEyMDp0bHMxLjJfdGlja2V0X2F1dGg6Wkc5MVlpNXBieTl6YzNwb1puZ3ZLalUzTmpBLz9yZW1hcmtzPTVweXM1WVdONkxTNTZMU201WS0zNXAybDZJZXFPbVJ2ZFdJdWFXOHZjM042YUdaNEx3";
        let decoded = base64_decode(input, true);
        assert_eq!(decoded, "64.137.228.35:5760:auth_sha1_v4:chacha20:tls1.2_ticket_auth:ZG91Yi5pby9zc3poZngvKjU3NjA/?remarks=5pys5YWN6LS56LSm5Y-35p2l6IeqOmRvdWIuaW8vc3N6aGZ4Lw");
    }
}
