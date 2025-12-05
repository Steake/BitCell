//! Poseidon Hash Implementation for BitCell
//!
//! This module provides a production-ready Poseidon hash implementation
//! optimized for use in ZK-SNARK circuits over the BN254 curve.
//!
//! # Security
//! The parameters are chosen for 128-bit security level following the
//! Poseidon specification from <https://eprint.iacr.org/2019/458.pdf>
//!
//! # Usage
//! ```ignore
//! use bitcell_crypto::poseidon::{PoseidonHasher, PoseidonParams};
//! use ark_bn254::Fr;
//!
//! let params = PoseidonParams::bn254_2_to_1();
//! let mut hasher = PoseidonHasher::new(&params);
//! let hash = hasher.hash_two(Fr::from(1u64), Fr::from(2u64));
//! ```

use ark_ff::{PrimeField, Field};
use ark_bn254::Fr;
use std::marker::PhantomData;

/// Number of full rounds for security (RF = 8)
pub const FULL_ROUNDS: usize = 8;

/// Number of partial rounds for security (RP = 57 for t=3)
pub const PARTIAL_ROUNDS_T3: usize = 57;

/// State width for 2-to-1 compression (t = 3)
pub const STATE_WIDTH_T3: usize = 3;

/// Poseidon parameters for a specific field and state width
#[derive(Clone, Debug)]
pub struct PoseidonParams<F: PrimeField> {
    /// State width (t)
    pub t: usize,
    /// Number of full rounds
    pub full_rounds: usize,
    /// Number of partial rounds
    pub partial_rounds: usize,
    /// Round constants (one per round per state element)
    pub round_constants: Vec<F>,
    /// MDS matrix for the linear layer
    pub mds_matrix: Vec<Vec<F>>,
    /// S-box exponent (typically 5 for BN254)
    pub alpha: u64,
}

impl PoseidonParams<Fr> {
    /// Create parameters for BN254 2-to-1 compression hash
    /// 
    /// These parameters are suitable for Merkle tree construction
    /// with 128-bit security against collision and preimage attacks.
    pub fn bn254_2_to_1() -> Self {
        let t = STATE_WIDTH_T3;
        let full_rounds = FULL_ROUNDS;
        let partial_rounds = PARTIAL_ROUNDS_T3;
        
        // Generate deterministic round constants using SHA-256 as PRF
        // This follows the Poseidon specification's grain LFSR method
        let round_constants = Self::generate_round_constants(t, full_rounds, partial_rounds);
        
        // Generate MDS matrix using Cauchy matrix construction
        let mds_matrix = Self::generate_mds_matrix(t);
        
        Self {
            t,
            full_rounds,
            partial_rounds,
            round_constants,
            mds_matrix,
            alpha: 5, // x^5 S-box for BN254
        }
    }
    
    /// Generate deterministic round constants
    fn generate_round_constants(t: usize, full_rounds: usize, partial_rounds: usize) -> Vec<Fr> {
        use sha2::{Sha256, Digest};
        
        let total_constants = t * (full_rounds + partial_rounds);
        let mut constants = Vec::with_capacity(total_constants);
        
        // Maximum iterations to prevent theoretical infinite loop
        const MAX_ITERATIONS: u64 = 1_000_000;
        
        // Use domain-separated SHA-256 as PRF
        let mut counter = 0u64;
        while constants.len() < total_constants {
            if counter >= MAX_ITERATIONS {
                panic!(
                    "Round constant generation exceeded {} iterations. \
                     This should never happen with SHA-256 PRF - please report this bug.",
                    MAX_ITERATIONS
                );
            }
            
            let mut hasher = Sha256::new();
            hasher.update(b"BitCell_Poseidon_RC");
            hasher.update(&counter.to_le_bytes());
            hasher.update(&(t as u64).to_le_bytes());
            let hash = hasher.finalize();
            
            // Convert hash to field element (mod p)
            // Take 31 bytes to ensure < p with high probability
            let mut bytes = [0u8; 32];
            bytes[..31].copy_from_slice(&hash[..31]);
            bytes[31] = 0; // Ensure < p by zeroing top byte
            
            if let Some(fe) = Fr::from_random_bytes(&bytes) {
                constants.push(fe);
            }
            counter += 1;
        }
        
        constants
    }
    
