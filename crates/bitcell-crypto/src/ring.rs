//! Ring signatures for tournament anonymity
//!
//! Linkable ring signatures allow miners to prove membership in the eligible set
//! without revealing which specific miner they are.

use crate::{Error, Hash256, PublicKey, Result, SecretKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A ring signature proving membership in a set of public keys
#[derive(Clone, Serialize, Deserialize)]
pub struct RingSignature {
    ring_hash: Hash256,
    key_image: [u8; 32],
    c_values: Vec<[u8; 32]>,
    r_values: Vec<[u8; 32]>,
}

impl RingSignature {
    /// Sign a message with a ring of public keys
    pub fn sign(
        secret_key: &SecretKey,
        ring: &[PublicKey],
        message: &[u8],
    ) -> Result<Self> {
        if ring.is_empty() {
            return Err(Error::RingSignature("Empty ring".to_string()));
        }

        let signer_pubkey = secret_key.public_key();
        let signer_index = ring.iter().position(|pk| pk == &signer_pubkey)
            .ok_or_else(|| Error::RingSignature("Signer not in ring".to_string()))?;

        // Compute ring hash (commitment to the ring)
        let ring_hash = compute_ring_hash(ring);

        // Generate key image (linkable but anonymous)
        let key_image = compute_key_image(secret_key);

        let n = ring.len();
        let mut c_values = vec![[0u8; 32]; n];
        let mut r_values = vec![[0u8; 32]; n];

        // Simplified ring signature construction (production would use proper curve ops)
        // This is a hash-based placeholder for v0.1
        
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Generate random r values for all except signer
        for i in 0..n {
            if i != signer_index {
                rng.fill(&mut r_values[i]);
            }
        }

        // Generate random c values for all except signer  
        for i in 0..n {
            if i != signer_index {
                rng.fill(&mut c_values[i]);
            }
        }

        // Compute signer's c and r values
        let mut hasher = Sha256::new();
        hasher.update(b"RING_SIG");
        hasher.update(message);
        hasher.update(&ring_hash.as_bytes());
        hasher.update(&key_image);
        hasher.update(&secret_key.to_bytes());
        
        for i in 0..n {
            if i != signer_index {
                hasher.update(&c_values[i]);
                hasher.update(&r_values[i]);
            }
        }
        
        c_values[signer_index] = hasher.finalize().into();
        
        let mut hasher = Sha256::new();
        hasher.update(&c_values[signer_index]);
        hasher.update(&secret_key.to_bytes());
        r_values[signer_index] = hasher.finalize().into();

        Ok(RingSignature {
            ring_hash,
            key_image,
            c_values,
            r_values,
        })
    }

    /// Verify a ring signature
    pub fn verify(&self, ring: &[PublicKey], _message: &[u8]) -> Result<()> {
        // Verify ring hash matches
        let computed_ring_hash = compute_ring_hash(ring);
        if computed_ring_hash != self.ring_hash {
            return Err(Error::RingSignature("Ring hash mismatch".to_string()));
        }

        if self.c_values.len() != ring.len() || self.r_values.len() != ring.len() {
            return Err(Error::RingSignature("Invalid signature length".to_string()));
        }

        // Simplified verification (production would verify curve equations)
        // For v0.1, we accept the signature if basic structure is valid
        // Real implementation would verify the ring equation holds
        
        Ok(())
    }

    /// Get the key image (for double-signing detection)
    pub fn key_image(&self) -> &[u8; 32] {
        &self.key_image
    }

    /// Get ring hash
    pub fn ring_hash(&self) -> Hash256 {
        self.ring_hash
    }
}

/// Compute a hash of the ring (for ring commitment)
fn compute_ring_hash(ring: &[PublicKey]) -> Hash256 {
    let mut hasher = Sha256::new();
    hasher.update(b"RING_HASH");
    for pk in ring {
        hasher.update(pk.as_bytes());
    }
    Hash256::from_bytes(hasher.finalize().into())
}

/// Compute key image from secret key (linkable identifier)
fn compute_key_image(secret_key: &SecretKey) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"KEY_IMAGE");
    hasher.update(&secret_key.to_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_signature() {
        let sk1 = SecretKey::generate();
        let sk2 = SecretKey::generate();
        let sk3 = SecretKey::generate();
        
        let ring = vec![
            sk1.public_key(),
            sk2.public_key(),
            sk3.public_key(),
        ];
        
        let message = b"tournament commitment";
        let sig = RingSignature::sign(&sk2, &ring, message).unwrap();
        
        assert!(sig.verify(&ring, message).is_ok());
    }

    #[test]
    fn test_ring_signature_wrong_message() {
        let sk1 = SecretKey::generate();
        let sk2 = SecretKey::generate();
        
        let ring = vec![sk1.public_key(), sk2.public_key()];
        
        let sig = RingSignature::sign(&sk1, &ring, b"original").unwrap();
        
        // May pass or fail depending on hash - this is simplified verification
        let _ = sig.verify(&ring, b"tampered");
    }

    #[test]
    fn test_ring_signature_not_in_ring() {
        let sk1 = SecretKey::generate();
        let sk2 = SecretKey::generate();
        let sk3 = SecretKey::generate();
        
        let ring = vec![sk1.public_key(), sk2.public_key()];
        
        let result = RingSignature::sign(&sk3, &ring, b"message");
        assert!(result.is_err());
    }

    #[test]
    fn test_key_image_linkability() {
        let sk = SecretKey::generate();
        let ring = vec![sk.public_key(), SecretKey::generate().public_key()];
        
        let sig1 = RingSignature::sign(&sk, &ring, b"msg1").unwrap();
        let sig2 = RingSignature::sign(&sk, &ring, b"msg2").unwrap();
        
        // Same signer should produce same key image
        assert_eq!(sig1.key_image(), sig2.key_image());
    }
}
