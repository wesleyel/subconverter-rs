use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn is_empty_option_string(s: &Option<String>) -> bool {
    s.is_none() || s.as_ref().unwrap().is_empty()
}

fn is_u32_option_zero(u: &Option<u32>) -> bool {
    if let Some(u) = u {
        *u == 0
    } else {
        true
    }
}

/// Represents a complete Clash configuration output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ClashYamlOutput {
    // General settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socks_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redir_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tproxy_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mixed_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_lan: Option<bool>,
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub bind_address: Option<String>,
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub log_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<bool>,

    // DNS settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns: Option<ClashDns>,

    // Proxy settings
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub proxies: Vec<ClashProxy>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub proxy_groups: Vec<ClashProxyGroup>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<String>,

    // Additional fields (for compatibility with ClashR and other variants)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tun: Option<ClashTun>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<ClashProfile>,

    #[serde(flatten)]
    pub extra_options: HashMap<String, serde_yaml::Value>,
}

/// DNS configuration for Clash
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ClashDns {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub listen: Option<String>,
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub enhanced_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nameserver: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_filter: Option<ClashDnsFallbackFilter>,
    #[serde(flatten)]
    pub extra_options: HashMap<String, serde_yaml::Value>,
}

/// DNS fallback filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ClashDnsFallbackFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geoip: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipcidr: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra_options: HashMap<String, serde_yaml::Value>,
}

/// TUN configuration for Clash
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ClashTun {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub device: Option<String>,
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_hijack: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_route: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_detect_interface: Option<bool>,
    #[serde(flatten)]
    pub extra_options: HashMap<String, serde_yaml::Value>,
}

/// Profile settings for Clash
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ClashProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_selected: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_fake_ip: Option<bool>,
    #[serde(flatten)]
    pub extra_options: HashMap<String, serde_yaml::Value>,
}
/// Common proxy options that can be used across different proxy types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CommonProxyOptions {
    pub name: String,
    pub server: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tfo: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_cert_verify: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<bool>,
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub fingerprint: Option<String>,
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub client_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub sni: Option<String>,
}

/// Factory methods for CommonProxyOptions
impl CommonProxyOptions {
    /// Create a new CommonProxyOptions with default values
    pub fn new(name: String, server: String, port: u16) -> Self {
        Self {
            name,
            server,
            port,
            udp: None,
            tfo: None,
            skip_cert_verify: None,
            tls: None,
            fingerprint: None,
            client_fingerprint: None,
            sni: None,
        }
    }

    /// Create a builder for CommonProxyOptions
    pub fn builder(name: String, server: String, port: u16) -> CommonProxyOptionsBuilder {
        CommonProxyOptionsBuilder {
            common: Self::new(name, server, port),
        }
    }
}

/// Builder for CommonProxyOptions
pub struct CommonProxyOptionsBuilder {
    common: CommonProxyOptions,
}

impl CommonProxyOptionsBuilder {
    /// Set UDP option
    pub fn udp(mut self, value: bool) -> Self {
        self.common.udp = Some(value);
        self
    }

    /// Set TFO (TCP Fast Open) option
    pub fn tfo(mut self, value: bool) -> Self {
        self.common.tfo = Some(value);
        self
    }

    /// Set skip_cert_verify option
    pub fn skip_cert_verify(mut self, value: bool) -> Self {
        self.common.skip_cert_verify = Some(value);
        self
    }

    /// Set TLS option
    pub fn tls(mut self, value: bool) -> Self {
        self.common.tls = Some(value);
        self
    }

    /// Set SNI option
    pub fn sni(mut self, value: String) -> Self {
        self.common.sni = Some(value);
        self
    }

    /// Set fingerprint option
    pub fn fingerprint(mut self, value: String) -> Self {
        self.common.fingerprint = Some(value);
        self
    }

    /// Set client_fingerprint option
    pub fn client_fingerprint(mut self, value: String) -> Self {
        self.common.client_fingerprint = Some(value);
        self
    }

    /// Build the final CommonProxyOptions
    pub fn build(self) -> CommonProxyOptions {
        self.common
    }
}

