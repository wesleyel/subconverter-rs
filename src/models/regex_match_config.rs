use serde::Deserialize;

/// Configuration for regex-based matching operations
#[derive(Debug, Clone, Deserialize)]
pub struct RegexMatchConfig {
    #[serde(rename = "match")]
    pub _match: String,
    pub replace: String,
    // pub script: String,
}

/// Collection of regex match configurations
pub type RegexMatchConfigs = Vec<RegexMatchConfig>;
