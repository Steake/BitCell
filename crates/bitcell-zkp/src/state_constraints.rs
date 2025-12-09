/// State transition circuit implementing Merkle tree verification
/// This module provides R1CS constraints for verifying state updates

use ark_ff::PrimeField;
use ark_r1cs_std::prelude::*;
use ark_r1cs_std::fields::fp::FpVar;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
use ark_snark::SNARK;
use ark_std::rand::thread_rng;

/// Merkle tree depth
pub const MERKLE_DEPTH: usize = 32;

/// State transition circuit
#[derive(Clone)]
pub struct StateCircuit<F: PrimeField> {
    /// Old state root (public)
    pub old_root: Option<F>,
    /// New state root (public)
    pub new_root: Option<F>,
    /// Nullifier (public)
    pub nullifier: Option<F>,
    /// New commitment (public)
    pub commitment: Option<F>,
    /// Leaf value (private)
    pub leaf: Option<F>,
    /// Merkle path (private)
    pub path: Option<Vec<F>>,
    /// Path indices (private)
    pub indices: Option<Vec<bool>>,
    /// New leaf value (private)
    pub new_leaf: Option<F>,
}

impl<F: PrimeField> StateCircuit<F> {
    pub fn new(old_root: F, new_root: F, nullifier: F, commitment: F) -> Self {
        Self {
            old_root: Some(old_root),
            new_root: Some(new_root),
            nullifier: Some(nullifier),
            commitment: Some(commitment),
            leaf: None,
            path: None,
            indices: None,
            new_leaf: None,
        }
    }

    pub fn with_witnesses(
        mut self,
        leaf: F,
        path: Vec<F>,
        indices: Vec<bool>,
        new_leaf: F,
    ) -> Self {
        self.leaf = Some(leaf);
        self.path = Some(path);
        self.indices = Some(indices);
        self.new_leaf = Some(new_leaf);
        self
    }
}

impl StateCircuit<Fr> {
    /// Setup the circuit and generate proving/verifying keys
    pub fn setup() -> crate::Result<(ProvingKey<Bn254>, VerifyingKey<Bn254>)> {
        let rng = &mut thread_rng();
        let dummy_circuit = Self {
            old_root: None,
            new_root: None,
            nullifier: None,
            commitment: None,
            leaf: None,
            path: None,
            indices: None,
            new_leaf: None,
        };
        
        Groth16::<Bn254>::circuit_specific_setup(dummy_circuit, rng)
            .map_err(|e| crate::Error::Setup(format!("StateCircuit setup failed: {}", e)))
    }

    /// Generate a proof for this circuit instance
    pub fn prove(&self, pk: &ProvingKey<Bn254>) -> crate::Result<crate::Groth16Proof> {
        let rng = &mut thread_rng();
        let proof = Groth16::<Bn254>::prove(pk, self.clone(), rng)
            .map_err(|e| crate::Error::ProofGeneration(e.to_string()))?;
        Ok(crate::Groth16Proof::new(proof))
    }

    /// Verify a proof against public inputs
    pub fn verify(
        vk: &VerifyingKey<Bn254>,
        proof: &crate::Groth16Proof,
        public_inputs: &[Fr],
    ) -> crate::Result<bool> {
        Groth16::<Bn254>::verify(vk, public_inputs, &proof.proof)
            .map_err(|_| crate::Error::ProofVerification)
    }

    /// Helper to construct public inputs in the correct order
    pub fn public_inputs(&self) -> Vec<Fr> {
        vec![
            self.old_root.unwrap_or(Fr::from(0u64)),
            self.new_root.unwrap_or(Fr::from(0u64)),
            self.nullifier.unwrap_or(Fr::from(0u64)),
            self.commitment.unwrap_or(Fr::from(0u64)),
        ]
    }
}

