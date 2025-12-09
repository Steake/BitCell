//! CLSAG (Concise Linkable Spontaneous Anonymous Group) Signatures
//!
//! Implements linkable ring signatures for tournament anonymity.
//! Based on the CLSAG construction from Monero.
//!
//! Ring sizes are configurable with the following constraints:
//! - Minimum: 11 (provides adequate anonymity set)
//! - Maximum: 64 (balances privacy with verification performance)
//! - Default: 16 (recommended for most use cases)

use crate::{Error, Result};
use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_TABLE,
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

/// Minimum ring size for CLSAG signatures
pub const MIN_RING_SIZE: usize = 11;

/// Maximum ring size for CLSAG signatures
pub const MAX_RING_SIZE: usize = 64;

/// Default ring size for CLSAG signatures
pub const DEFAULT_RING_SIZE: usize = 16;

/// Configuration for CLSAG ring signatures
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RingConfig {
    /// Size of the ring (must be between MIN_RING_SIZE and MAX_RING_SIZE)
    ring_size: usize,
}

impl RingConfig {
    /// Create a new ring configuration with the specified size
    pub fn new(ring_size: usize) -> Result<Self> {
        if ring_size < MIN_RING_SIZE {
            return Err(Error::RingSignature(format!(
                "Ring size {} is below minimum {}",
                ring_size, MIN_RING_SIZE
            )));
        }
        if ring_size > MAX_RING_SIZE {
            return Err(Error::RingSignature(format!(
                "Ring size {} exceeds maximum {}",
                ring_size, MAX_RING_SIZE
            )));
        }
        Ok(Self { ring_size })
    }

    /// Get the ring size
    pub fn ring_size(&self) -> usize {
        self.ring_size
    }
}

impl Default for RingConfig {
    fn default() -> Self {
        Self {
            ring_size: DEFAULT_RING_SIZE,
        }
    }
}

/// CLSAG public key (Ristretto point)
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ClsagPublicKey([u8; 32]);

impl ClsagPublicKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Result<Self> {
        // Validate it's a valid compressed point
        CompressedRistretto::from_slice(&bytes)
            .map_err(|_| Error::InvalidPublicKey)?;
        Ok(Self(bytes))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    fn to_point(&self) -> Result<RistrettoPoint> {
        CompressedRistretto::from_slice(&self.0)
            .map_err(|_| Error::InvalidPublicKey)?
            .decompress()
            .ok_or(Error::InvalidPublicKey)
    }
}

/// CLSAG secret key (scalar)
#[derive(Clone)]
pub struct ClsagSecretKey(Scalar);

impl ClsagSecretKey {
    /// Generate a new random key pair
    pub fn generate() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        Self(Scalar::from_bytes_mod_order(bytes))
    }

    /// Get the public key (x*G)
    pub fn public_key(&self) -> ClsagPublicKey {
        let point = &self.0 * RISTRETTO_BASEPOINT_TABLE;
        ClsagPublicKey(point.compress().to_bytes())
    }

    /// Get key image (x*Hp(P)) - linkable identifier
    pub fn key_image(&self) -> KeyImage {
        let pk = self.public_key();
        let hp = hash_to_point(&pk.0);
        let ki = hp * self.0;
        KeyImage(ki.compress().to_bytes())
    }
}

/// Key image for double-spending detection
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Hash)]
pub struct KeyImage([u8; 32]);

impl KeyImage {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    fn to_point(&self) -> Result<RistrettoPoint> {
        CompressedRistretto::from_slice(&self.0)
            .map_err(|_| Error::RingSignature("Invalid key image".to_string()))?
            .decompress()
            .ok_or_else(|| Error::RingSignature("Key image decompression failed".to_string()))
    }
}

/// CLSAG ring signature
#[derive(Clone, Serialize, Deserialize)]
pub struct ClsagSignature {
    key_image: KeyImage,
    #[serde(with = "scalar_serde")]
    c1: Scalar,
    #[serde(with = "scalar_vec_serde")]
    s: Vec<Scalar>,
}