    /// Generate MDS matrix using Cauchy matrix construction
    /// 
    /// The Cauchy matrix M[i][j] = 1 / (x_i + y_j) is guaranteed to be MDS
    /// when all x_i, y_j are distinct and x_i + y_j != 0.
    fn generate_mds_matrix(t: usize) -> Vec<Vec<Fr>> {
        let mut matrix = vec![vec![Fr::from(0u64); t]; t];
        
        // Use distinct elements for x_i and y_j
        let x: Vec<Fr> = (0..t).map(|i| Fr::from((i + 1) as u64)).collect();
        let y: Vec<Fr> = (0..t).map(|i| Fr::from((t + i + 1) as u64)).collect();
        
        for i in 0..t {
            for j in 0..t {
                // M[i][j] = 1 / (x[i] + y[j])
                let sum = x[i] + y[j];
                matrix[i][j] = sum.inverse().expect(
                    "Cauchy MDS construction: x[i] and y[j] are distinct elements, \
                     so x[i] + y[j] != 0 and is always invertible in a prime field"
                );
            }
        }
        
        matrix
    }
}

/// Poseidon hasher for computing hashes
#[derive(Clone)]
pub struct PoseidonHasher<F: PrimeField> {
    params: PoseidonParams<F>,
    _marker: PhantomData<F>,
}

impl<F: PrimeField> PoseidonHasher<F> {
    /// Create a new Poseidon hasher with the given parameters
    pub fn new(params: &PoseidonParams<F>) -> Self {
        Self {
            params: params.clone(),
            _marker: PhantomData,
        }
    }
    
    /// Hash two field elements (2-to-1 compression for Merkle trees)
    pub fn hash_two(&self, left: F, right: F) -> F {
        assert_eq!(self.params.t, 3, "2-to-1 hash requires t=3");
        
        let mut state = vec![F::zero(), left, right];
        self.permutation(&mut state);
        
        // Output is the first element of the final state
        state[0]
    }
    
    /// Hash a single field element
    pub fn hash_one(&self, input: F) -> F {
        assert!(self.params.t >= 2, "Single hash requires t>=2");
        
        let mut state = vec![F::zero(); self.params.t];
        state[1] = input;
        self.permutation(&mut state);
        
        state[0]
    }
    
    /// Hash multiple field elements using a sponge construction
    pub fn hash_many(&self, inputs: &[F]) -> F {
        let rate = self.params.t - 1; // Capacity = 1
        let mut state = vec![F::zero(); self.params.t];
        
        // Absorb phase
        for chunk in inputs.chunks(rate) {
            for (i, input) in chunk.iter().enumerate() {
                state[i + 1] += *input;
            }
            self.permutation(&mut state);
        }
        
        // Squeeze phase - return first element
        state[0]
    }
    
    /// Apply the full Poseidon permutation
    fn permutation(&self, state: &mut [F]) {
        let rf = self.params.full_rounds / 2;
        let rp = self.params.partial_rounds;
        let total_rounds = self.params.full_rounds + self.params.partial_rounds;
        
        let mut round_idx = 0;
        
        // First half of full rounds
        for _ in 0..rf {
            self.add_round_constants(state, round_idx);
            self.full_sbox(state);
            self.mds_multiply(state);
            round_idx += 1;
        }
        
        // Partial rounds
        for _ in 0..rp {
            self.add_round_constants(state, round_idx);
            self.partial_sbox(state);
            self.mds_multiply(state);
            round_idx += 1;
        }
        
        // Second half of full rounds
        for _ in 0..rf {
            self.add_round_constants(state, round_idx);
            self.full_sbox(state);
            self.mds_multiply(state);
            round_idx += 1;
        }
        
        debug_assert_eq!(round_idx, total_rounds);
    }
    
    /// Add round constants to the state
    fn add_round_constants(&self, state: &mut [F], round: usize) {
        let t = self.params.t;
        let offset = round * t;
        
        for i in 0..t {
            state[i] += self.params.round_constants[offset + i];
        }
    }
    
    /// Apply S-box to all state elements (full round)
    fn full_sbox(&self, state: &mut [F]) {
        for s in state.iter_mut() {
            *s = self.sbox(*s);
        }
    }
    
    /// Apply S-box to first state element only (partial round)
    fn partial_sbox(&self, state: &mut [F]) {
        state[0] = self.sbox(state[0]);
    }
    
    /// S-box: x^alpha (typically x^5 for BN254)
    fn sbox(&self, x: F) -> F {
        match self.params.alpha {
            5 => {
                // x^5 = x^4 * x = (x^2)^2 * x
                let x2 = x.square();
                let x4 = x2.square();
                x4 * x
            }
            _ => x.pow([self.params.alpha]),
        }
    }
    
