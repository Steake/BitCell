/// State transition circuit implementing Merkle tree verification
/// This module provides R1CS constraints for verifying state updates

use ark_ff::PrimeField;
use ark_r1cs_std::prelude::*;
use ark_r1cs_std::fields::fp::FpVar;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

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
}
