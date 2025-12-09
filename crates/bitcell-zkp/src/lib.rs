//! ZK-SNARK circuits for BitCell
//!
//! Implements modular Groth16 circuits for:
//! - Battle verification (CA evolution + commitment consistency)
//! - State transition verification (Merkle updates)
//! - Merkle tree inclusion proofs
//!
//! # Circuit Implementations
//!
//! ## Full Constraint Circuits (Production)
//! - [`battle_constraints::BattleCircuit`] - Full Conway's Game of Life verification (~6.7M constraints)
//! - [`state_constraints::StateCircuit`] - Complete state transition verification
//!
//! ## Simplified Circuits (Testing/Development)
//! - [`battle_circuit::BattleCircuit`] - Simplified battle circuit (mock, fast proofs)
//! - [`state_circuit::StateCircuit`] - Simplified state circuit (mock, fast proofs)
//!
//! By default, this module exports the full constraint implementations for production use.
//! Simplified circuits are available via their module paths or type aliases for testing.

pub mod battle_circuit;
pub mod state_circuit;

// Full constraint implementations (production)
pub mod battle_constraints;
pub mod state_constraints;

// Merkle tree verification gadgets
pub mod merkle_gadget;
// Production-ready Poseidon-based Merkle verification
pub mod poseidon_merkle;

// Export full constraint implementations as defaults
pub use battle_constraints::BattleCircuit;
pub use state_circuit::StateCircuit;

// Type aliases for simplified circuits (testing/development)
pub type SimpleBattleCircuit = battle_circuit::BattleCircuit;
pub type SimpleStateCircuit = state_circuit::StateCircuit;

pub use merkle_gadget::{MerklePathGadget, MERKLE_DEPTH};
pub use poseidon_merkle::{PoseidonMerkleGadget, POSEIDON_MERKLE_DEPTH};

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

use ark_bn254::Bn254;
use ark_groth16::Proof;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

/// Wrapper for Groth16 proof
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Groth16Proof {
    #[serde(with = "ark_serialize_wrapper")]
    pub proof: Proof<Bn254>,
}

mod ark_serialize_wrapper {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(proof: &Proof<Bn254>, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut bytes = Vec::new();
        proof.serialize_compressed(&mut bytes)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_bytes(&bytes)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Proof<Bn254>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        Proof::deserialize_compressed(&*bytes)
            .map_err(serde::de::Error::custom)
    }
}

impl Groth16Proof {
    pub fn new(proof: Proof<Bn254>) -> Self {
        Self { proof }
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        self.proof.serialize_compressed(&mut bytes)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        Ok(bytes)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        let proof = Proof::deserialize_compressed(bytes)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        Ok(Self { proof })
    }
}