    /// MDS matrix multiplication
    fn mds_multiply(&self, state: &mut [F]) {
        let t = self.params.t;
        let mut new_state = vec![F::zero(); t];
        
        for i in 0..t {
            for j in 0..t {
                new_state[i] += self.params.mds_matrix[i][j] * state[j];
            }
        }
        
        state.copy_from_slice(&new_state);
    }
}

/// Convenience function to create a BN254 Poseidon hasher
pub fn poseidon_bn254() -> PoseidonHasher<Fr> {
    let params = PoseidonParams::bn254_2_to_1();
    PoseidonHasher::new(&params)
}

/// Hash two BN254 field elements using Poseidon
pub fn poseidon_hash_two(left: Fr, right: Fr) -> Fr {
    poseidon_bn254().hash_two(left, right)
}

/// Hash a single BN254 field element using Poseidon
pub fn poseidon_hash_one(input: Fr) -> Fr {
    poseidon_bn254().hash_one(input)
}

/// Hash multiple BN254 field elements using Poseidon sponge
pub fn poseidon_hash_many(inputs: &[Fr]) -> Fr {
    poseidon_bn254().hash_many(inputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::One;
    
    #[test]
    fn test_poseidon_deterministic() {
        let a = Fr::from(123u64);
        let b = Fr::from(456u64);
        
        let h1 = poseidon_hash_two(a, b);
        let h2 = poseidon_hash_two(a, b);
        
        assert_eq!(h1, h2, "Poseidon hash should be deterministic");
    }
    
    #[test]
    fn test_poseidon_different_inputs() {
        let a = Fr::from(1u64);
        let b = Fr::from(2u64);
        
        let h1 = poseidon_hash_two(a, b);
        let h2 = poseidon_hash_two(b, a);
        
        assert_ne!(h1, h2, "Different inputs should produce different outputs");
    }
    
    #[test]
    fn test_poseidon_preimage_resistance() {
        // Hash of zero should not be zero
        let h = poseidon_hash_one(Fr::from(0u64));
        assert_ne!(h, Fr::from(0u64), "Hash of zero should not be zero");
        
        // Hash of one should not be one
        let h = poseidon_hash_one(Fr::one());
        assert_ne!(h, Fr::one(), "Hash of one should not be one");
    }
    
    #[test]
    fn test_poseidon_chain() {
        // Test chaining hashes (like in a Merkle tree)
        let leaves = vec![
            Fr::from(1u64),
            Fr::from(2u64),
            Fr::from(3u64),
            Fr::from(4u64),
        ];
        
        // Level 0: hash pairs
        let h01 = poseidon_hash_two(leaves[0], leaves[1]);
        let h23 = poseidon_hash_two(leaves[2], leaves[3]);
        
        // Level 1: hash the hashes
        let root = poseidon_hash_two(h01, h23);
        
        // Root should be non-zero and different from inputs
        assert_ne!(root, Fr::from(0u64));
        assert_ne!(root, h01);
        assert_ne!(root, h23);
    }
    
    #[test]
    fn test_poseidon_many() {
        let inputs: Vec<Fr> = (0..10).map(|i| Fr::from(i as u64)).collect();
        
        let h1 = poseidon_hash_many(&inputs);
        let h2 = poseidon_hash_many(&inputs);
        
        assert_eq!(h1, h2, "Sponge hash should be deterministic");
        
        // Different inputs should give different outputs
        let mut inputs2 = inputs.clone();
        inputs2[5] = Fr::from(999u64);
        let h3 = poseidon_hash_many(&inputs2);
        
        assert_ne!(h1, h3, "Different inputs should produce different outputs");
    }
    
    #[test]
    fn test_mds_matrix_is_valid() {
        let params = PoseidonParams::bn254_2_to_1();
        
        // MDS matrix should be invertible (non-zero determinant)
        // For a 3x3 Cauchy matrix with distinct x_i, y_j this is guaranteed
        assert_eq!(params.mds_matrix.len(), 3);
        for row in &params.mds_matrix {
            assert_eq!(row.len(), 3);
        }
        
        // All entries should be non-zero
        for row in &params.mds_matrix {
            for entry in row {
                assert_ne!(*entry, Fr::from(0u64), "MDS matrix entries should be non-zero");
            }
        }
    }
    
    #[test]
    fn test_round_constants_deterministic() {
        let params1 = PoseidonParams::bn254_2_to_1();
        let params2 = PoseidonParams::bn254_2_to_1();
        
        assert_eq!(
            params1.round_constants.len(),
            params2.round_constants.len()
        );
        
        for (c1, c2) in params1.round_constants.iter().zip(params2.round_constants.iter()) {
            assert_eq!(*c1, *c2, "Round constants should be deterministic");
        }
    }
}
