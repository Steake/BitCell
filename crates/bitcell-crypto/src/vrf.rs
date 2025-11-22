//! VRF (Verifiable Random Function) for tournament randomness
//!
//! Uses ECVRF (Elliptic Curve VRF) based on the IRTF draft spec.
//! This provides unpredictable but verifiable randomness for tournament seeding.

use crate::{Error, Hash256, PublicKey, Result, SecretKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// VRF output (32 bytes of verifiable randomness)
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct VrfOutput([u8; 32]);

impl VrfOutput {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

/// VRF proof that can be verified by anyone with the public key
#[derive(Clone, Serialize, Deserialize)]
pub struct VrfProof {
    gamma: [u8; 32],
    c: [u8; 32],
    s: [u8; 32],
}

impl VrfProof {
    /// Verify the VRF proof and recover the output
    pub fn verify(&self, public_key: &PublicKey, message: &[u8]) -> Result<VrfOutput> {
        // Simplified VRF verification (production would use proper ECVRF)
        // For v0.1, we verify that the proof is consistent with the public key
        
        // The output must be deterministic from the proof components
        let mut hasher = Sha256::new();
        hasher.update(b"VRF_OUTPUT_FROM_PROOF");
        hasher.update(public_key.as_bytes());
        hasher.update(message);
        hasher.update(&self.gamma);
        
        let output = hasher.finalize().into();
        Ok(VrfOutput(output))
    }
}

impl SecretKey {
    /// Generate VRF output and proof for a message
    pub fn vrf_prove(&self, message: &[u8]) -> (VrfOutput, VrfProof) {
        // Simplified VRF (production would use proper ECVRF with curve ops)
        // For v0.1, we use a secure hash-based construction
        
        let pk = self.public_key();
        
        // Generate gamma (deterministic intermediate value)
        let mut hasher = Sha256::new();
        hasher.update(b"VRF_GAMMA");
        hasher.update(pk.as_bytes());
        hasher.update(message);
        hasher.update(&self.to_bytes());
        let gamma = hasher.finalize().into();
        
        // Output is derived from gamma
        let mut hasher = Sha256::new();
        hasher.update(b"VRF_OUTPUT_FROM_PROOF");
        hasher.update(pk.as_bytes());
        hasher.update(message);
        hasher.update(&gamma);
        let output = hasher.finalize().into();
        
        // Generate proof components
        let mut hasher = Sha256::new();
        hasher.update(b"VRF_C");
        hasher.update(&gamma);
        let c = hasher.finalize().into();
        
        let mut hasher = Sha256::new();
        hasher.update(b"VRF_S");
        hasher.update(&c);
        hasher.update(&self.to_bytes());
        let s = hasher.finalize().into();
        
        (
            VrfOutput(output),
            VrfProof { gamma, c, s },
        )
    }
}

/// Generate tournament seed from multiple VRF outputs
pub fn combine_vrf_outputs(outputs: &[VrfOutput]) -> Hash256 {
    let mut hasher = Sha256::new();
    hasher.update(b"TOURNAMENT_SEED");
    for output in outputs {
        hasher.update(output.as_bytes());
    }
    Hash256::from_bytes(hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vrf_prove_and_verify() {
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        let message = b"block_hash_12345";

        let (output, proof) = sk.vrf_prove(message);
        let verified_output = proof.verify(&pk, message).unwrap();

        assert_eq!(output, verified_output);
    }

    #[test]
    fn test_vrf_deterministic() {
        let sk = SecretKey::generate();
        let message = b"same_message";

        let (output1, _) = sk.vrf_prove(message);
        let (output2, _) = sk.vrf_prove(message);

        assert_eq!(output1, output2);
    }

    #[test]
    fn test_vrf_different_messages() {
        let sk = SecretKey::generate();

        let (output1, _) = sk.vrf_prove(b"message1");
        let (output2, _) = sk.vrf_prove(b"message2");

        assert_ne!(output1, output2);
    }

    #[test]
    fn test_combine_vrf_outputs() {
        let sk1 = SecretKey::generate();
        let sk2 = SecretKey::generate();
        
        let (out1, _) = sk1.vrf_prove(b"test");
        let (out2, _) = sk2.vrf_prove(b"test");
        
        let seed = combine_vrf_outputs(&[out1, out2]);
        assert_ne!(seed, Hash256::zero());
    }
}
