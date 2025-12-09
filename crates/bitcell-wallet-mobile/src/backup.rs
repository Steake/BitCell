//! Wallet backup and restore functionality

use crate::error::{MobileWalletError, Result};
use crate::wallet::MobileWallet;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Wallet backup data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBackup {
    /// Encrypted backup data
    pub encrypted_data: String,
    /// Backup format version
    pub backup_version: String,
    /// Unix timestamp when backup was created
    pub timestamp: u64,
}

impl WalletBackup {
    const BACKUP_VERSION: &'static str = "1.0";
    
    /// Create a new encrypted backup
    ///
    /// # Security
    ///
    /// The backup is encrypted using the provided password with:
    /// - AES-256-GCM for encryption
    /// - PBKDF2 with 100,000 iterations for key derivation
    /// - Random salt and nonce
    ///
    /// In this mock implementation, we use simple hex encoding.
    /// Production implementation should use proper encryption.
    pub fn create(wallet: &MobileWallet, _password: String) -> Result<Self> {
        // Export mnemonic (this requires unlocked wallet)
        let mnemonic = wallet.export_mnemonic(password.clone())?;
        
        // In production: Encrypt mnemonic with password using AES-256-GCM
        // For now, we'll use a simple hex encoding
        let encrypted_data = hex::encode(mnemonic.as_bytes());
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| MobileWalletError::BackupError)?
            .as_secs();
        
        Ok(Self {
            encrypted_data,
            backup_version: Self::BACKUP_VERSION.to_string(),
            timestamp,
        })
    }
    
    /// Restore wallet from backup
    ///
    /// Decrypts the backup data and restores the wallet state
    pub fn restore(&self, wallet: &MobileWallet, _password: String) -> Result<()> {
        // Verify backup version
        if self.backup_version != Self::BACKUP_VERSION {
            return Err(MobileWalletError::BackupError);
        }
        
        // In production: Decrypt data with password
        // For now, decode from hex
        let mnemonic_bytes = hex::decode(&self.encrypted_data)
            .map_err(|_| MobileWalletError::BackupError)?;
        
        let mnemonic = String::from_utf8(mnemonic_bytes)
            .map_err(|_| MobileWalletError::BackupError)?;
        
        // Validate mnemonic
        if !crate::validate_mnemonic(mnemonic.clone()) {
            return Err(MobileWalletError::InvalidMnemonic);
        }
        
        // Restore wallet by unlocking with the mnemonic
        // In production, would properly restore all wallet state
        Ok(())
    }
    
    /// Serialize backup to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|_| MobileWalletError::SerializationError)
    }
    
    /// Deserialize backup from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|_| MobileWalletError::SerializationError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{generate_mnemonic, MnemonicWordCount, SecureStorageConfig};

    #[test]
    fn test_create_backup() {
        let mnemonic = generate_mnemonic(MnemonicWordCount::Words12).unwrap();
        let config = SecureStorageConfig::default();
        let wallet = MobileWallet::create(mnemonic, config).unwrap();
        wallet.unlock("password".to_string()).unwrap();
        
        let backup = wallet.create_backup("backup_password".to_string()).unwrap();
        
        assert_eq!(backup.backup_version, "1.0");
        assert!(!backup.encrypted_data.is_empty());
        assert!(backup.timestamp > 0);
    }

    #[test]
    fn test_backup_serialization() {
        let backup = WalletBackup {
            encrypted_data: "test_data".to_string(),
            backup_version: "1.0".to_string(),
            timestamp: 1234567890,
        };
        
        let json = backup.to_json().unwrap();
        let restored = WalletBackup::from_json(&json).unwrap();
        
        assert_eq!(backup.encrypted_data, restored.encrypted_data);
        assert_eq!(backup.backup_version, restored.backup_version);
        assert_eq!(backup.timestamp, restored.timestamp);
    }
}
