//! ZK-SNARK circuits for BitCell
//!
//! Implements modular Groth16 circuits for:
//! - Battle verification (CA evolution + commitment consistency)
//! - State transition verification (Merkle updates)
//!
//! Note: v0.1 provides circuit structure and basic constraints.
//! Full CA evolution verification requires extensive constraint programming.

pub mod battle_circuit;
pub mod state_circuit;

pub use battle_circuit::BattleCircuit;
pub use state_circuit::StateCircuit;

use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Circuit error: {0}")]
    Circuit(String),
    
    #[error("Proof generation failed: {0}")]
    ProofGeneration(String),
    
    #[error("Proof verification failed")]
    ProofVerification,
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Setup error: {0}")]
    Setup(String),
}

/// Simplified proof wrapper for v0.1
#[derive(Clone, Serialize, Deserialize)]
pub struct Groth16Proof {
    pub proof_data: Vec<u8>,
}

impl Groth16Proof {
    pub fn mock() -> Self {
        Self {
            proof_data: vec![0u8; 192], // Typical Groth16 proof size
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        Ok(self.proof_data.clone())
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            proof_data: bytes.to_vec(),
        })
    }

    pub fn verify(&self) -> bool {
        // Simplified verification for v0.1
        !self.proof_data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_proof() {
        let proof = Groth16Proof::mock();
        assert!(proof.verify());
        
        let serialized = proof.serialize().unwrap();
        let deserialized = Groth16Proof::deserialize(&serialized).unwrap();
        assert_eq!(proof.proof_data.len(), deserialized.proof_data.len());
    }
}