impl<F: PrimeField> ConstraintSynthesizer<F> for StateCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // Allocate public inputs
        let old_root_var = FpVar::new_input(cs.clone(), || {
            self.old_root.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let new_root_var = FpVar::new_input(cs.clone(), || {
            self.new_root.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let nullifier_var = FpVar::new_input(cs.clone(), || {
            self.nullifier.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let commitment_var = FpVar::new_input(cs.clone(), || {
            self.commitment.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        // Allocate private witnesses
        let leaf_var = FpVar::new_witness(cs.clone(), || {
            self.leaf.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let path_vars: Vec<FpVar<F>> = self
            .path
            .as_ref()
            .ok_or(SynthesisError::AssignmentMissing)?
            .iter()
            .map(|&p| FpVar::new_witness(cs.clone(), || Ok(p)))
            .collect::<Result<Vec<_>, _>>()?;
        
        let indices_vars: Vec<Boolean<F>> = self
            .indices
            .as_ref()
            .ok_or(SynthesisError::AssignmentMissing)?
            .iter()
            .map(|&b| Boolean::new_witness(cs.clone(), || Ok(b)))
            .collect::<Result<Vec<_>, _>>()?;
        
        let new_leaf_var = FpVar::new_witness(cs.clone(), || {
            self.new_leaf.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        // Constraint 1: Verify Merkle path for old leaf
        let computed_old_root = compute_merkle_root(
            cs.clone(),
            &leaf_var,
            &path_vars,
            &indices_vars,
        )?;
        computed_old_root.enforce_equal(&old_root_var)?;
        
        // Constraint 2: Verify nullifier derivation
        // nullifier = H(leaf)
        let computed_nullifier = hash_single(cs.clone(), &leaf_var)?;
        computed_nullifier.enforce_equal(&nullifier_var)?;
        
        // Constraint 3: Verify commitment for new leaf
        // commitment = H(new_leaf)
        let computed_commitment = hash_single(cs.clone(), &new_leaf_var)?;
        computed_commitment.enforce_equal(&commitment_var)?;
        
        // Constraint 4: Verify Merkle path for new leaf
        let computed_new_root = compute_merkle_root(
            cs.clone(),
            &new_leaf_var,
            &path_vars,
            &indices_vars,
        )?;
        computed_new_root.enforce_equal(&new_root_var)?;
        
        Ok(())
    }
}

/// Compute Merkle root from leaf and path
fn compute_merkle_root<F: PrimeField>(
    cs: ConstraintSystemRef<F>,
    leaf: &FpVar<F>,
    path: &[FpVar<F>],
    indices: &[Boolean<F>],
) -> Result<FpVar<F>, SynthesisError> {
    assert_eq!(path.len(), indices.len());
    assert_eq!(path.len(), MERKLE_DEPTH);
    
    let mut current = leaf.clone();
    
    for (sibling, index) in path.iter().zip(indices.iter()) {
        // If index is 0, hash(current, sibling)
        // If index is 1, hash(sibling, current)
        let (left, right) = (
            FpVar::conditionally_select(index, sibling, &current)?,
            FpVar::conditionally_select(index, &current, sibling)?,
        );
        
        current = hash_pair(cs.clone(), &left, &right)?;
    }
    
    Ok(current)
}

/// Hash a single field element (simplified hash function)
fn hash_single<F: PrimeField>(
    _cs: ConstraintSystemRef<F>,
    input: &FpVar<F>,
) -> Result<FpVar<F>, SynthesisError> {
    // Simplified hash: H(x) = x^2 + x + 1
    // In production, use Poseidon or another SNARK-friendly hash
    let squared = input.square()?;
    let result = &squared + input + FpVar::one();
    Ok(result)
}

/// Hash a pair of field elements
fn hash_pair<F: PrimeField>(
    _cs: ConstraintSystemRef<F>,
    left: &FpVar<F>,
    right: &FpVar<F>,
) -> Result<FpVar<F>, SynthesisError> {
    // Simplified hash: H(x, y) = x^2 + y^2 + x*y + 1
    // In production, use Poseidon or another SNARK-friendly hash
    let left_sq = left.square()?;
    let right_sq = right.square()?;
    let product = left * right;
    let result = &left_sq + &right_sq + &product + FpVar::one();
    Ok(result)
}

/// Nullifier set membership circuit
#[derive(Clone)]
pub struct NullifierCircuit<F: PrimeField> {
    /// Nullifier to check (public)
    pub nullifier: Option<F>,
    /// Nullifier set root (public)
    pub set_root: Option<F>,
    /// Is member (public - 1 if member, 0 if not)
    pub is_member: Option<bool>,
    /// Merkle path (private)
    pub path: Option<Vec<F>>,
    /// Path indices (private)
    pub indices: Option<Vec<bool>>,
}

impl<F: PrimeField> NullifierCircuit<F> {
    pub fn new(nullifier: F, set_root: F, is_member: bool) -> Self {
        Self {
            nullifier: Some(nullifier),
            set_root: Some(set_root),
            is_member: Some(is_member),
            path: None,
            indices: None,
        }
    }

    pub fn with_witnesses(mut self, path: Vec<F>, indices: Vec<bool>) -> Self {
        self.path = Some(path);
        self.indices = Some(indices);
        self
    }
}

impl NullifierCircuit<Fr> {
    /// Setup the circuit and generate proving/verifying keys
    pub fn setup() -> crate::Result<(ProvingKey<Bn254>, VerifyingKey<Bn254>)> {
        let rng = &mut thread_rng();
        let dummy_circuit = Self {
            nullifier: None,
            set_root: None,
            is_member: None,
            path: None,
            indices: None,
        };
        
        Groth16::<Bn254>::circuit_specific_setup(dummy_circuit, rng)
            .map_err(|e| crate::Error::Setup(format!("NullifierCircuit setup failed: {}", e)))
    }

    /// Generate a proof for this circuit instance
    pub fn prove(&self, pk: &ProvingKey<Bn254>) -> crate::Result<crate::Groth16Proof> {
        let rng = &mut thread_rng();
        let proof = Groth16::<Bn254>::prove(pk, self.clone(), rng)
            .map_err(|e| crate::Error::ProofGeneration(e.to_string()))?;
        Ok(crate::Groth16Proof::new(proof))
    }

    /// Verify a proof against public inputs
    pub fn verify(
        vk: &VerifyingKey<Bn254>,
        proof: &crate::Groth16Proof,
        public_inputs: &[Fr],
    ) -> crate::Result<bool> {
        Groth16::<Bn254>::verify(vk, public_inputs, &proof.proof)
            .map_err(|_| crate::Error::ProofVerification)
    }

    /// Helper to construct public inputs in the correct order
    pub fn public_inputs(&self) -> Vec<Fr> {
        vec![
            self.nullifier.unwrap_or(Fr::from(0u64)),
            self.set_root.unwrap_or(Fr::from(0u64)),
            Fr::from(if self.is_member.unwrap_or(false) { 1u64 } else { 0u64 }),
        ]
    }
}

impl<F: PrimeField> ConstraintSynthesizer<F> for NullifierCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // Allocate public inputs
        let nullifier_var = FpVar::new_input(cs.clone(), || {
            self.nullifier.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let set_root_var = FpVar::new_input(cs.clone(), || {
            self.set_root.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let is_member_var = Boolean::new_input(cs.clone(), || {
            self.is_member.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        // Allocate private witnesses
        let path_vars: Vec<FpVar<F>> = self
            .path
            .as_ref()
            .ok_or(SynthesisError::AssignmentMissing)?
            .iter()
            .map(|&p| FpVar::new_witness(cs.clone(), || Ok(p)))
            .collect::<Result<Vec<_>, _>>()?;
        
        let indices_vars: Vec<Boolean<F>> = self
            .indices
            .as_ref()
            .ok_or(SynthesisError::AssignmentMissing)?
            .iter()
            .map(|&b| Boolean::new_witness(cs.clone(), || Ok(b)))
            .collect::<Result<Vec<_>, _>>()?;
        
        // Compute Merkle root
        let computed_root = compute_merkle_root(
            cs.clone(),
            &nullifier_var,
            &path_vars,
            &indices_vars,
        )?;
        
        // If is_member, roots must match
        // If not is_member, roots can differ
        let roots_equal = computed_root.is_eq(&set_root_var)?;
        let should_be_equal = is_member_var.clone();
        
        roots_equal.enforce_equal(&should_be_equal)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;
    use ark_relations::r1cs::ConstraintSystem;
    
    #[test]
    fn test_state_circuit_satisfiable() {
        let cs = ConstraintSystem::<Fr>::new_ref();
        
        let leaf = Fr::from(100u64);
        let new_leaf = Fr::from(200u64);
        
        // Create a simple path (all zeros for simplicity)
        let path = vec![Fr::from(0u64); MERKLE_DEPTH];
        let indices = vec![false; MERKLE_DEPTH];
        
        // Compute roots manually
        let mut old_root = leaf;
        for i in 0..MERKLE_DEPTH {
            let left = if indices[i] { path[i] } else { old_root };
            let right = if indices[i] { old_root } else { path[i] };
            // Use simplified hash
            old_root = left * left + right * right + left * right + Fr::from(1u64);
        }
        
        let mut new_root = new_leaf;
        for i in 0..MERKLE_DEPTH {
            let left = if indices[i] { path[i] } else { new_root };
            let right = if indices[i] { new_root } else { path[i] };
            new_root = left * left + right * right + left * right + Fr::from(1u64);
        }
        
        // Compute nullifier and commitment
        let nullifier = leaf * leaf + leaf + Fr::from(1u64);
        let commitment = new_leaf * new_leaf + new_leaf + Fr::from(1u64);
        
        let circuit = StateCircuit {
            old_root: Some(old_root),
            new_root: Some(new_root),
            nullifier: Some(nullifier),
            commitment: Some(commitment),
            leaf: Some(leaf),
            path: Some(path),
            indices: Some(indices),
            new_leaf: Some(new_leaf),
        };
        
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(cs.is_satisfied().unwrap());
    }
    
    #[test]
    fn test_nullifier_circuit_member() {
        let cs = ConstraintSystem::<Fr>::new_ref();
        
        let nullifier = Fr::from(42u64);
        let path = vec![Fr::from(0u64); MERKLE_DEPTH];
        let indices = vec![false; MERKLE_DEPTH];
        
        // Compute root
        let mut root = nullifier;
        for i in 0..MERKLE_DEPTH {
            let left = if indices[i] { path[i] } else { root };
            let right = if indices[i] { root } else { path[i] };
            root = left * left + right * right + left * right + Fr::from(1u64);
        }
        
        let circuit = NullifierCircuit {
            nullifier: Some(nullifier),
            set_root: Some(root),
            is_member: Some(true),
            path: Some(path),
            indices: Some(indices),
        };
        
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(cs.is_satisfied().unwrap());
    }
    
    #[test]
    #[ignore] // Expensive test - requires 5+ minutes
    fn test_state_circuit_setup() {
        let result = StateCircuit::<Fr>::setup();
        assert!(result.is_ok(), "Circuit setup should succeed");
    }
    
    #[test]
    #[ignore] // Expensive test - requires setup + proof generation
    fn test_state_circuit_prove_verify() {
        // Setup
        let (pk, vk) = StateCircuit::<Fr>::setup().expect("Setup should succeed");
        
        // Create a valid state transition
        let leaf = Fr::from(100u64);
        let new_leaf = Fr::from(200u64);
        let path = vec![Fr::from(0u64); MERKLE_DEPTH];
        let indices = vec![false; MERKLE_DEPTH];
        
        // Compute roots using the same hash function as the circuit
        let mut old_root = leaf;
        for i in 0..MERKLE_DEPTH {
            let left = if indices[i] { path[i] } else { old_root };
            let right = if indices[i] { old_root } else { path[i] };
            old_root = left * left + right * right + left * right + Fr::from(1u64);
        }
        
        let mut new_root = new_leaf;
        for i in 0..MERKLE_DEPTH {
            let left = if indices[i] { path[i] } else { new_root };
            let right = if indices[i] { new_root } else { path[i] };
            new_root = left * left + right * right + left * right + Fr::from(1u64);
        }
        
        let nullifier = leaf * leaf + leaf + Fr::from(1u64);
        let commitment = new_leaf * new_leaf + new_leaf + Fr::from(1u64);
        
        // Create circuit with witnesses
        let circuit = StateCircuit::new(old_root, new_root, nullifier, commitment)
            .with_witnesses(leaf, path, indices, new_leaf);
        
        // Generate proof
        let proof = circuit.prove(&pk).expect("Proof generation should succeed");
        
        // Verify proof
        let public_inputs = circuit.public_inputs();
        let is_valid = StateCircuit::verify(&vk, &proof, &public_inputs)
            .expect("Verification should not error");
        assert!(is_valid, "Proof should verify successfully");
    }
    
    #[test]
    #[ignore] // Expensive test
    fn test_state_circuit_double_spend_prevention() {
        let (pk, vk) = StateCircuit::<Fr>::setup().expect("Setup should succeed");
        
        // Create two transactions trying to spend the same nullifier
        let leaf = Fr::from(100u64);
        let new_leaf_1 = Fr::from(200u64);
        let new_leaf_2 = Fr::from(300u64);
        let path = vec![Fr::from(0u64); MERKLE_DEPTH];
        let indices = vec![false; MERKLE_DEPTH];
        
        // Both transactions derive the same nullifier from the same leaf
        let nullifier = leaf * leaf + leaf + Fr::from(1u64);
        
        // Compute roots for first transaction
        let mut old_root = leaf;
        for i in 0..MERKLE_DEPTH {
            let left = if indices[i] { path[i] } else { old_root };
            let right = if indices[i] { old_root } else { path[i] };
            old_root = left * left + right * right + left * right + Fr::from(1u64);
        }
        
        let mut new_root_1 = new_leaf_1;
        for i in 0..MERKLE_DEPTH {
            let left = if indices[i] { path[i] } else { new_root_1 };
            let right = if indices[i] { new_root_1 } else { path[i] };
            new_root_1 = left * left + right * right + left * right + Fr::from(1u64);
        }
        
        let commitment_1 = new_leaf_1 * new_leaf_1 + new_leaf_1 + Fr::from(1u64);
        
        // First transaction proves successfully
        let circuit_1 = StateCircuit::new(old_root, new_root_1, nullifier, commitment_1)
            .with_witnesses(leaf, path.clone(), indices.clone(), new_leaf_1);
        let proof_1 = circuit_1.prove(&pk).expect("First proof should succeed");
        let is_valid_1 = StateCircuit::verify(&vk, &proof_1, &circuit_1.public_inputs())
            .expect("Verification should not error");
        assert!(is_valid_1, "First proof should verify");
        
        // Second transaction with same nullifier also proves (but system should reject it)
        // This test demonstrates that the circuit correctly derives the nullifier
        // The actual double-spend prevention happens at the consensus layer
        // by tracking used nullifiers in a set
        let mut new_root_2 = new_leaf_2;
        for i in 0..MERKLE_DEPTH {
            let left = if indices[i] { path[i] } else { new_root_2 };
            let right = if indices[i] { new_root_2 } else { path[i] };
            new_root_2 = left * left + right * right + left * right + Fr::from(1u64);
        }
        
        let commitment_2 = new_leaf_2 * new_leaf_2 + new_leaf_2 + Fr::from(1u64);
        
        let circuit_2 = StateCircuit::new(old_root, new_root_2, nullifier, commitment_2)
            .with_witnesses(leaf, path, indices, new_leaf_2);
        let proof_2 = circuit_2.prove(&pk).expect("Second proof should succeed");
        let is_valid_2 = StateCircuit::verify(&vk, &proof_2, &circuit_2.public_inputs())
            .expect("Verification should not error");
        assert!(is_valid_2, "Second proof should verify");
        
        // Both proofs have the same nullifier - consensus layer must reject the second
        assert_eq!(circuit_1.nullifier, circuit_2.nullifier, 
                   "Both transactions should produce the same nullifier");
    }
    
    #[test]
    #[ignore] // Expensive test
    fn test_nullifier_circuit_prove_verify() {
        // Setup
        let (pk, vk) = NullifierCircuit::<Fr>::setup().expect("Setup should succeed");
        
        let nullifier = Fr::from(42u64);
        let path = vec![Fr::from(0u64); MERKLE_DEPTH];
        let indices = vec![false; MERKLE_DEPTH];
        
        // Compute root
        let mut root = nullifier;
        for i in 0..MERKLE_DEPTH {
            let left = if indices[i] { path[i] } else { root };
            let right = if indices[i] { root } else { path[i] };
            root = left * left + right * right + left * right + Fr::from(1u64);
        }
        
        // Create circuit checking that nullifier is a member
        let circuit = NullifierCircuit::new(nullifier, root, true)
            .with_witnesses(path, indices);
        
        // Generate proof
        let proof = circuit.prove(&pk).expect("Proof generation should succeed");
        
        // Verify proof
        let public_inputs = circuit.public_inputs();
        let is_valid = NullifierCircuit::verify(&vk, &proof, &public_inputs)
            .expect("Verification should not error");
        assert!(is_valid, "Proof should verify successfully");
    }
    
    #[test]
    #[ignore] // Expensive test
    fn test_nullifier_circuit_rejects_non_member() {
        let (pk, vk) = NullifierCircuit::<Fr>::setup().expect("Setup should succeed");
        
        let nullifier = Fr::from(42u64);
        let path = vec![Fr::from(0u64); MERKLE_DEPTH];
        let indices = vec![false; MERKLE_DEPTH];
        
        // Compute a different root (nullifier is not in this tree)
        let wrong_root = Fr::from(999u64);
        
        // Circuit should prove non-membership
        let circuit = NullifierCircuit::new(nullifier, wrong_root, false)
            .with_witnesses(path, indices);
        
        let proof = circuit.prove(&pk).expect("Proof should generate");
        let is_valid = NullifierCircuit::verify(&vk, &proof, &circuit.public_inputs())
            .expect("Verification should not error");
        assert!(is_valid, "Proof should verify non-membership");
    }
    
    #[test]
    fn test_state_circuit_merkle_validation() {
        // Test that the circuit correctly validates merkle proofs
        let cs = ConstraintSystem::<Fr>::new_ref();
        
        let leaf = Fr::from(123u64);
        let new_leaf = Fr::from(456u64);
        
        // Create a path with some non-zero values
        let mut path = vec![Fr::from(0u64); MERKLE_DEPTH];
        path[0] = Fr::from(10u64);
        path[1] = Fr::from(20u64);
        path[5] = Fr::from(50u64);
        
        let mut indices = vec![false; MERKLE_DEPTH];
        indices[0] = true;
        indices[5] = true;
        
        // Compute correct roots
        let mut old_root = leaf;
        for i in 0..MERKLE_DEPTH {
            let left = if indices[i] { path[i] } else { old_root };
            let right = if indices[i] { old_root } else { path[i] };
            old_root = left * left + right * right + left * right + Fr::from(1u64);
        }
        
        let mut new_root = new_leaf;
        for i in 0..MERKLE_DEPTH {
            let left = if indices[i] { path[i] } else { new_root };
            let right = if indices[i] { new_root } else { path[i] };
            new_root = left * left + right * right + left * right + Fr::from(1u64);
        }
        
        let nullifier = leaf * leaf + leaf + Fr::from(1u64);
        let commitment = new_leaf * new_leaf + new_leaf + Fr::from(1u64);
        
        let circuit = StateCircuit {
            old_root: Some(old_root),
            new_root: Some(new_root),
            nullifier: Some(nullifier),
            commitment: Some(commitment),
            leaf: Some(leaf),
            path: Some(path),
            indices: Some(indices),
            new_leaf: Some(new_leaf),
        };
        
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(cs.is_satisfied().unwrap(), "Circuit should be satisfied with valid merkle proof");
    }
}
