//! P2P networking layer (Legacy - see deprecation notice)
//!
//! # Deprecation Notice
//! 
//! This crate (`bitcell-network`) provides a simplified/stub networking interface.
//! The actual production networking implementation is in:
//! - `bitcell-node/src/network.rs` - TCP-based P2P with real connections
//! - `bitcell-node/src/dht.rs` - libp2p Gossipsub integration
//!
//! This crate is maintained for:
//! 1. Type definitions used across the codebase (Message, PeerInfo, etc.)
//! 2. Trait definitions for network abstractions
//! 3. Testing and mock implementations
//!
//! For production networking, use the implementations in `bitcell-node`.
//!
//! # Future Plans
//! This crate may be refactored to provide only interfaces/traits, with the
//! actual implementations living in `bitcell-node`.

pub mod messages;
pub mod peer;

// Full libp2p transport integration (stub - see deprecation notice above)
pub mod transport;

pub use messages::{Message, MessageType};
pub use peer::{PeerInfo, PeerManager, PeerReputation};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Peer error: {0}")]
    Peer(String),
    
    #[error("Transport error: {0}")]
    Transport(String),
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_imports() {
        // Smoke test
    }
}
