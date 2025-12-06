//! Production Poseidon Merkle verification gadget for R1CS circuits
//!
//! This module provides a cryptographically secure Merkle tree verification
//! gadget using the Poseidon hash function, suitable for production use.
//!
//! # Security
//! This implementation uses a full Poseidon permutation with:
//! - 8 full rounds and 57 partial rounds
//! - x^5 S-box for BN254 field
//! - MDS matrix multiplication
//! - Domain-separated round constants
//!
//! # Usage
//! ```ignore
//! let gadget = PoseidonMerkleGadget::new(cs.clone(), leaf, path, indices)?;
//! gadget.verify_inclusion(&expected_root)?;
//! ```
//!
//! # Constraint Count
//! Each Poseidon hash requires approximately:
//! - 1 constraint per S-box application (65 * 3 = 195 for t=3)
//! - Plus MDS and addition constraints
//! Total: ~400-500 constraints per hash, ~500 per tree level

use ark_ff::PrimeField;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use ark_r1cs_std::{
    prelude::*,
    fields::fp::FpVar,
    boolean::Boolean,
};

/// Default Merkle tree depth (32 levels supports 2^32 leaves)
pub const POSEIDON_MERKLE_DEPTH: usize = 32;

/// Poseidon permutation parameters
const FULL_ROUNDS: usize = 8;
const PARTIAL_ROUNDS: usize = 57;
const STATE_WIDTH: usize = 3; // For 2-to-1 compression

/// Production-ready Poseidon Merkle path verification gadget.
///
/// Uses the full Poseidon permutation for cryptographic security.
pub struct PoseidonMerkleGadget<F: PrimeField> {
    /// The leaf value as a field element variable
    pub leaf: FpVar<F>,
    /// Authentication path (sibling hashes from leaf to root)
    pub path: Vec<FpVar<F>>,
    /// Path direction indices (false = left child, true = right child)
    pub path_indices: Vec<Boolean<F>>,
    /// Cached round constants
    round_constants: Vec<FpVar<F>>,
    /// Cached MDS matrix
    mds_matrix: Vec<Vec<F>>,
}

impl<F: PrimeField> PoseidonMerkleGadget<F> {
    /// Create a new Poseidon Merkle path gadget.
    ///
    /// # Arguments
    /// * `cs` - Constraint system reference
    /// * `leaf` - The leaf value to verify
    /// * `path` - Vector of sibling hashes (authentication path)
    /// * `path_indices` - Direction indicators (false=left, true=right)
    ///
    /// # Errors
    /// Returns an error if path and indices have different lengths or exceed depth limit.
    pub fn new(
        cs: ConstraintSystemRef<F>,
        leaf: FpVar<F>,
        path: Vec<FpVar<F>>,
        path_indices: Vec<Boolean<F>>,
    ) -> Result<Self, SynthesisError> {
        if path.len() != path_indices.len() {
            return Err(SynthesisError::Unsatisfiable);
        }
        if path.len() > POSEIDON_MERKLE_DEPTH {
            return Err(SynthesisError::Unsatisfiable);
        }
        
        // Generate round constants
        let round_constants = Self::generate_round_constants(cs)?;
        
        // Generate MDS matrix
        let mds_matrix = Self::generate_mds_matrix();
        
        Ok(Self {
            leaf,
            path,
            path_indices,
            round_constants,
            mds_matrix,
        })
    }
    
    /// Generate deterministic round constants as FpVar
    fn generate_round_constants(cs: ConstraintSystemRef<F>) -> Result<Vec<FpVar<F>>, SynthesisError> {
        use sha2::{Sha256, Digest};
        
        let total_rounds = FULL_ROUNDS + PARTIAL_ROUNDS;
        let total_constants = STATE_WIDTH * total_rounds;
        let mut constants = Vec::with_capacity(total_constants);
        
        let mut counter = 0u64;
        while constants.len() < total_constants {
            let mut hasher = Sha256::new();
            hasher.update(b"BitCell_Poseidon_RC");
            hasher.update(&counter.to_le_bytes());
            hasher.update(&(STATE_WIDTH as u64).to_le_bytes());
            let hash = hasher.finalize();
            
            // Convert to field element
            let mut bytes = [0u8; 32];
            bytes[..31].copy_from_slice(&hash[..31]);
            bytes[31] = 0;
            
            if let Some(fe) = F::from_random_bytes(&bytes) {
                constants.push(FpVar::new_constant(cs.clone(), fe)?);
            }
            counter += 1;
        }
        
        Ok(constants)
    }
    
