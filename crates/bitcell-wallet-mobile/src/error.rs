//! Error types for the mobile wallet SDK

use std::fmt;

/// Result type for mobile wallet operations
pub type Result<T> = std::result::Result<T, MobileWalletError>;

/// Mobile wallet error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum MobileWalletError {
    #[error("Invalid mnemonic phrase")]
    InvalidMnemonic,
    
    #[error("Invalid password")]
    InvalidPassword,
    
    #[error("Wallet is locked")]
    WalletLocked,
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Invalid address")]
    InvalidAddress,
    
    #[error("Transaction error")]
    TransactionError,
    
    #[error("Signing error")]
    SigningError,
    
    #[error("Storage error")]
    StorageError,
    
    #[error("Biometric authentication error")]
    BiometricError,
    
    #[error("Backup error")]
    BackupError,
    
    #[error("Cryptography error")]
    CryptoError,
    
    #[error("Serialization error")]
    SerializationError,
    
    #[error("Feature not yet implemented")]
    NotImplemented,
}

impl From<bitcell_wallet::Error> for MobileWalletError {
    fn from(e: bitcell_wallet::Error) -> Self {
        match e {
            bitcell_wallet::Error::InvalidMnemonic(_) => MobileWalletError::InvalidMnemonic,
            bitcell_wallet::Error::InvalidAddress(_) => MobileWalletError::InvalidAddress,
            bitcell_wallet::Error::InsufficientBalance { .. } => MobileWalletError::InsufficientBalance,
            bitcell_wallet::Error::TransactionError(_) => MobileWalletError::TransactionError,
            bitcell_wallet::Error::SigningError(_) => MobileWalletError::SigningError,
            bitcell_wallet::Error::WalletLocked => MobileWalletError::WalletLocked,
            bitcell_wallet::Error::Crypto(_) => MobileWalletError::CryptoError,
            bitcell_wallet::Error::Serialization(_) => MobileWalletError::SerializationError,
            _ => MobileWalletError::CryptoError,
        }
    }
}

impl From<serde_json::Error> for MobileWalletError {
    fn from(_: serde_json::Error) -> Self {
        MobileWalletError::SerializationError
    }
}
