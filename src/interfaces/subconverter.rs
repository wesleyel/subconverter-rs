use crate::generator::config::formats::{
    clash::proxy_to_clash, loon::proxy_to_loon, mellow::proxy_to_mellow, quan::proxy_to_quan,
    quanx::proxy_to_quanx, singbox::proxy_to_singbox, ss_sub::proxy_to_ss_sub,
    surge::proxy_to_surge,
};
use crate::models::{ExtraSettings, Proxy, ProxyGroupConfigs, RulesetContent};
use crate::parser::parse_settings::ParseSettings;
use crate::parser::subparser::add_nodes;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::path::Path;

/// The output format for subconverter
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum SubconverterTarget {
    Clash,
    ClashR,
    Surge(i32), // Surge version as parameter
    Surfboard,
    Mellow,
    SSSub,
    SS,
    SSR,
    V2Ray,
    Trojan,
    Mixed,
    Quantumult,
    QuantumultX,
    Loon,
    SSD,
    SingBox,
}

impl SubconverterTarget {
    /// Convert string to target enum
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "clash" => Some(SubconverterTarget::Clash),
            "clashr" => Some(SubconverterTarget::ClashR),
            "surge" => Some(SubconverterTarget::Surge(3)), // Default to Surge 3
            "surfboard" => Some(SubconverterTarget::Surfboard),
            "mellow" => Some(SubconverterTarget::Mellow),
            "sssub" => Some(SubconverterTarget::SSSub),
            "ss" => Some(SubconverterTarget::SS),
            "ssr" => Some(SubconverterTarget::SSR),
            "v2ray" => Some(SubconverterTarget::V2Ray),
            "trojan" => Some(SubconverterTarget::Trojan),
            "mixed" => Some(SubconverterTarget::Mixed),
            "quan" => Some(SubconverterTarget::Quantumult),
            "quanx" => Some(SubconverterTarget::QuantumultX),
            "loon" => Some(SubconverterTarget::Loon),
            "ssd" => Some(SubconverterTarget::SSD),
            "singbox" => Some(SubconverterTarget::SingBox),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn to_str(&self) -> String {
        match self {
            SubconverterTarget::Clash => "clash".to_string(),
            SubconverterTarget::ClashR => "clashr".to_string(),
            SubconverterTarget::Surge(ver) => format!("surge{}", ver),
            SubconverterTarget::Surfboard => "surfboard".to_string(),
            SubconverterTarget::Mellow => "mellow".to_string(),
            SubconverterTarget::SSSub => "sssub".to_string(),
            SubconverterTarget::SS => "ss".to_string(),
            SubconverterTarget::SSR => "ssr".to_string(),
            SubconverterTarget::V2Ray => "v2ray".to_string(),
            SubconverterTarget::Trojan => "trojan".to_string(),
            SubconverterTarget::Mixed => "mixed".to_string(),
            SubconverterTarget::Quantumult => "quan".to_string(),
            SubconverterTarget::QuantumultX => "quanx".to_string(),
            SubconverterTarget::Loon => "loon".to_string(),
            SubconverterTarget::SSD => "ssd".to_string(),
            SubconverterTarget::SingBox => "singbox".to_string(),
        }
    }

    /// Check if this is a simple subscription format (no rules and groups)
    pub fn is_simple(&self) -> bool {
        matches!(
            self,
            SubconverterTarget::SS
                | SubconverterTarget::SSR
                | SubconverterTarget::SSSub
                | SubconverterTarget::V2Ray
                | SubconverterTarget::Trojan
                | SubconverterTarget::Mixed
                | SubconverterTarget::SSD
        )
    }
}

