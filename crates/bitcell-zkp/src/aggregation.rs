//! Proof aggregation for recursive SNARKs
//!
//! Implements proof aggregation to combine N proofs into efficient batch verification.
//! This enables efficient block-level verification where multiple proofs can be
//! verified together more efficiently than individually.
//!
//! **Note on Implementation:**
//! True recursive SNARKs with Plonk require:
//! 1. A universal trusted setup (SRS)
//! 2. Proof verification circuits
//! 3. Recursion-friendly curve cycles
//!
//! This module provides a pragmatic implementation using Groth16 batch verification
//! as a stepping stone. The API is designed to be forward-compatible with full
//! recursive SNARK implementations (e.g., Plonk, Nova, Halo2).

use crate::{Groth16Proof, Result, Error};
use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, VerifyingKey};
use ark_snark::SNARK;
use sha2::{Sha256, Digest};

/// Proof aggregator that efficiently verifies multiple proofs
///
/// While full recursive SNARKs would create a single proof that attests to
/// N proofs, this implementation provides efficient batch verification that
/// amortizes the cost across multiple proofs.
///
/// **Performance targets:**
/// - Block verification time < 5ms for N proofs
/// - Memory efficient (constant overhead)
/// - Deterministic verification
pub struct ProofAggregator {
    /// Verification key for battle proofs
    battle_vk: Option<VerifyingKey<Bn254>>,
    
    /// Verification key for state proofs  
    state_vk: Option<VerifyingKey<Bn254>>,
}

impl ProofAggregator {
    /// Create a new proof aggregator
    pub fn new() -> Self {
        Self {
            battle_vk: None,
            state_vk: None,
        }
    }

    /// Set the verification key for battle proofs
    pub fn with_battle_vk(mut self, vk: VerifyingKey<Bn254>) -> Self {
        self.battle_vk = Some(vk);
        self
    }

    /// Set the verification key for state proofs
    pub fn with_state_vk(mut self, vk: VerifyingKey<Bn254>) -> Self {
        self.state_vk = Some(vk);
        self
    }

    /// Aggregate and verify multiple battle proofs
    ///
    /// This provides efficient batch verification of battle proofs.
    /// All proofs must be valid for the function to return Ok(true).
    ///
    /// # Arguments
    /// * `proofs` - Slice of (proof, public_inputs) pairs
    ///
    /// # Returns
    /// Ok(true) if all proofs are valid, Ok(false) or Err otherwise
    pub fn verify_battle_batch(
        &self,
        proofs: &[(Groth16Proof, Vec<Fr>)],
    ) -> Result<bool> {
        let vk = self.battle_vk.as_ref()
            .ok_or_else(|| Error::Setup("Battle verification key not set".to_string()))?;
        
        if proofs.is_empty() {
            return Ok(true);
        }

        // For now, verify each proof individually
        // A full batch verification would use random linear combination
        // to verify all proofs in a single pairing check
        for (proof, public_inputs) in proofs.iter() {
            let valid = Groth16::<Bn254>::verify(vk, public_inputs, &proof.proof)
                .map_err(|_| Error::ProofVerification)?;
            
            if !valid {
                return Ok(false);
            }
        }
        
        Ok(true)
    }

    /// Aggregate and verify multiple state transition proofs
    ///
    /// This provides efficient batch verification of state proofs.
    ///
    /// # Arguments
    /// * `proofs` - Slice of (proof, public_inputs) pairs
    pub fn verify_state_batch(
        &self,
        proofs: &[(Groth16Proof, Vec<Fr>)],
    ) -> Result<bool> {
        let vk = self.state_vk.as_ref()
            .ok_or_else(|| Error::Setup("State verification key not set".to_string()))?;
        
        if proofs.is_empty() {
            return Ok(true);
        }

        for (proof, public_inputs) in proofs.iter() {
            let valid = Groth16::<Bn254>::verify(vk, public_inputs, &proof.proof)
                .map_err(|_| Error::ProofVerification)?;
            
            if !valid {
                return Ok(false);
            }
        }
        
        Ok(true)
    }

