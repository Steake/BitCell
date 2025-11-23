//! ECVRF (Elliptic Curve VRF) Implementation
//!
//! Implements a VRF using Ristretto255 curve operations.
//! Provides verifiable random functions for tournament randomness.

use crate::{Error, Hash256, Result};
use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_TABLE,
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

/// ECVRF public key (Ristretto point)
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct EcvrfPublicKey([u8; 32]);

impl EcvrfPublicKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    fn to_point(&self) -> Result<RistrettoPoint> {
        CompressedRistretto::from_slice(&self.0)
            .map_err(|_| Error::VrfVerification("Invalid public key".to_string()))?
            .decompress()
            .ok_or_else(|| Error::VrfVerification("Public key decompression failed".to_string()))
    }
}

/// ECVRF secret key (scalar)
#[derive(Clone)]
pub struct EcvrfSecretKey {
    scalar: Scalar,
}

impl EcvrfSecretKey {
    /// Generate a new random ECVRF key pair
    pub fn generate() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        let scalar = Scalar::from_bytes_mod_order(bytes);
        Self { scalar }
    }

    /// Get the public key (x*G)
    pub fn public_key(&self) -> EcvrfPublicKey {
        let point = &self.scalar * RISTRETTO_BASEPOINT_TABLE;
        EcvrfPublicKey(point.compress().to_bytes())
    }

    /// Prove VRF evaluation for a message
    pub fn prove(&self, alpha: &[u8]) -> (EcvrfOutput, EcvrfProof) {
        // VRF using Ristretto255

        // Get public key
        let pk = self.public_key();

        // Hash to curve: H = hash_to_curve(alpha)
        let h_point = hash_to_curve(alpha);

        // Compute Gamma = x * H
        let gamma_point = h_point * self.scalar;

        // Generate k (nonce)
        let mut hasher = Sha512::new();
        hasher.update(b"ECVRF_NONCE");
        hasher.update(&self.scalar.to_bytes());
        hasher.update(alpha);
        let nonce_bytes: [u8; 64] = hasher.finalize().into();
        let mut k_bytes = [0u8; 32];
        k_bytes.copy_from_slice(&nonce_bytes[0..32]);
        let k_scalar = Scalar::from_bytes_mod_order(k_bytes);

        // Compute k*G and k*H
        let k_g = &k_scalar * RISTRETTO_BASEPOINT_TABLE;
        let k_h = h_point * k_scalar;

        // Compute c = hash(Y, H, Gamma, k*G, k*H)
        let mut hasher = Sha512::new();
        hasher.update(b"ECVRF_CHALLENGE");
        hasher.update(pk.as_bytes());
        hasher.update(&h_point.compress().to_bytes());
        hasher.update(&gamma_point.compress().to_bytes());
        hasher.update(&k_g.compress().to_bytes());
        hasher.update(&k_h.compress().to_bytes());
        let c_hash: [u8; 64] = hasher.finalize().into();
        let mut c_bytes = [0u8; 32];
        c_bytes.copy_from_slice(&c_hash[0..32]);
        let c_scalar = Scalar::from_bytes_mod_order(c_bytes);

        // Compute s = k - c*x (mod order)
        let s_scalar = k_scalar - (c_scalar * self.scalar);

        // Derive output from Gamma
        let output = proof_to_hash(&gamma_point);

        let proof = EcvrfProof {
            gamma: gamma_point.compress().to_bytes(),
            c: c_bytes,
            s: s_scalar.to_bytes(),
        };

        (output, proof)
    }
}

/// ECVRF output (32 bytes of verifiable randomness)
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct EcvrfOutput([u8; 32]);

