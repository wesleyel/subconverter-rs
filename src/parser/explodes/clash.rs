use crate::models::{
    Proxy, HTTP_DEFAULT_GROUP, HYSTERIA2_DEFAULT_GROUP, HYSTERIA_DEFAULT_GROUP,
    SNELL_DEFAULT_GROUP, SOCKS_DEFAULT_GROUP, SSR_DEFAULT_GROUP, SS_DEFAULT_GROUP,
    TROJAN_DEFAULT_GROUP, V2RAY_DEFAULT_GROUP, WG_DEFAULT_GROUP,
};
use serde_yaml::Value;

/// Parse a Clash YAML configuration into a vector of Proxy objects
pub fn explode_clash(content: &str, nodes: &mut Vec<Proxy>) -> bool {
    // Parse the YAML content
    let yaml: Value = match serde_yaml::from_str(content) {
        Ok(y) => y,
        Err(_) => return false,
    };

    // Extract proxies section
    let proxies = match yaml.get("proxies") {
        Some(Value::Sequence(seq)) => seq,
        _ => match yaml.get("Proxy") {
            Some(Value::Sequence(seq)) => seq,
            _ => return false,
        },
    };

    let mut success = false;

    // Process each proxy in the sequence
    for proxy in proxies {
        if let Some(node) = parse_clash_proxy(proxy) {
            nodes.push(node);
            success = true;
        }
    }

    success
}

/// Parse a single proxy from Clash YAML
fn parse_clash_proxy(proxy: &Value) -> Option<Proxy> {
    // Extract the proxy type
    let proxy_type = match proxy.get("type") {
        Some(Value::String(t)) => t.to_lowercase(),
        _ => return None,
    };

    // Extract common fields
    let name = proxy.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let server = proxy.get("server").and_then(|v| v.as_str()).unwrap_or("");
    let port_value = proxy.get("port").and_then(|v| v.as_u64()).unwrap_or(0);
    let port = port_value as u16;

    // Skip if missing essential information
    if name.is_empty() || server.is_empty() || port == 0 {
        return None;
    }

    // Extract common optional fields
    let udp = proxy.get("udp").and_then(|v| v.as_bool());
    let tfo = proxy.get("tfo").and_then(|v| v.as_bool());
    let skip_cert_verify = proxy.get("skip-cert-verify").and_then(|v| v.as_bool());

    // Process based on proxy type
    match proxy_type.as_str() {
        "ss" | "shadowsocks" => {
            parse_clash_ss(proxy, name, server, port, udp, tfo, skip_cert_verify)
        }
        "ssr" | "shadowsocksr" => {
            parse_clash_ssr(proxy, name, server, port, udp, tfo, skip_cert_verify)
        }
        "vmess" => parse_clash_vmess(proxy, name, server, port, udp, tfo, skip_cert_verify),
        "socks" | "socks5" => {
            parse_clash_socks(proxy, name, server, port, udp, tfo, skip_cert_verify)
        }
        "http" => parse_clash_http(proxy, name, server, port, false, tfo, skip_cert_verify),
        "https" => parse_clash_http(proxy, name, server, port, true, tfo, skip_cert_verify),
        "trojan" => parse_clash_trojan(proxy, name, server, port, udp, tfo, skip_cert_verify),
        "snell" => parse_clash_snell(proxy, name, server, port, udp, tfo, skip_cert_verify),
        "wireguard" => parse_clash_wireguard(proxy, name, server, port, udp),
        "hysteria" => parse_clash_hysteria(proxy, name, server, port, tfo, skip_cert_verify),
        "hysteria2" => parse_clash_hysteria2(proxy, name, server, port, tfo, skip_cert_verify),
        _ => None,
    }
}