/// Represents a single proxy in Clash configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ClashProxy {
    #[serde(rename = "ss")]
    Shadowsocks {
        #[serde(flatten)]
        common: CommonProxyOptions,
        cipher: String,
        password: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin_opts: Option<HashMap<String, serde_yaml::Value>>,
    },
    #[serde(rename = "ssr")]
    ShadowsocksR {
        #[serde(flatten)]
        common: CommonProxyOptions,
        cipher: String,
        password: String,
        protocol: String,
        obfs: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        protocol_param: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        obfs_param: Option<String>,
        // ClashR compatibility
        #[serde(skip_serializing_if = "Option::is_none")]
        protocolparam: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        obfsparam: Option<String>,
    },
    #[serde(rename = "vmess")]
    VMess {
        #[serde(flatten)]
        common: CommonProxyOptions,
        uuid: String,
        #[serde(rename = "alterId")]
        alter_id: u32,
        cipher: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        network: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ws_path: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ws_headers: Option<serde_yaml::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ws_opts: Option<HashMap<String, serde_yaml::Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        http_opts: Option<HashMap<String, serde_yaml::Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        h2_opts: Option<HashMap<String, serde_yaml::Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        grpc_opts: Option<HashMap<String, serde_yaml::Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        servername: Option<String>,
    },
    #[serde(rename = "trojan")]
    Trojan {
        #[serde(flatten)]
        common: CommonProxyOptions,
        password: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        network: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ws_opts: Option<serde_yaml::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        grpc_opts: Option<HashMap<String, serde_yaml::Value>>,
    },
    #[serde(rename = "http")]
    Http {
        #[serde(flatten)]
        common: CommonProxyOptions,
        #[serde(skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        password: Option<String>,
    },
    #[serde(rename = "socks5")]
    Socks5 {
        #[serde(flatten)]
        common: CommonProxyOptions,
        #[serde(skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        password: Option<String>,
    },
    #[serde(rename = "snell")]
    Snell {
        #[serde(flatten)]
        common: CommonProxyOptions,
        psk: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        obfs: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        obfs_opts: Option<HashMap<String, serde_yaml::Value>>,
    },
    #[serde(rename = "wireguard")]
    WireGuard {
        #[serde(flatten)]
        common: CommonProxyOptions,
        #[serde(rename = "private-key")]
        private_key: String,
        #[serde(rename = "public-key")]
        public_key: String,
        ip: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        ipv6: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        preshared_key: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dns: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mtu: Option<u32>,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        allowed_ips: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        keepalive: Option<u32>,
    },
    #[serde(rename = "hysteria")]
    Hysteria {
        #[serde(flatten)]
        common: CommonProxyOptions,
        #[serde(skip_serializing_if = "Option::is_none")]
        ports: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        protocol: Option<String>,
        #[serde(rename = "obfs-protocol", skip_serializing_if = "Option::is_none")]
        obfs_protocol: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        up: Option<String>,
        #[serde(rename = "up-speed", skip_serializing_if = "Option::is_none")]
        up_speed: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        down: Option<String>,
        #[serde(rename = "down-speed", skip_serializing_if = "Option::is_none")]
        down_speed: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        auth: Option<String>,
        #[serde(rename = "auth-str", skip_serializing_if = "Option::is_none")]
        auth_str: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        obfs: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        fingerprint: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        alpn: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ca: Option<String>,
        #[serde(rename = "ca-str", skip_serializing_if = "Option::is_none")]
        ca_str: Option<String>,
        #[serde(rename = "recv-window-conn", skip_serializing_if = "Option::is_none")]
        recv_window_conn: Option<u32>,
        #[serde(rename = "recv-window", skip_serializing_if = "Option::is_none")]
        recv_window: Option<u32>,
        #[serde(
            rename = "disable-mtu-discovery",
            skip_serializing_if = "Option::is_none"
        )]
        disable_mtu_discovery: Option<bool>,
        #[serde(rename = "fast-open", skip_serializing_if = "Option::is_none")]
        fast_open: Option<bool>,
        #[serde(rename = "hop-interval", skip_serializing_if = "Option::is_none")]
        hop_interval: Option<u32>,
    },
    #[serde(rename = "hysteria2")]
    Hysteria2 {
        #[serde(flatten)]
        common: CommonProxyOptions,
        #[serde(skip_serializing_if = "String::is_empty")]
        password: String,
        #[serde(skip_serializing_if = "String::is_empty")]
        ports: String,
        #[serde(rename = "hop-interval", skip_serializing_if = "Option::is_none")]
        hop_interval: Option<u32>,
        #[serde(skip_serializing_if = "String::is_empty")]
        up: String,
        #[serde(skip_serializing_if = "String::is_empty")]
        down: String,
        #[serde(skip_serializing_if = "String::is_empty")]
        obfs: String,
        #[serde(rename = "obfs-password", skip_serializing_if = "String::is_empty")]
        obfs_password: String,
        #[serde(skip_serializing_if = "String::is_empty")]
        fingerprint: String,
        #[serde(skip_serializing_if = "String::is_empty")]
        alpn: String,
        #[serde(skip_serializing_if = "String::is_empty")]
        ca: String,
        #[serde(rename = "ca-str", skip_serializing_if = "String::is_empty")]
        ca_str: String,
        #[serde(skip_serializing_if = "is_u32_option_zero")]
        cwnd: Option<u32>,
        #[serde(rename = "udp-mtu", skip_serializing_if = "is_u32_option_zero")]
        udp_mtu: Option<u32>,

        // quic-go special config
        #[serde(
            rename = "initial-stream-receive-window",
            skip_serializing_if = "Option::is_none"
        )]
        initial_stream_receive_window: Option<u64>,
        #[serde(
            rename = "max-stream-receive-window",
            skip_serializing_if = "Option::is_none"
        )]
        max_stream_receive_window: Option<u64>,
        #[serde(
            rename = "initial-connection-receive-window",
            skip_serializing_if = "Option::is_none"
        )]
        initial_connection_receive_window: Option<u64>,
        #[serde(
            rename = "max-connection-receive-window",
            skip_serializing_if = "Option::is_none"
        )]
        max_connection_receive_window: Option<u64>,
    },
    // Support for generic proxies with extra fields
    // #[serde(other)]
    // Other(HashMap<String, >),
}

