//! BitCell Mobile Wallet SDK
//!
//! Cross-platform wallet SDK with secure key management for iOS and Android.
//!
//! # Features
//!
//! - BIP39 mnemonic generation and recovery
//! - Secure key storage (iOS Keychain / Android Keystore)
//! - Biometric authentication support
//! - Transaction signing
//! - Backup and restore functionality
//! - FFI bindings for Swift and Kotlin
//!
//! # Platform Integration
//!
//! ## iOS
//! The SDK generates Swift bindings that integrate with iOS Keychain for secure storage
//! and FaceID/TouchID for biometric authentication.
//!
//! ## Android
//! The SDK generates Kotlin bindings that integrate with Android Keystore and
//! BiometricPrompt API for secure storage and biometric authentication.

mod error;
mod storage;
mod biometric;
mod wallet;
mod backup;

pub use error::{MobileWalletError, Result};
pub use storage::{SecureKeyStorage, SecureStorageConfig};
pub use biometric::{BiometricAuthProvider, BiometricResult};
pub use wallet::{MobileWallet, WalletLockState};
pub use backup::WalletBackup;

use bitcell_wallet::{Mnemonic, mnemonic::WordCount};

// Re-export types needed for UniFFI
pub use wallet::{AccountInfo, TransactionDetails, SignedTransactionResult};

/// Mnemonic word count options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MnemonicWordCount {
    /// 12-word mnemonic (128 bits entropy)
    Words12,
    /// 18-word mnemonic (192 bits entropy)
    Words18,
    /// 24-word mnemonic (256 bits entropy)
    Words24,
}

impl MnemonicWordCount {
    fn to_word_count(self) -> WordCount {
        match self {
            MnemonicWordCount::Words12 => WordCount::Words12,
            MnemonicWordCount::Words18 => WordCount::Words18,
            MnemonicWordCount::Words24 => WordCount::Words24,
        }
    }
}

/// Create a new wallet with a mnemonic phrase
pub fn create_wallet(
    mnemonic_phrase: String,
    storage_config: SecureStorageConfig,
) -> Result<MobileWallet> {
    MobileWallet::create(mnemonic_phrase, storage_config)
}

/// Restore a wallet from a mnemonic phrase
pub fn restore_wallet(
    mnemonic_phrase: String,
    storage_config: SecureStorageConfig,
) -> Result<MobileWallet> {
    MobileWallet::restore(mnemonic_phrase, storage_config)
}

/// Generate a new mnemonic phrase
pub fn generate_mnemonic(word_count: MnemonicWordCount) -> Result<String> {
    let mnemonic = Mnemonic::generate(word_count.to_word_count());
    Ok(mnemonic.phrase().to_string())
}

/// Validate a mnemonic phrase
pub fn validate_mnemonic(mnemonic_phrase: String) -> bool {
    Mnemonic::from_phrase(&mnemonic_phrase).is_ok()
}

// UniFFI scaffolding generation
uniffi::include_scaffolding!("bitcell_wallet_mobile");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mnemonic_12_words() {
        let mnemonic = generate_mnemonic(MnemonicWordCount::Words12).unwrap();
        assert_eq!(mnemonic.split_whitespace().count(), 12);
        assert!(validate_mnemonic(mnemonic));
    }

    #[test]
    fn test_generate_mnemonic_24_words() {
        let mnemonic = generate_mnemonic(MnemonicWordCount::Words24).unwrap();
        assert_eq!(mnemonic.split_whitespace().count(), 24);
        assert!(validate_mnemonic(mnemonic));
    }

    #[test]
    fn test_validate_mnemonic_valid() {
        let mnemonic = generate_mnemonic(MnemonicWordCount::Words12).unwrap();
        assert!(validate_mnemonic(mnemonic));
    }

    #[test]
    fn test_validate_mnemonic_invalid() {
        assert!(!validate_mnemonic("invalid mnemonic phrase".to_string()));
    }
}