/// Parse a Shadowsocks proxy from Clash YAML
fn parse_clash_ss(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    udp: Option<bool>,
    tfo: Option<bool>,
    skip_cert_verify: Option<bool>,
) -> Option<Proxy> {
    // Extract SS-specific fields
    let password = proxy.get("password").and_then(|v| v.as_str()).unwrap_or("");
    let method = proxy.get("cipher").and_then(|v| v.as_str()).unwrap_or("");

    if password.is_empty() || method.is_empty() {
        return None;
    }

    // Extract underlying proxy
    let underlying_proxy = proxy
        .get("underlying-proxy")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Extract plugin information
    let mut plugin = "";
    let mut pluginopts_mode = "";
    let mut pluginopts_host = "";
    let mut path = "";
    let mut tls = "";
    let mut pluginopts_mux = "";
    let mut pluginopts = String::new();

    // Check if plugin is defined
    if let Some(plugin_val) = proxy.get("plugin").and_then(|v| v.as_str()) {
        match plugin_val {
            "obfs" => {
                plugin = "obfs-local";
                if let Some(plugin_opts) = proxy.get("plugin-opts").and_then(|v| v.as_mapping()) {
                    if let Some(mode) = plugin_opts
                        .get(&Value::String("mode".to_string()))
                        .and_then(|v| v.as_str())
                    {
                        pluginopts_mode = mode;
                    }
                    if let Some(host) = plugin_opts
                        .get(&Value::String("host".to_string()))
                        .and_then(|v| v.as_str())
                    {
                        pluginopts_host = host;
                    }
                }
            }
            "v2ray-plugin" => {
                plugin = "v2ray-plugin";
                if let Some(plugin_opts) = proxy.get("plugin-opts").and_then(|v| v.as_mapping()) {
                    if let Some(mode) = plugin_opts
                        .get(&Value::String("mode".to_string()))
                        .and_then(|v| v.as_str())
                    {
                        pluginopts_mode = mode;
                    }
                    if let Some(host) = plugin_opts
                        .get(&Value::String("host".to_string()))
                        .and_then(|v| v.as_str())
                    {
                        pluginopts_host = host;
                    }
                    if let Some(plugin_tls) = plugin_opts
                        .get(&Value::String("tls".to_string()))
                        .and_then(|v| v.as_bool())
                    {
                        tls = if plugin_tls { "tls;" } else { "" };
                    }
                    if let Some(plugin_path) = plugin_opts
                        .get(&Value::String("path".to_string()))
                        .and_then(|v| v.as_str())
                    {
                        path = plugin_path;
                    }
                    if let Some(mux) = plugin_opts
                        .get(&Value::String("mux".to_string()))
                        .and_then(|v| v.as_bool())
                    {
                        pluginopts_mux = if mux { "mux=4;" } else { "" };
                    }
                }
            }
            _ => {}
        }
    } else if let Some(obfs) = proxy.get("obfs").and_then(|v| v.as_str()) {
        // Legacy support for obfs and obfs-host fields
        plugin = "obfs-local";
        pluginopts_mode = obfs;
        if let Some(obfs_host) = proxy.get("obfs-host").and_then(|v| v.as_str()) {
            pluginopts_host = obfs_host;
        }
    }

    // Format plugin options based on plugin type
    match plugin {
        "simple-obfs" | "obfs-local" => {
            pluginopts = format!("obfs={}", pluginopts_mode);
            if !pluginopts_host.is_empty() {
                pluginopts.push_str(&format!(";obfs-host={}", pluginopts_host));
            }
        }
        "v2ray-plugin" => {
            pluginopts = format!("mode={};{}{}", pluginopts_mode, tls, pluginopts_mux);
            if !pluginopts_host.is_empty() {
                pluginopts.push_str(&format!("host={};", pluginopts_host));
            }
            if !path.is_empty() {
                pluginopts.push_str(&format!("path={};", path));
            }
            if !pluginopts_mux.is_empty() {
                pluginopts.push_str(&format!("mux={};", pluginopts_mux));
            }
        }
        _ => {}
    }

    // Handle special cipher types (support for go-shadowsocks2)
    let mut cipher = method;
    if cipher == "AEAD_CHACHA20_POLY1305" {
        cipher = "chacha20-ietf-poly1305";
    } else if cipher.contains("AEAD") {
        // Not implementing the full C++ transformation for now
    }

    // Convert pluginopts String to &str
    let pluginopts_str = Box::leak(pluginopts.into_boxed_str());

    Some(Proxy::ss_construct(
        SS_DEFAULT_GROUP,
        name,
        server,
        port,
        password,
        cipher,
        plugin,
        pluginopts_str,
        udp,
        tfo,
        skip_cert_verify,
        None,
        underlying_proxy,
    ))
}

/// Parse a ShadowsocksR proxy from Clash YAML
fn parse_clash_ssr(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    udp: Option<bool>,
    tfo: Option<bool>,
    skip_cert_verify: Option<bool>,
) -> Option<Proxy> {
    // Extract SSR-specific fields
    let password = proxy.get("password").and_then(|v| v.as_str()).unwrap_or("");
    let method = proxy.get("cipher").and_then(|v| v.as_str()).unwrap_or("");
    let protocol = proxy.get("protocol").and_then(|v| v.as_str()).unwrap_or("");
    let protocol_param = proxy
        .get("protocol-param")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let obfs = proxy.get("obfs").and_then(|v| v.as_str()).unwrap_or("");
    let obfs_param = proxy
        .get("obfs-param")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Extract underlying proxy
    let underlying_proxy = proxy
        .get("underlying-proxy")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if password.is_empty() || method.is_empty() || protocol.is_empty() || obfs.is_empty() {
        return None;
    }

    Some(Proxy::ssr_construct(
        SSR_DEFAULT_GROUP,
        name,
        server,
        port,
        protocol,
        method,
        obfs,
        password,
        obfs_param,
        protocol_param,
        udp,
        tfo,
        skip_cert_verify,
        underlying_proxy,
    ))
}

