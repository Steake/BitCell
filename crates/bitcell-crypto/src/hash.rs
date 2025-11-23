//! Hash functions for BitCell
//!
//! Provides SHA-256 for general use and Blake3 for performance-critical paths.
//! Poseidon will be added for circuit-friendly hashing.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// 32-byte hash output
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash256([u8; 32]);

impl Hash256 {
    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Convert to bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Zero hash
    pub const fn zero() -> Self {
        Self([0u8; 32])
    }

    /// Hash arbitrary data with SHA-256
    pub fn hash(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        Self(hasher.finalize().into())
    }

    /// Hash multiple items
    pub fn hash_multiple(items: &[&[u8]]) -> Self {
        let mut hasher = Sha256::new();
        for item in items {
            hasher.update(item);
        }
        Self(hasher.finalize().into())
    }
}

impl fmt::Debug for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash256({})", hex::encode(&self.0[..8]))
    }
}

impl fmt::Display for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl From<[u8; 32]> for Hash256 {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8]> for Hash256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Trait for hashable types
pub trait Hashable {
    fn hash(&self) -> Hash256;
}

impl Hashable for &[u8] {
    fn hash(&self) -> Hash256 {
        Hash256::hash(self)
    }
}

impl Hashable for Vec<u8> {
    fn hash(&self) -> Hash256 {
        Hash256::hash(self)
    }
}

impl Hashable for String {
    fn hash(&self) -> Hash256 {
        Hash256::hash(self.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_deterministic() {
        let data = b"hello world";
        let h1 = Hash256::hash(data);
        let h2 = Hash256::hash(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_different_inputs() {
        let h1 = Hash256::hash(b"hello");
        let h2 = Hash256::hash(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_zero_hash() {
        let zero = Hash256::zero();
        assert_eq!(zero.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn test_hash_multiple() {
        let h1 = Hash256::hash_multiple(&[b"hello", b"world"]);
        let h2 = Hash256::hash(b"helloworld");
        // Without explicit domain separation, these will be the same
        assert_eq!(h1, h2);
        
        // Different ordering should give different results
        let h3 = Hash256::hash_multiple(&[b"world", b"hello"]);
        assert_ne!(h1, h3);
    }
}
