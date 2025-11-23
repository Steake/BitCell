//! CLSAG (Concise Linkable Spontaneous Anonymous Group) Signatures
//!
//! Implements linkable ring signatures for tournament anonymity.
//! Based on the CLSAG construction from Monero.

use crate::{Error, Result};
use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_TABLE,
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

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
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
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
    pub fn sign(
        secret_key: &ClsagSecretKey,
        ring: &[ClsagPublicKey],
        message: &[u8],
    ) -> Result<Self> {
        if ring.is_empty() {
            return Err(Error::RingSignature("Empty ring".to_string()));
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
        let sk1 = ClsagSecretKey::generate();
        let sk2 = ClsagSecretKey::generate();
        let sk3 = ClsagSecretKey::generate();

        let ring = vec![sk1.public_key(), sk2.public_key(), sk3.public_key()];

        let message = b"tournament commitment";
        let sig = ClsagSignature::sign(&sk2, &ring, message).unwrap();

        assert!(sig.verify(&ring, message).is_ok());
    }

    #[test]
    fn test_clsag_wrong_message() {
        let sk1 = ClsagSecretKey::generate();
        let sk2 = ClsagSecretKey::generate();

        let ring = vec![sk1.public_key(), sk2.public_key()];

        let sig = ClsagSignature::sign(&sk1, &ring, b"original").unwrap();

        // Verification with wrong message should fail
        assert!(sig.verify(&ring, b"tampered").is_err());
    }

    #[test]
    fn test_clsag_not_in_ring() {
        let sk1 = ClsagSecretKey::generate();
        let sk2 = ClsagSecretKey::generate();
        let sk3 = ClsagSecretKey::generate();

        let ring = vec![sk1.public_key(), sk2.public_key()];

        let result = ClsagSignature::sign(&sk3, &ring, b"message");
        assert!(result.is_err());
    }

    #[test]
    fn test_key_image_linkability() {
        let sk = ClsagSecretKey::generate();
        let ring = vec![sk.public_key(), ClsagSecretKey::generate().public_key()];

        let sig1 = ClsagSignature::sign(&sk, &ring, b"msg1").unwrap();
        let sig2 = ClsagSignature::sign(&sk, &ring, b"msg2").unwrap();

        // Same signer should produce same key image
        assert_eq!(sig1.key_image(), sig2.key_image());
    }

    #[test]
    fn test_different_signers_different_key_images() {
        let sk1 = ClsagSecretKey::generate();
        let sk2 = ClsagSecretKey::generate();
        let ring = vec![sk1.public_key(), sk2.public_key()];

        let sig1 = ClsagSignature::sign(&sk1, &ring, b"msg").unwrap();
        let sig2 = ClsagSignature::sign(&sk2, &ring, b"msg").unwrap();

        // Different signers should have different key images
        assert_ne!(sig1.key_image(), sig2.key_image());
    }

    #[test]
    fn test_wrong_ring() {
        let sk1 = ClsagSecretKey::generate();
        let sk2 = ClsagSecretKey::generate();
        let sk3 = ClsagSecretKey::generate();

        let ring1 = vec![sk1.public_key(), sk2.public_key()];
        let ring2 = vec![sk1.public_key(), sk3.public_key()];

        let sig = ClsagSignature::sign(&sk1, &ring1, b"msg").unwrap();

        // Verification with different ring should fail
        assert!(sig.verify(&ring2, b"msg").is_err());
    }
}