/// Configuration for subconverter
#[derive(Debug, Clone)]
pub struct SubconverterConfig {
    /// Target conversion format
    pub target: SubconverterTarget,
    /// URLs to parse
    pub urls: Vec<String>,
    /// URLs to insert
    pub insert_urls: Vec<String>,
    /// Whether to prepend inserted nodes
    pub prepend_insert: bool,
    /// Custom group name
    pub group_name: Option<String>,
    /// Base configuration content for the target format
    pub base_content: HashMap<SubconverterTarget, String>,
    /// Ruleset contents to apply
    pub ruleset_content: Vec<RulesetContent>,
    /// Custom proxy groups
    pub proxy_groups: ProxyGroupConfigs,
    /// Include nodes matching these remarks
    pub include_remarks: Vec<String>,
    /// Exclude nodes matching these remarks
    pub exclude_remarks: Vec<String>,
    /// Rename patterns
    pub rename_patterns: Vec<(String, String)>,
    /// Emoji patterns
    pub emoji_patterns: Vec<(String, String)>,
    /// Additional settings
    pub extra: ExtraSettings,
    /// Device ID for certain formats
    pub device_id: Option<String>,
    /// Filename for download
    pub filename: Option<String>,
    /// Update interval in seconds
    pub update_interval: i32,
    /// Filter script
    pub filter_script: Option<String>,
    /// Whether update is strict
    pub update_strict: bool,
    /// Managed config prefix
    pub managed_config_prefix: String,
    /// Upload path
    pub upload_path: Option<String>,
    /// Whether to upload the result
    pub upload: bool,
    /// Proxy for fetching subscriptions
    pub proxy: Option<String>,
    /// Authentication token
    pub token: Option<String>,
    /// Whether this request is authorized
    pub authorized: bool,
    /// Subscription information
    pub sub_info: Option<String>,
}

/// Builder for SubconverterConfig
#[derive(Debug, Clone)]
pub struct SubconverterConfigBuilder {
    config: SubconverterConfig,
}

impl Default for SubconverterConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SubconverterConfigBuilder {
    /// Create a new default builder
    pub fn new() -> Self {
        SubconverterConfigBuilder {
            config: SubconverterConfig {
                target: SubconverterTarget::Clash,
                urls: Vec::new(),
                insert_urls: Vec::new(),
                prepend_insert: false,
                group_name: None,
                base_content: HashMap::new(),
                ruleset_content: Vec::new(),
                proxy_groups: Vec::new(),
                include_remarks: Vec::new(),
                exclude_remarks: Vec::new(),
                rename_patterns: Vec::new(),
                emoji_patterns: Vec::new(),
                extra: ExtraSettings::default(),
                device_id: None,
                filename: None,
                update_interval: 86400, // 24 hours
                filter_script: None,
                update_strict: false,
                managed_config_prefix: String::new(),
                upload_path: None,
                upload: false,
                proxy: None,
                token: None,
                authorized: false,
                sub_info: None,
            },
        }
    }

    /// Set the target format
    pub fn target(mut self, target: SubconverterTarget) -> Self {
        self.config.target = target;
        self
    }

    /// Set target from string
    pub fn target_from_str(mut self, target: &str) -> Self {
        if let Some(t) = SubconverterTarget::from_str(target) {
            self.config.target = t;
        }
        self
    }

    /// Set Surge version if target is Surge
    pub fn surge_version(mut self, version: i32) -> Self {
        if let SubconverterTarget::Surge(_) = self.config.target {
            self.config.target = SubconverterTarget::Surge(version);
        }
        self
    }

    /// Add a URL to parse
    pub fn add_url(mut self, url: &str) -> Self {
        self.config.urls.push(url.to_string());
        self
    }

    /// Set URLs to parse
    pub fn urls(mut self, urls: Vec<String>) -> Self {
        self.config.urls = urls;
        self
    }

    /// Set URLs from pipe-separated string
    pub fn urls_from_str(mut self, urls: &str) -> Self {
        self.config.urls = urls.split('|').map(|s| s.trim().to_string()).collect();
        self
    }

    /// Add an insert URL
    pub fn add_insert_url(mut self, url: &str) -> Self {
        self.config.insert_urls.push(url.to_string());
        self
    }

    /// Set insert URLs
    pub fn insert_urls(mut self, urls: Vec<String>) -> Self {
        self.config.insert_urls = urls;
        self
    }

    /// Set insert URLs from pipe-separated string
    pub fn insert_urls_from_str(mut self, urls: &str) -> Self {
        self.config.insert_urls = urls.split('|').map(|s| s.trim().to_string()).collect();
        self
    }