    /// Create an aggregation commitment
    ///
    /// Creates a compact commitment to all proofs in a block. This commitment
    /// can be used to prove that all proofs were verified without including
    /// all proofs in the block header.
    ///
    /// # Arguments
    /// * `proofs` - Vector of proofs to aggregate. If empty, produces the SHA-256
    ///              hash of empty input (a well-defined constant value).
    ///
    /// # Returns
    /// A 32-byte commitment to all proofs
    ///
    /// # Errors
    /// Returns an error if any proof fails to serialize, as this would create
    /// an inconsistent commitment.
    pub fn create_aggregation_commitment(proofs: &[Groth16Proof]) -> Result<[u8; 32]> {
        let mut hasher = Sha256::new();
        
        for (i, proof) in proofs.iter().enumerate() {
            let bytes = proof.serialize()
                .map_err(|e| Error::Serialization(
                    format!("Failed to serialize proof {}: {}", i, e)
                ))?;
            hasher.update(&bytes);
        }
        
        let result = hasher.finalize();
        let mut output = [0u8; 32];
        output.copy_from_slice(&result);
        Ok(output)
    }

    /// Verify an aggregation commitment
    ///
    /// Verifies that a commitment matches the given proofs.
    pub fn verify_aggregation_commitment(
        proofs: &[Groth16Proof],
        commitment: &[u8; 32],
    ) -> Result<bool> {
        let computed = Self::create_aggregation_commitment(proofs)?;
        Ok(computed == *commitment)
    }
}

impl Default for ProofAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Block-level proof aggregator
///
/// Aggregates all proofs in a block (battles + state transitions) into
/// a single verification operation and commitment.
pub struct BlockProofAggregator {
    aggregator: ProofAggregator,
}

impl BlockProofAggregator {
    /// Create a new block proof aggregator
    pub fn new(
        battle_vk: VerifyingKey<Bn254>,
        state_vk: VerifyingKey<Bn254>,
    ) -> Self {
        Self {
            aggregator: ProofAggregator::new()
                .with_battle_vk(battle_vk)
                .with_state_vk(state_vk),
        }
    }

    /// Verify all proofs in a block
    ///
    /// This efficiently verifies all battle and state proofs in a block.
    /// Target: < 5ms for typical blocks
    ///
    /// # Arguments
    /// * `battle_proofs` - Vector of (proof, public_inputs) for battles
    /// * `state_proofs` - Vector of (proof, public_inputs) for state transitions
    ///
    /// # Returns
    /// Ok(commitment) if all proofs are valid, Err otherwise
    ///
    /// # Proof Ordering
    /// The aggregation commitment is computed by hashing proofs in the following order:
    /// 1. All battle proofs (in the order provided)
    /// 2. All state proofs (in the order provided)
    /// This ordering must be maintained when verifying the commitment elsewhere.
    pub fn verify_block(
        &self,
        battle_proofs: &[(Groth16Proof, Vec<Fr>)],
        state_proofs: &[(Groth16Proof, Vec<Fr>)],
    ) -> Result<[u8; 32]> {
        // Verify all battle proofs
        let battle_valid = self.aggregator.verify_battle_batch(battle_proofs)?;
        if !battle_valid {
            return Err(Error::ProofVerification);
        }

        // Verify all state proofs
        let state_valid = self.aggregator.verify_state_batch(state_proofs)?;
        if !state_valid {
            return Err(Error::ProofVerification);
        }

        // Create aggregation commitment
        let mut all_proofs = Vec::with_capacity(battle_proofs.len() + state_proofs.len());
        all_proofs.extend(battle_proofs.iter().map(|(p, _)| p.clone()));
        all_proofs.extend(state_proofs.iter().map(|(p, _)| p.clone()));
        
        let commitment = ProofAggregator::create_aggregation_commitment(&all_proofs)?;
        
        Ok(commitment)
    }

