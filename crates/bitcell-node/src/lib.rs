//! BitCell node implementation
//!
//! Implements miner, validator, and light client nodes

pub mod config;
pub mod validator;
pub mod miner;
pub mod monitoring;

pub use config::NodeConfig;
pub use validator::ValidatorNode;
pub use miner::MinerNode;
pub use monitoring::{MetricsRegistry, logging};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Node error: {0}")]
    Node(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_imports() {
        // Smoke test
    }
}
