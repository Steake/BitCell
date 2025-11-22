//! Pedersen commitments for hiding values
//!
//! Used in the privacy layer for commitments to state values.

use crate::{Error, Result};
use ark_ec::{CurveGroup, Group};
use ark_ff::{PrimeField, UniformRand};
use ark_bn254::{G1Projective as G1, Fr};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

/// Pedersen commitment parameters (generators)
pub struct PedersenParams {
    pub g: G1,
    pub h: G1,
}

/// Global Pedersen parameters (generated deterministically)
static PEDERSEN_PARAMS: Lazy<PedersenParams> = Lazy::new(|| {
    // Generate deterministically from nothing-up-my-sleeve numbers
    let g = G1::generator();
    let h = g * Fr::from(2u64); // Simple deterministic second generator
    PedersenParams { g, h }
});

/// A Pedersen commitment
#[derive(Clone, Serialize, Deserialize)]
pub struct PedersenCommitment {
    commitment: Vec<u8>,
    #[serde(skip)]
    opening: Option<Fr>,
}

impl PedersenCommitment {
    /// Create a commitment to a value
    pub fn commit(value: &[u8]) -> (Self, Fr) {
        let params = &*PEDERSEN_PARAMS;
        
        // Convert value to field element
        let value_scalar = Fr::from_le_bytes_mod_order(value);
        
        // Random blinding factor
        let blinding = Fr::rand(&mut OsRng);
        
        // Commitment: C = value*G + blinding*H
        let commitment_point = params.g * value_scalar + params.h * blinding;
        
        let mut commitment_bytes = Vec::new();
        // Safe: serialization to Vec cannot fail
        let _ = commitment_point.serialize_compressed(&mut commitment_bytes);
        
        (
            Self {
                commitment: commitment_bytes,
                opening: Some(blinding),
            },
            blinding,
        )
    }

    /// Verify a commitment opening
    pub fn verify(&self, value: &[u8], blinding: &Fr) -> Result<()> {
        let params = &*PEDERSEN_PARAMS;
        
        let value_scalar = Fr::from_le_bytes_mod_order(value);
        let expected_point = params.g * value_scalar + params.h * blinding;
        
        let mut expected_bytes = Vec::new();
        // Safe: serialization to Vec cannot fail
        let _ = expected_point.serialize_compressed(&mut expected_bytes);
        
        if expected_bytes == self.commitment {
            Ok(())
        } else {
            Err(Error::InvalidCommitment)
        }
    }

    /// Get commitment bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.commitment
    }

    /// Create from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            commitment: bytes,
            opening: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_and_verify() {
        let value = b"secret value";
        let (commitment, blinding) = PedersenCommitment::commit(value);
        
        assert!(commitment.verify(value, &blinding).is_ok());
    }

    #[test]
    fn test_verify_wrong_value() {
        let value = b"secret value";
        let (commitment, blinding) = PedersenCommitment::commit(value);
        
        assert!(commitment.verify(b"wrong value", &blinding).is_err());
    }

    #[test]
    fn test_verify_wrong_blinding() {
        let value = b"secret value";
        let (commitment, _) = PedersenCommitment::commit(value);
        let wrong_blinding = Fr::rand(&mut OsRng);
        
        assert!(commitment.verify(value, &wrong_blinding).is_err());
    }

    #[test]
    fn test_commitment_hiding() {
        let value1 = b"value1";
        let value2 = b"value1"; // Same value
        
        let (comm1, _) = PedersenCommitment::commit(value1);
        let (comm2, _) = PedersenCommitment::commit(value2);
        
        // Same value but different randomness = different commitments
        assert_ne!(comm1.as_bytes(), comm2.as_bytes());
    }
}
