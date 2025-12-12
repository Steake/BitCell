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

    /// Create ECVRF secret key from a scalar
    /// Used for deterministic key derivation
    pub fn from_scalar(scalar: Scalar) -> Self {
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

    /// Test vector 1: Fixed key produces deterministic output
    /// This test vector can be used to verify implementation consistency
    #[test]
    fn test_vector_deterministic_fixed_key() {
        // Create a deterministic key from fixed bytes
        let scalar_bytes = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
        ];
        let scalar = Scalar::from_bytes_mod_order(scalar_bytes);
        let sk = EcvrfSecretKey::from_scalar(scalar);
        let pk = sk.public_key();

        // Test with a specific message
        let message = b"BitCell_ECVRF_TestVector_1";
        let (output1, proof1) = sk.prove(message);
        
        // Verify the proof
        let verified_output = proof1.verify(&pk, message)
            .expect("Test vector proof should verify");
        assert_eq!(output1, verified_output);

        // Generate again with same key and message - should be identical
        let (output2, _proof2) = sk.prove(message);
        assert_eq!(output1, output2, "VRF output must be deterministic");

        // Different message should produce different output
        let (output3, _) = sk.prove(b"different_message");
        assert_ne!(output1, output3, "Different messages must produce different outputs");
    }

    /// Test vector 2: VRF chaining for block proposer selection
    /// Simulates the blockchain use case where each block's VRF uses the previous output
    #[test]
    fn test_vector_vrf_chaining() {
        let sk = EcvrfSecretKey::generate();
        let pk = sk.public_key();

        // Genesis block: use fixed seed
        let genesis_seed = b"BitCell_Genesis_Block_Seed";
        let (output0, proof0) = sk.prove(genesis_seed);
        
        // Verify genesis proof
        let verified0 = proof0.verify(&pk, genesis_seed)
            .expect("Genesis VRF should verify");
        assert_eq!(output0, verified0);

        // Block 1: use genesis VRF output as input
        let (output1, proof1) = sk.prove(output0.as_bytes());
        let verified1 = proof1.verify(&pk, output0.as_bytes())
            .expect("Block 1 VRF should verify");
        assert_eq!(output1, verified1);

        // Block 2: use block 1 VRF output as input
        let (output2, proof2) = sk.prove(output1.as_bytes());
        let verified2 = proof2.verify(&pk, output1.as_bytes())
            .expect("Block 2 VRF should verify");
        assert_eq!(output2, verified2);

        // Verify outputs are all different (non-trivial chaining)
        assert_ne!(output0, output1);
        assert_ne!(output1, output2);
        assert_ne!(output0, output2);

        // Verify determinism: regenerating the chain produces same outputs
        let (output0_again, _) = sk.prove(genesis_seed);
        assert_eq!(output0, output0_again);
        
        let (output1_again, _) = sk.prove(output0_again.as_bytes());
        assert_eq!(output1, output1_again);
        
        let (output2_again, _) = sk.prove(output1_again.as_bytes());
        assert_eq!(output2, output2_again);
    }

    /// Test vector 3: Multiple proposers
    /// Verifies that different validators produce different VRF outputs
    #[test]
    fn test_vector_multiple_proposers() {
        // Create 3 different validator keys
        let sk1 = EcvrfSecretKey::generate();
        let sk2 = EcvrfSecretKey::generate();
        let sk3 = EcvrfSecretKey::generate();

        let pk1 = sk1.public_key();
        let pk2 = sk2.public_key();
        let pk3 = sk3.public_key();

        // All validators use the same block hash as VRF input
        let block_hash = b"shared_block_hash_for_proposer_selection";

        // Each validator generates their VRF proof
        let (output1, proof1) = sk1.prove(block_hash);
        let (output2, proof2) = sk2.prove(block_hash);
        let (output3, proof3) = sk3.prove(block_hash);

        // Verify all proofs
        let verified1 = proof1.verify(&pk1, block_hash)
            .expect("Validator 1 proof should verify");
        let verified2 = proof2.verify(&pk2, block_hash)
            .expect("Validator 2 proof should verify");
        let verified3 = proof3.verify(&pk3, block_hash)
            .expect("Validator 3 proof should verify");

        assert_eq!(output1, verified1);
        assert_eq!(output2, verified2);
        assert_eq!(output3, verified3);

        // Different validators produce different outputs for same input
        assert_ne!(output1, output2);
        assert_ne!(output2, output3);
        assert_ne!(output1, output3);

        // Wrong key cannot verify another validator's proof
        assert!(proof1.verify(&pk2, block_hash).is_err(),
                "Validator 2 key should not verify Validator 1's proof");
        assert!(proof2.verify(&pk3, block_hash).is_err(),
                "Validator 3 key should not verify Validator 2's proof");
    }

    /// Test vector 4: Proof serialization roundtrip
    /// Verifies that proofs can be serialized and deserialized correctly
    #[test]
    fn test_vector_proof_serialization() {
        let sk = EcvrfSecretKey::generate();
        let pk = sk.public_key();
        let message = b"serialization_test_message";

        let (output, proof) = sk.prove(message);

        // Serialize the proof
        let serialized = bincode::serialize(&proof)
            .expect("Proof should serialize");

        // Deserialize the proof
        let deserialized_proof: EcvrfProof = bincode::deserialize(&serialized)
            .expect("Proof should deserialize");

        // Verify the deserialized proof produces same output
        let verified_output = deserialized_proof.verify(&pk, message)
            .expect("Deserialized proof should verify");
        
        assert_eq!(output, verified_output);

        // Verify proof size is reasonable (gamma + c + s = 32 + 32 + 32 = 96 bytes minimum)
        assert!(serialized.len() >= 96, "Proof size should be at least 96 bytes");
        assert!(serialized.len() < 200, "Proof size should be compact (< 200 bytes)");
    }

    /// Test vector 5: Grinding resistance
    /// Verifies that changing a single bit in the message produces a completely different output
    #[test]
    fn test_vector_grinding_resistance() {
        let sk = EcvrfSecretKey::generate();
        
        // Original message
        let mut message1 = vec![0u8; 32];
        message1[0] = 0xAA;
        
        // Message with single bit flipped
        let mut message2 = message1.clone();
        message2[0] = 0xAB; // Changed from 0xAA to 0xAB (1 bit flip)

        let (output1, _) = sk.prove(&message1);
        let (output2, _) = sk.prove(&message2);

        // Even a single bit change should produce completely different output
        // (avalanche effect)
        assert_ne!(output1, output2);

        // Count differing bits to ensure good avalanche effect
        let mut diff_bits = 0;
        for i in 0..32 {
            diff_bits += (output1.as_bytes()[i] ^ output2.as_bytes()[i]).count_ones();
        }
        
        // With good cryptographic hashing, about 50% of bits should differ
        // We check for at least 25% to be conservative
        assert!(diff_bits >= 64, 
                "Single bit change should affect many output bits (avalanche effect), got {} differing bits", 
                diff_bits);
    }

    /// Test vector 6: Non-malleability
    /// Verifies that proof components cannot be tampered with
    #[test]
    fn test_vector_non_malleability() {
        let sk = EcvrfSecretKey::generate();
        let pk = sk.public_key();
        let message = b"non_malleability_test";

        let (_output, mut proof) = sk.prove(message);

        // Try to tamper with gamma
        let original_gamma = proof.gamma;
        proof.gamma[0] ^= 0x01; // Flip one bit
        assert!(proof.verify(&pk, message).is_err(),
                "Tampered gamma should fail verification");
        proof.gamma = original_gamma; // Restore

        // Try to tamper with c (challenge)
        let original_c = proof.c;
        proof.c[0] ^= 0x01;
        assert!(proof.verify(&pk, message).is_err(),
                "Tampered challenge should fail verification");
        proof.c = original_c; // Restore

        // Try to tamper with s (response)
        let original_s = proof.s;
        proof.s[0] ^= 0x01;
        assert!(proof.verify(&pk, message).is_err(),
                "Tampered response should fail verification");
        proof.s = original_s; // Restore

        // Original proof should still verify after restoration
        assert!(proof.verify(&pk, message).is_ok(),
                "Restored proof should verify");
    }
}
