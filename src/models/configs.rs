/// Configuration for regex-based matching operations
#[derive(Debug, Clone)]
pub struct RegexMatchConfig {
    pub regex: String,
    pub replacement: String,
}

/// Collection of regex match configurations
pub type RegexMatchConfigs = Vec<RegexMatchConfig>;

/// Settings for subscription export operations
#[derive(Debug, Clone)]
pub struct ExtraSettings {
    /// Whether to enable the rule generator
    pub enable_rule_generator: bool,
    /// Whether to overwrite original rules
    pub overwrite_original_rules: bool,
    /// Rename operations to apply
    pub rename_array: RegexMatchConfigs,
    /// Emoji operations to apply
    pub emoji_array: RegexMatchConfigs,
    /// Whether to add emoji
    pub add_emoji: bool,
    /// Whether to remove emoji
    pub remove_emoji: bool,
    /// Whether to append proxy type
    pub append_proxy_type: bool,
    /// Whether to output as node list
    pub nodelist: bool,
    /// Whether to sort nodes
    pub sort_flag: bool,
    /// Whether to filter deprecated nodes
    pub filter_deprecated: bool,
    /// Whether to use new field names in Clash
    pub clash_new_field_name: bool,
    /// Whether to use scripts in Clash
    pub clash_script: bool,
    /// Path to Surge SSR binary
    pub surge_ssr_path: String,
    /// Prefix for managed configs
    pub managed_config_prefix: String,
    /// QuantumultX device ID
    pub quanx_dev_id: String,
    /// UDP support flag
    pub udp: Option<bool>,
    /// TCP Fast Open support flag
    pub tfo: Option<bool>,
    /// Skip certificate verification flag
    pub skip_cert_verify: Option<bool>,
    /// TLS 1.3 support flag
    pub tls13: Option<bool>,
    /// Whether to use classical ruleset in Clash
    pub clash_classical_ruleset: bool,
    /// Script for sorting nodes
    pub sort_script: String,
    /// Style for Clash proxies output
    pub clash_proxies_style: String,
    /// Style for Clash proxy groups output
    pub clash_proxy_groups_style: String,
    /// Whether the export is authorized
    pub authorized: bool,
    /// JavaScript runtime context (not implemented in Rust version)
    pub js_context: Option<()>,
}

impl Default for ExtraSettings {
    fn default() -> Self {
        ExtraSettings {
            enable_rule_generator: true,
            overwrite_original_rules: true,
            rename_array: Vec::new(),
            emoji_array: Vec::new(),
            add_emoji: false,
            remove_emoji: false,
            append_proxy_type: false,
            nodelist: false,
            sort_flag: false,
            filter_deprecated: false,
            clash_new_field_name: false,
            clash_script: false,
            surge_ssr_path: String::new(),
            managed_config_prefix: String::new(),
            quanx_dev_id: String::new(),
            udp: None,
            tfo: None,
            skip_cert_verify: None,
            tls13: None,
            clash_classical_ruleset: false,
            sort_script: String::new(),
            clash_proxies_style: "flow".to_string(),
            clash_proxy_groups_style: "flow".to_string(),
            authorized: false,
            js_context: None,
        }
    }
}

/// Type of proxy group
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyGroupType {
    Select,
    URLTest,
    Fallback,
    LoadBalance,
    Relay,
    SSID,
    Smart,
}

impl ProxyGroupType {
    /// Get string representation of the proxy group type
    pub fn as_str(&self) -> &'static str {
        match self {
            ProxyGroupType::Select => "select",
            ProxyGroupType::URLTest => "url-test",
            ProxyGroupType::LoadBalance => "load-balance",
            ProxyGroupType::Fallback => "fallback",
            ProxyGroupType::Relay => "relay",
            ProxyGroupType::SSID => "ssid",
            ProxyGroupType::Smart => "smart",
        }
    }
}

/// Load balancing strategy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BalanceStrategy {
    ConsistentHashing,
    RoundRobin,
}

impl BalanceStrategy {
    /// Get string representation of the balance strategy
    pub fn as_str(&self) -> &'static str {
        match self {
            BalanceStrategy::ConsistentHashing => "consistent-hashing",
            BalanceStrategy::RoundRobin => "round-robin",
        }
    }
}

/// Configuration for a proxy group
#[derive(Debug, Clone)]
pub struct ProxyGroupConfig {
    /// Name of the proxy group
    pub name: String,
    /// Type of the proxy group
    pub group_type: ProxyGroupType,
    /// List of proxy names in this group
    pub proxies: Vec<String>,
    /// List of provider names used by this group
    pub using_provider: Vec<String>,
    /// URL for testing
    pub url: String,
    /// Interval in seconds between tests
    pub interval: i32,
    /// Timeout in seconds for tests
    pub timeout: i32,
    /// Tolerance value for tests
    pub tolerance: i32,
    /// Strategy for load balancing
    pub strategy: BalanceStrategy,
    /// Whether to use lazy loading
    pub lazy: bool,
    /// Whether to disable UDP support
    pub disable_udp: bool,
    /// Whether to persist connections
    pub persistent: bool,
    /// Whether to evaluate before use
    pub evaluate_before_use: bool,
}

impl Default for ProxyGroupConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            group_type: ProxyGroupType::Select,
            proxies: Vec::new(),
            using_provider: Vec::new(),
            url: String::new(),
            interval: 0,
            timeout: 0,
            tolerance: 0,
            strategy: BalanceStrategy::ConsistentHashing,
            lazy: false,
            disable_udp: false,
            persistent: false,
            evaluate_before_use: false,
        }
    }
}

impl ProxyGroupConfig {
    /// Create a new proxy group config
    pub fn new(name: String, group_type: ProxyGroupType) -> Self {
        Self {
            name,
            group_type,
            ..Default::default()
        }
    }

    /// Get string representation of the group type
    pub fn type_str(&self) -> &'static str {
        self.group_type.as_str()
    }

    /// Get string representation of the balance strategy
    pub fn strategy_str(&self) -> &'static str {
        self.strategy.as_str()
    }
}

/// A collection of proxy group configurations
pub type ProxyGroupConfigs = Vec<ProxyGroupConfig>;
