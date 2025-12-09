//! Key image registry for double-spend prevention
//!
//! Tracks used key images from CLSAG ring signatures to prevent
//! double-signing attacks in tournaments.

use bitcell_crypto::KeyImage;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Registry of used key images for double-spend prevention
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyImageRegistry {
    /// Set of used key images (O(1) lookup)
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
    /// 
    /// Returns true if the key image is already in the registry (double-spend attempt)
    pub fn is_used(&self, key_image: &KeyImage) -> bool {
        self.used_images.contains(key_image)
    }

    /// Mark a key image as used
    /// 
    /// Returns Ok(()) if successfully added, Err if already used (double-spend)
    pub fn mark_used(&mut self, key_image: KeyImage) -> Result<(), KeyImageError> {
        if self.used_images.contains(&key_image) {
            return Err(KeyImageError::AlreadyUsed);
        }
        self.used_images.insert(key_image);
        Ok(())
    }

    /// Check and mark a key image in one operation
    /// 
    /// This is an atomic operation that checks for double-spend and marks as used.
    /// Returns Ok(()) if the key image was new and is now marked as used.
    /// Returns Err if the key image was already used.
    pub fn check_and_mark(&mut self, key_image: KeyImage) -> Result<(), KeyImageError> {
        if !self.used_images.insert(key_image) {
            return Err(KeyImageError::AlreadyUsed);
        }
        Ok(())
    }

    /// Remove a key image (for rollback scenarios)
    /// 
    /// This should only be used during chain reorganization
    pub fn remove(&mut self, key_image: &KeyImage) -> bool {
        self.used_images.remove(key_image)
    }

    /// Get the number of used key images
    pub fn len(&self) -> usize {
        self.used_images.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.used_images.is_empty()
    }

    /// Clear all key images (for testing or rollback)
    pub fn clear(&mut self) {
        self.used_images.clear();
    }

    /// Get an iterator over all used key images
    pub fn iter(&self) -> impl Iterator<Item = &KeyImage> {
        self.used_images.iter()
    }
}

impl Default for KeyImageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors related to key image tracking
#[derive(Debug, thiserror::Error)]
pub enum KeyImageError {
    #[error("Key image already used - double-spend attempt detected")]
    AlreadyUsed,
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
    fn test_mark_used() {
        let mut registry = KeyImageRegistry::new();
        let sk = ClsagSecretKey::generate();
        let key_image = sk.key_image();

        assert!(!registry.is_used(&key_image));
        assert!(registry.mark_used(key_image).is_ok());
        assert!(registry.is_used(&key_image));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_double_spend_detection() {
        let mut registry = KeyImageRegistry::new();
        let sk = ClsagSecretKey::generate();
        let key_image = sk.key_image();

        // First use should succeed
        assert!(registry.mark_used(key_image).is_ok());

        // Second use should fail (double-spend)
        let result = registry.mark_used(key_image);
        assert!(result.is_err());
        assert!(matches!(result, Err(KeyImageError::AlreadyUsed)));
    }

    #[test]
    fn test_check_and_mark() {
        let mut registry = KeyImageRegistry::new();
        let sk = ClsagSecretKey::generate();
        let key_image = sk.key_image();

        // First check_and_mark should succeed
        assert!(registry.check_and_mark(key_image).is_ok());
        assert!(registry.is_used(&key_image));

        // Second check_and_mark should fail
        assert!(registry.check_and_mark(key_image).is_err());
    }

    #[test]
    fn test_multiple_key_images() {
        let mut registry = KeyImageRegistry::new();
        
        let sk1 = ClsagSecretKey::generate();
        let sk2 = ClsagSecretKey::generate();
        let sk3 = ClsagSecretKey::generate();

        let ki1 = sk1.key_image();
        let ki2 = sk2.key_image();
        let ki3 = sk3.key_image();

        // All should be different
        assert_ne!(ki1, ki2);
        assert_ne!(ki2, ki3);
        assert_ne!(ki1, ki3);

        // Mark all as used
        assert!(registry.mark_used(ki1).is_ok());
        assert!(registry.mark_used(ki2).is_ok());
        assert!(registry.mark_used(ki3).is_ok());

        assert_eq!(registry.len(), 3);
        assert!(registry.is_used(&ki1));
        assert!(registry.is_used(&ki2));
        assert!(registry.is_used(&ki3));
    }

    #[test]
    fn test_remove_key_image() {
        let mut registry = KeyImageRegistry::new();
        let sk = ClsagSecretKey::generate();
        let key_image = sk.key_image();

        registry.mark_used(key_image).unwrap();
        assert!(registry.is_used(&key_image));

        // Remove the key image
        assert!(registry.remove(&key_image));
        assert!(!registry.is_used(&key_image));
        assert!(registry.is_empty());

        // Removing again should return false
        assert!(!registry.remove(&key_image));
    }

    #[test]
    fn test_clear() {
        let mut registry = KeyImageRegistry::new();
        
        for _ in 0..10 {
            let sk = ClsagSecretKey::generate();
            registry.mark_used(sk.key_image()).unwrap();
        }

        assert_eq!(registry.len(), 10);
        
        registry.clear();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_iterator() {
        let mut registry = KeyImageRegistry::new();
        let mut key_images = vec![];
        
        for _ in 0..5 {
            let sk = ClsagSecretKey::generate();
            let ki = sk.key_image();
            key_images.push(ki);
            registry.mark_used(ki).unwrap();
        }

        let collected: Vec<_> = registry.iter().copied().collect();
        assert_eq!(collected.len(), 5);
        
        // All key images should be in the registry
        for ki in &key_images {
            assert!(collected.contains(ki));
        }
    }

    #[test]
    fn test_same_key_different_signatures() {
        let mut registry = KeyImageRegistry::new();
        let sk = ClsagSecretKey::generate();
        
        // Same secret key should always produce the same key image
        let ki1 = sk.key_image();
        let ki2 = sk.key_image();
        
        assert_eq!(ki1, ki2);
        
        // First use succeeds
        assert!(registry.mark_used(ki1).is_ok());
        
        // Second use fails even if we derive the key image again
        assert!(registry.mark_used(ki2).is_err());
    }
}
