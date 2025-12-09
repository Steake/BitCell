//! ZK-SNARK circuits for BitCell
//!
//! Implements production-ready Groth16 circuits for:
//! - Battle verification (full CA evolution with Conway's rules)
//! - State transition verification (Merkle tree updates)
//! - Merkle tree inclusion proofs
//!
//! # Production Status (RC2)
//!
//! This module provides production-ready implementations with full R1CS constraints:
//! - [`BattleCircuit`]: Full Conway's Game of Life evolution verification (~6.7M constraints)
//! - [`StateCircuit`]: State transition with Merkle proofs
//!
//! # Simplified Circuits for Testing
//!
//! For fast testing and development, simplified mock circuits are available:
//! - [`SimpleBattleCircuit`]: Mock battle circuit with minimal constraints
//! - Original state circuit (via `state_circuit` module)
//!
//! # Usage
//!
//! ```rust,ignore
//! use bitcell_zkp::{BattleCircuit, Groth16Proof};
//!
//! // Setup (generate proving/verifying keys)
//! let (pk, vk) = BattleCircuit::setup()?;
//!
//! // Create circuit and generate proof
//! let circuit = BattleCircuit::new(/* ... */);
//! let proof = circuit.prove(&pk)?;
//!
//! // Verify proof
//! let public_inputs = circuit.public_inputs();
//! let valid = BattleCircuit::verify(&vk, &proof, &public_inputs)?;
//! ```

pub mod battle_circuit;
pub mod state_circuit;

// New: Full constraint implementations
pub mod battle_constraints;
pub mod state_constraints;

// Merkle tree verification gadgets
pub mod merkle_gadget;
// Production-ready Poseidon-based Merkle verification
pub mod poseidon_merkle;

// Default exports point to full constraint implementations (production-ready)
pub use battle_constraints::BattleCircuit;
pub use battle_constraints::{GRID_SIZE, BATTLE_STEPS};

// Simplified circuits available via explicit aliases for testing
pub use battle_circuit::BattleCircuit as SimpleBattleCircuit;
pub use state_circuit::StateCircuit;

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


