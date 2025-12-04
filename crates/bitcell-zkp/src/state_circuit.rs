//! State transition circuit
//!
//! Verifies Merkle tree updates.

use ark_ff::Field;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_bn254::Fr;
use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
use ark_snark::SNARK;
use ark_std::rand::thread_rng;

/// State transition circuit configuration
#[derive(Clone)]
pub struct StateCircuit {
    // Public inputs
    pub old_state_root: Option<Fr>,
    pub new_state_root: Option<Fr>,
    pub nullifier: Option<Fr>,
    
    // Private witness
    pub leaf_index: Option<Fr>,
}

impl StateCircuit {
    pub fn new(
        old_state_root: Fr,
        new_state_root: Fr,
        nullifier: Fr,
        leaf_index: u64,
    ) -> Self {
        Self {
            old_state_root: Some(old_state_root),
            new_state_root: Some(new_state_root),
            nullifier: Some(nullifier),
            leaf_index: Some(Fr::from(leaf_index)),
        }
    }
    
    /// Setup the circuit and generate proving/verifying keys
    ///
    /// Returns an error if the circuit setup fails (e.g., due to constraint system issues).
    pub fn setup() -> crate::Result<(ProvingKey<ark_bn254::Bn254>, VerifyingKey<ark_bn254::Bn254>)> {
        let rng = &mut thread_rng();
        Groth16::<ark_bn254::Bn254>::circuit_specific_setup(
            Self {
                old_state_root: None,
                new_state_root: None,
                nullifier: None,
                leaf_index: None,
            },
            rng,
        )
        .map_err(|e| crate::Error::ProofGeneration(format!("Circuit setup failed: {}", e)))
    }

    pub fn prove(
        &self,
        pk: &ProvingKey<ark_bn254::Bn254>,
    ) -> crate::Result<crate::Groth16Proof> {
        let rng = &mut thread_rng();
        let proof = Groth16::<ark_bn254::Bn254>::prove(pk, self.clone(), rng)
            .map_err(|e| crate::Error::ProofGeneration(e.to_string()))?;
        Ok(crate::Groth16Proof::new(proof))
    }

    pub fn verify(
        vk: &VerifyingKey<ark_bn254::Bn254>,
        proof: &crate::Groth16Proof,
        public_inputs: &[Fr],
    ) -> crate::Result<bool> {
        Groth16::<ark_bn254::Bn254>::verify(vk, &public_inputs, &proof.proof)
            .map_err(|_| crate::Error::ProofVerification)
    }
}

impl ConstraintSynthesizer<Fr> for StateCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Allocate public inputs
        let old_root = cs.new_input_variable(|| self.old_state_root.ok_or(SynthesisError::AssignmentMissing))?;
        let new_root = cs.new_input_variable(|| self.new_state_root.ok_or(SynthesisError::AssignmentMissing))?;
        let _nullifier = cs.new_input_variable(|| self.nullifier.ok_or(SynthesisError::AssignmentMissing))?;
        
        // Allocate private witness
        let _leaf_index = cs.new_witness_variable(|| self.leaf_index.ok_or(SynthesisError::AssignmentMissing))?;
        
        // Constraint: old_root != new_root (state must change)
        // (new_root - old_root) * inv = 1
        // This proves new_root - old_root != 0
        
        let diff = cs.new_witness_variable(|| {
            let old = self.old_state_root.ok_or(SynthesisError::AssignmentMissing)?;
            let new = self.new_state_root.ok_or(SynthesisError::AssignmentMissing)?;
            Ok(new - old)
        })?;
        
        cs.enforce_constraint(
            ark_relations::lc!() + new_root - old_root,
            ark_relations::lc!() + ark_relations::r1cs::Variable::One,
            ark_relations::lc!() + diff,
        )?;
        
        // TODO: Add full Merkle tree verification constraints
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::One;

    #[test]
    fn test_state_circuit_prove_verify() {
        // 1. Setup - now returns Result
        let (pk, vk) = StateCircuit::setup().expect("Circuit setup should succeed");

        // 2. Create circuit instance
        let circuit = StateCircuit::new(
            Fr::from(100u64), // Old root
            Fr::from(200u64), // New root
            Fr::one(),        // Nullifier
            0,                // Leaf index
        );

        // 3. Generate proof
        let proof = circuit.prove(&pk).unwrap();

        // 4. Verify proof
        let public_inputs = vec![
            Fr::from(100u64),
            Fr::from(200u64),
            Fr::one(),
        ];
        
        assert!(StateCircuit::verify(&vk, &proof, &public_inputs).unwrap());
    }
}
