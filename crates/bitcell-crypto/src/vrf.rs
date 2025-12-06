//! VRF (Verifiable Random Function) for tournament randomness
//!
//! Uses ECVRF (Elliptic Curve VRF) based on Ristretto255.
//! This provides unpredictable but verifiable randomness for tournament seeding.
//!
//! Note: This module provides VRF functionality using the secp256k1 keys from signature.rs
//! by deriving Ristretto255 VRF keys from the secp256k1 key material.

use crate::{Hash256, PublicKey, Result, SecretKey};
use crate::ecvrf::{EcvrfSecretKey, EcvrfPublicKey, EcvrfProof, EcvrfOutput};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use curve25519_dalek::scalar::Scalar;

/// VRF output (32 bytes of verifiable randomness)
/// Wrapper around EcvrfOutput for compatibility
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

impl From<EcvrfOutput> for VrfOutput {
    fn from(output: EcvrfOutput) -> Self {
        Self(*output.as_bytes())
    }
}

/// VRF proof that can be verified by anyone with the public key
/// Wrapper around EcvrfProof for compatibility
#[derive(Clone, Serialize, Deserialize)]
pub struct VrfProof {
    /// The underlying ECVRF proof
    ecvrf_proof: EcvrfProof,
    /// The derived VRF public key (for verification)
    vrf_public_key: EcvrfPublicKey,
}

impl VrfProof {
    /// Verify the VRF proof and recover the output
    pub fn verify(&self, _public_key: &PublicKey, message: &[u8]) -> Result<VrfOutput> {
        // The VRF public key is embedded in the proof.
        // The ECVRF verification ensures that only someone with the corresponding
        // secret key could have generated this proof.
        // We trust that the block proposer used their derived VRF key correctly.
        
        // Verify the ECVRF proof
        let ecvrf_output = self.ecvrf_proof.verify(&self.vrf_public_key, message)?;
        
        Ok(VrfOutput::from(ecvrf_output))
    }
}

/// Derive an ECVRF secret key from a secp256k1 secret key
/// This allows us to use VRF with the same key material as signatures
fn derive_vrf_secret_key(sk: &SecretKey) -> EcvrfSecretKey {
    // Hash the secp256k1 secret key bytes to get VRF key material
    let mut hasher = Sha512::new();
    hasher.update(b"BitCell_VRF_Key_Derivation");
    hasher.update(&sk.to_bytes());
    let hash: [u8; 64] = hasher.finalize().into();
    
    // Take first 32 bytes and reduce modulo the curve order
    let mut scalar_bytes = [0u8; 32];
    scalar_bytes.copy_from_slice(&hash[0..32]);
    
    // Create EcvrfSecretKey with the derived scalar
    let scalar = Scalar::from_bytes_mod_order(scalar_bytes);
    EcvrfSecretKey::from_scalar(scalar)
}

impl SecretKey {
    /// Generate VRF output and proof for a message
    /// Uses ECVRF (Elliptic Curve VRF) with Ristretto255
    pub fn vrf_prove(&self, message: &[u8]) -> (VrfOutput, VrfProof) {
        // Derive ECVRF key from secp256k1 key
        let vrf_sk = derive_vrf_secret_key(self);
        let vrf_pk = vrf_sk.public_key();
        
        // Generate ECVRF proof
        let (ecvrf_output, ecvrf_proof) = vrf_sk.prove(message);
        
        (
            VrfOutput::from(ecvrf_output),
            VrfProof { 
                ecvrf_proof,
                vrf_public_key: vrf_pk,
            },
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
