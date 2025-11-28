//! BitCell Wallet
//!
//! Modular wallet implementation for BitCell blockchain with multi-chain support.
//!
//! Features:
//! - Wallet creation and key management
//! - BIP39 seed phrase generation and recovery
//! - Address generation and management
//! - Transaction handling (signing, creation)
//! - Balance display
//! - Transaction history
//!
//! Supports:
//! - BitCell native blockchain
//! - Bitcoin (basic structures)
//! - Ethereum (basic structures)

pub mod address;
pub mod balance;
pub mod chain;
pub mod history;
pub mod mnemonic;
pub mod transaction;
pub mod wallet;

pub use address::{Address, AddressType};
pub use balance::Balance;
pub use chain::{Chain, ChainConfig};
pub use history::{TransactionRecord, TransactionHistory};
pub use mnemonic::Mnemonic;
pub use transaction::{Transaction, TransactionBuilder, SignedTransaction};
pub use wallet::{Wallet, WalletConfig};

/// Standard result type for wallet operations
pub type Result<T> = std::result::Result<T, Error>;

/// Wallet errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),

    #[error("Invalid derivation path: {0}")]
    InvalidDerivationPath(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: u64, need: u64 },

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Signing error: {0}")]
    SigningError(String),

    #[error("Chain not supported: {0}")]
    UnsupportedChain(String),

    #[error("Wallet locked")]
    WalletLocked,

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("IO error: {0}")]
    Io(String),
}

impl From<bitcell_crypto::Error> for Error {
    fn from(e: bitcell_crypto::Error) -> Self {
        Error::Crypto(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_imports() {
        // Smoke test to ensure all modules compile
        let _ = Chain::BitCell;
    }
}