// Serde helpers for Scalar
mod scalar_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(scalar: &Scalar, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&scalar.to_bytes())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Scalar, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = serde::Deserialize::deserialize(deserializer)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Invalid scalar length"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Scalar::from_bytes_mod_order(arr))
    }
}

mod scalar_vec_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(scalars: &Vec<Scalar>, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes: Vec<Vec<u8>> = scalars.iter().map(|s| s.to_bytes().to_vec()).collect();
        bytes.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Vec<Scalar>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes_vec: Vec<Vec<u8>> = serde::Deserialize::deserialize(deserializer)?;
        bytes_vec
            .into_iter()
            .map(|bytes| {
                if bytes.len() != 32 {
                    return Err(serde::de::Error::custom("Invalid scalar length"));
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Ok(Scalar::from_bytes_mod_order(arr))
            })
            .collect()
    }
}

impl ClsagSignature {
    /// Sign a message with a ring of public keys
    /// 
    /// # Arguments
    /// * `secret_key` - The signer's secret key
    /// * `ring` - The ring of public keys (must contain the signer's public key)
    /// * `message` - The message to sign
    /// 
    /// # Ring Size Validation
    /// The ring size is validated to ensure it meets minimum/maximum requirements.
    /// For optimal privacy-performance tradeoff, use the default ring size of 16.
    pub fn sign(
        secret_key: &ClsagSecretKey,
        ring: &[ClsagPublicKey],
        message: &[u8],
    ) -> Result<Self> {
        if ring.is_empty() {
            return Err(Error::RingSignature("Empty ring".to_string()));
        }

        // Validate ring size
        if ring.len() < MIN_RING_SIZE {
            return Err(Error::RingSignature(format!(
                "Ring size {} is below minimum {}",
                ring.len(),
                MIN_RING_SIZE
            )));
        }
        if ring.len() > MAX_RING_SIZE {
            return Err(Error::RingSignature(format!(
                "Ring size {} exceeds maximum {}",
                ring.len(),
                MAX_RING_SIZE
            )));
        }

        let signer_pk = secret_key.public_key();
        let pi = ring
            .iter()
            .position(|pk| pk == &signer_pk)
            .ok_or_else(|| Error::RingSignature("Signer not in ring".to_string()))?;

        let n = ring.len();
        let key_image = secret_key.key_image();

        // Convert ring to points
        let ring_points: Result<Vec<RistrettoPoint>> =
            ring.iter().map(|pk| pk.to_point()).collect();
        let ring_points = ring_points?;

        // Hash key image to point
        let ki_point = key_image.to_point()?;
        let hp_pi = hash_to_point(&ring[pi].0);

        // Generate random alpha
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let alpha_bytes: [u8; 32] = rng.gen();
        let alpha = Scalar::from_bytes_mod_order(alpha_bytes);

        // Compute L_pi = alpha*G and R_pi = alpha*Hp(P_pi)
        let l_pi = &alpha * RISTRETTO_BASEPOINT_TABLE;
        let r_pi = hp_pi * alpha;

        // Initialize challenge array
        let mut c = vec![Scalar::ZERO; n];
        let mut s = vec![Scalar::ZERO; n];

        // Compute c_{pi+1}
        let next_idx = (pi + 1) % n;
        c[next_idx] = hash_to_scalar(&[
            &message.to_vec(),
            &ring_points.iter().map(|p| p.compress().to_bytes().to_vec()).collect::<Vec<_>>().concat(),
            &key_image.0.to_vec(),
            &l_pi.compress().to_bytes().to_vec(),
            &r_pi.compress().to_bytes().to_vec(),
        ]);

        // Generate random s values and compute challenges for all indices except pi
        let mut idx = (pi + 1) % n;
        while idx != pi {
            s[idx] = Scalar::from_bytes_mod_order(rng.gen());

            // Compute L_j = s_j*G + c_j*P_j
            let l_j = (&s[idx] * RISTRETTO_BASEPOINT_TABLE) + (c[idx] * ring_points[idx]);

            // Compute R_j = s_j*Hp(P_j) + c_j*KI
            let hp_j = hash_to_point(&ring[idx].0);
            let r_j = (hp_j * s[idx]) + (ki_point * c[idx]);

            // Compute next challenge
            let next_idx = (idx + 1) % n;
            c[next_idx] = hash_to_scalar(&[
                &message.to_vec(),
                &ring_points.iter().map(|p| p.compress().to_bytes().to_vec()).collect::<Vec<_>>().concat(),
                &key_image.0.to_vec(),
                &l_j.compress().to_bytes().to_vec(),
                &r_j.compress().to_bytes().to_vec(),
            ]);

            idx = next_idx;
        }

        // Complete the ring: compute s_pi
        s[pi] = alpha - (c[pi] * secret_key.0);

        Ok(ClsagSignature {
            key_image,
            c1: c[0],
            s,
        })
    }

