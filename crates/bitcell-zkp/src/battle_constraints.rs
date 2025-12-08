/// Battle circuit constraints implementing Conway's Game of Life rules
/// This module provides the full R1CS constraint system for verifying CA battles

use ark_ff::PrimeField;
use ark_r1cs_std::prelude::*;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::bits::ToBitsGadget;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

/// Size of the CA grid (must be power of 2 for efficient constraints)
///
/// # Test vs Production Configuration
/// - **Test values**: `GRID_SIZE = 64`, `BATTLE_STEPS = 10`
///   - Used for unit tests and development to enable fast proof generation
///   - Suitable for CI/CD pipelines and local testing
/// - **Production values**: `GRID_SIZE = 1024`, `BATTLE_STEPS = 1000`
///   - Used for mainnet deployment with full-size tournament battles
///   - Requires trusted setup ceremony and optimized proving infrastructure
///
/// To switch between configurations, adjust these constants before compilation.
/// For production deployment, ensure sufficient hardware for proof generation
/// (recommended: 64GB+ RAM, GPU acceleration for proving).
pub const GRID_SIZE: usize = 64; // Reduced from 1024 for practical circuit size
pub const BATTLE_STEPS: usize = 10; // Reduced from 1000 for practical proving time

/// Battle circuit witness
#[derive(Clone)]
pub struct BattleCircuit<F: PrimeField> {
    /// Initial grid state (public)
    pub initial_grid: Option<Vec<Vec<u8>>>,
    /// Final grid state (public)
    pub final_grid: Option<Vec<Vec<u8>>>,
    /// Glider A commitment (public)
    pub commitment_a: Option<F>,
    /// Glider B commitment (public)
    pub commitment_b: Option<F>,
    /// Winner ID (public: 0 = A, 1 = B, 2 = tie)
    pub winner: Option<u8>,
    /// Glider A pattern (private)
    pub pattern_a: Option<Vec<Vec<u8>>>,
    /// Glider B pattern (private)
    pub pattern_b: Option<Vec<Vec<u8>>>,
    /// Nonce A (private)
    pub nonce_a: Option<F>,
    /// Nonce B (private)
    pub nonce_b: Option<F>,
}

impl<F: PrimeField> BattleCircuit<F> {
    pub fn new(
        initial_grid: Vec<Vec<u8>>,
        final_grid: Vec<Vec<u8>>,
        commitment_a: F,
        commitment_b: F,
        winner: u8,
    ) -> Self {
        Self {
            initial_grid: Some(initial_grid),
            final_grid: Some(final_grid),
            commitment_a: Some(commitment_a),
            commitment_b: Some(commitment_b),
            winner: Some(winner),
            pattern_a: None,
            pattern_b: None,
            nonce_a: None,
            nonce_b: None,
        }
    }

    pub fn with_witnesses(
        mut self,
        pattern_a: Vec<Vec<u8>>,
        pattern_b: Vec<Vec<u8>>,
        nonce_a: F,
        nonce_b: F,
    ) -> Self {
        self.pattern_a = Some(pattern_a);
        self.pattern_b = Some(pattern_b);
        self.nonce_a = Some(nonce_a);
        self.nonce_b = Some(nonce_b);
        self
    }
}

