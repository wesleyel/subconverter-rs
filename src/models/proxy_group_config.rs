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
    pub interval: u32,
    /// Timeout in seconds for tests
    pub timeout: u32,
    /// Tolerance value for tests
    pub tolerance: u32,
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
