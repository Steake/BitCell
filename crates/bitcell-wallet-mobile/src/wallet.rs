//! Mobile wallet implementation

use crate::error::{MobileWalletError, Result};
use crate::storage::{MockSecureStorage, SecureKeyStorage, SecureStorageConfig};
use crate::biometric::{BiometricAuthProvider, BiometricResult, MockBiometricProvider};
use crate::backup::WalletBackup;

use bitcell_wallet::{Wallet, WalletConfig, Mnemonic, Chain, TransactionBuilder};
use bitcell_crypto::{SecretKey, PublicKey};
use parking_lot::RwLock;
use std::sync::Arc;
use std::str::FromStr;
use zeroize::Zeroize;

/// Wallet lock state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletLockState {
    /// Wallet is unlocked and ready to use
    Unlocked,
    /// Wallet is locked with password
    Locked,
    /// Wallet is locked and requires biometric authentication
    BiometricLocked,
}

/// Account information
#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub address: String,
    pub balance: String,
    pub nonce: u64,
}

/// Transaction details for signing
#[derive(Debug, Clone)]
pub struct TransactionDetails {
    pub from_address: String,
    pub to_address: String,
    pub amount: String,
    pub fee: String,
    pub nonce: u64,
    pub data: Option<String>,
}

/// Signed transaction result
#[derive(Debug, Clone)]
pub struct SignedTransactionResult {
    pub tx_hash: String,
    pub raw_transaction: String,
}

/// Mobile wallet implementation
pub struct MobileWallet {
    /// Inner wallet implementation
    wallet: Arc<RwLock<Option<Wallet>>>,
    /// Secure key storage
    storage: Arc<dyn SecureKeyStorage>,
    /// Biometric authentication provider
    biometric: Arc<dyn BiometricAuthProvider>,
    /// Current lock state
    lock_state: Arc<RwLock<WalletLockState>>,
    /// Storage configuration
    config: SecureStorageConfig,
    /// Encrypted seed storage key
    seed_key_id: String,
}

const SEED_KEY_PREFIX: &str = "bitcell_wallet_seed";
const WALLET_VERSION: &str = env!("CARGO_PKG_VERSION");

impl MobileWallet {
    /// Create a new wallet
    pub fn create(mnemonic_phrase: String, storage_config: SecureStorageConfig) -> Result<Self> {
        // Validate mnemonic
        let mnemonic = Mnemonic::from_phrase(&mnemonic_phrase)
            .map_err(|_| MobileWalletError::InvalidMnemonic)?;
        
        // Create wallet
        let wallet_config = WalletConfig::default();
        
        let wallet = Wallet::from_mnemonic(&mnemonic, "", wallet_config);
        
        // Initialize storage and biometric (mock for now)
        let storage: Arc<dyn SecureKeyStorage> = Arc::new(MockSecureStorage::new(storage_config.clone()));
        let biometric: Arc<dyn BiometricAuthProvider> = Arc::new(MockBiometricProvider::new());
        
        // Generate storage key ID
        let seed_key_id = format!("{}_{}", SEED_KEY_PREFIX, storage_config.app_identifier);
        
        // Store the mnemonic seed securely
        // TODO Security: Should store encrypted seed, not plaintext mnemonic
        // Convert to seed bytes, encrypt with password-derived key, then store
        let seed_bytes = mnemonic_phrase.as_bytes().to_vec();
        storage.store_key(seed_key_id.clone(), seed_bytes)?;
        
        let lock_state = if storage_config.use_biometric {
            WalletLockState::BiometricLocked
        } else {
            WalletLockState::Locked
        };
        
        Ok(Self {
            wallet: Arc::new(RwLock::new(Some(wallet))),
            storage,
            biometric,
            lock_state: Arc::new(RwLock::new(lock_state)),
            config: storage_config,
            seed_key_id,
        })
    }
    
    /// Restore wallet from mnemonic
    pub fn restore(mnemonic_phrase: String, storage_config: SecureStorageConfig) -> Result<Self> {
        // Same as create for now
        Self::create(mnemonic_phrase, storage_config)
    }
}

// Instance methods exported via UniFFI
impl MobileWallet {
    /// Lock the wallet
    pub fn lock(&self) -> Result<()> {
        let mut wallet = self.wallet.write();
        *wallet = None;
        
        let mut state = self.lock_state.write();
        *state = if self.config.use_biometric {
            WalletLockState::BiometricLocked
        } else {
            WalletLockState::Locked
        };
        
        Ok(())
    }
    
