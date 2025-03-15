use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ConfType {
    Unknown,
    SS,
    SSR,
    V2Ray,
    SSConf,
    SSTap,
    Netch,
    SOCKS,
    HTTP,
    SUB,
    Local,
}

#[derive(Debug, Clone)]
pub struct Proxy {
    pub proxy_type: ProxyType,
    pub group: String,
    pub remarks: String,
    pub hostname: String,
    pub port: u16,
    pub udp: Option<bool>,
    pub tfo: Option<bool>,
    pub scv: Option<bool>,
    pub tls13: Option<bool>,
    pub underlying_proxy: Option<String>,
    // Add other fields as needed
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProxyType {
    Unknown,
    VMess,
    Shadowsocks,
    ShadowsocksR,
    SOCKS5,
    HTTP,
    HTTPS,
    Trojan,
    Snell,
    WireGuard,
    // Add other types as needed
}

impl Default for Proxy {
    fn default() -> Self {
        Proxy {
            proxy_type: ProxyType::Unknown,
            group: String::new(),
            remarks: String::new(),
            hostname: String::new(),
            port: 0,
            udp: None,
            tfo: None,
            scv: None,
            tls13: None,
            underlying_proxy: None,
        }
    }
}
