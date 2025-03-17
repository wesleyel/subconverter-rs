// pub mod generator;
pub mod models;
pub mod parser;
pub mod utils;

// Re-export the main proxy types for easier access
pub use models::{Proxy, ProxyType};

// Re-export configuration types
pub use parser::types::ConfType;