/// Factory methods for creating various proxy types
impl ClashProxy {
    /// Create a new Shadowsocks proxy
    pub fn new_shadowsocks(common: CommonProxyOptions) -> Self {
        ClashProxy::Shadowsocks {
            common,
            cipher: String::new(),
            password: String::new(),
            plugin: None,
            plugin_opts: None,
        }
    }

    /// Create a new ShadowsocksR proxy
    pub fn new_shadowsocksr(common: CommonProxyOptions) -> Self {
        ClashProxy::ShadowsocksR {
            common,
            cipher: String::new(),
            password: String::new(),
            protocol: String::new(),
            obfs: String::new(),
            protocol_param: None,
            obfs_param: None,
            protocolparam: None,
            obfsparam: None,
        }
    }

    /// Create a new VMess proxy
    pub fn new_vmess(common: CommonProxyOptions) -> Self {
        ClashProxy::VMess {
            common,
            uuid: String::new(),
            alter_id: 0,
            cipher: String::new(),
            network: None,
            ws_path: None,
            ws_headers: None,
            ws_opts: None,
            http_opts: None,
            h2_opts: None,
            grpc_opts: None,
            servername: None,
        }
    }

    /// Create a new HTTP proxy
    pub fn new_http(common: CommonProxyOptions) -> Self {
        ClashProxy::Http {
            common,
            username: None,
            password: None,
        }
    }

    /// Create a new Trojan proxy
    pub fn new_trojan(common: CommonProxyOptions) -> Self {
        ClashProxy::Trojan {
            common,
            password: String::new(),
            network: None,
            ws_opts: None,
            grpc_opts: None,
        }
    }

    /// Create a new Socks5 proxy
    pub fn new_socks5(common: CommonProxyOptions) -> Self {
        ClashProxy::Socks5 {
            common,
            username: None,
            password: None,
        }
    }

    /// Create a new Snell proxy
    pub fn new_snell(common: CommonProxyOptions) -> Self {
        ClashProxy::Snell {
            common,
            psk: String::new(),
            version: None,
            obfs: None,
            obfs_opts: None,
        }
    }

    /// Create a new WireGuard proxy
    pub fn new_wireguard(common: CommonProxyOptions) -> Self {
        ClashProxy::WireGuard {
            common,
            ip: String::new(),
            ipv6: None,
            preshared_key: None,
            dns: None,
            mtu: None,
            allowed_ips: Vec::new(),
            keepalive: None,
            private_key: String::new(),
            public_key: String::new(),
        }
    }

    /// Create a new Hysteria proxy
    pub fn new_hysteria(common: CommonProxyOptions) -> Self {
        Self::Hysteria {
            common,
            ports: None,
            protocol: None,
            obfs_protocol: None,
            up: None,
            up_speed: None,
            down: None,
            down_speed: None,
            auth: None,
            auth_str: None,
            obfs: None,
            fingerprint: None,
            alpn: None,
            ca: None,
            ca_str: None,
            recv_window_conn: None,
            recv_window: None,
            disable_mtu_discovery: None,
            fast_open: None,
            hop_interval: None,
        }
    }

    /// Create a new Hysteria2 proxy
    pub fn new_hysteria2(common: CommonProxyOptions) -> Self {
        ClashProxy::Hysteria2 {
            common,
            password: String::new(),
            ports: String::new(),
            hop_interval: None,
            up: String::new(),
            down: String::new(),
            obfs: String::new(),
            obfs_password: String::new(),
            fingerprint: String::new(),
            alpn: String::new(),
            ca: String::new(),
            ca_str: String::new(),
            cwnd: None,
            udp_mtu: None,
            initial_stream_receive_window: None,
            max_stream_receive_window: None,
            initial_connection_receive_window: None,
            max_connection_receive_window: None,
        }
    }
}