/// Parse a VMess proxy from Clash YAML
fn parse_clash_vmess(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    udp: Option<bool>,
    tfo: Option<bool>,
    skip_cert_verify: Option<bool>,
) -> Option<Proxy> {
    // Extract VMess-specific fields
    let uuid = proxy.get("uuid").and_then(|v| v.as_str()).unwrap_or("");
    let alter_id_val = proxy.get("alterId").and_then(|v| v.as_u64()).unwrap_or(0);
    let alter_id = alter_id_val as u16;
    let cipher = proxy
        .get("cipher")
        .and_then(|v| v.as_str())
        .unwrap_or("auto");

    // Extract underlying proxy
    let underlying_proxy = proxy
        .get("underlying-proxy")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if uuid.is_empty() {
        return None;
    }

    // Get network settings
    let network = proxy
        .get("network")
        .and_then(|v| v.as_str())
        .unwrap_or("tcp");

    // Get TLS settings
    let tls = proxy.get("tls").and_then(|v| v.as_bool()).unwrap_or(false);
    let sni = proxy
        .get("servername")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Parse network specific options
    let mut host = String::new();
    let mut path = String::new();

    // Handle WebSocket options
    if let Some(ws_opts) = proxy.get("ws-opts").and_then(|v| v.as_mapping()) {
        if let Some(path_val) = ws_opts
            .get(&Value::String("path".to_string()))
            .and_then(|v| v.as_str())
        {
            path = path_val.to_string();
        }

        if let Some(headers) = ws_opts
            .get(&Value::String("headers".to_string()))
            .and_then(|v| v.as_mapping())
        {
            if let Some(host_val) = headers
                .get(&Value::String("Host".to_string()))
                .and_then(|v| v.as_str())
            {
                host = host_val.to_string();
            }
        }
    }
    // Handle HTTP/2 options
    else if let Some(h2_opts) = proxy.get("h2-opts").and_then(|v| v.as_mapping()) {
        if let Some(path_val) = h2_opts
            .get(&Value::String("path".to_string()))
            .and_then(|v| v.as_str())
        {
            path = path_val.to_string();
        }

        if let Some(hosts) = h2_opts
            .get(&Value::String("host".to_string()))
            .and_then(|v| v.as_sequence())
        {
            if !hosts.is_empty() {
                if let Some(first_host) = hosts.get(0).and_then(|v| v.as_str()) {
                    host = first_host.to_string();
                }
            }
        }
    }
    // Handle HTTP options
    else if let Some(http_opts) = proxy.get("http-opts").and_then(|v| v.as_mapping()) {
        if let Some(paths) = http_opts
            .get(&Value::String("path".to_string()))
            .and_then(|v| v.as_sequence())
        {
            if !paths.is_empty() {
                if let Some(first_path) = paths.get(0).and_then(|v| v.as_str()) {
                    path = first_path.to_string();
                }
            }
        }

        if let Some(hosts) = http_opts
            .get(&Value::String("host".to_string()))
            .and_then(|v| v.as_sequence())
        {
            if !hosts.is_empty() {
                if let Some(first_host) = hosts.get(0).and_then(|v| v.as_str()) {
                    host = first_host.to_string();
                }
            }
        }
    }
    // Handle gRPC options
    else if let Some(grpc_opts) = proxy.get("grpc-opts").and_then(|v| v.as_mapping()) {
        if let Some(service_name) = grpc_opts
            .get(&Value::String("grpc-service-name".to_string()))
            .and_then(|v| v.as_str())
        {
            path = service_name.to_string();
        }
    }

    // Prepare path
    let final_path = if path.is_empty() { "/" } else { &path };

    // Get edge value
    let edge = "";

    Some(Proxy::vmess_construct(
        V2RAY_DEFAULT_GROUP,
        name,
        server,
        port,
        "", // type
        uuid,
        alter_id,
        network,
        cipher,
        final_path,
        &host,
        edge,
        if tls { "tls" } else { "" },
        sni,
        udp,
        tfo,
        skip_cert_verify,
        None,
        underlying_proxy,
    ))
}

/// Parse a SOCKS5 proxy from Clash YAML
fn parse_clash_socks(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    udp: Option<bool>,
    tfo: Option<bool>,
    skip_cert_verify: Option<bool>,
) -> Option<Proxy> {
    // Extract SOCKS-specific fields
    let username = proxy.get("username").and_then(|v| v.as_str()).unwrap_or("");
    let password = proxy.get("password").and_then(|v| v.as_str()).unwrap_or("");

    // Extract underlying proxy
    let underlying_proxy = proxy
        .get("underlying-proxy")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    Some(Proxy::socks_construct(
        SOCKS_DEFAULT_GROUP,
        name,
        server,
        port,
        username,
        password,
        udp,
        tfo,
        skip_cert_verify,
        underlying_proxy,
    ))
}

/// Parse an HTTP/HTTPS proxy from Clash YAML
fn parse_clash_http(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    is_https: bool,
    tfo: Option<bool>,
    skip_cert_verify: Option<bool>,
) -> Option<Proxy> {
    // Extract HTTP-specific fields
    let username = proxy.get("username").and_then(|v| v.as_str()).unwrap_or("");
    let password = proxy.get("password").and_then(|v| v.as_str()).unwrap_or("");

    // Extract underlying proxy
    let underlying_proxy = proxy
        .get("underlying-proxy")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    Some(Proxy::http_construct(
        HTTP_DEFAULT_GROUP,
        name,
        server,
        port,
        username,
        password,
        is_https,
        tfo,
        skip_cert_verify,
        None,
        underlying_proxy,
    ))
}

/// Parse a Trojan proxy from Clash YAML
fn parse_clash_trojan(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    udp: Option<bool>,
    tfo: Option<bool>,
    skip_cert_verify: Option<bool>,
) -> Option<Proxy> {
    // Extract Trojan-specific fields
    let password = proxy.get("password").and_then(|v| v.as_str()).unwrap_or("");

    if password.is_empty() {
        return None;
    }

    // Extract underlying proxy
    let underlying_proxy = proxy
        .get("underlying-proxy")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Get SNI and network settings
    let sni = proxy.get("sni").and_then(|v| v.as_str()).unwrap_or("");
    let network = proxy.get("network").and_then(|v| v.as_str()).unwrap_or("");

    // Get path and host, if any
    let mut host = String::new();
    let mut path = String::new();

    // Handle WebSocket options if specified
    if network == "ws" && proxy.get("ws-opts").is_some() {
        if let Some(ws_opts) = proxy.get("ws-opts").and_then(|v| v.as_mapping()) {
            if let Some(path_val) = ws_opts
                .get(&Value::String("path".to_string()))
                .and_then(|v| v.as_str())
            {
                path = path_val.to_string();
            }

            if let Some(headers) = ws_opts
                .get(&Value::String("headers".to_string()))
                .and_then(|v| v.as_mapping())
            {
                if let Some(host_val) = headers
                    .get(&Value::String("Host".to_string()))
                    .and_then(|v| v.as_str())
                {
                    host = host_val.to_string();
                }
            }
        }
    }

    Some(Proxy::trojan_construct(
        TROJAN_DEFAULT_GROUP.to_string(),
        name.to_string(),
        server.to_string(),
        port,
        password.to_string(),
        Some(network.to_string()),
        Some(host),
        Some(path),
        true, // tls_secure, Trojan always uses TLS
        udp,
        tfo,
        skip_cert_verify,
        None,
        Some(underlying_proxy.to_string()),
    ))
}

