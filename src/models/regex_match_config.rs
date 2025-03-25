use serde::Deserialize;

use crate::utils::{matcher::reg_find, reg_replace};

/// Configuration for regex-based matching operations
#[derive(Debug, Clone, Deserialize)]
pub struct RegexMatchConfig {
    #[serde(rename = "match")]
    pub _match: String,
    pub replace: String,
    // pub script: String,
}

impl RegexMatchConfig {
    pub fn process(&self, remark: &mut String) {
        if reg_find(remark, &self._match) {
            *remark = reg_replace(remark, &self._match, &self.replace, true, false);
        }
    }
}

/// Collection of regex match configurations
pub type RegexMatchConfigs = Vec<RegexMatchConfig>;
