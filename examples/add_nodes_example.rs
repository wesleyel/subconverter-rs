use subconverter_rs::{
    models::Proxy,
    parser::parse_settings::{ParseSettings, RegexMatchConfig, RegexMatchConfigs},
    parser::subparser::add_nodes,
};

fn main() {
    println!("Demonstrating add_nodes functionality");

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

    // Example 1: Add nodes from a sample direct link
    println!("\nExample 1: Adding a single VMess node from direct link");
    let vmess_node = "vmess://eyJ2IjogIjIiLCAicHMiOiAiSEstQSIsICJhZGQiOiAiZXhhbXBsZS5jb20iLCAicG9ydCI6IDQ0MywgImlkIjogIjEyMzQ1Njc4LTEyMzQtMTIzNC0xMjM0LTEyMzQ1Njc4OTAxMiIsICJhaWQiOiAwLCAibmV0IjogIndzIiwgInR5cGUiOiAibm9uZSIsICJob3N0IjogImV4YW1wbGUuY29tIiwgInBhdGgiOiAiL3BhdGgiLCAidGxzIjogInRscyJ9";

    match add_nodes(vmess_node.to_string(), &mut all_nodes, 1, &parse_settings) {
        Ok(_) => println!("Successfully added VMess node"),
        Err(e) => println!("Error adding VMess node: {}", e),
    }

    // Print the nodes
    print_nodes(&all_nodes);

    // Example 2: Adding a single SS node
    println!("\nExample 2: Adding a single Shadowsocks node");
    all_nodes.clear();

    let ss_node = "ss://YWVzLTI1Ni1nY206cGFzc3dvcmQxMjM=@example.com:8388#SS-Demo";

    match add_nodes(ss_node.to_string(), &mut all_nodes, 2, &parse_settings) {
        Ok(_) => println!("Successfully added SS node"),
        Err(e) => println!("Error adding SS node: {}", e),
    }

    // Print the nodes
    print_nodes(&all_nodes);

    // Example 3: Adding a Trojan node
    println!("\nExample 3: Adding a Trojan node");
    all_nodes.clear();

    let trojan_node =
        "trojan://password123@example.com:443?sni=example.com&allowInsecure=1#Trojan-Demo";

    match add_nodes(trojan_node.to_string(), &mut all_nodes, 3, &parse_settings) {
        Ok(_) => println!("Successfully added Trojan node"),
        Err(e) => println!("Error adding Trojan node: {}", e),
    }

    // Print the nodes
    print_nodes(&all_nodes);

    // Example 4: Test filtering with include/exclude patterns
    println!("\nExample 4: Testing include/exclude patterns");
    all_nodes.clear();

    // Define our filter patterns
    let mut include_remarks = Vec::new();
    include_remarks.push("HK|香港|Hong Kong".to_string());

    let mut exclude_remarks = Vec::new();
    exclude_remarks.push("Test|测试".to_string());

    parse_settings.include_remarks = Some(include_remarks);
    parse_settings.exclude_remarks = Some(exclude_remarks);

    // Add HK, Test, and JP nodes
    let all_vmess_nodes = [
        ("HK-Server", "vmess://eyJ2IjogIjIiLCAicHMiOiAiSEstU2VydmVyIiwgImFkZCI6ICJoay5leGFtcGxlLmNvbSIsICJwb3J0IjogNDQzLCAiaWQiOiAiMTIzNDU2NzgtMTIzNC0xMjM0LTEyMzQtMTIzNDU2Nzg5MDEyIiwgImFpZCI6IDAsICJuZXQiOiAid3MiLCAidHlwZSI6ICJub25lIiwgImhvc3QiOiAiaGsuZXhhbXBsZS5jb20iLCAicGF0aCI6ICIvcGF0aCIsICJ0bHMiOiAidGxzIn0="),
        ("Test-Server", "vmess://eyJ2IjogIjIiLCAicHMiOiAiVGVzdC1TZXJ2ZXIiLCAiYWRkIjogInRlc3QuZXhhbXBsZS5jb20iLCAicG9ydCI6IDQ0MywgImlkIjogIjEyMzQ1Njc4LTEyMzQtMTIzNC0xMjM0LTEyMzQ1Njc4OTAxMiIsICJhaWQiOiAwLCAibmV0IjogIndzIiwgInR5cGUiOiAibm9uZSIsICJob3N0IjogInRlc3QuZXhhbXBsZS5jb20iLCAicGF0aCI6ICIvcGF0aCIsICJ0bHMiOiAidGxzIn0="),
        ("JP-Server", "vmess://eyJ2IjogIjIiLCAicHMiOiAiSlAtU2VydmVyIiwgImFkZCI6ICJqcC5leGFtcGxlLmNvbSIsICJwb3J0IjogNDQzLCAiaWQiOiAiMTIzNDU2NzgtMTIzNC0xMjM0LTEyMzQtMTIzNDU2Nzg5MDEyIiwgImFpZCI6IDAsICJuZXQiOiAid3MiLCAidHlwZSI6ICJub25lIiwgImhvc3QiOiAianAuZXhhbXBsZS5jb20iLCAicGF0aCI6ICIvcGF0aCIsICJ0bHMiOiAidGxzIn0=")
    ];

    // Add each node
    for (name, vmess) in all_vmess_nodes.iter() {
        match add_nodes(vmess.to_string(), &mut all_nodes, 4, &parse_settings) {
            Ok(_) => println!("Successfully added {} node", name),
            Err(e) => println!("Error adding {} node: {}", name, e),
        }
    }

    // Print the nodes after filtering
    println!("\nAfter filtering, should only see nodes matching 'HK|香港|Hong Kong' pattern but not 'Test|测试':");
    print_nodes(&all_nodes);

    // Now let's try one more test with a simpler pattern for clarity
    println!("\nExample 5: Testing simpler filtering pattern");
    all_nodes.clear();

    // Define simpler filter patterns
    let mut include_remarks = Vec::new();
    include_remarks.push("HK".to_string()); // Just match "HK" exactly

    let mut exclude_remarks = Vec::new();
    exclude_remarks.push("Test".to_string()); // Just match "Test" exactly

    parse_settings.include_remarks = Some(include_remarks);
    parse_settings.exclude_remarks = Some(exclude_remarks);

    // Add each node again
    for (name, vmess) in all_vmess_nodes.iter() {
        match add_nodes(vmess.to_string(), &mut all_nodes, 5, &parse_settings) {
            Ok(_) => println!("Successfully added {} node", name),
            Err(e) => println!("Error adding {} node: {}", name, e),
        }
    }

    // Print the nodes after filtering with simpler pattern
    println!("\nAfter simple filtering, should only see nodes containing 'HK' but not 'Test':");
    print_nodes(&all_nodes);

    // Example 6: What happens with an invalid link
    println!("\nExample 6: What happens with an invalid link");
    all_nodes.clear();
    parse_settings.include_remarks = None;
    parse_settings.exclude_remarks = None;

    match add_nodes(
        "invalid://link".to_string(),
        &mut all_nodes,
        6,
        &parse_settings,
    ) {
        Ok(_) => println!("Successfully added nodes from invalid link (unexpected)"),
        Err(e) => println!("Expected error: {}", e),
    }
}

fn print_nodes(nodes: &[Proxy]) {
    if !nodes.is_empty() {
        println!("Added {} node(s):", nodes.len());
        for node in nodes {
            println!("  Node: {} - {}", node.proxy_type.to_string(), node.remark);
            println!("    Server: {}:{}", node.hostname, node.port);

            // Handle Option<String> fields
            if let Some(plugin) = &node.plugin {
                if !plugin.is_empty() {
                    println!("    Plugin: {}", plugin);
                }
            }

            if let Some(plugin_opts) = &node.plugin_option {
                if !plugin_opts.is_empty() {
                    println!("    Plugin Options: {}", plugin_opts);
                }
            }

            if let Some(method) = &node.encrypt_method {
                println!("    Method: {}", method);
            }
        }
    } else {
        println!("No nodes were added.");
    }
}