/// Parse a Snell proxy from Clash YAML
fn parse_clash_snell(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    udp: Option<bool>,
    tfo: Option<bool>,
    skip_cert_verify: Option<bool>,
) -> Option<Proxy> {
    // Extract Snell-specific fields
    let psk = proxy.get("psk").and_then(|v| v.as_str()).unwrap_or("");

    if psk.is_empty() {
        return None;
    }

    // Extract underlying proxy
    let underlying_proxy = proxy
        .get("underlying-proxy")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Get obfs settings
    let version = proxy.get("version").and_then(|v| v.as_u64()).unwrap_or(1) as u16;
    let obfs = proxy.get("obfs").and_then(|v| v.as_str()).unwrap_or("");
    let obfs_host = proxy
        .get("obfs-host")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    Some(Proxy::snell_construct(
        SNELL_DEFAULT_GROUP.to_string(),
        name.to_string(),
        server.to_string(),
        port,
        psk.to_string(),
        obfs.to_string(),
        obfs_host.to_string(),
        version,
        udp,
        tfo,
        skip_cert_verify,
        Some(underlying_proxy.to_string()),
    ))
}

/// Parse a WireGuard proxy from Clash YAML
fn parse_clash_wireguard(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    udp: Option<bool>,
) -> Option<Proxy> {
    // Extract WireGuard-specific fields
    let private_key = proxy
        .get("privateKey")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let public_key = proxy
        .get("publicKey")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let preshared_key = proxy
        .get("presharedKey")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if private_key.is_empty() || public_key.is_empty() {
        return None;
    }

    // Extract underlying proxy
    let underlying_proxy = proxy
        .get("underlying-proxy")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Get IP addresses
    let self_ip = proxy.get("ip").and_then(|v| v.as_str()).unwrap_or("");
    let self_ipv6 = proxy.get("ipv6").and_then(|v| v.as_str()).unwrap_or("");

    // Get MTU and keepalive
    let mtu_value = proxy.get("mtu").and_then(|v| v.as_u64()).unwrap_or(0);
    let mtu = if mtu_value > 0 {
        Some(mtu_value as u16)
    } else {
        None
    };

    let keepalive_value = proxy.get("keepalive").and_then(|v| v.as_u64()).unwrap_or(0);
    let keepalive = if keepalive_value > 0 {
        Some(keepalive_value as u16)
    } else {
        None
    };

    // Get DNS servers
    let mut dns_servers = Vec::new();
    if let Some(Value::Sequence(dns_seq)) = proxy.get("dns") {
        for dns in dns_seq {
            if let Some(dns_str) = dns.as_str() {
                dns_servers.push(dns_str.to_string());
            }
        }
    }

    // Get client ID and test URL
    let client_id = proxy.get("clientId").and_then(|v| v.as_str()).unwrap_or("");
    let test_url = proxy.get("testUrl").and_then(|v| v.as_str()).unwrap_or("");

    Some(Proxy::wireguard_construct(
        WG_DEFAULT_GROUP.to_string(),
        name.to_string(),
        server.to_string(),
        port,
        self_ip.to_string(),
        self_ipv6.to_string(),
        private_key.to_string(),
        public_key.to_string(),
        preshared_key.to_string(),
        dns_servers,
        mtu,
        keepalive,
        test_url.to_string(),
        client_id.to_string(),
        udp,
        Some(underlying_proxy.to_string()),
    ))
}

