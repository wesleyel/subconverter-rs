use std::collections::HashMap;
use std::str::FromStr;

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

/// Rust equivalent of RegexMatchConfig in C++
#[derive(Debug, Clone)]
pub struct RegexMatchConfig {
    pub match_pattern: String,
    pub replace: String,
    pub script: String,
}

/// Represents a collection of RegexMatchConfig
pub type RegexMatchConfigs = Vec<RegexMatchConfig>;

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
        ParseSettings {
            proxy: None,
            exclude_remarks: None,
            include_remarks: None,
            stream_rules: None,
            time_rules: None,
            sub_info: None,
            authorized: false,
            request_header: None,
            #[cfg(feature = "js_runtime")]
            js_runtime: None,
            #[cfg(feature = "js_runtime")]
            js_context: None,
        }
    }
}
