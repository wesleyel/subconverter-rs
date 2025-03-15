use std::collections::HashSet;

/// Represents the type of a proxy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProxyType {
    Unknown,
    Shadowsocks,
    ShadowsocksR,
    VMess,
    Trojan,
    Snell,
    HTTP,
    HTTPS,
    SOCKS5,
    WireGuard,
    Hysteria,
    Hysteria2,
}

/// Converts a `ProxyType` into a human-readable name.
impl ProxyType {
    pub fn to_string(self) -> &'static str {
        match self {
            ProxyType::Shadowsocks => "SS",
            ProxyType::ShadowsocksR => "SSR",
            ProxyType::VMess => "VMess",
            ProxyType::Trojan => "Trojan",
            ProxyType::Snell => "Snell",
            ProxyType::HTTP => "HTTP",
            ProxyType::HTTPS => "HTTPS",
            ProxyType::SOCKS5 => "SOCKS5",
            ProxyType::WireGuard => "WireGuard",
            ProxyType::Hysteria => "Hysteria",
            ProxyType::Hysteria2 => "Hysteria2",
            ProxyType::Unknown => "Unknown",
        }
    }
}

/// Represents a proxy configuration.
#[derive(Debug, Clone)]
pub struct Proxy {
    pub proxy_type: ProxyType,
    pub id: u32,
    pub group_id: u32,
    pub group: String,
    pub remark: String,
    pub hostname: String,
    pub port: u16,

    pub username: Option<String>,
    pub password: Option<String>,
    pub encrypt_method: Option<String>,
    pub plugin: Option<String>,
    pub plugin_option: Option<String>,
    pub protocol: Option<String>,
    pub protocol_param: Option<String>,
    pub obfs: Option<String>,
    pub obfs_param: Option<String>,
    pub user_id: Option<String>,
    pub alter_id: u16,
    pub transfer_protocol: Option<String>,
    pub fake_type: Option<String>,
    pub tls_secure: bool,

    pub host: Option<String>,
    pub path: Option<String>,
    pub edge: Option<String>,

    pub quic_secure: Option<String>,
    pub quic_secret: Option<String>,

    pub udp: Option<bool>,
    pub tcp_fast_open: Option<bool>,
    pub allow_insecure: Option<bool>,
    pub tls13: Option<bool>,

    pub underlying_proxy: Option<String>,

    pub snell_version: u16,
    pub server_name: Option<String>,

    pub self_ip: Option<String>,
    pub self_ipv6: Option<String>,
    pub public_key: Option<String>,
    pub private_key: Option<String>,
    pub pre_shared_key: Option<String>,
    pub dns_servers: HashSet<String>,
    pub mtu: u16,
    pub allowed_ips: String,
    pub keep_alive: u16,
    pub test_url: Option<String>,
    pub client_id: Option<String>,

    pub ports: Option<String>,
    pub up: Option<String>,
    pub up_speed: u32,
    pub down: Option<String>,
    pub down_speed: u32,
    pub auth_str: Option<String>,
    pub sni: Option<String>,
    pub fingerprint: Option<String>,
    pub ca: Option<String>,
    pub ca_str: Option<String>,
    pub recv_window_conn: u32,
    pub recv_window: u32,
    pub disable_mtu_discovery: Option<bool>,
    pub hop_interval: u32,
    pub alpn: HashSet<String>,

    pub cwnd: u32,
}

/// Implement Default for Proxy
impl Default for Proxy {
    fn default() -> Self {
        Proxy {
            proxy_type: ProxyType::Unknown,
            id: 0,
            group_id: 0,
            group: String::new(),
            remark: String::new(),
            hostname: String::new(),
            port: 0,
            username: None,
            password: None,
            encrypt_method: None,
            plugin: None,
            plugin_option: None,
            protocol: None,
            protocol_param: None,
            obfs: None,
            obfs_param: None,
            user_id: None,
            alter_id: 0,
            transfer_protocol: None,
            fake_type: None,
            tls_secure: false,
            host: None,
            path: None,
            edge: None,
            quic_secure: None,
            quic_secret: None,
            udp: None,
            tcp_fast_open: None,
            allow_insecure: None,
            tls13: None,
            underlying_proxy: None,
            snell_version: 0,
            server_name: None,
            self_ip: None,
            self_ipv6: None,
            public_key: None,
            private_key: None,
            pre_shared_key: None,
            dns_servers: HashSet::new(),
            mtu: 0,
            allowed_ips: String::new(),
            keep_alive: 0,
            test_url: None,
            client_id: None,
            ports: None,
            up: None,
            up_speed: 0,
            down: None,
            down_speed: 0,
            auth_str: None,
            sni: None,
            fingerprint: None,
            ca: None,
            ca_str: None,
            recv_window_conn: 0,
            recv_window: 0,
            disable_mtu_discovery: None,
            hop_interval: 0,
            alpn: HashSet::new(),
            cwnd: 0,
        }
    }
}

/// Default provider group names as constants.
pub const SS_DEFAULT_GROUP: &str = "SSProvider";
pub const SSR_DEFAULT_GROUP: &str = "SSRProvider";
pub const V2RAY_DEFAULT_GROUP: &str = "V2RayProvider";
pub const SOCKS_DEFAULT_GROUP: &str = "SocksProvider";
pub const HTTP_DEFAULT_GROUP: &str = "HTTPProvider";
pub const TROJAN_DEFAULT_GROUP: &str = "TrojanProvider";
pub const SNELL_DEFAULT_GROUP: &str = "SnellProvider";
pub const WG_DEFAULT_GROUP: &str = "WireGuardProvider";
pub const HYSTERIA_DEFAULT_GROUP: &str = "HysteriaProvider";
pub const HYSTERIA2_DEFAULT_GROUP: &str = "Hysteria2Provider";