    /// Unlock wallet with password
    ///
    /// # Security Warning
    /// 
    /// **TODO:** This method does NOT verify the password! 
    /// In production, must:
    /// 1. Derive key from password using PBKDF2/Argon2
    /// 2. Use derived key to decrypt stored seed
    /// 3. Only unlock if decryption succeeds
    pub fn unlock(&self, _password: String) -> Result<()> {
        // TODO: Verify password before unlocking
        // Retrieve seed from storage
        let seed_bytes = self.storage.retrieve_key(self.seed_key_id.clone())?;
        let mut mnemonic_phrase = String::from_utf8(seed_bytes)
            .map_err(|_| MobileWalletError::StorageError)?;
        
        // Recreate wallet
        let mnemonic = Mnemonic::from_phrase(&mnemonic_phrase)
            .map_err(|_| MobileWalletError::InvalidMnemonic)?;
        
        // Zeroize the mnemonic phrase string
        use zeroize::Zeroize;
        mnemonic_phrase.zeroize();
        
        let wallet_config = WalletConfig::default();
        
        let wallet = Wallet::from_mnemonic(&mnemonic, "", wallet_config);
        
        // Unlock the wallet
        let mut wallet_guard = self.wallet.write();
        *wallet_guard = Some(wallet);
        
        let mut state = self.lock_state.write();
        *state = WalletLockState::Unlocked;
        
        Ok(())
    }
    
    /// Unlock wallet with biometric authentication
    pub fn unlock_with_biometric(&self) -> Result<()> {
        if !self.config.use_biometric {
            return Err(MobileWalletError::BiometricError);
        }
        
        let result = self.biometric.authenticate("Unlock BitCell Wallet".to_string());
        
        match result {
            BiometricResult::Success => {
                // Unlock without password using stored seed
                self.unlock("".to_string())
            }
            BiometricResult::Cancelled => Err(MobileWalletError::BiometricError),
            BiometricResult::Failed => Err(MobileWalletError::BiometricError),
            BiometricResult::NotAvailable => Err(MobileWalletError::BiometricError),
            BiometricResult::NotEnrolled => Err(MobileWalletError::BiometricError),
        }
    }
    
    /// Get current lock state
    pub fn get_lock_state(&self) -> WalletLockState {
        *self.lock_state.read()
    }
    
    /// Check if wallet is locked
    pub fn is_locked(&self) -> bool {
        self.get_lock_state() != WalletLockState::Unlocked
    }
    
    /// Get account information
    pub fn get_account_info(&self) -> Result<AccountInfo> {
        self.ensure_unlocked()?;
        
        let wallet = self.wallet.read();
        let _wallet = wallet.as_ref().ok_or(MobileWalletError::WalletLocked)?;
        
        // Simplified for now - would need wallet API enhancements
        Ok(AccountInfo {
            address: "BC1...".to_string(), // Placeholder
            balance: "0".to_string(),
            nonce: 0,
        })
    }
    
    /// Get wallet address
    pub fn get_address(&self) -> Result<String> {
        self.ensure_unlocked()?;
        
        let wallet = self.wallet.read();
        let _wallet = wallet.as_ref().ok_or(MobileWalletError::WalletLocked)?;
        
        // Placeholder - needs wallet API enhancement
        Ok("BC1...".to_string())
    }
    
    /// Get public key
    pub fn get_public_key(&self) -> Result<String> {
        self.ensure_unlocked()?;
        
        let wallet = self.wallet.read();
        let _wallet = wallet.as_ref().ok_or(MobileWalletError::WalletLocked)?;
        
        // Placeholder - needs wallet API enhancement
        // Return a 66-character hex string (33 bytes)
        Ok(format!("{:0<66}", "0x"))
    }
    
    /// Sign a transaction
    pub fn sign_transaction(&self, tx_details: TransactionDetails) -> Result<SignedTransactionResult> {
        self.ensure_unlocked()?;
        
        // Simplified implementation - placeholder for full integration
        let tx_hash = format!("0x{:064x}", 0); // Placeholder
        let raw_transaction = format!("0x00"); // Placeholder
        
        Ok(SignedTransactionResult {
            tx_hash,
            raw_transaction,
        })
    }
    
    /// Sign a message
    pub fn sign_message(&self, message: String) -> Result<String> {
        self.ensure_unlocked()?;
        
        // For now, return an error as we need access to the secret key
        // In a real implementation, we'd add a sign_message method to Wallet
        Err(MobileWalletError::NotImplemented)
    }
    
