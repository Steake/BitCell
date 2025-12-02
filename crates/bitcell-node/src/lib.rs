//! BitCell node implementation
//!
//! Implements miner, validator, and light client nodes

pub mod rpc;
pub mod ws;
pub mod config;
pub mod validator;
pub mod miner;
pub mod monitoring;
pub mod blockchain;
pub mod tx_pool;
pub mod tournament;
pub mod network;
pub mod dht;
pub mod keys;

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

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Network(e.to_string())
    }
}

impl From<libp2p::TransportError<std::io::Error>> for Error {
    fn from(e: libp2p::TransportError<std::io::Error>) -> Self {
        Error::Network(e.to_string())
    }
}

impl From<libp2p::gossipsub::SubscriptionError> for Error {
    fn from(e: libp2p::gossipsub::SubscriptionError) -> Self {
        Error::Network(e.to_string())
    }
}

impl From<libp2p::gossipsub::PublishError> for Error {
    fn from(e: libp2p::gossipsub::PublishError) -> Self {
        Error::Network(e.to_string())
    }
}

impl From<libp2p::multiaddr::Error> for Error {
    fn from(e: libp2p::multiaddr::Error) -> Self {
        Error::Network(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_imports() {
        // Smoke test
    }
}
