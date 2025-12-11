//! BitCell Light Client
//!
//! Lightweight client implementation for resource-constrained devices.
//!
//! Features:
//! - Header-only sync with checkpoint support
//! - Merkle proof verification for state queries
//! - Balance queries via state proofs
//! - Transaction submission without full state
//! - <100MB resource usage
//!
//! # Architecture
//!
//! The light client maintains only block headers and verifies state via
//! Merkle proofs obtained from full nodes. This allows wallet functionality
//! on devices with limited storage and bandwidth.
//!
//! ## Components
//!
//! - `header_chain`: Header-only blockchain with checkpoint support
//! - `sync`: Header synchronization protocol
//! - `proofs`: Merkle proof verification
//! - `wallet`: Wallet mode for balance queries and transactions
//! - `protocol`: Light client network protocol

pub mod header_chain;
pub mod sync;
pub mod proofs;
pub mod wallet;
pub mod protocol;
pub mod checkpoints;

pub use header_chain::{HeaderChain, HeaderChainConfig};
pub use sync::{HeaderSync, SyncStatus};
pub use proofs::{StateProof, StateProofRequest};
pub use wallet::{LightWallet, WalletMode};
pub use protocol::{LightClientMessage, LightClientProtocol};
pub use checkpoints::{Checkpoint, CheckpointManager};

/// Standard result type for light client operations
pub type Result<T> = std::result::Result<T, Error>;

/// Light client errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    #[error("Invalid checkpoint: {0}")]
    InvalidCheckpoint(String),

    #[error("Invalid proof: {0}")]
    InvalidProof(String),

    #[error("Sync error: {0}")]
    SyncError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("State error: {0}")]
    StateError(String),

    #[error("Wallet error: {0}")]
    WalletError(String),

    #[error("Consensus error: {0}")]
    ConsensusError(#[from] bitcell_consensus::Error),

    #[error("Crypto error: {0}")]
    CryptoError(#[from] bitcell_crypto::Error),

    #[error("Network layer error: {0}")]
    NetworkLayerError(#[from] bitcell_network::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Error::SerializationError(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_imports() {
        // Smoke test to ensure all modules compile
    }
}