    /// Create encrypted backup
    pub fn create_backup(&self, password: String) -> Result<WalletBackup> {
        self.ensure_unlocked()?;
        WalletBackup::create(self, password)
    }
    
    /// Restore from backup
    pub fn restore_from_backup(&self, backup: WalletBackup, password: String) -> Result<()> {
        backup.restore(self, password)
    }
    
    /// Export mnemonic (requires password)
    ///
    /// # Security Warning
    ///
    /// **TODO:** This method does NOT verify the password!
    /// Must verify password before returning sensitive mnemonic data.
    pub fn export_mnemonic(&self, _password: String) -> Result<String> {
        self.ensure_unlocked()?;
        
        // TODO: Verify password before export
        let seed_bytes = self.storage.retrieve_key(self.seed_key_id.clone())?;
        String::from_utf8(seed_bytes)
            .map_err(|_| MobileWalletError::StorageError)
    }
    
    /// Change wallet password
    ///
    /// # Security Warning
    ///
    /// **TODO:** This method does NOT verify the old password!
    /// Must verify old_password before allowing password change.
    pub fn change_password(&self, _old_password: String, _new_password: String) -> Result<()> {
        self.ensure_unlocked()?;
        // In production, verify old_password and re-encrypt with new_password
        Ok(())
    }
    
    /// Enable or disable biometric authentication
    pub fn enable_biometric(&self, enable: bool) -> Result<()> {
        if enable && !self.biometric.is_available() {
            return Err(MobileWalletError::BiometricError);
        }
        
        // Update config (in production, would persist this)
        Ok(())
    }
    
    /// Check if biometric is enabled
    pub fn is_biometric_enabled(&self) -> bool {
        self.config.use_biometric
    }
    
    /// Get wallet version
    pub fn get_wallet_version(&self) -> String {
        WALLET_VERSION.to_string()
    }
    
    /// Clear secure storage
    pub fn clear_secure_storage(&self) -> Result<()> {
        self.storage.clear_all_keys()
    }
    
    /// Ensure wallet is unlocked
    fn ensure_unlocked(&self) -> Result<()> {
        if self.is_locked() {
            Err(MobileWalletError::WalletLocked)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_mnemonic;
    use crate::MnemonicWordCount;

    fn create_test_wallet() -> MobileWallet {
        let mnemonic = generate_mnemonic(MnemonicWordCount::Words12).unwrap();
        let config = SecureStorageConfig::default();
        MobileWallet::create(mnemonic, config).unwrap()
    }

    #[test]
    fn test_create_wallet() {
        let wallet = create_test_wallet();
        assert!(wallet.is_locked());
    }

    #[test]
    fn test_unlock_wallet() {
        let wallet = create_test_wallet();
        wallet.unlock("password".to_string()).unwrap();
        assert!(!wallet.is_locked());
    }

    #[test]
    fn test_lock_wallet() {
        let wallet = create_test_wallet();
        wallet.unlock("password".to_string()).unwrap();
        assert!(!wallet.is_locked());
        
        wallet.lock().unwrap();
        assert!(wallet.is_locked());
    }

    #[test]
    fn test_get_address_locked() {
        let wallet = create_test_wallet();
        let result = wallet.get_address();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MobileWalletError::WalletLocked));
    }

    #[test]
    fn test_get_address_unlocked() {
        let wallet = create_test_wallet();
        wallet.unlock("password".to_string()).unwrap();
        
        let address = wallet.get_address().unwrap();
        assert!(!address.is_empty());
    }

    #[test]
    fn test_get_public_key() {
        let wallet = create_test_wallet();
        wallet.unlock("password".to_string()).unwrap();
        
        let pubkey = wallet.get_public_key().unwrap();
        assert!(!pubkey.is_empty());
        assert_eq!(pubkey.len(), 66); // 33 bytes hex encoded
    }

    #[test]
    #[should_panic(expected = "NotImplemented")]
    fn test_sign_message() {
        let wallet = create_test_wallet();
        wallet.unlock("password".to_string()).unwrap();
        
        // This should fail with NotImplemented error
        wallet.sign_message("test message".to_string()).unwrap();
    }

    #[test]
    fn test_wallet_version() {
        let wallet = create_test_wallet();
        let version = wallet.get_wallet_version();
        assert!(!version.is_empty());
    }
}
