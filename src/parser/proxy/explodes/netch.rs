use crate::parser::proxy::{
    Proxy, ProxyType, HTTP_DEFAULT_GROUP, SOCKS_DEFAULT_GROUP, SSR_DEFAULT_GROUP, SS_DEFAULT_GROUP,
    TROJAN_DEFAULT_GROUP, V2RAY_DEFAULT_GROUP,
};
use base64;
use serde_json::{json, Value};
use url::Url;

/// Parse a Netch JSON configuration into a Proxy object
pub fn explode_netch(content: &str, node: &mut Proxy) -> bool {
    // Parse the JSON content
    let json: Value = match serde_json::from_str(content) {
        Ok(j) => j,
        Err(_) => return false,
    };

    // Check if required fields exist
    let type_str = match json.get("Type").and_then(Value::as_str) {
        Some(t) => t,
        None => return false,
    };

    let remark = match json.get("Remark").and_then(Value::as_str) {
        Some(r) => r,
        None => return false,
    };

    let server = match json.get("Hostname").and_then(Value::as_str) {
        Some(s) => s,
        None => return false,
    };

    let port = match json.get("Port").and_then(Value::as_u64) {
        Some(p) => p as u16,
        None => return false,
    };

    // Process based on proxy type
    match type_str {
        "Shadowsocks" => {
            let method = json.get("Method").and_then(Value::as_str).unwrap_or("");
            let password = json.get("Password").and_then(Value::as_str).unwrap_or("");

            if method.is_empty() || password.is_empty() {
                return false;
            }

            let plugin = json.get("Plugin").and_then(Value::as_str).unwrap_or("");
            let plugin_opts = json
                .get("PluginOption")
                .and_then(Value::as_str)
                .unwrap_or("");

            *node = Proxy::ss_construct(
                SS_DEFAULT_GROUP,
                remark,
                server,
                port,
                password,
                method,
                plugin,
                plugin_opts,
                None,
                None,
                None,
                None,
                "",
            );

            true
        }
        "ShadowsocksR" => {
            let method = json.get("Method").and_then(Value::as_str).unwrap_or("");
            let password = json.get("Password").and_then(Value::as_str).unwrap_or("");
            let protocol = json.get("Protocol").and_then(Value::as_str).unwrap_or("");
            let obfs = json.get("OBFS").and_then(Value::as_str).unwrap_or("");
            let protocol_param = json
                .get("ProtocolParam")
                .and_then(Value::as_str)
                .unwrap_or("");
            let obfs_param = json.get("OBFSParam").and_then(Value::as_str).unwrap_or("");

            if method.is_empty() || password.is_empty() || protocol.is_empty() || obfs.is_empty() {
                return false;
            }

            *node = Proxy::ssr_construct(
                SSR_DEFAULT_GROUP,
                remark,
                server,
                port,
                protocol,
                method,
                obfs,
                password,
                obfs_param,
                protocol_param,
                None,
                None,
                None,
                "",
            );

            true
        }
        "SOCKS5" => {
            let username = json.get("Username").and_then(Value::as_str).unwrap_or("");
            let password = json.get("Password").and_then(Value::as_str).unwrap_or("");

            *node = Proxy::socks_construct(
                SOCKS_DEFAULT_GROUP,
                remark,
                server,
                port,
                username,
                password,
                None,
                None,
                None,
                "",
            );

            true
        }
        "HTTP" | "HTTPS" => {
            let username = json.get("Username").and_then(Value::as_str).unwrap_or("");
            let password = json.get("Password").and_then(Value::as_str).unwrap_or("");
            let is_https = type_str == "HTTPS";

            *node = Proxy::http_construct(
                HTTP_DEFAULT_GROUP,
                remark,
                server,
                port,
                username,
                password,
                is_https,
                None,
                None,
                None,
                "",
            );

            true
        }
        "Trojan" => {
            let password = json.get("Password").and_then(Value::as_str).unwrap_or("");
            if password.is_empty() {
                return false;
            }

            let sni = json
                .get("Host")
                .and_then(Value::as_str)
                .map(|s| s.to_string());

            *node = Proxy::trojan_construct(
                TROJAN_DEFAULT_GROUP.to_string(),
                remark.to_string(),
                server.to_string(),
                port,
                password.to_string(),
                None,
                sni,
                None,
                true,
                None,
                None,
                None,
                None,
                None,
            );

            true
        }
        "VMess" => {
            let uuid = json.get("UserID").and_then(Value::as_str).unwrap_or("");
            if uuid.is_empty() {
                return false;
            }

            let alter_id = json.get("AlterID").and_then(Value::as_u64).unwrap_or(0) as u16;
            let network = json
                .get("TransferProtocol")
                .and_then(Value::as_str)
                .unwrap_or("tcp");
            let security = json
                .get("EncryptMethod")
                .and_then(Value::as_str)
                .unwrap_or("auto");
            let tls = json
                .get("TLSSecure")
                .and_then(Value::as_bool)
                .unwrap_or(false);

            let host = json.get("Host").and_then(Value::as_str).unwrap_or("");
            let path = json.get("Path").and_then(Value::as_str).unwrap_or("/");
            let sni = json.get("ServerName").and_then(Value::as_str).unwrap_or("");

            *node = Proxy::vmess_construct(
                &V2RAY_DEFAULT_GROUP.to_string(),
                &remark.to_string(),
                &server.to_string(),
                port,
                "",
                &uuid.to_string(),
                alter_id,
                &network.to_string(),
                &security.to_string(),
                &path.to_string(),
                &host.to_string(),
                "",
                if tls { "tls" } else { "" },
                &sni.to_string(),
                None,
                None,
                None,
                None,
                "",
            );

            true
        }
        _ => false,
    }
}

/// Parse a Netch configuration file into a vector of Proxy objects
pub fn explode_netch_conf(content: &str, nodes: &mut Vec<Proxy>) -> bool {
    // Parse JSON
    let json: Value = match serde_json::from_str(content) {
        Ok(j) => j,
        Err(_) => return false,
    };

    // Check if it's an array (multiple servers)
    if let Some(servers) = json.as_array() {
        let mut success = false;

        // Process each server
        for server in servers {
            let server_str = match serde_json::to_string(server) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let mut node = Proxy::default();
            if explode_netch(&server_str, &mut node) {
                nodes.push(node);
                success = true;
            }
        }

        return success;
    }

    // If not an array, try as a single server
    let mut node = Proxy::default();
    if explode_netch(content, &mut node) {
        nodes.push(node);
        return true;
    }

    false
}