/// Trait for common operations on all ClashProxy variants
pub trait ClashProxyCommon {
    /// Get a reference to the common options
    fn common(&self) -> &CommonProxyOptions;

    /// Get a mutable reference to the common options
    fn common_mut(&mut self) -> &mut CommonProxyOptions;

    /// Set a TFO (TCP Fast Open) option
    fn set_tfo(&mut self, value: bool) {
        self.common_mut().tfo = Some(value);
    }

    /// Set a UDP option
    fn set_udp(&mut self, value: bool) {
        self.common_mut().udp = Some(value);
    }

    /// Set skip certificate verification option
    fn set_skip_cert_verify(&mut self, value: bool) {
        self.common_mut().skip_cert_verify = Some(value);
    }

    /// Set TLS option
    fn set_tls(&mut self, value: bool) {
        self.common_mut().tls = Some(value);
    }

    /// Set SNI option
    fn set_sni(&mut self, value: String) {
        self.common_mut().sni = Some(value);
    }

    /// Set fingerprint option
    fn set_fingerprint(&mut self, value: String) {
        self.common_mut().fingerprint = Some(value);
    }
}

impl ClashProxyCommon for ClashProxy {
    fn common(&self) -> &CommonProxyOptions {
        match self {
            ClashProxy::Shadowsocks { common, .. } => common,
            ClashProxy::ShadowsocksR { common, .. } => common,
            ClashProxy::VMess { common, .. } => common,
            ClashProxy::Trojan { common, .. } => common,
            ClashProxy::Http { common, .. } => common,
            ClashProxy::Socks5 { common, .. } => common,
            ClashProxy::Snell { common, .. } => common,
            ClashProxy::WireGuard { common, .. } => common,
            ClashProxy::Hysteria { common, .. } => common,
            ClashProxy::Hysteria2 { common, .. } => common,
        }
    }

    fn common_mut(&mut self) -> &mut CommonProxyOptions {
        match self {
            ClashProxy::Shadowsocks { common, .. } => common,
            ClashProxy::ShadowsocksR { common, .. } => common,
            ClashProxy::VMess { common, .. } => common,
            ClashProxy::Trojan { common, .. } => common,
            ClashProxy::Http { common, .. } => common,
            ClashProxy::Socks5 { common, .. } => common,
            ClashProxy::Snell { common, .. } => common,
            ClashProxy::WireGuard { common, .. } => common,
            ClashProxy::Hysteria { common, .. } => common,
            ClashProxy::Hysteria2 { common, .. } => common,
        }
    }
}

/// Represents a proxy group in Clash configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ClashProxyGroup {
    #[serde(rename = "select")]
    Select {
        name: String,
        proxies: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        r#use: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_udp: Option<bool>,
    },
    #[serde(rename = "url-test")]
    UrlTest {
        name: String,
        proxies: Vec<String>,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        interval: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tolerance: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        lazy: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_udp: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        r#use: Option<Vec<String>>,
    },
    #[serde(rename = "fallback")]
    Fallback {
        name: String,
        proxies: Vec<String>,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        interval: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tolerance: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_udp: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        r#use: Option<Vec<String>>,
    },
    #[serde(rename = "load-balance")]
    LoadBalance {
        name: String,
        proxies: Vec<String>,
        strategy: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        interval: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tolerance: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        lazy: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_udp: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        r#use: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        persistent: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        evaluate_before_use: Option<bool>,
    },
    #[serde(rename = "relay")]
    Relay {
        name: String,
        proxies: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_udp: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        r#use: Option<Vec<String>>,
    },
    // Support for generic proxy groups with extra fields
    // #[serde(other)]
    // Other(HashMap<String, serde_yaml::Value>),
}

// Implement Default trait for ClashYamlOutput
impl Default for ClashYamlOutput {
    fn default() -> Self {
        Self {
            port: None,
            socks_port: None,
            redir_port: None,
            tproxy_port: None,
            mixed_port: None,
            allow_lan: None,
            bind_address: None,
            mode: Some("rule".to_string()),
            log_level: Some("info".to_string()),
            ipv6: None,
            dns: None,
            proxies: Vec::new(),
            proxy_groups: Vec::new(),
            rules: Vec::new(),
            tun: None,
            profile: None,
            extra_options: HashMap::new(),
        }
    }
}