    /// Verify the ring signature
    pub fn verify(&self, ring: &[ClsagPublicKey], message: &[u8]) -> Result<()> {
        let n = ring.len();
        if self.s.len() != n {
            return Err(Error::RingSignature("Invalid signature length".to_string()));
        }

        // Convert ring to points
        let ring_points: Result<Vec<RistrettoPoint>> =
            ring.iter().map(|pk| pk.to_point()).collect();
        let ring_points = ring_points?;

        let ki_point = self.key_image.to_point()?;

        // Recompute all challenges
        let mut c = vec![Scalar::ZERO; n];
        c[0] = self.c1;

        for j in 0..n {
            // Compute L_j = s_j*G + c_j*P_j
            let l_j = (&self.s[j] * RISTRETTO_BASEPOINT_TABLE) + (c[j] * ring_points[j]);

            // Compute R_j = s_j*Hp(P_j) + c_j*KI
            let hp_j = hash_to_point(&ring[j].0);
            let r_j = (hp_j * self.s[j]) + (ki_point * c[j]);

            // Compute next challenge
            let next_j = (j + 1) % n;
            let next_c = hash_to_scalar(&[
                &message.to_vec(),
                &ring_points.iter().map(|p| p.compress().to_bytes().to_vec()).collect::<Vec<_>>().concat(),
                &self.key_image.0.to_vec(),
                &l_j.compress().to_bytes().to_vec(),
                &r_j.compress().to_bytes().to_vec(),
            ]);
            
            if next_j == 0 {
                // Verify the ring closes
                if next_c != self.c1 {
                    return Err(Error::RingSignature("Ring equation verification failed".to_string()));
                }
                break;
            } else {
                c[next_j] = next_c;
            }
        }

        Ok(())
    }

    /// Get the key image (for double-signing detection)
    pub fn key_image(&self) -> &KeyImage {
        &self.key_image
    }
}

/// Hash data to a curve point (for Hp function)
fn hash_to_point(data: &[u8]) -> RistrettoPoint {
    let mut hasher = Sha512::new();
    hasher.update(b"CLSAG_HASH_TO_POINT");
    hasher.update(data);
    let hash = hasher.finalize();

    // Use hash to derive scalar, then multiply by base point
    let mut scalar_bytes = [0u8; 32];
    scalar_bytes.copy_from_slice(&hash[0..32]);
    let scalar = Scalar::from_bytes_mod_order(scalar_bytes);

    &scalar * RISTRETTO_BASEPOINT_TABLE
}