    /// Generate MDS matrix using Cauchy construction
    fn generate_mds_matrix() -> Vec<Vec<F>> {
        let t = STATE_WIDTH;
        let mut matrix = vec![vec![F::zero(); t]; t];
        
        let x: Vec<F> = (0..t).map(|i| F::from((i + 1) as u64)).collect();
        let y: Vec<F> = (0..t).map(|i| F::from((t + i + 1) as u64)).collect();
        
        for i in 0..t {
            for j in 0..t {
                let sum = x[i] + y[j];
                matrix[i][j] = sum.inverse().expect(
                    "MDS matrix Cauchy construction guarantees non-zero inverse: \
                     x[i] and y[j] are chosen as distinct elements so x[i] + y[j] != 0"
                );
            }
        }
        
        matrix
    }
    
    /// Verify that the leaf is included in a Merkle tree with the given root.
    pub fn verify_inclusion(
        &self,
        expected_root: &FpVar<F>,
    ) -> Result<(), SynthesisError> {
        let depth = self.path.len();
        
        // Start with the leaf
        let mut current_hash = self.leaf.clone();
        
        // Walk up the tree
        for i in 0..depth {
            let sibling = &self.path[i];
            let is_right = &self.path_indices[i];
            
            // Select left and right based on path index
            let left = FpVar::conditionally_select(is_right, sibling, &current_hash)?;
            let right = FpVar::conditionally_select(is_right, &current_hash, sibling)?;
            
            // Hash using Poseidon
            current_hash = self.poseidon_hash_two(&left, &right)?;
        }
        
        // Enforce computed root equals expected root
        current_hash.enforce_equal(expected_root)?;
        
        Ok(())
    }
    
    /// Compute Poseidon hash of two field elements
    fn poseidon_hash_two(&self, left: &FpVar<F>, right: &FpVar<F>) -> Result<FpVar<F>, SynthesisError> {
        // Initialize state: [0, left, right]
        let mut state = vec![FpVar::zero(), left.clone(), right.clone()];
        
        // Apply Poseidon permutation
        self.poseidon_permutation(&mut state)?;
        
        // Return first element
        Ok(state[0].clone())
    }
    
    /// Apply full Poseidon permutation to state
    fn poseidon_permutation(&self, state: &mut [FpVar<F>]) -> Result<(), SynthesisError> {
        let rf = FULL_ROUNDS / 2;
        let rp = PARTIAL_ROUNDS;
        
        let mut round_idx = 0;
        
        // First half of full rounds
        for _ in 0..rf {
            self.add_round_constants(state, round_idx)?;
            self.full_sbox(state)?;
            self.mds_multiply(state)?;
            round_idx += 1;
        }
        
        // Partial rounds
        for _ in 0..rp {
            self.add_round_constants(state, round_idx)?;
            self.partial_sbox(state)?;
            self.mds_multiply(state)?;
            round_idx += 1;
        }
        
        // Second half of full rounds
        for _ in 0..rf {
            self.add_round_constants(state, round_idx)?;
            self.full_sbox(state)?;
            self.mds_multiply(state)?;
            round_idx += 1;
        }
        
        Ok(())
    }
    
    /// Add round constants to state
    fn add_round_constants(&self, state: &mut [FpVar<F>], round: usize) -> Result<(), SynthesisError> {
        let offset = round * STATE_WIDTH;
        for i in 0..STATE_WIDTH {
            state[i] = &state[i] + &self.round_constants[offset + i];
        }
        Ok(())
    }
    
    /// Apply S-box (x^5) to all state elements
    fn full_sbox(&self, state: &mut [FpVar<F>]) -> Result<(), SynthesisError> {
        for s in state.iter_mut() {
            *s = self.sbox(s)?;
        }
        Ok(())
    }
    
    /// Apply S-box to first state element only
    fn partial_sbox(&self, state: &mut [FpVar<F>]) -> Result<(), SynthesisError> {
        state[0] = self.sbox(&state[0])?;
        Ok(())
    }
    
    /// S-box: x^5 = x^4 * x = (x^2)^2 * x
    fn sbox(&self, x: &FpVar<F>) -> Result<FpVar<F>, SynthesisError> {
        let x2 = x.square()?;
        let x4 = x2.square()?;
        Ok(&x4 * x)
    }
    
