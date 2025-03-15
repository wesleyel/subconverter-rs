use subconverter_rs::generator::config::proxy::Proxy;
use subconverter_rs::generator::{
    add_nodes, filter_nodes, node_rename, preprocess_nodes, ExtraSettings, ParseSettings,
    RegexMatchConfig,
};

fn main() {
    // Create a sample proxy node
    let mut node = Proxy::default();
    node.remark = "HK_Hong Kong".to_string();
    node.group = "Sample Group".to_string();

    // Create a vector of nodes
    let mut nodes = vec![node];

    // Create sample rename rules
    let rename_rules = vec![RegexMatchConfig {
        regex: "Hong Kong".to_string(),
        replacement: "HK".to_string(),
    }];

    // Create sample emoji rules
    let emoji_rules = vec![RegexMatchConfig {
        regex: "HK".to_string(),
        replacement: "🇭🇰".to_string(),
    }];

    // Create extra settings
    let mut ext = ExtraSettings::default();
    ext.rename_array = rename_rules;
    ext.emoji_array = emoji_rules;
    ext.add_emoji = true;

    // Process the nodes
    println!("Original node: {}", nodes[0].remark);

    // Apply rename
    node_rename(&mut nodes[0], &ext.rename_array, &ext);
    println!("After rename: {}", nodes[0].remark);

    // Apply full preprocessing
    preprocess_nodes(&mut nodes, &ext);
    println!("After preprocessing: {}", nodes[0].remark);

    // Try adding nodes from a link
    let mut all_nodes = Vec::new();
    let parse_settings = ParseSettings::default();

    // This is just a placeholder - in a real application, you would use actual proxy links
    match add_nodes("nullnode", &mut all_nodes, 1, &parse_settings) {
        Ok(_) => println!("Added nullnode successfully"),
        Err(e) => println!("Error adding node: {}", e),
    }

    // Filter nodes
    let exclude_remarks = vec!["exclude_pattern".to_string()];
    let include_remarks = vec!["include_pattern".to_string()];

    filter_nodes(&mut all_nodes, &exclude_remarks, &include_remarks, 1);

    println!("Total nodes after filtering: {}", all_nodes.len());
}
