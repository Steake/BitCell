//! ZK-SNARK circuits for BitCell
//!
//! Implements modular circuits for:
//! - Battle verification (CA evolution + commitment consistency)
//! - State transition verification (Merkle updates)
//! - Merkle tree inclusion proofs
//! - Proof aggregation for efficient block verification
//!
//! ## Circuit Implementations
//!
//! This crate provides two tiers of circuit implementations:
//!
//! ### Simplified Circuits (battle_circuit, state_circuit)
//! - **Purpose**: Fast testing, development, and basic validation
//! - **Constraints**: Minimal (winner validation, root non-equality)
//! - **Performance**: Very fast proof generation (~1-2 seconds)
//! - **Security**: Cryptographically sound but doesn't verify full computation
//!
//! ### Full Constraint Circuits (battle_constraints, state_constraints)
//! - **Purpose**: Production deployment with complete verification
//! - **Constraints**: Complete CA evolution simulation and Merkle tree verification
//! - **Performance**: Slower proof generation (30-60 seconds for battles)
//! - **Security**: Fully verifies all computation steps
//!
//! ## Proof Aggregation
//!
//! **Proof System:**
//! Currently uses Groth16 with batch verification and aggregation commitments.
//! The aggregation module provides a forward-compatible API for future migration
//! to universal SNARKs (Plonk, Halo2) or recursive schemes (Nova, Arecibo).
//!
//! ## Usage
//!
//! ```rust,ignore
//! use bitcell_zkp::{battle_constraints::BattleCircuit, Groth16Proof};
//! use ark_bn254::Fr;
//!
//! // Setup (one-time, reusable)
//! let (pk, vk) = BattleCircuit::<Fr>::setup().unwrap();
//!
//! // Create circuit instance
//! let circuit = BattleCircuit::new(
//!     initial_grid,
//!     final_grid,
//!     commitment_a,
//!     commitment_b,
//!     winner_id,
//! ).with_witnesses(pattern_a, pattern_b, nonce_a, nonce_b);
//!
//! // Generate proof
//! let proof = circuit.prove(&pk).unwrap();
//!
//! // Verify proof
//! let public_inputs = circuit.public_inputs();
//! assert!(BattleCircuit::verify(&vk, &proof, &public_inputs).unwrap());
//! ```

// Groth16 circuits
pub mod battle_circuit;
pub mod state_circuit;

// Full constraint implementations for production
pub mod battle_constraints;
pub mod state_constraints;

// Merkle tree verification gadgets
pub mod merkle_gadget;
pub mod poseidon_merkle;

// Key management for trusted setup ceremony
pub mod key_management;

// Proof aggregation and batch verification
pub mod aggregation;

// Export simplified circuits for backward compatibility
pub use battle_circuit::BattleCircuit as SimpleBattleCircuit;
pub use state_circuit::StateCircuit as SimpleStateCircuit;

// Export full circuits as recommended defaults
pub use battle_constraints::BattleCircuit;
pub use state_constraints::{StateCircuit, NullifierCircuit};

pub use merkle_gadget::{MerklePathGadget, MERKLE_DEPTH};
pub use poseidon_merkle::{PoseidonMerkleGadget, POSEIDON_MERKLE_DEPTH};

// Aggregation exports
pub use aggregation::{ProofAggregator, BlockProofAggregator, BatchVerifier};

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
    
    #[error("Key management error: {0}")]
    KeyManagement(String),
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