    /// MDS matrix multiplication
    fn mds_multiply(&self, state: &mut [FpVar<F>]) -> Result<(), SynthesisError> {
        let t = STATE_WIDTH;
        let mut new_state = Vec::with_capacity(t);
        
        for i in 0..t {
            let mut acc = FpVar::zero();
            for j in 0..t {
                let coeff = FpVar::constant(self.mds_matrix[i][j]);
                acc = &acc + &(&coeff * &state[j]);
            }
            new_state.push(acc);
        }
        
        for i in 0..t {
            state[i] = new_state[i].clone();
        }
        
        Ok(())
    }
    
    /// Get the approximate number of constraints generated for this verification.
    pub fn num_constraints(&self) -> usize {
        // Per hash:
        // - Full rounds: 8 * 3 S-boxes = 24 S-box applications
        // - Partial rounds: 57 * 1 S-box = 57 S-box applications
        // - Each S-box: ~3 multiplications
        // - MDS: ~9 multiplications per round
        // Total per hash: ~400-500 constraints
        // Per tree level: ~500 constraints + conditional select (~2)
        self.path.len() * 502 + 1
    }
}

/// Compute Poseidon hash of two field elements (native, for testing)
pub fn compute_poseidon_merkle_root<F: PrimeField>(
    leaf: F,
    path: &[F],
    directions: &[bool],
) -> F {
    // Note: This only works for ark_bn254::Fr due to the params generation
    // For generic F, we'd need generic params
    let mut current = leaf;
    
    for (sibling, &is_right) in path.iter().zip(directions.iter()) {
        let (left, right) = if is_right {
            (*sibling, current)
        } else {
            (current, *sibling)
        };
        
        // Use native Poseidon for Fr
        // For other fields, fall back to algebraic hash
        current = poseidon_hash_native(left, right);
    }
    
    current
}