/// Parse a Hysteria proxy from Clash YAML
fn parse_clash_hysteria(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    tfo: Option<bool>,
    skip_cert_verify: Option<bool>,
) -> Option<Proxy> {
    // Extract Hysteria-specific fields
    let auth = proxy.get("auth").and_then(|v| v.as_str()).unwrap_or("");
    let auth_str = proxy.get("auth-str").and_then(|v| v.as_str()).unwrap_or("");
    let obfs = proxy.get("obfs").and_then(|v| v.as_str()).unwrap_or("");
    let protocol = proxy
        .get("protocol")
        .and_then(|v| v.as_str())
        .unwrap_or("udp");

    // Extract underlying proxy
    let underlying_proxy = proxy
        .get("underlying-proxy")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Get ports range if specified
    let ports = proxy.get("ports").and_then(|v| v.as_str()).unwrap_or("");

    // Get up/down speeds
    let up_mbps = proxy.get("up").and_then(|v| v.as_u64()).unwrap_or(0);
    let down_mbps = proxy.get("down").and_then(|v| v.as_u64()).unwrap_or(0);
    let up_speed = if up_mbps > 0 {
        Some(up_mbps as u32)
    } else {
        None
    };
    let down_speed = if down_mbps > 0 {
        Some(down_mbps as u32)
    } else {
        None
    };

    // Get TLS settings
    let sni = proxy.get("sni").and_then(|v| v.as_str()).unwrap_or("");
    let alpn_value = proxy.get("alpn").and_then(|v| v.as_str()).unwrap_or("");
    let mut alpn = Vec::new();
    if !alpn_value.is_empty() {
        alpn.push(alpn_value.to_string());
    }

    let fingerprint = proxy
        .get("fingerprint")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let ca = proxy.get("ca").and_then(|v| v.as_str()).unwrap_or("");
    let ca_str = proxy.get("ca-str").and_then(|v| v.as_str()).unwrap_or("");

    // Get advanced settings
    let recv_window_conn_value = proxy
        .get("recv-window-conn")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let recv_window_value = proxy
        .get("recv-window")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let recv_window_conn = if recv_window_conn_value > 0 {
        Some(recv_window_conn_value as u32)
    } else {
        None
    };
    let recv_window = if recv_window_value > 0 {
        Some(recv_window_value as u32)
    } else {
        None
    };

    let disable_mtu_discovery = proxy.get("disable-mtu-discovery").and_then(|v| v.as_bool());

    let hop_interval_value = proxy
        .get("hop-interval")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let hop_interval = if hop_interval_value > 0 {
        Some(hop_interval_value as u32)
    } else {
        None
    };

    Some(Proxy::hysteria_construct(
        HYSTERIA_DEFAULT_GROUP.to_string(),
        name.to_string(),
        server.to_string(),
        port,
        ports.to_string(),
        protocol.to_string(),
        "".to_string(), // obfs_param
        up_speed,
        down_speed,
        if !auth.is_empty() {
            auth.to_string()
        } else {
            auth_str.to_string()
        },
        obfs.to_string(),
        sni.to_string(),
        fingerprint.to_string(),
        ca.to_string(),
        ca_str.to_string(),
        recv_window_conn,
        recv_window,
        disable_mtu_discovery,
        hop_interval,
        alpn,
        tfo,
        skip_cert_verify,
        Some(underlying_proxy.to_string()),
    ))
}

/// Parse a Hysteria2 proxy from Clash YAML
fn parse_clash_hysteria2(
    proxy: &Value,
    name: &str,
    server: &str,
    port: u16,
    tfo: Option<bool>,
    skip_cert_verify: Option<bool>,
) -> Option<Proxy> {
    // Extract Hysteria2-specific fields
    let password = proxy.get("password").and_then(|v| v.as_str()).unwrap_or("");

    // Extract underlying proxy
    let underlying_proxy = proxy
        .get("underlying-proxy")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Get obfs settings
    let obfs = proxy.get("obfs").and_then(|v| v.as_str()).unwrap_or("");
    let obfs_password = proxy
        .get("obfs-password")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Get ports range if specified
    let ports = proxy.get("ports").and_then(|v| v.as_str()).unwrap_or("");

    // Get up/down speeds
    let up_mbps = proxy.get("up").and_then(|v| v.as_u64()).unwrap_or(0);
    let down_mbps = proxy.get("down").and_then(|v| v.as_u64()).unwrap_or(0);
    let up_speed = if up_mbps > 0 {
        Some(up_mbps as u32)
    } else {
        None
    };
    let down_speed = if down_mbps > 0 {
        Some(down_mbps as u32)
    } else {
        None
    };

    // Get TLS settings
    let sni = proxy.get("sni").and_then(|v| v.as_str()).unwrap_or("");
    let alpn_value = proxy.get("alpn").and_then(|v| v.as_str()).unwrap_or("");
    let mut alpn = Vec::new();
    if !alpn_value.is_empty() {
        alpn.push(alpn_value.to_string());
    }

    let fingerprint = proxy
        .get("fingerprint")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let ca = proxy.get("ca").and_then(|v| v.as_str()).unwrap_or("");
    let ca_str = proxy.get("ca-str").and_then(|v| v.as_str()).unwrap_or("");

    // Get congestion window
    let cwnd_value = proxy.get("cwnd").and_then(|v| v.as_u64()).unwrap_or(0);
    let cwnd = if cwnd_value > 0 {
        Some(cwnd_value as u32)
    } else {
        None
    };

    Some(Proxy::hysteria2_construct(
        HYSTERIA2_DEFAULT_GROUP.to_string(),
        name.to_string(),
        server.to_string(),
        port,
        ports.to_string(),
        up_speed,
        down_speed,
        password.to_string(),
        obfs.to_string(),
        obfs_password.to_string(),
        sni.to_string(),
        fingerprint.to_string(),
        alpn,
        ca.to_string(),
        ca_str.to_string(),
        cwnd,
        tfo,
        skip_cert_verify,
        Some(underlying_proxy.to_string()),
    ))
}

#[cfg(test)]
mod tests {
    use crate::ProxyType;

    use super::*;

    // Helper function to create a basic YAML Value for SS with simple-obfs plugin
    fn create_ss_obfs_yaml() -> Value {
        let yaml_str = r#"
        {
          "name": "SS Simple-Obfs Test",
          "type": "ss",
          "server": "example.com",
          "port": 8388,
          "cipher": "aes-256-gcm",
          "password": "password123",
          "plugin": "simple-obfs",
          "plugin-opts": {
            "mode": "http",
            "host": "example.org"
          },
          "udp": true
        }
        "#;
        serde_json::from_str(yaml_str).unwrap()
    }