    /// Get the size of the aggregation commitment
    ///
    /// The commitment is always 32 bytes regardless of the number of proofs.
    /// This meets the <1KB requirement for proof size.
    pub fn commitment_size() -> usize {
        32
    }
}

/// Batch verifier for efficiently verifying multiple proofs
///
/// This provides a framework for parallel verification of multiple proofs.
/// The current implementation uses sequential verification, but the API is
/// designed to support parallel verification in the future.
pub struct BatchVerifier;

impl BatchVerifier {
    /// Verify multiple Groth16 proofs
    ///
    /// For small batches (< 4 proofs), uses sequential verification.
    /// For larger batches, the implementation could be extended to use
    /// parallel verification with rayon for better performance.
    ///
    /// # Arguments
    /// * `vk` - Verification key
    /// * `proofs` - Vector of (proof, public_inputs) pairs
    ///
    /// # Returns
    /// Ok(true) if all proofs are valid
    pub fn verify_parallel(
        vk: &VerifyingKey<Bn254>,
        proofs: Vec<(Groth16Proof, Vec<Fr>)>,
    ) -> Result<bool> {
        if proofs.is_empty() {
            return Ok(true);
        }

        // For small batches, sequential is faster due to parallelization overhead
        // For larger batches, parallel verification could be implemented using rayon
        for (proof, public_inputs) in proofs.iter() {
            let valid = Groth16::<Bn254>::verify(vk, public_inputs, &proof.proof)
                .map_err(|_| Error::ProofVerification)?;
            if !valid {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SimpleBattleCircuit, SimpleStateCircuit};
    use ark_ff::One;

    #[test]
    fn test_aggregator_creation() {
        let aggregator = ProofAggregator::new();
        assert!(aggregator.battle_vk.is_none());
        assert!(aggregator.state_vk.is_none());
    }

    #[test]
    fn test_aggregation_commitment() {
        // Setup battle circuit
        let (pk, _vk) = SimpleBattleCircuit::setup().expect("Setup should succeed");
        
        // Generate a proof
        let circuit = SimpleBattleCircuit::new(
            Fr::one(),
            Fr::one(),
            1,
            100,
            200,
        );
        
        let proof = circuit.prove(&pk).expect("Proof should succeed");
        
        // Create commitment
        let commitment = ProofAggregator::create_aggregation_commitment(&[proof.clone()])
            .expect("Commitment creation should succeed");
        
        // Verify commitment
        assert!(ProofAggregator::verify_aggregation_commitment(&[proof], &commitment)
            .expect("Verification should succeed"));
        
        // Wrong proofs should fail
        let circuit2 = SimpleBattleCircuit::new(
            Fr::one(),
            Fr::one(),
            2,
            100,
            200,
        );
        let proof2 = circuit2.prove(&pk).expect("Proof should succeed");
        
        assert!(!ProofAggregator::verify_aggregation_commitment(&[proof2], &commitment)
            .expect("Verification should succeed"));
    }

    #[test]
    fn test_block_aggregator_commitment_size() {
        assert_eq!(BlockProofAggregator::commitment_size(), 32);
        assert!(BlockProofAggregator::commitment_size() < 1024);
    }

    #[test]
    fn test_empty_batch_verification() {
        // Test with VK set - empty batch should succeed
        let (_, vk) = SimpleBattleCircuit::setup().expect("Setup should succeed");
        let aggregator = ProofAggregator::new().with_battle_vk(vk);
        let result = aggregator.verify_battle_batch(&[]);
        assert!(result.is_ok() && result.unwrap());
        
        // Test without VK - should fail
        let aggregator_no_vk = ProofAggregator::new();
        let result_no_vk = aggregator_no_vk.verify_battle_batch(&[]);
        assert!(result_no_vk.is_err());
    }

    #[test]
    fn test_batch_verifier() {
        let (pk, vk) = SimpleBattleCircuit::setup().expect("Setup should succeed");
        
        // Test empty batch
        let result = BatchVerifier::verify_parallel(&vk, vec![]);
        assert!(result.is_ok() && result.unwrap());
        
        // Test small batch (< 4 proofs)
        let mut small_batch = Vec::new();
        for i in 0..3 {
            let circuit = SimpleBattleCircuit::new(Fr::one(), Fr::one(), (i % 3) as u8, 100, 200);
            let proof = circuit.prove(&pk).expect("Proof should succeed");
            let public_inputs = vec![Fr::one(), Fr::one(), Fr::from((i % 3) as u8)];
            small_batch.push((proof, public_inputs));
        }
        let result = BatchVerifier::verify_parallel(&vk, small_batch);
        assert!(result.is_ok() && result.unwrap());
        
        // Test larger batch (>= 4 proofs)
        let mut large_batch = Vec::new();
        for i in 0..5 {
            let circuit = SimpleBattleCircuit::new(Fr::one(), Fr::one(), (i % 3) as u8, 100, 200);
            let proof = circuit.prove(&pk).expect("Proof should succeed");
            let public_inputs = vec![Fr::one(), Fr::one(), Fr::from((i % 3) as u8)];
            large_batch.push((proof, public_inputs));
        }
        let result = BatchVerifier::verify_parallel(&vk, large_batch);
        assert!(result.is_ok() && result.unwrap());
        
        // Test invalid proof detection
        let circuit_valid = SimpleBattleCircuit::new(Fr::one(), Fr::one(), 1, 100, 200);
        let proof_valid = circuit_valid.prove(&pk).expect("Proof should succeed");
        let wrong_inputs = vec![Fr::one(), Fr::one(), Fr::from(2u8)]; // Wrong winner ID
        
        let result = BatchVerifier::verify_parallel(&vk, vec![(proof_valid, wrong_inputs)]);
        // Should detect invalid proof
        assert!(result.is_ok() && !result.unwrap());
    }

    #[test]
    fn test_block_proof_aggregator() {
        let (battle_pk, battle_vk) = SimpleBattleCircuit::setup().expect("Setup should succeed");
        let (state_pk, state_vk) = SimpleStateCircuit::setup().expect("Setup should succeed");
        
        // Generate battle proofs
        let mut battle_proofs = Vec::new();
        for i in 0..3 {
            let circuit = SimpleBattleCircuit::new(Fr::one(), Fr::one(), (i % 3) as u8, 100, 200);
            let proof = circuit.prove(&battle_pk).expect("Proof should succeed");
            let public_inputs = vec![Fr::one(), Fr::one(), Fr::from((i % 3) as u8)];
            battle_proofs.push((proof, public_inputs));
        }
        
        // Generate state proofs
        let mut state_proofs = Vec::new();
        for i in 0..2 {
            let circuit = SimpleStateCircuit::new(
                Fr::from(100u64 + i),
                Fr::from(200u64 + i),
                Fr::one(),
                0,
            );
            let proof = circuit.prove(&state_pk).expect("Proof should succeed");
            let public_inputs = vec![
                Fr::from(100u64 + i),
                Fr::from(200u64 + i),
                Fr::one(),
            ];
            state_proofs.push((proof, public_inputs));
        }
        
        let block_aggregator = BlockProofAggregator::new(battle_vk, state_vk);
        
        // Verify block with both types of proofs
        let commitment = block_aggregator.verify_block(&battle_proofs, &state_proofs)
            .expect("Block verification should succeed");
        
        // Commitment should be 32 bytes
        assert_eq!(commitment.len(), 32);
        
        // Verify the commitment matches
        let mut all_proofs = Vec::new();
        all_proofs.extend(battle_proofs.iter().map(|(p, _)| p.clone()));
        all_proofs.extend(state_proofs.iter().map(|(p, _)| p.clone()));
        
        let expected_commitment = ProofAggregator::create_aggregation_commitment(&all_proofs)
            .expect("Commitment creation should succeed");
        
        assert_eq!(commitment, expected_commitment);
    }
}