    /// Set whether to prepend inserted nodes
    pub fn prepend_insert(mut self, prepend: bool) -> Self {
        self.config.prepend_insert = prepend;
        self
    }

    /// Set custom group name
    pub fn group_name(mut self, name: Option<String>) -> Self {
        self.config.group_name = name;
        self
    }

    /// Add base content for a specific target
    pub fn add_base_content(mut self, target: SubconverterTarget, content: String) -> Self {
        self.config.base_content.insert(target, content);
        self
    }

    /// Set base content for a target from a file path
    pub fn add_base_content_from_file<P: AsRef<Path>>(
        mut self,
        target: SubconverterTarget,
        path: P,
    ) -> Self {
        if let Ok(content) = std::fs::read_to_string(path) {
            self.config.base_content.insert(target, content);
        }
        self
    }

    /// Add a ruleset content
    pub fn add_ruleset(mut self, ruleset: RulesetContent) -> Self {
        self.config.ruleset_content.push(ruleset);
        self
    }

    /// Set ruleset contents
    pub fn ruleset_content(mut self, rulesets: Vec<RulesetContent>) -> Self {
        self.config.ruleset_content = rulesets;
        self
    }

    /// Set proxy groups
    pub fn proxy_groups(mut self, groups: ProxyGroupConfigs) -> Self {
        self.config.proxy_groups = groups;
        self
    }

    /// Add an include remark pattern
    pub fn add_include_remark(mut self, pattern: &str) -> Self {
        self.config.include_remarks.push(pattern.to_string());
        self
    }

    /// Set include remark patterns
    pub fn include_remarks(mut self, patterns: Vec<String>) -> Self {
        self.config.include_remarks = patterns;
        self
    }

    /// Add an exclude remark pattern
    pub fn add_exclude_remark(mut self, pattern: &str) -> Self {
        self.config.exclude_remarks.push(pattern.to_string());
        self
    }

    /// Set exclude remark patterns
    pub fn exclude_remarks(mut self, patterns: Vec<String>) -> Self {
        self.config.exclude_remarks = patterns;
        self
    }

    /// Add a rename pattern
    pub fn add_rename_pattern(mut self, pattern: &str, replacement: &str) -> Self {
        self.config
            .rename_patterns
            .push((pattern.to_string(), replacement.to_string()));
        self
    }

    /// Set rename patterns
    pub fn rename_patterns(mut self, patterns: Vec<(String, String)>) -> Self {
        self.config.rename_patterns = patterns;
        self
    }

    /// Add an emoji pattern
    pub fn add_emoji_pattern(mut self, pattern: &str, emoji: &str) -> Self {
        self.config
            .emoji_patterns
            .push((pattern.to_string(), emoji.to_string()));
        self
    }

    /// Set emoji patterns
    pub fn emoji_patterns(mut self, patterns: Vec<(String, String)>) -> Self {
        self.config.emoji_patterns = patterns;
        self
    }

    /// Set extra settings
    pub fn extra(mut self, extra: ExtraSettings) -> Self {
        self.config.extra = extra;
        self
    }

    /// Set whether to append proxy type to remarks
    pub fn append_proxy_type(mut self, append: bool) -> Self {
        self.config.extra.append_proxy_type = append;
        self
    }

    /// Set whether to enable TCP Fast Open
    pub fn tfo(mut self, tfo: Option<bool>) -> Self {
        self.config.extra.tfo = tfo;
        self
    }

    /// Set whether to enable UDP
    pub fn udp(mut self, udp: Option<bool>) -> Self {
        self.config.extra.udp = udp;
        self
    }

    /// Set whether to skip certificate verification
    pub fn skip_cert_verify(mut self, skip: Option<bool>) -> Self {
        self.config.extra.skip_cert_verify = skip;
        self
    }

    /// Set whether to enable TLS 1.3
    pub fn tls13(mut self, tls13: Option<bool>) -> Self {
        self.config.extra.tls13 = tls13;
        self
    }

    /// Set whether to sort nodes
    pub fn sort(mut self, sort: bool) -> Self {
        self.config.extra.sort_flag = sort;
        self
    }