    // Helper function to create a basic YAML Value for SS with v2ray-plugin
    fn create_ss_v2ray_yaml() -> Value {
        let yaml_str = r#"
        {
          "name": "SS V2Ray Test",
          "type": "ss",
          "server": "example.com",
          "port": 8388,
          "cipher": "aes-256-gcm",
          "password": "password123",
          "plugin": "v2ray-plugin",
          "plugin-opts": {
            "mode": "websocket",
            "host": "example.org",
            "path": "/v2ray",
            "tls": true,
            "mux": true,
            "headers": {
              "custom": "value"
            }
          },
          "udp": true
        }
        "#;
        serde_json::from_str(yaml_str).unwrap()
    }

    // Helper function to create a basic YAML Value for SS with legacy obfs fields
    fn create_ss_legacy_obfs_yaml() -> Value {
        let yaml_str = r#"
        {
          "name": "SS Legacy Obfs Test",
          "type": "ss",
          "server": "example.com",
          "port": 8388,
          "cipher": "aes-256-gcm",
          "password": "password123",
          "obfs": "http",
          "obfs-host": "example.org",
          "udp": true
        }
        "#;
        serde_json::from_str(yaml_str).unwrap()
    }

    // Helper function to create a YAML Value for SS with shadow-tls plugin
    fn create_ss_shadow_tls_yaml() -> Value {
        let yaml_str = r#"
        {
          "name": "SS Shadow-TLS Test",
          "type": "ss",
          "server": "example.com",
          "port": 8388,
          "cipher": "aes-256-gcm",
          "password": "password123",
          "plugin": "shadow-tls",
          "plugin-opts": {
            "host": "example.org",
            "password": "shadowpassword",
            "version": "2"
          },
          "udp": true
        }
        "#;
        serde_json::from_str(yaml_str).unwrap()
    }

    // Helper function to create a basic YAML Value for VMess testing
    fn create_vmess_yaml() -> Value {
        let yaml_str = r#"
        {
          "name": "VMess Test",
          "type": "vmess",
          "server": "example.com",
          "port": 443,
          "uuid": "a3c14e2a-a37f-11ec-b909-0242ac120002",
          "alterId": 0,
          "cipher": "auto",
          "network": "ws",
          "tls": true,
          "servername": "example.com",
          "ws-opts": {
            "path": "/path",
            "headers": {
              "Host": "example.com"
            }
          },
          "udp": true,
          "underlying-proxy": "proxy-1"
        }
        "#;
        serde_json::from_str(yaml_str).unwrap()
    }

    // Helper function to create a basic YAML Value for WireGuard testing
    fn create_wireguard_yaml() -> Value {
        let yaml_str = r#"
        {
          "name": "WireGuard Test",
          "type": "wireguard",
          "server": "example.com",
          "port": 51820,
          "ip": "10.0.0.2",
          "ipv6": "fd00::2",
          "privateKey": "eCtXsJZ27+4PbhDkHnB923tkUn2Gj59wZw5wFA75MnA=",
          "publicKey": "Cr7L3k4R89NjLRYUvxSaQQHcRNDwse9P9FY+9wPD1jE=",
          "presharedKey": "dQ9uRw8+H3cIPJDGYOqAlPPQ/dxx/RUX8YnhhJ+aBkQ=",
          "dns": ["1.1.1.1", "8.8.8.8"],
          "mtu": 1420,
          "keepalive": 25,
          "udp": true,
          "underlying-proxy": "proxy-2"
        }
        "#;
        serde_json::from_str(yaml_str).unwrap()
    }

    // Helper function to create a basic YAML Value for Hysteria testing
    fn create_hysteria_yaml() -> Value {
        let yaml_str = r#"
        {
          "name": "Hysteria Test",
          "type": "hysteria",
          "server": "example.com",
          "port": 9000,
          "auth": "password123",
          "protocol": "udp",
          "up": 20,
          "down": 100,
          "obfs": "xplus",
          "sni": "example.com",
          "alpn": ["h3"],
          "skip-cert-verify": true,
          "underlying-proxy": "proxy-3"
        }
        "#;
        serde_json::from_str(yaml_str).unwrap()
    }

    // Helper function to create a basic YAML Value for Hysteria2 testing
    fn create_hysteria2_yaml() -> Value {
        let yaml_str = r#"
        {
          "name": "Hysteria2 Test",
          "type": "hysteria2",
          "server": "example.com",
          "port": 9000,
          "password": "password123",
          "up": 20,
          "down": 100,
          "obfs": "salamander",
          "obfs-password": "obfs-pass",
          "sni": "example.com",
          "skip-cert-verify": true,
          "underlying-proxy": "proxy-4"
        }
        "#;
        serde_json::from_str(yaml_str).unwrap()
    }