impl EcvrfOutput {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

/// ECVRF proof that can be verified by anyone with the public key
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EcvrfProof {
    gamma: [u8; 32],  // Gamma point (compressed)
    c: [u8; 32],       // Challenge
    s: [u8; 32],       // Response
}

impl EcvrfProof {
    /// Verify the ECVRF proof and recover the output
    pub fn verify(&self, public_key: &EcvrfPublicKey, alpha: &[u8]) -> Result<EcvrfOutput> {
        // Decompress Gamma
        let gamma_point = CompressedRistretto::from_slice(&self.gamma)
            .map_err(|_| Error::VrfVerification("Invalid gamma".to_string()))?
            .decompress()
            .ok_or_else(|| Error::VrfVerification("Gamma decompression failed".to_string()))?;

        // Hash to curve: H = hash_to_curve(alpha)
        let h_point = hash_to_curve(alpha);

        // Get public key point Y
        let y_point = public_key.to_point()?;

        // Parse c and s
        let c_scalar = Scalar::from_bytes_mod_order(self.c);
        let s_scalar = Scalar::from_bytes_mod_order(self.s);

        // Compute U = s*G + c*Y
        let u_point = (&s_scalar * RISTRETTO_BASEPOINT_TABLE) + (c_scalar * y_point);

        // Compute V = s*H + c*Gamma
        let v_point = (h_point * s_scalar) + (gamma_point * c_scalar);

        // Recompute challenge
        let mut hasher = Sha512::new();
        hasher.update(b"ECVRF_CHALLENGE");
        hasher.update(public_key.as_bytes());
        hasher.update(&h_point.compress().to_bytes());
        hasher.update(&gamma_point.compress().to_bytes());
        hasher.update(&u_point.compress().to_bytes());
        hasher.update(&v_point.compress().to_bytes());
        let computed_c_hash: [u8; 64] = hasher.finalize().into();
        let mut computed_c = [0u8; 32];
        computed_c.copy_from_slice(&computed_c_hash[0..32]);

        // Verify challenge matches
        if computed_c != self.c {
            return Err(Error::VrfVerification("Challenge mismatch".to_string()));
        }

        // Derive output from Gamma
        let output = proof_to_hash(&gamma_point);
        Ok(output)
    }
}

/// Hash arbitrary data to a curve point
fn hash_to_curve(data: &[u8]) -> RistrettoPoint {
    let mut hasher = Sha512::new();
    hasher.update(b"ECVRF_HASH_TO_CURVE");
    hasher.update(data);
    let hash_output: [u8; 64] = hasher.finalize().into();
    
    let mut scalar_bytes = [0u8; 32];
    scalar_bytes.copy_from_slice(&hash_output[0..32]);
    let scalar = Scalar::from_bytes_mod_order(scalar_bytes);
    &scalar * RISTRETTO_BASEPOINT_TABLE
}

/// Derive output hash from Gamma point
fn proof_to_hash(gamma: &RistrettoPoint) -> EcvrfOutput {
    let mut hasher = Sha512::new();
    hasher.update(b"ECVRF_PROOF_TO_HASH");
    hasher.update(&gamma.compress().to_bytes());
    let hash: [u8; 64] = hasher.finalize().into();
    let mut output = [0u8; 32];
    output.copy_from_slice(&hash[0..32]);
    EcvrfOutput(output)
}

/// Combine multiple ECVRF outputs into a single tournament seed
pub fn combine_ecvrf_outputs(outputs: &[EcvrfOutput]) -> Hash256 {
    let mut hasher = Sha512::new();
    hasher.update(b"TOURNAMENT_SEED_V2");
    for output in outputs {
        hasher.update(output.as_bytes());
    }
    let hash: [u8; 64] = hasher.finalize().into();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash[0..32]);
    Hash256::from_bytes(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecvrf_prove_and_verify() {
        let sk = EcvrfSecretKey::generate();
        let pk = sk.public_key();
        let alpha = b"block_hash_12345";

        let (output, proof) = sk.prove(alpha);
        let verified_output = proof.verify(&pk, alpha).unwrap();

        assert_eq!(output, verified_output);
    }

    #[test]
    fn test_ecvrf_deterministic() {
        let sk = EcvrfSecretKey::generate();
        let alpha = b"same_message";

        let (output1, _) = sk.prove(alpha);
        let (output2, _) = sk.prove(alpha);

        assert_eq!(output1, output2);
    }

    #[test]
    fn test_ecvrf_different_messages() {
        let sk = EcvrfSecretKey::generate();

        let (output1, _) = sk.prove(b"message1");
        let (output2, _) = sk.prove(b"message2");

        assert_ne!(output1, output2);
    }

    #[test]
    fn test_ecvrf_wrong_public_key() {
        let sk1 = EcvrfSecretKey::generate();
        let sk2 = EcvrfSecretKey::generate();
        let pk2 = sk2.public_key();
        
        let alpha = b"test_message";
        let (_, proof) = sk1.prove(alpha);
        
        // Verification with wrong key should fail
        let result = proof.verify(&pk2, alpha);
        assert!(result.is_err());
    }

    #[test]
    fn test_ecvrf_wrong_message() {
        let sk = EcvrfSecretKey::generate();
        let pk = sk.public_key();
        
        let (_, proof) = sk.prove(b"original");
        
        // Verification with wrong message should fail
        let result = proof.verify(&pk, b"tampered");
        assert!(result.is_err());
    }

    #[test]
    fn test_combine_outputs() {
        let sk1 = EcvrfSecretKey::generate();
        let sk2 = EcvrfSecretKey::generate();
        
        let (out1, _) = sk1.prove(b"test");
        let (out2, _) = sk2.prove(b"test");
        
        let seed = combine_ecvrf_outputs(&[out1, out2]);
        assert_ne!(seed, Hash256::zero());
    }
}
