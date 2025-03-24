use std::collections::HashMap;
use std::str::FromStr;

use crate::models::RegexMatchConfigs;
use crate::Settings;

/// Case-insensitive string for use as HashMap keys
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CaseInsensitiveString(String);

impl FromStr for CaseInsensitiveString {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(CaseInsensitiveString(s.to_string()))
    }
}

impl std::fmt::Display for CaseInsensitiveString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<str> for CaseInsensitiveString {
    fn eq(&self, other: &str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

/// Rust equivalent of the parse_settings struct in C++
/// Used for controlling the behavior of parsing functions
#[derive(Debug, Clone)]
pub struct ParseSettings {
    /// Proxy to use for downloading subscriptions
    pub proxy: Option<String>,

    /// Array of remarks to exclude
    pub exclude_remarks: Option<Vec<String>>,

    /// Array of remarks to include
    pub include_remarks: Option<Vec<String>>,

    /// Rules for stream matching
    pub stream_rules: Option<RegexMatchConfigs>,

    /// Rules for time matching
    pub time_rules: Option<RegexMatchConfigs>,

    /// Subscription information
    pub sub_info: Option<String>,

    /// Whether operations requiring authorization are allowed
    pub authorized: bool,

    /// HTTP request headers
    pub request_header: Option<HashMap<CaseInsensitiveString, String>>,

    /// JavaScript runtime - optional depending on feature flags
    #[cfg(feature = "js_runtime")]
    pub js_runtime: Option<()>, // Placeholder for actual JS runtime type

    /// JavaScript context - optional depending on feature flags
    #[cfg(feature = "js_runtime")]
    pub js_context: Option<()>, // Placeholder for actual JS context type
}

impl Default for ParseSettings {
    fn default() -> Self {
        // Get global settings
        let settings = Settings::current();

        ParseSettings {
            proxy: Some(settings.proxy_subscription.clone()),
            exclude_remarks: if settings.exclude_remarks.is_empty() {
                None
            } else {
                Some(settings.exclude_remarks.clone())
            },
            include_remarks: if settings.include_remarks.is_empty() {
                None
            } else {
                Some(settings.include_remarks.clone())
            },
            stream_rules: None, // TODO: Get from global settings
            time_rules: None,   // TODO: Get from global settings
            sub_info: None,
            authorized: !settings.api_access_token.is_empty(),
            request_header: None,
            #[cfg(feature = "js_runtime")]
            js_runtime: None,
            #[cfg(feature = "js_runtime")]
            js_context: None,
        }
    }
}

/// Create a new ParseSettings instance with defaults from global settings
pub fn create_parse_settings() -> ParseSettings {
    ParseSettings::default()
}

/// Create a new ParseSettings instance with authorization
pub fn create_authorized_settings() -> ParseSettings {
    let mut settings = ParseSettings::default();
    settings.authorized = true;
    settings
}
