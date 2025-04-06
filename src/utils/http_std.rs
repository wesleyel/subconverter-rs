use crate::parser::parse_settings::CaseInsensitiveString;
use crate::utils::system::get_system_proxy;
use std::collections::HashMap;
use std::time::Duration;

use reqwest::{Client, Proxy, StatusCode};

/// Default timeout for HTTP requests in seconds
const DEFAULT_TIMEOUT: u64 = 15;

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
    proxy_config: &ProxyConfig,
    headers: Option<&HashMap<CaseInsensitiveString, String>>,
) -> Result<(String, HashMap<String, String>), String> {
    // Build client with proxy if specified
    let mut client_builder = Client::builder()
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
        .user_agent("subconverter-rs");

    if let Some(proxy) = &proxy_config.proxy {
        if !proxy.is_empty() {
            match Proxy::all(proxy) {
                Ok(proxy) => {
                    client_builder = client_builder.proxy(proxy);
                }
                Err(e) => {
                    return Err(format!("Failed to set proxy: {}", e));
                }
            }
        }
    }

    let client = match client_builder.build() {
        Ok(client) => client,
        Err(e) => {
            return Err(format!("Failed to build HTTP client: {}", e));
        }
    };

    // Build request with headers if specified
    let mut request_builder = client.get(url);
    if let Some(custom_headers) = headers {
        for (key, value) in custom_headers {
            request_builder = request_builder.header(key.to_string(), value);
        }
    }

    // Send request and get response
    let response = match request_builder.send().await {
        Ok(resp) => resp,
        Err(e) => {
            return Err(format!("Failed to send request: {}", e));
        }
    };

    // Get response headers
    let mut resp_headers = HashMap::new();
    for (key, value) in response.headers() {
        if let Ok(v) = value.to_str() {
            resp_headers.insert(key.to_string(), v.to_string());
        }
    }

    // Check status code
    if response.status() != StatusCode::OK {
        return Err(format!("HTTP error: {}", response.status()));
    }

    // Get response body
    match response.text().await {
        Ok(body) => Ok((body, resp_headers)),
        Err(e) => Err(format!("Failed to read response body: {}", e)),
    }
}

/// Synchronous version of web_get_async that uses tokio runtime to run the async function
///
/// This function is provided for compatibility with the existing codebase.
pub fn web_get(
    url: &str,
    proxy_config: &ProxyConfig,
    headers: Option<&HashMap<CaseInsensitiveString, String>>,
) -> Result<(String, HashMap<String, String>), String> {
    // Create a tokio runtime for running the async function
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            return Err(format!("Failed to create tokio runtime: {}", e));
        }
    };

    // Run the async function in the runtime
    rt.block_on(web_get_async(url, proxy_config, headers))
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