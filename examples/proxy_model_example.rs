//! Example demonstrating the correct way to use the Proxy model
//!
//! This example shows how to create, manipulate, and use Proxy instances
//! according to the latest architecture.

use subconverter_rs::models::{Proxy, ProxyType};
// You can also use the re-exports from the crate root:
// use subconverter_rs::{Proxy, ProxyType};

fn main() {
    // Create a new proxy instance
    let mut proxy = Proxy::default();

    // Set basic properties
    proxy.proxy_type = ProxyType::VMess;
    proxy.hostname = "example.com".to_string();
    proxy.port = 443;
    proxy.remark = "Example VMess Server".to_string();

    // Set optional properties
    proxy.user_id = Some("b831381d-6324-4d53-ad4f-8cda48b30811".to_string());
    proxy.encrypt_method = Some("auto".to_string());
    proxy.transfer_protocol = Some("ws".to_string());
    proxy.tls_secure = true;

    // Demonstrate the correct way to work with Option fields

    // Bad: This would cause a compilation error because of Option<String>
    // if !proxy.user_id.is_empty() { /* ... */ }

    // Good: Check if an Option<String> field is Some and not empty
    if proxy.user_id.as_ref().map_or(false, |s| !s.is_empty()) {
        println!("User ID: {}", proxy.user_id.as_ref().unwrap());
    }

    // Good: Providing a default for an Option<String>
    let protocol = proxy.transfer_protocol.as_deref().unwrap_or("tcp");
    println!("Protocol: {}", protocol);

    // Demonstrate the correct way to check proxy types
    match proxy.proxy_type {
        ProxyType::VMess => println!("This is a VMess proxy"),
        ProxyType::Shadowsocks => println!("This is a Shadowsocks proxy"),
        ProxyType::ShadowsocksR => println!("This is a ShadowsocksR proxy"),
        _ => println!("This is another type of proxy"),
    }

    // Print the proxy details
    println!(
        "Proxy: {} ({}:{})",
        proxy.remark, proxy.hostname, proxy.port
    );
    println!("Type: {}", proxy.proxy_type.to_string());

    // Create a vec of proxies
    let mut proxies = Vec::new();
    proxies.push(proxy);

    // Add another proxy
    let mut ss_proxy = Proxy::default();
    ss_proxy.proxy_type = ProxyType::Shadowsocks;
    ss_proxy.hostname = "ss.example.com".to_string();
    ss_proxy.port = 8388;
    ss_proxy.remark = "Example SS Server".to_string();
    ss_proxy.encrypt_method = Some("aes-256-gcm".to_string());
    ss_proxy.password = Some("password123".to_string());
    proxies.push(ss_proxy);

    // Process the proxies
    for proxy in &proxies {
        println!(
            "\nProxy: {} ({})",
            proxy.remark,
            proxy.proxy_type.to_string()
        );
        println!("Address: {}:{}", proxy.hostname, proxy.port);

        // Handle encryption method properly with Option
        if let Some(method) = &proxy.encrypt_method {
            println!("Encryption: {}", method);
        }

        // Handle password properly with Option
        if let Some(pass) = &proxy.password {
            println!("Password: {}", pass);
        }
    }
}
