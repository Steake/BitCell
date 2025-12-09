//! Key Image Registry for Double-Spend Prevention
//!
//! Provides O(1) lookup to track used key images and prevent double-signing
//! in CLSAG ring signatures.

use bitcell_crypto::KeyImage;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Key image already exists (double-spend attempt)")]
    KeyImageExists,
}

/// Registry for tracking used key images to prevent double-spending
#[derive(Debug, Clone)]
pub struct KeyImageRegistry {
    used_images: HashSet<KeyImage>,
}

impl KeyImageRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            used_images: HashSet::new(),
        }
    }

    /// Check if a key image has been used
    pub fn contains(&self, key_image: &KeyImage) -> bool {
        self.used_images.contains(key_image)
    }

    /// Mark a key image as used
    /// Returns an error if the key image was already used (double-spend)
    pub fn mark(&mut self, key_image: KeyImage) -> Result<()> {
        if self.used_images.contains(&key_image) {
            return Err(Error::KeyImageExists);
        }
        self.used_images.insert(key_image);
        Ok(())
    }

    /// Check and mark a key image atomically
    /// This is the preferred method for concurrent scenarios
    pub fn check_and_mark(&mut self, key_image: KeyImage) -> Result<()> {
        self.mark(key_image)
    }

    /// Get the number of tracked key images
    pub fn len(&self) -> usize {
        self.used_images.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.used_images.is_empty()
    }

    /// Clear all key images (for testing)
    #[cfg(test)]
    pub fn clear(&mut self) {
        self.used_images.clear();
    }

    /// Create a thread-safe registry wrapped in Arc<Mutex<>>
    pub fn new_shared() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new()))
    }
}

impl Default for KeyImageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcell_crypto::ClsagSecretKey;

    #[test]
    fn test_new_registry_empty() {
        let registry = KeyImageRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_mark_key_image() {
        let mut registry = KeyImageRegistry::new();
        let sk = ClsagSecretKey::generate();
        let key_image = sk.key_image();

        assert!(!registry.contains(&key_image));
        assert!(registry.mark(key_image).is_ok());
        assert!(registry.contains(&key_image));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_double_spend_detection() {
        let mut registry = KeyImageRegistry::new();
        let sk = ClsagSecretKey::generate();
        let key_image = sk.key_image();

        // First use should succeed
        assert!(registry.mark(key_image).is_ok());

        // Second use should fail (double-spend)
        let result = registry.mark(key_image);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::KeyImageExists));
    }

    #[test]
    fn test_check_and_mark() {
        let mut registry = KeyImageRegistry::new();
        let sk1 = ClsagSecretKey::generate();
        let sk2 = ClsagSecretKey::generate();
        let ki1 = sk1.key_image();
        let ki2 = sk2.key_image();

        // First key image
        assert!(registry.check_and_mark(ki1).is_ok());
        assert!(registry.contains(&ki1));

        // Second key image
        assert!(registry.check_and_mark(ki2).is_ok());
        assert!(registry.contains(&ki2));

        // Duplicate should fail
        assert!(registry.check_and_mark(ki1).is_err());
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_multiple_key_images() {
        let mut registry = KeyImageRegistry::new();
        let count = 10;

        for _ in 0..count {
            let sk = ClsagSecretKey::generate();
            let ki = sk.key_image();
            assert!(registry.mark(ki).is_ok());
        }

        assert_eq!(registry.len(), count);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut registry = KeyImageRegistry::new();
        let sk = ClsagSecretKey::generate();
        let ki = sk.key_image();

        registry.mark(ki).unwrap();
        assert_eq!(registry.len(), 1);

        registry.clear();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        // Can mark again after clear
        assert!(registry.mark(ki).is_ok());
    }

    #[test]
    fn test_shared_registry() {
        let registry = KeyImageRegistry::new_shared();
        let sk = ClsagSecretKey::generate();
        let ki = sk.key_image();

        {
            let mut reg = registry.lock().unwrap();
            assert!(reg.mark(ki).is_ok());
        }

        {
            let reg = registry.lock().unwrap();
            assert!(reg.contains(&ki));
        }
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;

        let registry = KeyImageRegistry::new_shared();
        let sk1 = ClsagSecretKey::generate();
        let sk2 = ClsagSecretKey::generate();
        let ki1 = sk1.key_image();
        let ki2 = sk2.key_image();

        let reg1 = Arc::clone(&registry);
        let reg2 = Arc::clone(&registry);

        let h1 = thread::spawn(move || {
            let mut reg = reg1.lock().unwrap();
            reg.check_and_mark(ki1)
        });

        let h2 = thread::spawn(move || {
            let mut reg = reg2.lock().unwrap();
            reg.check_and_mark(ki2)
        });

        assert!(h1.join().unwrap().is_ok());
        assert!(h2.join().unwrap().is_ok());

        let reg = registry.lock().unwrap();
        assert_eq!(reg.len(), 2);
    }
}