    /// Set sort script
    pub fn sort_script(mut self, script: String) -> Self {
        self.config.extra.sort_script = script;
        self
    }

    /// Set whether to filter deprecated nodes
    pub fn filter_deprecated(mut self, filter: bool) -> Self {
        self.config.extra.filter_deprecated = filter;
        self
    }

    /// Set whether to use new field names in Clash
    pub fn clash_new_field_name(mut self, new_field: bool) -> Self {
        self.config.extra.clash_new_field_name = new_field;
        self
    }

    /// Set whether to enable Clash script
    pub fn clash_script(mut self, enable: bool) -> Self {
        self.config.extra.clash_script = enable;
        self
    }

    /// Set whether to generate node list
    pub fn nodelist(mut self, nodelist: bool) -> Self {
        self.config.extra.nodelist = nodelist;
        self
    }

    /// Set whether to enable rule generator
    pub fn enable_rule_generator(mut self, enable: bool) -> Self {
        self.config.extra.enable_rule_generator = enable;
        self
    }

    /// Set whether to overwrite original rules
    pub fn overwrite_original_rules(mut self, overwrite: bool) -> Self {
        self.config.extra.overwrite_original_rules = overwrite;
        self
    }

    /// Set device ID
    pub fn device_id(mut self, device_id: Option<String>) -> Self {
        self.config.device_id = device_id;
        self
    }

    /// Set filename
    pub fn filename(mut self, filename: Option<String>) -> Self {
        self.config.filename = filename;
        self
    }

    /// Set update interval
    pub fn update_interval(mut self, interval: i32) -> Self {
        self.config.update_interval = interval;
        self
    }

    /// Set filter script
    pub fn filter_script(mut self, script: Option<String>) -> Self {
        self.config.filter_script = script;
        self
    }

    /// Set whether update is strict
    pub fn update_strict(mut self, strict: bool) -> Self {
        self.config.update_strict = strict;
        self
    }

    /// Set managed config prefix
    pub fn managed_config_prefix(mut self, prefix: String) -> Self {
        self.config.managed_config_prefix = prefix;
        self
    }

    /// Set upload path
    pub fn upload_path(mut self, path: Option<String>) -> Self {
        self.config.upload_path = path;
        self
    }

    /// Set whether to upload the result
    pub fn upload(mut self, upload: bool) -> Self {
        self.config.upload = upload;
        self
    }

    /// Set proxy for fetching subscriptions
    pub fn proxy(mut self, proxy: Option<String>) -> Self {
        self.config.proxy = proxy;
        self
    }

    /// Set authentication token
    pub fn token(mut self, token: Option<String>) -> Self {
        self.config.token = token;
        self
    }

    /// Set whether this request is authorized
    pub fn authorized(mut self, authorized: bool) -> Self {
        self.config.authorized = authorized;
        self
    }

    /// Set subscription information
    pub fn sub_info(mut self, sub_info: Option<String>) -> Self {
        self.config.sub_info = sub_info;
        self
    }

    /// Build the final configuration
    pub fn build(self) -> Result<SubconverterConfig, String> {
        let config = self.config;

        // Basic validation
        if config.urls.is_empty() && config.insert_urls.is_empty() {
            return Err("No URLs provided".to_string());
        }

        Ok(config)
    }
}

/// Result of subscription conversion
#[derive(Debug, Clone)]
pub struct SubconverterResult {
    /// Converted content
    pub content: String,
    /// Response headers
    pub headers: HashMap<String, String>,
}

/// Options for parsing subscriptions
#[derive(Debug, Clone)]
pub struct ParseOptions {
    /// Remarks to include in parsing
    pub include_remarks: Vec<String>,

    /// Remarks to exclude from parsing
    pub exclude_remarks: Vec<String>,

    /// Whether the request is authorized
    pub authorized: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            include_remarks: Vec::new(),
            exclude_remarks: Vec::new(),
            authorized: false,
        }
    }
}

