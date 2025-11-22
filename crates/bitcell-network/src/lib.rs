//! P2P networking layer
//!
//! Handles peer discovery, message propagation, and block relay.
//! v0.1 provides message structures; full libp2p integration in v0.2.

pub mod messages;
pub mod peer;

pub use messages::{Message, MessageType};
pub use peer::{Peer, PeerManager};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Peer error: {0}")]
    Peer(String),
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_imports() {
        // Smoke test
    }
}
