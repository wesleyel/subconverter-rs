use std::env;
use subconverter_rs::{
    models::Proxy,
    parser::parse_settings::{ParseSettings, RegexMatchConfig, RegexMatchConfigs},
    parser::subparser::add_nodes,
};

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <url_or_subscription_link>", args[0]);
        println!("Examples:");
        println!("  {} vmess://eyJ2IjogIjIiLCAicHMiOiAiSEstQSIsICJhZGQiOiAiZXhhbXBsZS5jb20iLCAicG9ydCI6IDQ0MywgImlkIjogIjEyMzQ1Njc4LTEyMzQtMTIzNC0xMjM0LTEyMzQ1Njc4OTAxMiIsICJhaWQiOiAwLCAibmV0IjogIndzIiwgInR5cGUiOiAibm9uZSIsICJob3N0IjogImV4YW1wbGUuY29tIiwgInBhdGgiOiAiL3BhdGgiLCAidGxzIjogInRscyJ9", args[0]);
        println!("  {} https://example.com/subscription_link", args[0]);
        return;
    }

    let url = &args[1];
    println!("Parsing URL: {}", url);

    // Create a vector to store the nodes
    let mut all_nodes: Vec<Proxy> = Vec::new();

    // Create basic parse settings
    let mut parse_settings = ParseSettings::default();

    // Add some example stream and time rules for subscription info extraction
    let mut stream_rules = RegexMatchConfigs::new();
    stream_rules.push(RegexMatchConfig {
        match_pattern: r"剩余流量：(.*?)".to_string(),
        replace: "left=$1".to_string(),
        script: "".to_string(),
    });

    let mut time_rules = RegexMatchConfigs::new();
    time_rules.push(RegexMatchConfig {
        match_pattern: r"过期时间：(.*?)".to_string(),
        replace: "expire=$1".to_string(),
        script: "".to_string(),
    });

    // Set up the parse settings
    parse_settings.stream_rules = Some(stream_rules);
    parse_settings.time_rules = Some(time_rules);
    parse_settings.authorized = true; // Allow local file access

    // Optional: Add filtering
    if args.len() > 2 {
        let include_pattern = &args[2];
        println!("Using include filter: {}", include_pattern);

        let mut include_remarks = Vec::new();
        include_remarks.push(include_pattern.clone());
        parse_settings.include_remarks = Some(include_remarks);
    }

    if args.len() > 3 {
        let exclude_pattern = &args[3];
        println!("Using exclude filter: {}", exclude_pattern);

        let mut exclude_remarks = Vec::new();
        exclude_remarks.push(exclude_pattern.clone());
        parse_settings.exclude_remarks = Some(exclude_remarks);
    }

    // Parse the URL
    match add_nodes(url.to_string(), &mut all_nodes, 0, &parse_settings) {
        Ok(_) => {
            if all_nodes.is_empty() {
                println!("No nodes were successfully parsed from the URL.");
            } else {
                println!(
                    "Successfully parsed {} nodes from the URL.",
                    all_nodes.len()
                );
                print_nodes(&all_nodes);
            }
        }
        Err(e) => println!("Error parsing URL: {}", e),
    }
}

fn print_nodes(nodes: &[Proxy]) {
    if !nodes.is_empty() {
        println!("Parsed {} node(s):", nodes.len());
        for (i, node) in nodes.iter().enumerate() {
            println!(
                "Node #{}: {} - {}",
                i + 1,
                node.proxy_type.to_string(),
                node.remark
            );
            println!("  Server: {}:{}", node.hostname, node.port);

            // Handle Option<String> fields
            if let Some(method) = &node.encrypt_method {
                println!("  Method: {}", method);
            }

            if let Some(password) = &node.password {
                println!("  Password: {}", password);
            }

            if let Some(plugin) = &node.plugin {
                if !plugin.is_empty() {
                    println!("  Plugin: {}", plugin);
                }
            }

            if let Some(plugin_opts) = &node.plugin_option {
                if !plugin_opts.is_empty() {
                    println!("  Plugin Options: {}", plugin_opts);
                }
            }

            // Extra information for different proxy types
            match node.proxy_type.to_string() {
                "VMess" => {
                    if let Some(id) = &node.user_id {
                        println!("  User ID: {}", id);
                    }
                    if let Some(protocol) = &node.transfer_protocol {
                        println!("  Protocol: {}", protocol);
                    }
                }
                "Trojan" => {
                    if let Some(sni) = &node.sni {
                        println!("  SNI: {}", sni);
                    }
                }
                _ => {}
            }

            if let Some(udp) = node.udp {
                println!("  UDP: {}", udp);
            }

            if node.tls_secure {
                println!("  TLS: enabled");
            }

            println!(); // Add a blank line between nodes
        }
    } else {
        println!("No nodes were found.");
    }
}
