//! BitCell node implementation
//!
//! Implements miner, validator, and light client nodes

pub mod config;
pub mod validator;
pub mod miner;
pub mod monitoring;
pub mod blockchain;
pub mod tx_pool;
pub mod tournament;
pub mod network;

pub use config::NodeConfig;
pub use validator::ValidatorNode;
pub use miner::MinerNode;
pub use monitoring::{MetricsRegistry, logging};
pub use blockchain::Blockchain;
pub use tx_pool::TransactionPool;
pub use tournament::TournamentManager;
pub use network::NetworkManager;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Node error: {0}")]
    Node(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Network error: {0}")]
    Network(String),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Network(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Network(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_imports() {
        // Smoke test
    }
}
