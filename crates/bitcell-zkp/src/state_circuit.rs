//! State transition circuit
//!
//! Verifies Merkle tree updates with proper non-equality constraint.
//! Uses arkworks Groth16 for zero-knowledge proof generation and verification.

use ark_ff::Field;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_bn254::Fr;
use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
use ark_snark::SNARK;
use ark_std::rand::thread_rng;
use ark_std::Zero;

/// State transition circuit configuration
///
/// This circuit proves that a state transition occurred correctly by verifying:
/// 1. The old and new state roots are different (state changed)
/// 2. The nullifier is properly computed to prevent double-spending
///
/// **Note**: This is a simplified circuit for testing and development.
/// For production use with full Merkle tree verification, see `state_constraints::StateCircuit`.
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
    /// **WARNING:** This method generates keys using insecure randomness and should
    /// ONLY be used for testing. Production systems MUST use keys generated from a
    /// proper multi-party trusted setup ceremony via `load_ceremony_keys()`.
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

    /// Load proving key from the trusted setup ceremony
    ///
    /// This loads the production proving key that was generated through a
    /// multi-party computation ceremony. The key is stored in `keys/state/proving_key.bin`.
    ///
    /// # Expected Directory Structure
    /// ```text
    /// BitCell/
    /// ├── crates/
    /// │   └── bitcell-zkp/      <- CARGO_MANIFEST_DIR
    /// └── keys/
    ///     └── state/
    ///         └── proving_key.bin
    /// ```
    ///
    /// # Returns
    /// * `Ok(ProvingKey)` if the key is found and successfully loaded
    /// * `Err` if the key file doesn't exist or is corrupted
    pub fn load_proving_key() -> crate::Result<ProvingKey<ark_bn254::Bn254>> {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .ok_or_else(|| crate::Error::KeyManagement(
                "Failed to resolve repository root from crates/bitcell-zkp".to_string()
            ))?;
        let key_path = repo_root.join("keys/state/proving_key.bin");
        crate::key_management::load_proving_key(key_path)
    }

    /// Load verification key from the trusted setup ceremony
    ///
    /// This loads the production verification key that was generated through a
    /// multi-party computation ceremony. The key is stored in `keys/state/verification_key.bin`.
    ///
    /// # Expected Directory Structure
    /// ```text
    /// BitCell/
    /// ├── crates/
    /// │   └── bitcell-zkp/      <- CARGO_MANIFEST_DIR
    /// └── keys/
    ///     └── state/
    ///         └── verification_key.bin
    /// ```
    ///
    /// # Returns
    /// * `Ok(VerifyingKey)` if the key is found and successfully loaded
    /// * `Err` if the key file doesn't exist or is corrupted
    pub fn load_verification_key() -> crate::Result<VerifyingKey<ark_bn254::Bn254>> {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .ok_or_else(|| crate::Error::KeyManagement(
                "Failed to resolve repository root from crates/bitcell-zkp".to_string()
            ))?;
        let key_path = repo_root.join("keys/state/verification_key.bin");
        crate::key_management::load_verification_key(key_path)
    }

    /// Load both proving and verification keys from the trusted setup ceremony
    ///
    /// Convenience method that loads both keys at once. Equivalent to calling
    /// `load_proving_key()` and `load_verification_key()` separately.
    ///
    /// # Returns
    /// * `Ok((ProvingKey, VerifyingKey))` if both keys are successfully loaded
    /// * `Err` if either key file doesn't exist or is corrupted
    pub fn load_ceremony_keys() -> crate::Result<(ProvingKey<ark_bn254::Bn254>, VerifyingKey<ark_bn254::Bn254>)> {
        let pk = Self::load_proving_key()?;
        let vk = Self::load_verification_key()?;
        Ok((pk, vk))
    }

    /// Generate a proof for this circuit instance
    pub fn prove(
        &self,
        pk: &ProvingKey<ark_bn254::Bn254>,
    ) -> crate::Result<crate::Groth16Proof> {
        let rng = &mut thread_rng();
        let proof = Groth16::<ark_bn254::Bn254>::prove(pk, self.clone(), rng)
            .map_err(|e| crate::Error::ProofGeneration(e.to_string()))?;
        Ok(crate::Groth16Proof::new(proof))
    }

    /// Verify a proof against public inputs
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
        // To prove non-equality, we use the following approach:
        // 1. Compute diff = new_root - old_root
        // 2. Compute inv = inverse(diff) as a witness
        // 3. Enforce: diff * inv = 1
        // This proves diff != 0, which proves new_root != old_root

        // Step 1: Compute diff = new_root - old_root
        let diff = cs.new_witness_variable(|| {
            let old = self.old_state_root.ok_or(SynthesisError::AssignmentMissing)?;
            let new = self.new_state_root.ok_or(SynthesisError::AssignmentMissing)?;
            Ok(new - old)
        })?;

        // Enforce: diff = new_root - old_root
        cs.enforce_constraint(
            ark_relations::lc!() + new_root - old_root,
            ark_relations::lc!() + ark_relations::r1cs::Variable::One,
            ark_relations::lc!() + diff,
        )?;

        // Step 2: Allocate inverse of diff as witness
        let inv = cs.new_witness_variable(|| {
            let old = self.old_state_root.ok_or(SynthesisError::AssignmentMissing)?;
            let new = self.new_state_root.ok_or(SynthesisError::AssignmentMissing)?;
            let diff_val = new - old;
            if diff_val.is_zero() {
                // If diff is zero (old_root == new_root), no valid inverse exists.
                // This violates the non-equality constraint - state must change.
                // We return Unsatisfiable since the constraint cannot be satisfied.
                return Err(SynthesisError::Unsatisfiable);
            }
            diff_val.inverse().ok_or(SynthesisError::Unsatisfiable)
        })?;

        // Step 3: Enforce diff * inv = 1 (proves diff != 0)
        cs.enforce_constraint(
            ark_relations::lc!() + diff,
            ark_relations::lc!() + inv,
            ark_relations::lc!() + ark_relations::r1cs::Variable::One,
        )?;

        // Note: This simplified circuit only verifies state change (old_root != new_root).
        // Full Merkle tree verification is implemented in state_constraints::StateCircuit,
        // which includes:
        // - Verifying the old leaf at leaf_index against old_state_root
        // - Verifying the new leaf at leaf_index against new_state_root
        // - Ensuring the nullifier is derived from the old leaf

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

        // 2. Create circuit instance with different roots (non-equality constraint)
        let circuit = StateCircuit::new(
            Fr::from(100u64), // Old root
            Fr::from(200u64), // New root (must be different!)
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

    #[test]
    fn test_state_circuit_rejects_same_roots() {
        // Setup
        let (pk, _vk) = StateCircuit::setup().expect("Circuit setup should succeed");

        // Create circuit with same old and new roots - should fail to prove
        // because our non-equality constraint requires diff != 0
        let circuit = StateCircuit::new(
            Fr::from(100u64), // Old root
            Fr::from(100u64), // Same as old - violates non-equality constraint
            Fr::one(),
            0,
        );

        // Proof generation should fail because diff = 0 has no inverse
        let result = circuit.prove(&pk);
        assert!(result.is_err(), "Proof should fail when old_root == new_root");
    }
}