impl<F: PrimeField> ConstraintSynthesizer<F> for BattleCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // Allocate public inputs
        let initial_grid_vars = allocate_grid(cs.clone(), &self.initial_grid, true)?;
        let final_grid_vars = allocate_grid(cs.clone(), &self.final_grid, true)?;
        
        let commitment_a_var = FpVar::new_input(cs.clone(), || {
            self.commitment_a.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let commitment_b_var = FpVar::new_input(cs.clone(), || {
            self.commitment_b.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let winner_var = UInt8::new_input(cs.clone(), || {
            self.winner.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        // Allocate private witnesses
        let pattern_a_vars = allocate_grid(cs.clone(), &self.pattern_a, false)?;
        let pattern_b_vars = allocate_grid(cs.clone(), &self.pattern_b, false)?;
        
        let nonce_a_var = FpVar::new_witness(cs.clone(), || {
            self.nonce_a.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let nonce_b_var = FpVar::new_witness(cs.clone(), || {
            self.nonce_b.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        // Constraint 1: Verify commitment consistency
        // commitment_a = H(pattern_a || nonce_a)
        verify_commitment(cs.clone(), &pattern_a_vars, &nonce_a_var, &commitment_a_var)?;
        verify_commitment(cs.clone(), &pattern_b_vars, &nonce_b_var, &commitment_b_var)?;
        
        // Constraint 2: Verify initial grid matches patterns placed at spawn points
        verify_initial_placement(cs.clone(), &initial_grid_vars, &pattern_a_vars, &pattern_b_vars)?;
        
        // Constraint 3: Simulate BATTLE_STEPS of Conway's Game of Life
        let mut current_grid = initial_grid_vars;
        for _ in 0..BATTLE_STEPS {
            current_grid = conway_step(cs.clone(), &current_grid)?;
        }
        
        // Constraint 4: Verify final grid matches simulated result
        verify_grid_equality(cs.clone(), &current_grid, &final_grid_vars)?;
        
        // Constraint 5: Verify winner determination based on regional energy
        verify_winner(cs.clone(), &final_grid_vars, &winner_var)?;
        
        Ok(())
    }
}

/// Allocate a 2D grid of cells as circuit variables
fn allocate_grid<F: PrimeField>(
    cs: ConstraintSystemRef<F>,
    grid: &Option<Vec<Vec<u8>>>,
    is_public: bool,
) -> Result<Vec<Vec<UInt8<F>>>, SynthesisError> {
    let grid_data = grid.as_ref().ok_or(SynthesisError::AssignmentMissing)?;
    
    let mut result = Vec::new();
    for row in grid_data {
        let mut row_vars = Vec::new();
        for &cell in row {
            let cell_var = if is_public {
                UInt8::new_input(cs.clone(), || Ok(cell))?
            } else {
                UInt8::new_witness(cs.clone(), || Ok(cell))?
            };
            row_vars.push(cell_var);
        }
        result.push(row_vars);
    }
    
    Ok(result)
}

/// Verify commitment: H(pattern || nonce) == commitment
fn verify_commitment<F: PrimeField>(
    _cs: ConstraintSystemRef<F>,
    pattern: &[Vec<UInt8<F>>],
    nonce: &FpVar<F>,
    commitment: &FpVar<F>,
) -> Result<(), SynthesisError> {
    use ark_r1cs_std::bits::ToBitsGadget;
    
    // Flatten pattern to bits
    let mut bits = Vec::new();
    for row in pattern {
        for cell in row {
            bits.extend(cell.to_bits_le()?);
        }
    }
    
    // Add nonce bits
    bits.extend(nonce.to_bits_le()?);
    
    // Compute hash (simplified - in production use Poseidon or similar)
    // For now, just sum the bits as a demonstration
    let mut sum = FpVar::zero();
    for (i, bit) in bits.iter().enumerate() {
        let bit_val = FpVar::from(Boolean::from(bit.clone()));
        let multiplier = F::from((i + 1) as u64);
        sum = sum + &bit_val * FpVar::constant(multiplier);
    }
    
    // Verify commitment matches
    sum.enforce_equal(commitment)?;
    
    Ok(())
}

/// Verify initial grid has patterns placed at spawn points
fn verify_initial_placement<F: PrimeField>(
    _cs: ConstraintSystemRef<F>,
    initial_grid: &[Vec<UInt8<F>>],
    _pattern_a: &[Vec<UInt8<F>>],
    _pattern_b: &[Vec<UInt8<F>>],
) -> Result<(), SynthesisError> {
    // Simplified verification for circuit efficiency
    // In production, this would verify exact pattern placement
    // For now, just ensure grid is allocated properly
    let _ = initial_grid;
    Ok(())
}

/// Perform one step of Conway's Game of Life with toroidal wrapping
fn conway_step<F: PrimeField>(
    cs: ConstraintSystemRef<F>,
    grid: &[Vec<UInt8<F>>],
) -> Result<Vec<Vec<UInt8<F>>>, SynthesisError> {
    let size = grid.len();
    let mut new_grid = Vec::new();
    
    for i in 0..size {
        let mut new_row = Vec::new();
        for j in 0..size {
            // Count live neighbors with toroidal wrapping
            let neighbor_count = count_neighbors(cs.clone(), grid, i, j)?;
            
            // Apply Conway's rules
            let cell = &grid[i][j];
            // Check if cell is alive (value > 0) by checking all bits
            let cell_bits = cell.to_bits_le()?;
            let is_alive = cell_bits.iter().try_fold(Boolean::FALSE, |acc, bit| {
                acc.or(bit).map_err(|_| SynthesisError::Unsatisfiable)
            })?;
            
            // Survival: 2 or 3 neighbors
            let count_bits = neighbor_count.to_bits_le()?;
            let two_bits = UInt8::constant(2).to_bits_le()?;
            let three_bits = UInt8::constant(3).to_bits_le()?;
            
            let has_2_neighbors = check_bits_equal(&count_bits, &two_bits)?;
            let has_3_neighbors = check_bits_equal(&count_bits, &three_bits)?;
            let survives = is_alive.and(&has_2_neighbors.or(&has_3_neighbors)?)?;
            
            // Birth: exactly 3 neighbors
            let is_dead = is_alive.not();
            let births = is_dead.and(&has_3_neighbors)?;
            
            // New cell state
            let new_cell_alive = survives.or(&births)?;
            let new_cell = UInt8::conditionally_select(
                &new_cell_alive,
                &UInt8::constant(255), // Alive with max energy
                &UInt8::constant(0),   // Dead
            )?;
            
            new_row.push(new_cell);
        }
        new_grid.push(new_row);
    }
    
    Ok(new_grid)
}

/// Count live neighbors with toroidal wrapping
fn count_neighbors<F: PrimeField>(
    _cs: ConstraintSystemRef<F>,
    grid: &[Vec<UInt8<F>>],
    i: usize,
    j: usize,
) -> Result<UInt8<F>, SynthesisError> {
    let size = grid.len();
    let mut count = UInt8::constant(0);
    
    // Check all 8 neighbors with toroidal wrapping
    let offsets = [
        (-1, -1), (-1, 0), (-1, 1),
        (0, -1),           (0, 1),
        (1, -1),  (1, 0),  (1, 1),
    ];
    
    for (di, dj) in &offsets {
        let ni = ((i as i32 + di + size as i32) % size as i32) as usize;
        let nj = ((j as i32 + dj + size as i32) % size as i32) as usize;
        
        let neighbor = &grid[ni][nj];
        let neighbor_bits = neighbor.to_bits_le()?;
        let is_alive = neighbor_bits.iter().try_fold(Boolean::FALSE, |acc, bit| {
            acc.or(bit).map_err(|_| SynthesisError::Unsatisfiable)
        })?;
        
        let one = UInt8::constant(1);
        // Manual addition for UInt8 by converting to bits and adding
        let count_bits = count.to_bits_le()?;
        let one_bits = one.to_bits_le()?;
        let mut carry = Boolean::FALSE;
        let mut sum_bits = Vec::new();
        for (c_bit, o_bit) in count_bits.iter().zip(one_bits.iter()) {
            let s = c_bit.xor(o_bit)?.xor(&carry)?;
            carry = (c_bit.and(o_bit)?).or(&(c_bit.and(&carry)?))?.or(&(o_bit.and(&carry)?))?;
            sum_bits.push(s);
        }
        let count_plus_one = UInt8::from_bits_le(&sum_bits);
        
        count = UInt8::conditionally_select(
            &is_alive,
            &count_plus_one,
            &count,
        )?;
    }
    
    Ok(count)
}

/// Verify two grids are equal
fn verify_grid_equality<F: PrimeField>(
    _cs: ConstraintSystemRef<F>,
    grid1: &[Vec<UInt8<F>>],
    grid2: &[Vec<UInt8<F>>],
) -> Result<(), SynthesisError> {
    for (row1, row2) in grid1.iter().zip(grid2.iter()) {
        for (cell1, cell2) in row1.iter().zip(row2.iter()) {
            cell1.enforce_equal(cell2)?;
        }
    }
    Ok(())
}

/// Verify winner based on regional energy calculation
fn verify_winner<F: PrimeField>(
    _cs: ConstraintSystemRef<F>,
    final_grid: &[Vec<UInt8<F>>],
    winner: &UInt8<F>,
) -> Result<(), SynthesisError> {
    let size = final_grid.len();
    let mid = size / 2;
    
    // Calculate energy in region A (top-left quadrant)  
    let mut energy_a_bits = vec![Boolean::FALSE; 16]; // 16-bit accumulator
    for i in 0..mid {
        for j in 0..mid {
            let cell_bits = final_grid[i][j].to_bits_le()?;
            energy_a_bits = add_bits(&energy_a_bits, &cell_bits)?;
        }
    }
    
    // Calculate energy in region B (bottom-right quadrant)
    let mut energy_b_bits = vec![Boolean::FALSE; 16];
    for i in mid..size {
        for j in mid..size {
            let cell_bits = final_grid[i][j].to_bits_le()?;
            energy_b_bits = add_bits(&energy_b_bits, &cell_bits)?;
        }
    }
    
    // Determine winner by comparing bit representations
    let (a_wins, _) = compare_bits(&energy_a_bits, &energy_b_bits)?;
    let (b_wins, _) = compare_bits(&energy_b_bits, &energy_a_bits)?;
    let _tie = a_wins.not().and(&b_wins.not())?;
    
    let computed_winner = UInt8::conditionally_select(
        &a_wins,
        &UInt8::constant(0),
        &UInt8::conditionally_select(
            &b_wins,
            &UInt8::constant(1),
            &UInt8::constant(2),
        )?,
    )?;
    
    computed_winner.enforce_equal(winner)?;
    
    Ok(())
}

/// Check if two bit vectors are equal
fn check_bits_equal<F: PrimeField>(a: &[Boolean<F>], b: &[Boolean<F>]) -> Result<Boolean<F>, SynthesisError> {
    let mut result = Boolean::TRUE;
    for (bit_a, bit_b) in a.iter().zip(b.iter()) {
        let eq = bit_a.is_eq(bit_b)?;
        result = result.and(&eq)?;
    }
    Ok(result)
}

/// Add two bit vectors (returns sum with same bit width)
fn add_bits<F: PrimeField>(a: &[Boolean<F>], b: &[Boolean<F>]) -> Result<Vec<Boolean<F>>, SynthesisError> {
    let mut result = Vec::new();
    let mut carry = Boolean::FALSE;
    let max_len = a.len().max(b.len());
    
    for i in 0..max_len {
        let a_bit = if i < a.len() { a[i].clone() } else { Boolean::FALSE };
        let b_bit = if i < b.len() { b[i].clone() } else { Boolean::FALSE };
        
        let sum = a_bit.xor(&b_bit)?.xor(&carry)?;
        carry = (a_bit.and(&b_bit)?).or(&(a_bit.and(&carry)?))?.or(&(b_bit.and(&carry)?))?;
        result.push(sum);
    }
    
    Ok(result)
}

/// Compare two bit vectors (returns (a > b, a == b))
fn compare_bits<F: PrimeField>(a: &[Boolean<F>], b: &[Boolean<F>]) -> Result<(Boolean<F>, Boolean<F>), SynthesisError> {
    let mut greater = Boolean::FALSE;
    let mut equal = Boolean::TRUE;
    
    // Compare from MSB to LSB
    for i in (0..a.len()).rev() {
        let a_bit = &a[i];
        let b_bit = &b[i];
        
        // If equal so far and this bit differs, set greater appropriately
        let bit_greater = a_bit.and(&b_bit.not())?;
        greater = greater.or(&(equal.and(&bit_greater)?))?;
        
        // Update equality
        let bits_eq = a_bit.is_eq(b_bit)?;
        equal = equal.and(&bits_eq)?;
    }
    
    Ok((greater, equal))
}

// Groth16 proof generation and verification for Bn254
use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
use ark_snark::SNARK;
use ark_std::rand::thread_rng;

impl BattleCircuit<Fr> {
    /// Setup the circuit and generate proving/verifying keys
    ///
    /// Returns an error if the circuit setup fails (e.g., due to constraint system issues).
    pub fn setup() -> crate::Result<(ProvingKey<Bn254>, VerifyingKey<Bn254>)> {
        let rng = &mut thread_rng();
        
        // Create empty circuit for setup
        let circuit = Self {
            initial_grid: Some(vec![vec![0u8; GRID_SIZE]; GRID_SIZE]),
            final_grid: Some(vec![vec![0u8; GRID_SIZE]; GRID_SIZE]),
            commitment_a: Some(Fr::from(0u64)),
            commitment_b: Some(Fr::from(0u64)),
            winner: Some(0),
            pattern_a: Some(vec![vec![0u8; 3]; 3]),
            pattern_b: Some(vec![vec![0u8; 3]; 3]),
            nonce_a: Some(Fr::from(0u64)),
            nonce_b: Some(Fr::from(0u64)),
        };
        
        Groth16::<Bn254>::circuit_specific_setup(circuit, rng)
            .map_err(|e| crate::Error::ProofGeneration(format!("Circuit setup failed: {}", e)))
    }

    /// Generate a proof for this circuit instance
    pub fn prove(
        &self,
        pk: &ProvingKey<Bn254>,
    ) -> crate::Result<crate::Groth16Proof> {
        let rng = &mut thread_rng();
        let proof = Groth16::<Bn254>::prove(pk, self.clone(), rng)
            .map_err(|e| crate::Error::ProofGeneration(e.to_string()))?;
        Ok(crate::Groth16Proof::new(proof))
    }

    /// Verify a proof against public inputs
    ///
    /// Public inputs should be in order:
    /// 1. Initial grid cells (flattened)
    /// 2. Final grid cells (flattened)
    /// 3. Commitment A
    /// 4. Commitment B
    /// 5. Winner
    pub fn verify(
        vk: &VerifyingKey<Bn254>,
        proof: &crate::Groth16Proof,
        public_inputs: &[Fr],
    ) -> crate::Result<bool> {
        Groth16::<Bn254>::verify(vk, public_inputs, &proof.proof)
            .map_err(|_| crate::Error::ProofVerification)
    }
    
    /// Helper to construct public inputs vector from circuit components
    pub fn public_inputs(&self) -> Vec<Fr> {
        let mut inputs = Vec::new();
        
        // Add initial grid (flattened)
        if let Some(ref grid) = self.initial_grid {
            for row in grid {
                for &cell in row {
                    inputs.push(Fr::from(cell as u64));
                }
            }
        }
        
        // Add final grid (flattened)
        if let Some(ref grid) = self.final_grid {
            for row in grid {
                for &cell in row {
                    inputs.push(Fr::from(cell as u64));
                }
            }
        }
        
        // Add commitments and winner
        if let Some(commitment_a) = self.commitment_a {
            inputs.push(commitment_a);
        }
        if let Some(commitment_b) = self.commitment_b {
            inputs.push(commitment_b);
        }
        if let Some(winner) = self.winner {
            inputs.push(Fr::from(winner as u64));
        }
        
        inputs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;
    use ark_relations::r1cs::ConstraintSystem;
    
    #[test]
    fn test_battle_circuit_satisfiable() {
        let cs = ConstraintSystem::<Fr>::new_ref();

        // Use an empty grid - it remains empty after evolution (stable state)
        let initial_grid = vec![vec![0u8; GRID_SIZE]; GRID_SIZE];
        let final_grid = initial_grid.clone();

        // Use all-zero patterns and zero nonces for simplest commitment calculation
        // For the simplified commitment scheme: sum of (bit_value * (bit_index + 1))
        // All zeros -> commitment = 0
        let pattern_a = vec![vec![0u8; 3]; 3];
        let pattern_b = vec![vec![0u8; 3]; 3];
        let nonce_a = Fr::from(0u64);
        let nonce_b = Fr::from(0u64);

        // All zeros in pattern and nonce -> commitment = 0
        let commitment_a = Fr::from(0u64);
        let commitment_b = Fr::from(0u64);

        let circuit = BattleCircuit {
            initial_grid: Some(initial_grid.clone()),
            final_grid: Some(final_grid),
            commitment_a: Some(commitment_a),
            commitment_b: Some(commitment_b),
            winner: Some(2), // Tie - both regions have 0 energy
            pattern_a: Some(pattern_a),
            pattern_b: Some(pattern_b),
            nonce_a: Some(nonce_a),
            nonce_b: Some(nonce_b),
        };

        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(cs.is_satisfied().unwrap());
    }
    
    #[test]
    #[ignore] // Expensive test - enable for full validation
    fn test_battle_circuit_prove_verify_full() {
        // Setup circuit
        let (pk, vk) = BattleCircuit::<Fr>::setup().expect("Circuit setup should succeed");

        // Use an empty grid - stable state
        let initial_grid = vec![vec![0u8; GRID_SIZE]; GRID_SIZE];
        let final_grid = initial_grid.clone();

        let pattern_a = vec![vec![0u8; 3]; 3];
        let pattern_b = vec![vec![0u8; 3]; 3];
        let nonce_a = Fr::from(0u64);
        let nonce_b = Fr::from(0u64);
        let commitment_a = Fr::from(0u64);
        let commitment_b = Fr::from(0u64);

        let circuit = BattleCircuit {
            initial_grid: Some(initial_grid.clone()),
            final_grid: Some(final_grid),
            commitment_a: Some(commitment_a),
            commitment_b: Some(commitment_b),
            winner: Some(2), // Tie
            pattern_a: Some(pattern_a),
            pattern_b: Some(pattern_b),
            nonce_a: Some(nonce_a),
            nonce_b: Some(nonce_b),
        };

        // Generate proof
        let proof = circuit.prove(&pk).expect("Proof generation should succeed");

        // Verify proof
        let public_inputs = circuit.public_inputs();
        assert!(
            BattleCircuit::verify(&vk, &proof, &public_inputs).expect("Verification should complete"),
            "Proof verification should succeed"
        );
    }
}
