use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;
use serde_yaml::Value;
use std::collections::HashMap;
use std::fmt;

// 添加一个辅助函数来反序列化既可能是数字也可能是字符串的字段
fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // 定义一个访问者来处理不同类型
    struct StringOrNumberVisitor;

    impl<'de> Visitor<'de> for StringOrNumberVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or number")
        }

        // 处理字符串
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value.to_string()))
        }

        // 处理i64
        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value.to_string()))
        }

        // 处理u64
        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value.to_string()))
        }

        // 处理f64
        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value.to_string()))
        }

        // 处理None
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        // 处理null
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(StringOrNumberVisitor)
}

/// Represents a single proxy in Clash configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ClashProxyYamlInput {
    #[serde(rename = "ss")]
    Shadowsocks {
        name: String,
        server: String,
        port: u16,
        cipher: String,
        password: String,
        #[serde(default)]
        udp: Option<bool>,
        #[serde(default)]
        tfo: Option<bool>,
        #[serde(rename = "skip-cert-verify", default)]
        skip_cert_verify: Option<bool>,
        #[serde(default)]
        plugin: Option<String>,
        #[serde(rename = "plugin-opts", default)]
        plugin_opts: Option<HashMap<String, Value>>,
    },

    #[serde(rename = "ssr")]
    ShadowsocksR {
        name: String,
        server: String,
        port: u16,
        cipher: String,
        password: String,
        protocol: String,
        obfs: String,
        #[serde(default)]
        udp: Option<bool>,
        #[serde(default)]
        tfo: Option<bool>,
        #[serde(rename = "skip-cert-verify", default)]
        skip_cert_verify: Option<bool>,
        #[serde(rename = "protocol-param", default)]
        protocol_param: Option<String>,
        #[serde(rename = "obfs-param", default)]
        obfs_param: Option<String>,
    },

    #[serde(rename = "vmess")]
    VMess {
        name: String,
        server: String,
        port: u16,
        uuid: String,
        #[serde(rename = "alterId")]
        alter_id: u32,
        cipher: String,
        #[serde(default)]
        udp: Option<bool>,
        #[serde(default)]
        tfo: Option<bool>,
        #[serde(rename = "skip-cert-verify", default)]
        skip_cert_verify: Option<bool>,
        #[serde(default)]
        network: Option<String>,
        #[serde(rename = "ws-path", default)]
        ws_path: Option<String>,
        #[serde(rename = "ws-headers", default)]
        ws_headers: Option<HashMap<String, String>>,
        #[serde(default)]
        tls: Option<bool>,
        #[serde(default)]
        servername: Option<String>,
    },

    #[serde(rename = "trojan")]
    Trojan {
        name: String,
        server: String,
        port: u16,
        password: String,
        #[serde(default)]
        udp: Option<bool>,
        #[serde(default)]
        tfo: Option<bool>,
        #[serde(rename = "skip-cert-verify", default)]
        skip_cert_verify: Option<bool>,
        #[serde(default)]
        network: Option<String>,
        #[serde(default)]
        sni: Option<String>,
    },

    #[serde(rename = "http")]
    Http {
        name: String,
        server: String,
        port: u16,
        #[serde(default)]
        username: Option<String>,
        #[serde(default)]
        password: Option<String>,
        #[serde(default)]
        tls: Option<bool>,
        #[serde(rename = "skip-cert-verify", default)]
        skip_cert_verify: Option<bool>,
    },

    #[serde(rename = "socks5")]
    Socks5 {
        name: String,
        server: String,
        port: u16,
        #[serde(default)]
        username: Option<String>,
        #[serde(default)]
        password: Option<String>,
        #[serde(rename = "skip-cert-verify", default)]
        skip_cert_verify: Option<bool>,
        #[serde(default)]
        udp: Option<bool>,
        #[serde(default)]
        tfo: Option<bool>,
    },

    #[serde(rename = "snell")]
    Snell {
        name: String,
        server: String,
        port: u16,
        psk: String,
        #[serde(default)]
        version: Option<u32>,
        #[serde(default)]
        obfs: Option<String>,
        #[serde(rename = "obfs-opts", default)]
        obfs_opts: Option<HashMap<String, Value>>,
        #[serde(default)]
        udp: Option<bool>,
        #[serde(default)]
        tfo: Option<bool>,
    },

    #[serde(rename = "wireguard")]
    WireGuard {
        name: String,
        server: String,
        port: u16,
        #[serde(rename = "private-key")]
        private_key: String,
        #[serde(rename = "public-key")]
        public_key: String,
        ip: String,
        #[serde(default)]
        ipv6: Option<String>,
        #[serde(rename = "preshared-key", default)]
        preshared_key: Option<String>,
        #[serde(default)]
        dns: Option<Vec<String>>,
        #[serde(default)]
        mtu: Option<u32>,
        #[serde(default)]
        allowed_ips: Vec<String>,
        #[serde(default)]
        keepalive: Option<u32>,
        #[serde(default)]
        udp: Option<bool>,
    },

    #[serde(rename = "hysteria")]
    Hysteria {
        name: String,
        server: String,
        port: u16,
        #[serde(default)]
        ports: Option<String>,
        #[serde(default)]
        protocol: Option<String>,
        #[serde(alias = "obfs-protocol", default)]
        obfs_protocol: Option<String>,
        #[serde(default, deserialize_with = "deserialize_string_or_number")]
        up: Option<String>,
        #[serde(alias = "up-speed", default)]
        up_speed: Option<u32>,
        #[serde(default, deserialize_with = "deserialize_string_or_number")]
        down: Option<String>,
        #[serde(alias = "down-speed", default)]
        down_speed: Option<u32>,
        #[serde(default)]
        auth: Option<String>,
        #[serde(alias = "auth-str", default)]
        auth_str: Option<String>,
        #[serde(default)]
        obfs: Option<String>,
        #[serde(default)]
        sni: Option<String>,
        #[serde(default)]
        fingerprint: Option<String>,
        #[serde(default)]
        alpn: Option<Vec<String>>,
        #[serde(default)]
        ca: Option<String>,
        #[serde(alias = "ca-str", default)]
        ca_str: Option<String>,
        #[serde(alias = "recv-window-conn", default)]
        recv_window_conn: Option<u32>,
        #[serde(alias = "recv-window", default)]
        recv_window: Option<u32>,
        #[serde(alias = "disable-mtu-discovery", default)]
        disable_mtu_discovery: Option<bool>,
        #[serde(alias = "fast-open", default)]
        fast_open: Option<bool>,
        #[serde(alias = "hop-interval", default)]
        hop_interval: Option<u32>,
        #[serde(alias = "skip-cert-verify", default)]
        skip_cert_verify: Option<bool>,
        #[serde(default)]
        tfo: Option<bool>,
    },

    #[serde(rename = "hysteria2")]
    Hysteria2 {
        name: String,
        server: String,
        port: u16,
        password: String,
        #[serde(default)]
        ports: Option<String>,
        #[serde(alias = "hop-interval", default)]
        hop_interval: Option<u32>,
        #[serde(default, deserialize_with = "deserialize_string_or_number")]
        up: Option<String>,
        #[serde(default, deserialize_with = "deserialize_string_or_number")]
        down: Option<String>,
        #[serde(default)]
        obfs: Option<String>,
        #[serde(alias = "obfs-password", default)]
        obfs_password: Option<String>,
        #[serde(default)]
        fingerprint: Option<String>,
        #[serde(default)]
        alpn: Option<String>,
        #[serde(default)]
        ca: Option<String>,
        #[serde(alias = "ca-str", default)]
        ca_str: Option<String>,
        #[serde(default)]
        cwnd: Option<u32>,
        #[serde(alias = "udp-mtu", default)]
        udp_mtu: Option<u32>,
        #[serde(default)]
        sni: Option<String>,
        #[serde(alias = "skip-cert-verify", default)]
        skip_cert_verify: Option<bool>,
        #[serde(alias = "fast-open", default)]
        fast_open: Option<bool>,
        #[serde(default)]
        tfo: Option<bool>,
    },

    // 处理其他未知类型的代理
    #[serde(other)]
    Unknown,
}
