use crate::generator::config::subexport::{process_remark, ExtraSettings};
use crate::parser::proxy::{Proxy, ProxyType};
use base64::{engine::general_purpose, Engine as _};

/// Convert proxies to SSSub format
///
/// This function converts a list of proxies to the SSSub configuration format,
/// which is a base64-encoded list of SS URIs.
///
/// # Arguments
/// * `nodes` - List of proxy nodes to convert
/// * `ext` - Extra settings for conversion
pub fn proxy_to_ss_sub(nodes: &mut Vec<Proxy>, ext: &mut ExtraSettings) -> String {
    let mut sub_content = String::new();

    for node in nodes.iter_mut() {
        // Skip non-SS nodes
        if node.proxy_type != ProxyType::Shadowsocks {
            continue;
        }

        // Process remark
        let mut remark = node.remark.clone();
        process_remark(&mut remark, ext, false);

        // Create SS URI
        let mut uri = String::new();

        // Add method and password
        let user_info = format!("{}:{}", node.cipher, node.password);
        let encoded_user_info = general_purpose::STANDARD.encode(user_info);

        // Format: ss://BASE64(method:password)@server:port/?plugin=plugin_data#remark
        uri.push_str(&format!(
            "ss://{}@{}:{}",
            encoded_user_info, node.server, node.port
        ));

        // Add plugin if present
        if !node.plugin.is_empty() && !node.plugin_opts.is_empty() {
            uri.push_str(&format!(
                "/?plugin={}",
                urlencoding::encode(&format!("{};{}", node.plugin, node.plugin_opts))
            ));
        }

        // Add remark
        uri.push_str(&format!("#{}", urlencoding::encode(&remark)));

        // Add to subscription content
        sub_content.push_str(&uri);
        sub_content.push('\n');
    }

    // Base64 encode the entire subscription
    general_purpose::STANDARD.encode(sub_content)
}