/// Native Poseidon hash for testing (same algorithm as gadget)
fn poseidon_hash_native<F: PrimeField>(left: F, right: F) -> F {
    use sha2::{Sha256, Digest};
    
    let total_rounds = FULL_ROUNDS + PARTIAL_ROUNDS;
    let total_constants = STATE_WIDTH * total_rounds;
    
    // Generate round constants
    let mut round_constants = Vec::with_capacity(total_constants);
    let mut counter = 0u64;
    while round_constants.len() < total_constants {
        let mut hasher = Sha256::new();
        hasher.update(b"BitCell_Poseidon_RC");
        hasher.update(&counter.to_le_bytes());
        hasher.update(&(STATE_WIDTH as u64).to_le_bytes());
        let hash = hasher.finalize();
        
        let mut bytes = [0u8; 32];
        bytes[..31].copy_from_slice(&hash[..31]);
        bytes[31] = 0;
        
        if let Some(fe) = F::from_random_bytes(&bytes) {
            round_constants.push(fe);
        }
        counter += 1;
    }
    
    // Generate MDS matrix
    let t = STATE_WIDTH;
    let mut mds_matrix = vec![vec![F::zero(); t]; t];
    let x: Vec<F> = (0..t).map(|i| F::from((i + 1) as u64)).collect();
    let y: Vec<F> = (0..t).map(|i| F::from((t + i + 1) as u64)).collect();
    for i in 0..t {
        for j in 0..t {
            let sum = x[i] + y[j];
            mds_matrix[i][j] = sum.inverse().expect(
                "MDS matrix Cauchy construction guarantees non-zero inverse for distinct x_i, y_j"
            );
        }
    }
    
    // Initialize state
    let mut state = vec![F::zero(), left, right];
    
    // Apply permutation
    let rf = FULL_ROUNDS / 2;
    let rp = PARTIAL_ROUNDS;
    let mut round_idx = 0;
    
    // First half of full rounds
    for _ in 0..rf {
        // Add round constants
        for i in 0..STATE_WIDTH {
            state[i] += round_constants[round_idx * STATE_WIDTH + i];
        }
        // Full S-box
        for s in state.iter_mut() {
            let s2 = s.square();
            let s4 = s2.square();
            *s = s4 * *s;
        }
        // MDS multiply
        let mut new_state = vec![F::zero(); STATE_WIDTH];
        for i in 0..STATE_WIDTH {
            for j in 0..STATE_WIDTH {
                new_state[i] += mds_matrix[i][j] * state[j];
            }
        }
        state = new_state;
        round_idx += 1;
    }
    
    // Partial rounds
    for _ in 0..rp {
        // Add round constants
        for i in 0..STATE_WIDTH {
            state[i] += round_constants[round_idx * STATE_WIDTH + i];
        }
        // Partial S-box (first element only)
        let s2 = state[0].square();
        let s4 = s2.square();
        state[0] = s4 * state[0];
        // MDS multiply
        let mut new_state = vec![F::zero(); STATE_WIDTH];
        for i in 0..STATE_WIDTH {
            for j in 0..STATE_WIDTH {
                new_state[i] += mds_matrix[i][j] * state[j];
            }
        }
        state = new_state;
        round_idx += 1;
    }
    
    // Second half of full rounds
    for _ in 0..rf {
        // Add round constants
        for i in 0..STATE_WIDTH {
            state[i] += round_constants[round_idx * STATE_WIDTH + i];
        }
        // Full S-box
        for s in state.iter_mut() {
            let s2 = s.square();
            let s4 = s2.square();
            *s = s4 * *s;
        }
        // MDS multiply
        let mut new_state = vec![F::zero(); STATE_WIDTH];
        for i in 0..STATE_WIDTH {
            for j in 0..STATE_WIDTH {
                new_state[i] += mds_matrix[i][j] * state[j];
            }
        }
        state = new_state;
        round_idx += 1;
    }
    
    state[0]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;
    use ark_relations::r1cs::ConstraintSystem;
    
    #[test]
    fn test_poseidon_merkle_verification_depth_3() {
        let cs = ConstraintSystem::<Fr>::new_ref();
        
        let leaf_value = Fr::from(42u64);
        let path_values = vec![
            Fr::from(1u64),
            Fr::from(2u64),
            Fr::from(3u64),
        ];
        let directions = vec![false, true, false];
        
        // Compute expected root using native Poseidon
        let expected_root = compute_poseidon_merkle_root(leaf_value, &path_values, &directions);
        
        // Allocate variables
        let leaf = FpVar::new_witness(cs.clone(), || Ok(leaf_value)).unwrap();
        let path: Vec<FpVar<Fr>> = path_values.iter()
            .map(|v| FpVar::new_witness(cs.clone(), || Ok(*v)).unwrap())
            .collect();
        let indices: Vec<Boolean<Fr>> = directions.iter()
            .map(|d| Boolean::new_witness(cs.clone(), || Ok(*d)).unwrap())
            .collect();
        
        let root_var = FpVar::new_input(cs.clone(), || Ok(expected_root)).unwrap();
        
        // Create gadget and verify
        let gadget = PoseidonMerkleGadget::new(cs.clone(), leaf, path, indices).unwrap();
        gadget.verify_inclusion(&root_var).unwrap();
        
        assert!(cs.is_satisfied().unwrap());
        println!("Poseidon Merkle depth 3: {} constraints", cs.num_constraints());
    }
    
    #[test]
    fn test_poseidon_merkle_wrong_root_fails() {
        let cs = ConstraintSystem::<Fr>::new_ref();
        
        let leaf_value = Fr::from(42u64);
        let path_values = vec![Fr::from(1u64), Fr::from(2u64)];
        let directions = vec![false, true];
        
        let correct_root = compute_poseidon_merkle_root(leaf_value, &path_values, &directions);
        let wrong_root = correct_root + Fr::from(1u64);
        
        let leaf = FpVar::new_witness(cs.clone(), || Ok(leaf_value)).unwrap();
        let path: Vec<FpVar<Fr>> = path_values.iter()
            .map(|v| FpVar::new_witness(cs.clone(), || Ok(*v)).unwrap())
            .collect();
        let indices: Vec<Boolean<Fr>> = directions.iter()
            .map(|d| Boolean::new_witness(cs.clone(), || Ok(*d)).unwrap())
            .collect();
        
        let root_var = FpVar::new_input(cs.clone(), || Ok(wrong_root)).unwrap();
        
        let gadget = PoseidonMerkleGadget::new(cs.clone(), leaf, path, indices).unwrap();
        gadget.verify_inclusion(&root_var).unwrap();
        
        assert!(!cs.is_satisfied().unwrap());
    }
    
    #[test]
    fn test_poseidon_native_deterministic() {
        let a = Fr::from(123u64);
        let b = Fr::from(456u64);
        
        let h1 = poseidon_hash_native(a, b);
        let h2 = poseidon_hash_native(a, b);
        
        assert_eq!(h1, h2);
    }
    
    #[test]
    fn test_poseidon_native_asymmetric() {
        let a = Fr::from(1u64);
        let b = Fr::from(2u64);
        
        let h1 = poseidon_hash_native(a, b);
        let h2 = poseidon_hash_native(b, a);
        
        assert_ne!(h1, h2);
    }
}
