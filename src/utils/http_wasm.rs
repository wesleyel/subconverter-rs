use crate::utils::system::get_system_proxy;
use case_insensitive_string::CaseInsensitiveString;
use std::collections::HashMap;

use js_sys::{Array, Object};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub proxy: Option<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        ProxyConfig { proxy: None }
    }
}

pub fn parse_proxy(proxy_str: &str) -> ProxyConfig {
    if proxy_str == "SYSTEM" {
        return ProxyConfig {
            proxy: Some(get_system_proxy()),
        };
    } else if proxy_str == "NONE" {
        return ProxyConfig { proxy: None };
    } else if !proxy_str.is_empty() {
        return ProxyConfig {
            proxy: Some(proxy_str.to_string()),
        };
    }
    ProxyConfig { proxy: None }
}

/// Makes an HTTP request to the specified URL
///
/// # Arguments
/// * `url` - The URL to request
/// * `proxy_str` - Optional proxy string (e.g., "http://127.0.0.1:8080")
/// * `headers` - Optional custom headers
///
/// # Returns
/// * `Ok(String)` - The response body as a string
/// * `Err(String)` - Error message if the request failed
pub async fn web_get_async(
    url: &str,
    _proxy_config: &ProxyConfig,
    headers: Option<&HashMap<CaseInsensitiveString, String>>,
) -> Result<(String, HashMap<String, String>), String> {
    // In WASM environment, we use the fetch API
    // Note: Proxy configuration is not supported in WASM environment
    #[allow(unused_mut)]
    let mut opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    // Create request object
    let request = match Request::new_with_str_and_init(url, &opts) {
        Ok(req) => req,
        Err(e) => return Err(format!("Failed to create request: {:?}", e)),
    };

    // Add headers if specified
    if let Some(custom_headers) = headers {
        let headers_obj = request.headers();

        for (key, value) in custom_headers {
            if let Err(e) = headers_obj.set(key.to_string().as_str(), value) {
                return Err(format!("Failed to set header {}: {:?}", key, e));
            }
        }
    }

    // Get window object
    let window = match web_sys::window() {
        Some(w) => w,
        None => return Err("No window object available".to_string()),
    };

    // Fetch the request
    let resp_promise = window.fetch_with_request(&request);

    // Wait for the response
    let resp_value = match JsFuture::from(resp_promise).await {
        Ok(val) => val,
        Err(e) => return Err(format!("Failed to get response: {:?}", e)),
    };

    // Convert to Response object
    let response: Response = match resp_value.dyn_into() {
        Ok(resp) => resp,
        Err(_) => return Err("Failed to convert response".to_string()),
    };

    // Check status
    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    // Get response headers
    let mut resp_headers = HashMap::new();
    let headers = response.headers();
    let js_headers: Object = headers.into();
    let entries = Object::entries(&js_headers);
    let entries_array: Array = entries.into();
    for i in 0..entries_array.length() {
        let entry = entries_array.get(i);
        let entry_array: Array = entry.into();
        let key = entry_array.get(0);
        let value = entry_array.get(1);
        if let (Some(key_str), Some(value_str)) = (key.as_string(), value.as_string()) {
            resp_headers.insert(key_str, value_str);
        }
    }

    // Get response body as text
    let text_promise = match response.text() {
        Ok(p) => p,
        Err(e) => return Err(format!("Failed to get response text: {:?}", e)),
    };

    let text_value = match JsFuture::from(text_promise).await {
        Ok(val) => val,
        Err(e) => return Err(format!("Failed to read response body: {:?}", e)),
    };

    let body = match text_value.as_string() {
        Some(s) => s,
        None => return Err("Failed to convert response to string".to_string()),
    };

    Ok((body, resp_headers))
}

/// Synchronous version of web_get_async that uses tokio runtime to run the async function
///
/// This function is provided for compatibility with the existing codebase.
pub fn web_get(
    _url: &str,
    _proxy_config: &ProxyConfig,
    _headers: Option<&HashMap<CaseInsensitiveString, String>>,
) -> Result<(String, HashMap<String, String>), String> {
    // In WASM environment, we can't block and wait for async operations
    // Users should use web_get_async directly
    Err("在WASM环境中，请直接使用web_get_async函数而不是web_get".to_string())
}

/// Version of web_get that returns only the body content
/// This is for backward compatibility where headers are not needed
/// Asynchronous version of web_get_content that returns only the body content
/// This is useful in WASM environment where synchronous functions can't be used
pub async fn web_get_content_async(
    url: &str,
    proxy_config: &ProxyConfig,
    headers: Option<&HashMap<CaseInsensitiveString, String>>,
) -> Result<String, String> {
    match web_get_async(url, proxy_config, headers).await {
        Ok((body, _)) => Ok(body),
        Err(e) => Err(e),
    }
}

/// Extract subscription info from HTTP headers
///
/// # Arguments
/// * `headers` - HTTP response headers
///
/// # Returns
/// * Subscription info string with key-value pairs
pub fn get_sub_info_from_header(headers: &HashMap<String, String>) -> String {
    let mut sub_info = String::new();

    // Extract upload and download
    let mut upload: u64 = 0;
    let mut download: u64 = 0;
    let mut total: u64 = 0;
    let mut expire: String = String::new();

    // Look for subscription-userinfo header
    if let Some(userinfo) = headers.get("subscription-userinfo") {
        for info_item in userinfo.split(';') {
            let info_item = info_item.trim();
            if info_item.starts_with("upload=") {
                if let Ok(value) = info_item[7..].parse::<u64>() {
                    upload = value;
                }
            } else if info_item.starts_with("download=") {
                if let Ok(value) = info_item[9..].parse::<u64>() {
                    download = value;
                }
            } else if info_item.starts_with("total=") {
                if let Ok(value) = info_item[6..].parse::<u64>() {
                    total = value;
                }
            } else if info_item.starts_with("expire=") {
                expire = info_item[7..].to_string();
            }
        }
    }

    // Add traffic info
    if upload > 0 || download > 0 {
        sub_info.push_str(&format!("upload={}, download={}", upload, download));
    }

    // Add total traffic
    if total > 0 {
        if !sub_info.is_empty() {
            sub_info.push_str(", ");
        }
        sub_info.push_str(&format!("total={}", total));
    }

    // Add expiry info
    if !expire.is_empty() {
        if !sub_info.is_empty() {
            sub_info.push_str(", ");
        }
        sub_info.push_str(&format!("expire={}", expire));
    }

    sub_info
}

/// Get subscription info from response headers with additional formatting
///
/// # Arguments
/// * `headers` - HTTP response headers
/// * `sub_info` - Mutable string to append info to
///
/// # Returns
/// * `true` if info was extracted, `false` otherwise
pub fn get_sub_info_from_response(
    headers: &HashMap<String, String>,
    sub_info: &mut String,
) -> bool {
    let header_info = get_sub_info_from_header(headers);
    if !header_info.is_empty() {
        if !sub_info.is_empty() {
            sub_info.push_str(", ");
        }
        sub_info.push_str(&header_info);
        true
    } else {
        false
    }
}