/// Parse a subscription URL and return a vector of proxies
///
/// # Arguments
/// * `url` - The subscription URL to parse
/// * `options` - Options for parsing
///
/// # Returns
/// * `Ok(Vec<Proxy>)` - The parsed proxies
/// * `Err(String)` - Error message if parsing fails
pub fn parse_subscription(url: &str, options: ParseOptions) -> Result<Vec<Proxy>, String> {
    // Create a new parse settings instance
    let mut parse_settings = ParseSettings::default();

    // Set options from the provided config
    if !options.include_remarks.is_empty() {
        parse_settings.include_remarks = Some(options.include_remarks.clone());
    }

    if !options.exclude_remarks.is_empty() {
        parse_settings.exclude_remarks = Some(options.exclude_remarks.clone());
    }

    parse_settings.authorized = options.authorized;

    // Create a vector to hold the nodes
    let mut nodes = Vec::new();

    // Call add_nodes to do the actual parsing
    // We use group_id = 0 since we don't care about it in this context
    add_nodes(url.to_string(), &mut nodes, 0, &mut parse_settings)?;

    Ok(nodes)
}

/// Process a subscription conversion request
pub fn subconverter(config: SubconverterConfig) -> Result<SubconverterResult, String> {
    let mut response_headers = HashMap::new();
    let mut nodes = Vec::new();

    info!(
        "Processing subscription conversion request to {}",
        config.target.to_str()
    );

    // Parse subscription URLs
    let opts = ParseOptions {
        include_remarks: config.include_remarks.clone(),
        exclude_remarks: config.exclude_remarks.clone(),
        authorized: config.authorized,
    };

    // Parse insert URLs first if needed
    let mut insert_nodes = Vec::new();
    if !config.insert_urls.is_empty() {
        info!("Fetching node data from insert URLs");
        for url in &config.insert_urls {
            debug!("Parsing insert URL: {}", url);
            match parse_subscription(url, opts.clone()) {
                Ok(mut parsed_nodes) => {
                    info!("Found {} nodes from insert URL", parsed_nodes.len());
                    insert_nodes.append(&mut parsed_nodes);
                }
                Err(e) => {
                    warn!("Failed to parse insert URL '{}': {}", url, e);
                    // Continue with other URLs even if this one failed
                }
            }
        }
    }

    // Parse main URLs
    info!("Fetching node data from main URLs");
    for url in &config.urls {
        debug!("Parsing URL: {}", url);
        match parse_subscription(url, opts.clone()) {
            Ok(mut parsed_nodes) => {
                info!("Found {} nodes from URL", parsed_nodes.len());
                nodes.append(&mut parsed_nodes);
            }
            Err(e) => {
                error!("Failed to parse URL '{}': {}", url, e);
                return Err(format!("Failed to parse URL '{}': {}", url, e));
            }
        }
    }

    // Exit if found nothing
    if nodes.is_empty() && insert_nodes.is_empty() {
        return Err("No nodes were found!".to_string());
    }

    // Merge insert nodes and main nodes
    if config.prepend_insert {
        // Prepend insert nodes
        info!(
            "Prepending {} insert nodes to {} main nodes",
            insert_nodes.len(),
            nodes.len()
        );
        let mut combined = insert_nodes;
        combined.append(&mut nodes);
        nodes = combined;
    } else {
        // Append insert nodes
        info!(
            "Appending {} insert nodes to {} main nodes",
            insert_nodes.len(),
            nodes.len()
        );
        nodes.append(&mut insert_nodes);
    }

    // Apply group name if specified
    if let Some(group_name) = &config.group_name {
        info!("Setting group name to '{}'", group_name);
        for node in &mut nodes {
            (*node).group = group_name.clone();
        }
    }

    // Apply filter script if available
    if let Some(script) = &config.filter_script {
        info!("Applying filter script");
        // In the real implementation, this would involve running a JavaScript engine
        // to filter nodes based on the script. Left as placeholder.
    }

    // Process nodes (rename, emoji, sort, etc.)
    preprocess_nodes(
        &mut nodes,
        &config.extra,
        &config.rename_patterns,
        &config.emoji_patterns,
    );

    // Pass subscription info if provided
    if let Some(sub_info) = &config.sub_info {
        response_headers.insert("Subscription-UserInfo".to_string(), sub_info.clone());
    }

    // Generate output based on target
    let output_content = match &config.target {
        SubconverterTarget::Clash => {
            info!("Generate target: Clash");
            let base = config
                .base_content
                .get(&SubconverterTarget::Clash)
                .cloned()
                .unwrap_or_default();
            proxy_to_clash(
                &mut nodes,
                &base,
                &mut config.ruleset_content.clone(),
                &config.proxy_groups,
                false,
                &mut config.extra.clone(),
            )
        }
        SubconverterTarget::ClashR => {
            info!("Generate target: ClashR");
            let base = config
                .base_content
                .get(&SubconverterTarget::ClashR)
                .cloned()
                .unwrap_or_default();
            proxy_to_clash(
                &mut nodes,
                &base,
                &mut config.ruleset_content.clone(),
                &config.proxy_groups,
                true,
                &mut config.extra.clone(),
            )
        }
        SubconverterTarget::Surge(ver) => {
            info!("Generate target: Surge {}", ver);
            let base = config
                .base_content
                .get(&config.target)
                .cloned()
                .unwrap_or_default();
            let output = proxy_to_surge(
                &mut nodes,
                &base,
                &mut config.ruleset_content.clone(),
                &config.proxy_groups,
                *ver,
                &mut config.extra.clone(),
            );

            // Add managed configuration header if needed
            if !config.managed_config_prefix.is_empty() && config.extra.enable_rule_generator {
                let managed_url = format!(
                    "{}sub?target=surge&ver={}&url={}",
                    config.managed_config_prefix,
                    ver,
                    // URL would need to be properly encoded
                    config.urls.join("|")
                );

                format!(
                    "#!MANAGED-CONFIG {} interval={} strict={}\n\n{}",
                    managed_url,
                    config.update_interval,
                    if config.update_strict {
                        "true"
                    } else {
                        "false"
                    },
                    output
                )
            } else {
                output
            }
        }
        SubconverterTarget::Surfboard => {
            info!("Generate target: Surfboard");
            let base = config
                .base_content
                .get(&config.target)
                .cloned()
                .unwrap_or_default();
            let output = proxy_to_surge(
                &mut nodes,
                &base,
                &mut config.ruleset_content.clone(),
                &config.proxy_groups,
                -3, // Special version for Surfboard
                &mut config.extra.clone(),
            );

            // Add managed configuration header if needed
            if !config.managed_config_prefix.is_empty() && config.extra.enable_rule_generator {
                let managed_url = format!(
                    "{}sub?target=surfboard&url={}",
                    config.managed_config_prefix,
                    // URL would need to be properly encoded
                    config.urls.join("|")
                );

                format!(
                    "#!MANAGED-CONFIG {} interval={} strict={}\n\n{}",
                    managed_url,
                    config.update_interval,
                    if config.update_strict {
                        "true"
                    } else {
                        "false"
                    },
                    output
                )
            } else {
                output
            }
        }
        SubconverterTarget::Mellow => {
            info!("Generate target: Mellow");
            let base = config
                .base_content
                .get(&config.target)
                .cloned()
                .unwrap_or_default();
            proxy_to_mellow(
                &mut nodes,
                &base,
                &mut config.ruleset_content.clone(),
                &config.proxy_groups,
                &mut config.extra.clone(),
            )
        }
        SubconverterTarget::SSSub => {
            info!("Generate target: SS Subscription");
            let base = config
                .base_content
                .get(&config.target)
                .cloned()
                .unwrap_or_default();
            proxy_to_ss_sub(&base, &mut nodes, &mut config.extra.clone())
        }
        SubconverterTarget::SS => {
            info!("Generate target: SS");
            // To be implemented: convert nodes to SS format
            String::new() // placeholder
        }
        SubconverterTarget::SSR => {
            info!("Generate target: SSR");
            // To be implemented: convert nodes to SSR format
            String::new() // placeholder
        }
        SubconverterTarget::V2Ray => {
            info!("Generate target: V2Ray");
            // To be implemented: convert nodes to V2Ray format
            String::new() // placeholder
        }
        SubconverterTarget::Trojan => {
            info!("Generate target: Trojan");
            // To be implemented: convert nodes to Trojan format
            String::new() // placeholder
        }
        SubconverterTarget::Mixed => {
            info!("Generate target: Mixed");
            // To be implemented: convert nodes to mixed format
            String::new() // placeholder
        }
        SubconverterTarget::Quantumult => {
            info!("Generate target: Quantumult");
            let base = config
                .base_content
                .get(&config.target)
                .cloned()
                .unwrap_or_default();
            proxy_to_quan(
                &mut nodes,
                &base,
                &mut config.ruleset_content.clone(),
                &config.proxy_groups,
                &mut config.extra.clone(),
            )
        }
        SubconverterTarget::QuantumultX => {
            info!("Generate target: Quantumult X");
            let base = config
                .base_content
                .get(&config.target)
                .cloned()
                .unwrap_or_default();
            proxy_to_quanx(
                &mut nodes,
                &base,
                &mut config.ruleset_content.clone(),
                &config.proxy_groups,
                &mut config.extra.clone(),
            )
        }
        SubconverterTarget::Loon => {
            info!("Generate target: Loon");
            let base = config
                .base_content
                .get(&config.target)
                .cloned()
                .unwrap_or_default();
            proxy_to_loon(
                &mut nodes,
                &base,
                &mut config.ruleset_content.clone(),
                &config.proxy_groups,
                &mut config.extra.clone(),
            )
        }
        SubconverterTarget::SSD => {
            info!("Generate target: SSD");
            // To be implemented: convert nodes to SSD format
            String::new() // placeholder
        }
        SubconverterTarget::SingBox => {
            info!("Generate target: SingBox");
            let base = config
                .base_content
                .get(&config.target)
                .cloned()
                .unwrap_or_default();
            proxy_to_singbox(
                &mut nodes,
                &base,
                &mut config.ruleset_content.clone(),
                &config.proxy_groups,
                &mut config.extra.clone(),
            )
        }
    };

    // Set filename header if provided
    if let Some(filename) = &config.filename {
        response_headers.insert(
            "Content-Disposition".to_string(),
            format!("attachment; filename=\"{}\"; filename*=utf-8''", filename),
        );
    }

    // Upload result if needed
    if config.upload {
        if let Some(upload_path) = &config.upload_path {
            info!("Uploading result to path: {}", upload_path);
            // Implement upload functionality here
            // This is typically a separate function like `upload_gist`
        }
    }

    info!("Conversion completed");
    Ok(SubconverterResult {
        content: output_content,
        headers: response_headers,
    })
}