    #[test]
    fn test_parse_clash_ss_obfs() {
        let ss_yaml = create_ss_obfs_yaml();
        let node = parse_clash_ss(
            &ss_yaml,
            "SS Simple-Obfs Test",
            "example.com",
            8388,
            Some(true),
            None,
            None,
        )
        .unwrap();

        assert_eq!(node.proxy_type, ProxyType::Shadowsocks);
        assert_eq!(node.remark, "SS Simple-Obfs Test");
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 8388);
        assert_eq!(node.encrypt_method, Some("aes-256-gcm".to_string()));
        assert_eq!(node.password, Some("password123".to_string()));
        assert_eq!(node.plugin, Some("simple-obfs".to_string()));
        assert_eq!(
            node.plugin_option,
            Some("obfs=http;obfs-host=example.org".to_string())
        );
        assert_eq!(node.udp, Some(true));
    }

    #[test]
    fn test_parse_clash_ss_v2ray() {
        let ss_yaml = create_ss_v2ray_yaml();
        let node = parse_clash_ss(
            &ss_yaml,
            "SS V2Ray Test",
            "example.com",
            8388,
            Some(true),
            None,
            None,
        )
        .unwrap();

        assert_eq!(node.proxy_type, ProxyType::Shadowsocks);
        assert_eq!(node.remark, "SS V2Ray Test");
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 8388);
        assert_eq!(node.encrypt_method, Some("aes-256-gcm".to_string()));
        assert_eq!(node.password, Some("password123".to_string()));
        assert_eq!(node.plugin, Some("v2ray-plugin".to_string()));

        let plugin_opts = node.plugin_option.unwrap();
        assert!(plugin_opts.contains("mode=websocket"));
        assert!(plugin_opts.contains("host=example.org"));
        assert!(plugin_opts.contains("path=/v2ray"));
        assert!(plugin_opts.contains("tls"));
        assert!(plugin_opts.contains("mux=1"));
        assert!(plugin_opts.contains("custom=value"));

        assert_eq!(node.udp, Some(true));
    }

    #[test]
    fn test_parse_clash_ss_legacy_obfs() {
        let ss_yaml = create_ss_legacy_obfs_yaml();
        let node = parse_clash_ss(
            &ss_yaml,
            "SS Legacy Obfs Test",
            "example.com",
            8388,
            Some(true),
            None,
            None,
        )
        .unwrap();

        assert_eq!(node.proxy_type, ProxyType::Shadowsocks);
        assert_eq!(node.remark, "SS Legacy Obfs Test");
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 8388);
        assert_eq!(node.encrypt_method, Some("aes-256-gcm".to_string()));
        assert_eq!(node.password, Some("password123".to_string()));
        assert_eq!(node.plugin, Some("simple-obfs".to_string()));
        assert_eq!(
            node.plugin_option,
            Some("obfs=http;obfs-host=example.org".to_string())
        );
        assert_eq!(node.udp, Some(true));
    }

    #[test]
    fn test_parse_clash_ss_shadow_tls() {
        let ss_yaml = create_ss_shadow_tls_yaml();
        let node = parse_clash_ss(
            &ss_yaml,
            "SS Shadow-TLS Test",
            "example.com",
            8388,
            Some(true),
            None,
            None,
        )
        .unwrap();

        assert_eq!(node.proxy_type, ProxyType::Shadowsocks);
        assert_eq!(node.remark, "SS Shadow-TLS Test");
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 8388);
        assert_eq!(node.encrypt_method, Some("aes-256-gcm".to_string()));
        assert_eq!(node.password, Some("password123".to_string()));
        assert_eq!(node.plugin, Some("shadow-tls".to_string()));

        let plugin_opts = node.plugin_option.unwrap();
        assert!(plugin_opts.contains("host=example.org"));
        assert!(plugin_opts.contains("password=shadowpassword"));
        assert!(plugin_opts.contains("version=2"));

        assert_eq!(node.udp, Some(true));
    }

    #[test]
    fn test_parse_clash_vmess() {
        let vmess_yaml = create_vmess_yaml();
        let node = parse_clash_vmess(
            &vmess_yaml,
            "VMess Test",
            "example.com",
            443,
            Some(true),
            None,
            None,
        )
        .unwrap();

        assert_eq!(node.proxy_type, ProxyType::VMess);
        assert_eq!(node.remark, "VMess Test");
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 443);
        assert_eq!(
            node.user_id,
            Some("a3c14e2a-a37f-11ec-b909-0242ac120002".to_string())
        );
        assert_eq!(node.alter_id, 0);
        assert_eq!(node.encrypt_method, Some("auto".to_string()));
        assert_eq!(node.transfer_protocol, Some("ws".to_string()));
        assert_eq!(node.tls_secure, true);
        assert_eq!(node.path, Some("/path".to_string()));
        assert_eq!(node.host, Some("example.com".to_string()));
        assert_eq!(node.server_name, Some("example.com".to_string()));
        assert_eq!(node.udp, Some(true));
        assert_eq!(node.underlying_proxy, Some("proxy-1".to_string()));
    }

    #[test]
    fn test_parse_clash_wireguard() {
        let wireguard_yaml = create_wireguard_yaml();
        let node = parse_clash_wireguard(
            &wireguard_yaml,
            "WireGuard Test",
            "example.com",
            51820,
            Some(true),
        )
        .unwrap();

        assert_eq!(node.proxy_type, ProxyType::WireGuard);
        assert_eq!(node.remark, "WireGuard Test");
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 51820);
        assert_eq!(node.self_ip, Some("10.0.0.2".to_string()));
        assert_eq!(node.self_ipv6, Some("fd00::2".to_string()));
        assert_eq!(
            node.private_key,
            Some("eCtXsJZ27+4PbhDkHnB923tkUn2Gj59wZw5wFA75MnA=".to_string())
        );
        assert_eq!(
            node.public_key,
            Some("Cr7L3k4R89NjLRYUvxSaQQHcRNDwse9P9FY+9wPD1jE=".to_string())
        );
        assert_eq!(
            node.pre_shared_key,
            Some("dQ9uRw8+H3cIPJDGYOqAlPPQ/dxx/RUX8YnhhJ+aBkQ=".to_string())
        );
        assert_eq!(node.mtu, 1420);
        assert_eq!(node.keep_alive, 25);
        assert_eq!(node.udp, Some(true));

        assert!(node.dns_servers.contains("1.1.1.1"));
        assert!(node.dns_servers.contains("8.8.8.8"));

        assert_eq!(node.underlying_proxy, Some("proxy-2".to_string()));
    }

    #[test]
    fn test_parse_clash_hysteria() {
        let hysteria_yaml = create_hysteria_yaml();
        let node = parse_clash_hysteria(
            &hysteria_yaml,
            "Hysteria Test",
            "example.com",
            9000,
            None,
            Some(true),
        )
        .unwrap();

        assert_eq!(node.proxy_type, ProxyType::Hysteria);
        assert_eq!(node.remark, "Hysteria Test");
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 9000);
        assert_eq!(node.auth_str, Some("password123".to_string()));
        assert_eq!(node.protocol, Some("udp".to_string()));
        assert_eq!(node.up_speed, 20);
        assert_eq!(node.down_speed, 100);
        assert_eq!(node.obfs, Some("xplus".to_string()));
        assert_eq!(node.sni, Some("example.com".to_string()));
        assert_eq!(node.allow_insecure, Some(true));

        assert!(node.alpn.contains("h3"));

        assert_eq!(node.underlying_proxy, Some("proxy-3".to_string()));
    }

    #[test]
    fn test_parse_clash_hysteria2() {
        let hysteria2_yaml = create_hysteria2_yaml();
        let node = parse_clash_hysteria2(
            &hysteria2_yaml,
            "Hysteria2 Test",
            "example.com",
            9000,
            None,
            Some(true),
        )
        .unwrap();

        assert_eq!(node.proxy_type, ProxyType::Hysteria2);
        assert_eq!(node.remark, "Hysteria2 Test");
        assert_eq!(node.hostname, "example.com");
        assert_eq!(node.port, 9000);
        assert_eq!(node.password, Some("password123".to_string()));
        assert_eq!(node.up_speed, 20);
        assert_eq!(node.down_speed, 100);
        assert_eq!(node.obfs, Some("salamander".to_string()));
        assert_eq!(node.obfs_param, Some("obfs-pass".to_string()));
        assert_eq!(node.sni, Some("example.com".to_string()));
        assert_eq!(node.allow_insecure, Some(true));

        assert_eq!(node.underlying_proxy, Some("proxy-4".to_string()));
    }

    #[test]
    fn test_explode_clash() {
        let yaml_str = r#"
        proxies:
          - name: "SS Test"
            type: ss
            server: example.com
            port: 8388
            cipher: aes-256-gcm
            password: "password123"
            udp: true
            underlying-proxy: "ss-proxy"
          - name: "VMess Test"
            type: vmess
            server: example.org
            port: 443
            uuid: a3c14e2a-a37f-11ec-b909-0242ac120002
            alterId: 0
            cipher: auto
            network: ws
            tls: true
            servername: example.org
            ws-opts:
              path: /path
              headers:
                Host: example.org
            underlying-proxy: "vmess-proxy"
          - name: "WireGuard Test"
            type: wireguard
            server: wg.example.com
            port: 51820
            ip: 10.0.0.2
            privateKey: eCtXsJZ27+4PbhDkHnB923tkUn2Gj59wZw5wFA75MnA=
            publicKey: Cr7L3k4R89NjLRYUvxSaQQHcRNDwse9P9FY+9wPD1jE=
            udp: true
            underlying-proxy: "wg-proxy"
          - name: "Hysteria Test"
            type: hysteria
            server: hy.example.com
            port: 9000
            auth: password123
            up: 20
            down: 100
            underlying-proxy: "hy-proxy"
          - name: "Hysteria2 Test"
            type: hysteria2
            server: hy2.example.com
            port: 9000
            password: password123
            up: 20
            down: 100
            underlying-proxy: "hy2-proxy"
        "#;

        let mut nodes = Vec::new();
        let result = explode_clash(yaml_str, &mut nodes);

        assert!(result);
        assert_eq!(nodes.len(), 5);

        let ss_node = &nodes[0];
        assert_eq!(ss_node.proxy_type, ProxyType::Shadowsocks);
        assert_eq!(ss_node.remark, "SS Test");
        assert_eq!(ss_node.underlying_proxy, Some("ss-proxy".to_string()));

        let vmess_node = &nodes[1];
        assert_eq!(vmess_node.proxy_type, ProxyType::VMess);
        assert_eq!(vmess_node.remark, "VMess Test");
        assert_eq!(vmess_node.underlying_proxy, Some("vmess-proxy".to_string()));

        let wg_node = &nodes[2];
        assert_eq!(wg_node.proxy_type, ProxyType::WireGuard);
        assert_eq!(wg_node.remark, "WireGuard Test");
        assert_eq!(wg_node.hostname, "wg.example.com");
        assert_eq!(wg_node.underlying_proxy, Some("wg-proxy".to_string()));

        let hy_node = &nodes[3];
        assert_eq!(hy_node.proxy_type, ProxyType::Hysteria);
        assert_eq!(hy_node.remark, "Hysteria Test");
        assert_eq!(hy_node.hostname, "hy.example.com");
        assert_eq!(hy_node.underlying_proxy, Some("hy-proxy".to_string()));

        let hy2_node = &nodes[4];
        assert_eq!(hy2_node.proxy_type, ProxyType::Hysteria2);
        assert_eq!(hy2_node.remark, "Hysteria2 Test");
        assert_eq!(hy2_node.hostname, "hy2.example.com");
        assert_eq!(hy2_node.underlying_proxy, Some("hy2-proxy".to_string()));
    }
}
