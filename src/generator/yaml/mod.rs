pub mod clash_output;
pub mod proxy_group_output;

// Example function showing how to use the ClashProxyCommon trait
#[cfg(test)]
mod tests {
    use super::clash_output::{ClashProxy, ClashProxyCommon};

    #[test]
    fn test_common_proxy_operations() {
        // Create a Shadowsocks proxy
        let mut ss_proxy = ClashProxy::Shadowsocks {
            common: super::clash_output::CommonProxyOptions {
                name: "example-ss".to_string(),
                server: "example.com".to_string(),
                port: 8388,
                udp: None,
                tfo: None,
                skip_cert_verify: None,
                tls: None,
                fingerprint: None,
                client_fingerprint: None,
                sni: None,
            },
            cipher: "aes-256-gcm".to_string(),
            password: "password".to_string(),
            plugin: None,
            plugin_opts: None,
        };

        // Use the trait methods to set common values
        ss_proxy.set_tfo(true);
        ss_proxy.set_udp(true);
        ss_proxy.set_skip_cert_verify(false);

        // Access common fields
        assert_eq!(ss_proxy.common().name, "example-ss");
        assert_eq!(ss_proxy.common().tfo, Some(true));
        assert_eq!(ss_proxy.common().udp, Some(true));
        assert_eq!(ss_proxy.common().skip_cert_verify, Some(false));
    }
}