/// Preprocess nodes before conversion
pub fn preprocess_nodes(
    nodes: &mut Vec<Proxy>,
    extra: &ExtraSettings,
    rename_patterns: &[(String, String)],
    emoji_patterns: &[(String, String)],
) {
    // Apply renames
    if !rename_patterns.is_empty() {
        info!(
            "Applying {} rename patterns to {} nodes",
            rename_patterns.len(),
            nodes.len()
        );
        for node in nodes.iter_mut() {
            for (pattern, replacement) in rename_patterns {
                // Apply regex replace
                // This is a simplified version; actual implementation would use regex
                if node.remark.contains(pattern) {
                    node.remark = node.remark.replace(pattern, replacement);
                }
            }
        }
    }

    // Apply emojis
    if extra.add_emoji && !emoji_patterns.is_empty() {
        info!("Applying emoji patterns to {} nodes", nodes.len());
        for node in nodes.iter_mut() {
            // Remove existing emoji if needed
            if extra.remove_emoji {
                // Simplified emoji removal; actual implementation would use regex
                // to remove emoji patterns
            }

            // Add emoji based on patterns
            for (pattern, emoji) in emoji_patterns {
                if node.remark.contains(pattern) {
                    node.remark = format!("{} {}", emoji, node.remark);
                    break; // Only add one emoji
                }
            }
        }
    }

    // Sort nodes if needed
    if extra.sort_flag {
        info!("Sorting {} nodes", nodes.len());
        if !extra.sort_script.is_empty() {
            // Apply sort using script
            // This would involve running a JavaScript engine
        } else {
            // Simple default sort by remark
            nodes.sort_by(|a, b| a.remark.cmp(&b.remark));
        }
    }
}
