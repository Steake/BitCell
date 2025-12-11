//! Secure key storage abstraction
//!
//! This module provides a platform-agnostic interface for secure key storage.
//! On iOS, this should be implemented using iOS Keychain.
//! On Android, this should be implemented using Android Keystore.

use crate::error::{MobileWalletError, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use zeroize::Zeroize;

/// Configuration for secure storage
#[derive(Debug, Clone)]
pub struct SecureStorageConfig {
    /// Enable biometric authentication for key access
    pub use_biometric: bool,
    /// Use platform keychain (iOS Keychain / Android Keystore)
    pub use_keychain: bool,
    /// Application identifier for keychain isolation
    pub app_identifier: String,
}

impl Default for SecureStorageConfig {
    fn default() -> Self {
        Self {
            use_biometric: false,
            use_keychain: true,
            app_identifier: "com.bitcell.wallet".to_string(),
        }
    }
}

/// Secure key storage trait
///
/// Platform-specific implementations should:
/// - iOS: Use Keychain Services API with kSecAttrAccessibleWhenUnlockedThisDeviceOnly
/// - Android: Use AndroidKeyStore with StrongBox if available
///
/// # Security Considerations
///
/// - Keys should never be stored in plaintext
/// - Keys should be bound to the device (no cloud backup)
/// - Biometric authentication should use platform APIs (Face ID, Touch ID, BiometricPrompt)
/// - Key material should be zeroized after use
pub trait SecureKeyStorage: Send + Sync {
    /// Store a key securely
    ///
    /// # Platform-specific behavior
    ///
    /// ## iOS
    /// Should use `SecItemAdd` with attributes:
    /// - `kSecClass`: `kSecClassGenericPassword`
    /// - `kSecAttrAccessible`: `kSecAttrAccessibleWhenUnlockedThisDeviceOnly`
    /// - `kSecAttrSynchronizable`: `false`
    /// - `kSecUseAuthenticationContext`: For biometric protection
    ///
    /// ## Android
    /// Should use `AndroidKeyStore` with:
    /// - `setUserAuthenticationRequired(true)` for biometric
    /// - `setIsStrongBoxBacked(true)` if available
    /// - `setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_RSA_OAEP)`
    fn store_key(&self, key_id: String, key_data: Vec<u8>) -> Result<()>;
    
    /// Retrieve a key from secure storage
    ///
    /// May trigger biometric authentication if configured
    fn retrieve_key(&self, key_id: String) -> Result<Vec<u8>>;
    
    /// Delete a key from secure storage
    fn delete_key(&self, key_id: String) -> Result<()>;
    
    /// Check if a key exists in storage
    fn key_exists(&self, key_id: String) -> bool;
    
    /// Clear all keys from storage
    ///
    /// Should be used carefully - typically only for wallet reset/logout
    fn clear_all_keys(&self) -> Result<()>;
}

/// Mock implementation for testing
///
/// In production, this would be replaced by:
/// - `IosKeychainStorage` for iOS
/// - `AndroidKeystoreStorage` for Android
pub struct MockSecureStorage {
    storage: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    config: SecureStorageConfig,
}

impl MockSecureStorage {
    pub fn new(config: SecureStorageConfig) -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
}

impl SecureKeyStorage for MockSecureStorage {
    fn store_key(&self, key_id: String, key_data: Vec<u8>) -> Result<()> {
        // In production, this would use platform keychain
        let mut storage = self.storage.write();
        storage.insert(key_id, key_data);
        
        Ok(())
    }
    
    fn retrieve_key(&self, key_id: String) -> Result<Vec<u8>> {
        let storage = self.storage.read();
        storage
            .get(&key_id)
            .cloned()
            .ok_or(MobileWalletError::StorageError)
    }
    
    fn delete_key(&self, key_id: String) -> Result<()> {
        let mut storage = self.storage.write();
        storage.remove(&key_id);
        Ok(())
    }
    
    fn key_exists(&self, key_id: String) -> bool {
        self.storage.read().contains_key(&key_id)
    }
    
    fn clear_all_keys(&self) -> Result<()> {
        let mut storage = self.storage.write();
        
        // Zeroize all key data before clearing
        for (_, mut value) in storage.drain() {
            value.zeroize();
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_storage_store_retrieve() {
        let config = SecureStorageConfig::default();
        let storage = MockSecureStorage::new(config);
        
        let key_data = vec![1, 2, 3, 4, 5];
        storage.store_key("test_key".to_string(), key_data.clone()).unwrap();
        
        let retrieved = storage.retrieve_key("test_key".to_string()).unwrap();
        assert_eq!(retrieved, key_data);
    }

    #[test]
    fn test_mock_storage_key_exists() {
        let config = SecureStorageConfig::default();
        let storage = MockSecureStorage::new(config);
        
        assert!(!storage.key_exists("test_key".to_string()));
        
        storage.store_key("test_key".to_string(), vec![1, 2, 3]).unwrap();
        assert!(storage.key_exists("test_key".to_string()));
    }

    #[test]
    fn test_mock_storage_delete() {
        let config = SecureStorageConfig::default();
        let storage = MockSecureStorage::new(config);
        
        storage.store_key("test_key".to_string(), vec![1, 2, 3]).unwrap();
        assert!(storage.key_exists("test_key".to_string()));
        
        storage.delete_key("test_key".to_string()).unwrap();
        assert!(!storage.key_exists("test_key".to_string()));
    }

    #[test]
    fn test_mock_storage_clear_all() {
        let config = SecureStorageConfig::default();
        let storage = MockSecureStorage::new(config);
        
        storage.store_key("key1".to_string(), vec![1]).unwrap();
        storage.store_key("key2".to_string(), vec![2]).unwrap();
        
        storage.clear_all_keys().unwrap();
        
        assert!(!storage.key_exists("key1".to_string()));
        assert!(!storage.key_exists("key2".to_string()));
    }
}