/// Hash data to a scalar (for challenges)
fn hash_to_scalar(data_parts: &[&Vec<u8>]) -> Scalar {
    let mut hasher = Sha512::new();
    hasher.update(b"CLSAG_HASH_TO_SCALAR");
    for part in data_parts {
        hasher.update(part);
    }
    let hash = hasher.finalize();

    let mut scalar_bytes = [0u8; 32];
    scalar_bytes.copy_from_slice(&hash[0..32]);
    Scalar::from_bytes_mod_order(scalar_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clsag_sign_and_verify() {
        // Create a ring with minimum size
        let mut ring = vec![];
        let mut keys = vec![];
        
        for _ in 0..MIN_RING_SIZE {
            let sk = ClsagSecretKey::generate();
            keys.push(sk.clone());
            ring.push(sk.public_key());
        }

        // Use the middle key as signer
        let signer_idx = MIN_RING_SIZE / 2;
        let message = b"tournament commitment";
        let sig = ClsagSignature::sign(&keys[signer_idx], &ring, message).unwrap();

        assert!(sig.verify(&ring, message).is_ok());
    }

    #[test]
    fn test_clsag_wrong_message() {
        // Create a ring with minimum size
        let mut ring = vec![];
        let mut keys = vec![];
        
        for _ in 0..MIN_RING_SIZE {
            let sk = ClsagSecretKey::generate();
            keys.push(sk.clone());
            ring.push(sk.public_key());
        }

        let sig = ClsagSignature::sign(&keys[0], &ring, b"original").unwrap();

        // Verification with wrong message should fail
        assert!(sig.verify(&ring, b"tampered").is_err());
    }

    #[test]
    fn test_clsag_not_in_ring() {
        let sk_outside = ClsagSecretKey::generate();
        
        // Create a ring without the signer
        let mut ring = vec![];
        for _ in 0..MIN_RING_SIZE {
            ring.push(ClsagSecretKey::generate().public_key());
        }

        let result = ClsagSignature::sign(&sk_outside, &ring, b"message");
        assert!(result.is_err());
    }

    #[test]
    fn test_key_image_linkability() {
        let sk = ClsagSecretKey::generate();
        
        // Create a ring with minimum size
        let mut ring = vec![sk.public_key()];
        for _ in 1..MIN_RING_SIZE {
            ring.push(ClsagSecretKey::generate().public_key());
        }

        let sig1 = ClsagSignature::sign(&sk, &ring, b"msg1").unwrap();
        let sig2 = ClsagSignature::sign(&sk, &ring, b"msg2").unwrap();

        // Same signer should produce same key image
        assert_eq!(sig1.key_image(), sig2.key_image());
    }

    #[test]
    fn test_different_signers_different_key_images() {
        // Create a ring with minimum size
        let mut ring = vec![];
        let mut keys = vec![];
        
        for _ in 0..MIN_RING_SIZE {
            let sk = ClsagSecretKey::generate();
            keys.push(sk.clone());
            ring.push(sk.public_key());
        }

        let sig1 = ClsagSignature::sign(&keys[0], &ring, b"msg").unwrap();
        let sig2 = ClsagSignature::sign(&keys[1], &ring, b"msg").unwrap();

        // Different signers should have different key images
        assert_ne!(sig1.key_image(), sig2.key_image());
    }

    #[test]
    fn test_wrong_ring() {
        let sk1 = ClsagSecretKey::generate();
        
        // Create ring1 with minimum size
        let mut ring1 = vec![sk1.public_key()];
        for _ in 1..MIN_RING_SIZE {
            ring1.push(ClsagSecretKey::generate().public_key());
        }
        
        // Create ring2 with minimum size (different keys)
        let mut ring2 = vec![sk1.public_key()];
        for _ in 1..MIN_RING_SIZE {
            ring2.push(ClsagSecretKey::generate().public_key());
        }

        let sig = ClsagSignature::sign(&sk1, &ring1, b"msg").unwrap();

        // Verification with different ring should fail
        assert!(sig.verify(&ring2, b"msg").is_err());
    }

    #[test]
    fn test_ring_config_default() {
        let config = RingConfig::default();
        assert_eq!(config.ring_size(), DEFAULT_RING_SIZE);
    }

    #[test]
    fn test_ring_config_valid_sizes() {
        // Minimum
        let config = RingConfig::new(MIN_RING_SIZE).unwrap();
        assert_eq!(config.ring_size(), MIN_RING_SIZE);

        // Default
        let config = RingConfig::new(DEFAULT_RING_SIZE).unwrap();
        assert_eq!(config.ring_size(), DEFAULT_RING_SIZE);

        // Maximum
        let config = RingConfig::new(MAX_RING_SIZE).unwrap();
        assert_eq!(config.ring_size(), MAX_RING_SIZE);

        // Mid-range
        let config = RingConfig::new(32).unwrap();
        assert_eq!(config.ring_size(), 32);
    }

    #[test]
    fn test_ring_config_invalid_sizes() {
        // Below minimum
        assert!(RingConfig::new(10).is_err());
        assert!(RingConfig::new(5).is_err());
        assert!(RingConfig::new(0).is_err());

        // Above maximum
        assert!(RingConfig::new(65).is_err());
        assert!(RingConfig::new(100).is_err());
    }

    #[test]
    fn test_ring_size_validation_minimum() {
        let sk1 = ClsagSecretKey::generate();
        
        // Create a ring with less than minimum size (10 < 11)
        let mut ring = vec![];
        for _ in 0..10 {
            ring.push(ClsagSecretKey::generate().public_key());
        }
        ring[0] = sk1.public_key(); // Ensure signer is in ring

        let result = ClsagSignature::sign(&sk1, &ring, b"test");
        assert!(result.is_err());
        if let Err(Error::RingSignature(msg)) = result {
            assert!(msg.contains("below minimum"));
        }
    }

    #[test]
    fn test_ring_size_validation_maximum() {
        let sk1 = ClsagSecretKey::generate();
        
        // Create a ring with more than maximum size (65 > 64)
        let mut ring = vec![];
        for _ in 0..65 {
            ring.push(ClsagSecretKey::generate().public_key());
        }
        ring[0] = sk1.public_key(); // Ensure signer is in ring

        let result = ClsagSignature::sign(&sk1, &ring, b"test");
        assert!(result.is_err());
        if let Err(Error::RingSignature(msg)) = result {
            assert!(msg.contains("exceeds maximum"));
        }
    }

    #[test]
    fn test_clsag_with_minimum_ring() {
        let sk1 = ClsagSecretKey::generate();
        
        // Create ring with exactly minimum size
        let mut ring = vec![];
        for _ in 0..MIN_RING_SIZE {
            ring.push(ClsagSecretKey::generate().public_key());
        }
        ring[5] = sk1.public_key(); // Put signer in the middle

        let message = b"test with minimum ring";
        let sig = ClsagSignature::sign(&sk1, &ring, message).unwrap();
        assert!(sig.verify(&ring, message).is_ok());
    }

    #[test]
    fn test_clsag_with_maximum_ring() {
        let sk1 = ClsagSecretKey::generate();
        
        // Create ring with exactly maximum size
        let mut ring = vec![];
        for _ in 0..MAX_RING_SIZE {
            ring.push(ClsagSecretKey::generate().public_key());
        }
        ring[32] = sk1.public_key(); // Put signer in the middle

        let message = b"test with maximum ring";
        let sig = ClsagSignature::sign(&sk1, &ring, message).unwrap();
        assert!(sig.verify(&ring, message).is_ok());
    }

    #[test]
    fn test_clsag_with_default_ring_size() {
        let sk1 = ClsagSecretKey::generate();
        
        // Create ring with default size
        let mut ring = vec![];
        for _ in 0..DEFAULT_RING_SIZE {
            ring.push(ClsagSecretKey::generate().public_key());
        }
        ring[8] = sk1.public_key(); // Put signer in the middle

        let message = b"test with default ring";
        let sig = ClsagSignature::sign(&sk1, &ring, message).unwrap();
        assert!(sig.verify(&ring, message).is_ok());
    }
}
