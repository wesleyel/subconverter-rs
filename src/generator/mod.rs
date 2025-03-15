pub mod config;
pub mod node_manip;

// Re-export common types
pub use config::subexport::{
    ExtraSettings, ProxyGroupConfig, ProxyGroupConfigs, RegexMatchConfig, RegexMatchConfigs,
};

// Re-export format converters
pub use config::formats::clash::{proxy_to_clash, proxy_to_clash_yaml};
pub use config::formats::loon::proxy_to_loon;
pub use config::formats::mellow::{proxy_to_mellow, proxy_to_mellow_ini};
pub use config::formats::quan::{proxy_to_quan, proxy_to_quan_ini};
pub use config::formats::quanx::{proxy_to_quan_x, proxy_to_quan_x_ini};
pub use config::formats::singbox::proxy_to_sing_box;
pub use config::formats::single::proxy_to_single;
pub use config::formats::ss_sub::proxy_to_ss_sub;
pub use config::formats::ssd::proxy_to_ssd;
pub use config::formats::surge::proxy_to_surge;

// Re-export node manipulation functions
pub use node_manip::{
    add_emoji, add_nodes, check_ignore, explode, explode_conf, explode_conf_content, filter_nodes,
    node_rename, preprocess_nodes, remove_emoji, ParseSettings,
};
