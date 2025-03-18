pub mod config;
pub mod ruleconvert;

// Re-export common types
pub use config::subexport::{
    ExtraSettings, ProxyGroupConfig, ProxyGroupConfigs, RegexMatchConfig, RegexMatchConfigs,
};
// Re-export rule conversion functions
pub use ruleconvert::{
    convert_ruleset, ruleset_to_clash_str, ruleset_to_sing_box, ruleset_to_surge,
};

// Re-export node manipulation functions
// pub use node_manip::{
//     add_emoji, add_nodes, check_ignore, explode, explode_conf, explode_conf_content, filter_nodes,
//     node_rename, preprocess_nodes, remove_emoji, ParseSettings,
// };
