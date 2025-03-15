use super::types::{ConfType, Proxy, ProxyType};
use crate::utils::base64::{base64_decode, base64_encode};
use serde_json::Value as JsonValue;

fn parse_json(input: &str) -> Option<JsonValue> {
    serde_json::from_str(input).ok()
}

fn is_link_valid(link: &str) -> bool {
    link.starts_with("http") || link.starts_with("https")
}
