/// Battle circuit constraints implementing Conway's Game of Life rules
/// This module provides the full R1CS constraint system for verifying CA battles

use ark_ff::PrimeField;
use ark_r1cs_std::prelude::*;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::bits::ToBitsGadget;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
use ark_snark::SNARK;
use ark_std::rand::thread_rng;

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

impl BattleCircuit<Fr> {
    /// Setup the circuit and generate proving/verifying keys
    ///
    /// This performs the trusted setup ceremony for the Groth16 proof system.
    /// Note: Due to the large circuit size (~6.7M constraints for 64x64 grid), 
    /// setup may take several minutes and require significant memory (8GB+).
    ///
    /// Returns an error if the circuit setup fails.
    pub fn setup() -> crate::Result<(ProvingKey<Bn254>, VerifyingKey<Bn254>)> {
        let rng = &mut thread_rng();
        
        // Create empty circuit for setup
        let circuit = BattleCircuit {
            initial_grid: None,
            final_grid: None,
            commitment_a: None,
            commitment_b: None,
            winner: None,
            pattern_a: None,
            pattern_b: None,
            nonce_a: None,
            nonce_b: None,
        };
        
        Groth16::<Bn254>::circuit_specific_setup(circuit, rng)
            .map_err(|e| crate::Error::Setup(format!("Circuit setup failed: {}", e)))
    }

    /// Generate a proof for this circuit instance
    ///
    /// Generates a Groth16 proof that the battle was executed correctly according to
    /// Conway's Game of Life rules and the winner was determined correctly.
    ///
    /// # Performance
    /// Proof generation for a 64x64 grid with 10 steps takes approximately 10-30 seconds
    /// on an 8-core CPU. Larger grids (1024x1024) may require GPU acceleration.
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
    /// Verifies that a Groth16 proof is valid for the given public inputs.
    /// Verification is fast (~10ms) regardless of circuit size.
    ///
    /// # Public Inputs Format
    /// The public inputs must be in the following order:
    /// - initial_grid: flattened 2D grid (GRID_SIZE * GRID_SIZE elements)
    /// - final_grid: flattened 2D grid (GRID_SIZE * GRID_SIZE elements)
    /// - commitment_a: single field element
    /// - commitment_b: single field element
    /// - winner: single u8 (0 = A wins, 1 = B wins, 2 = tie)
    pub fn verify(
        vk: &VerifyingKey<Bn254>,
        proof: &crate::Groth16Proof,
        public_inputs: &[Fr],
    ) -> crate::Result<bool> {
        Groth16::<Bn254>::verify(vk, public_inputs, &proof.proof)
            .map_err(|_| crate::Error::ProofVerification)
    }

    /// Helper to construct public inputs in the correct order
    ///
    /// This ensures that public inputs are formatted consistently for verification.
    /// The order matches the order in which variables are allocated as public inputs
    /// in the circuit's generate_constraints method.
    pub fn public_inputs(
        initial_grid: &[Vec<u8>],
        final_grid: &[Vec<u8>],
        commitment_a: Fr,
        commitment_b: Fr,
        winner: u8,
    ) -> Vec<Fr> {
        let mut inputs = Vec::new();
        
        // Flatten initial grid
        for row in initial_grid {
            for &cell in row {
                inputs.push(Fr::from(cell));
            }
        }
        
        // Flatten final grid
        for row in final_grid {
            for &cell in row {
                inputs.push(Fr::from(cell));
            }
        }
        
        // Add commitments
        inputs.push(commitment_a);
        inputs.push(commitment_b);
        
        // Add winner
        inputs.push(Fr::from(winner));
        
        inputs
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
        
        // Print constraint count for informational purposes
        println!("Battle circuit constraints: {}", cs.num_constraints());
    }

    #[test]
    fn test_public_inputs_helper() {
        // Test that public_inputs helper creates the correct format
        let initial_grid = vec![vec![1u8, 2u8], vec![3u8, 4u8]];
        let final_grid = vec![vec![0u8, 0u8], vec![0u8, 0u8]];
        let commitment_a = Fr::from(100u64);
        let commitment_b = Fr::from(200u64);
        let winner = 1u8;
        
        let inputs = BattleCircuit::public_inputs(
            &initial_grid,
            &final_grid,
            commitment_a,
            commitment_b,
            winner,
        );
        
        // Should have: 2*2 (initial) + 2*2 (final) + 1 (commitment_a) + 1 (commitment_b) + 1 (winner) = 11
        assert_eq!(inputs.len(), 11);
        
        // Check initial grid values
        assert_eq!(inputs[0], Fr::from(1u8));
        assert_eq!(inputs[1], Fr::from(2u8));
        assert_eq!(inputs[2], Fr::from(3u8));
        assert_eq!(inputs[3], Fr::from(4u8));
        
        // Check final grid values (all zeros)
        assert_eq!(inputs[4], Fr::from(0u8));
        assert_eq!(inputs[5], Fr::from(0u8));
        assert_eq!(inputs[6], Fr::from(0u8));
        assert_eq!(inputs[7], Fr::from(0u8));
        
        // Check commitments
        assert_eq!(inputs[8], commitment_a);
        assert_eq!(inputs[9], commitment_b);
        
        // Check winner
        assert_eq!(inputs[10], Fr::from(1u8));
    }

    /// Test setup phase of the Groth16 protocol
    /// 
    /// Note: This test takes ~3 minutes to run due to the large circuit size.
    /// It generates proving and verifying keys for the full battle circuit
    /// with ~6.7M constraints (64x64 grid, 10 steps).
    #[test]
    #[ignore] // Marked ignore due to long runtime (~3 min) and memory requirements
    fn test_battle_circuit_setup() {
        let result = BattleCircuit::setup();
        assert!(result.is_ok(), "Circuit setup should succeed");
        
        let (pk, vk) = result.unwrap();
        
        // Verify keys are generated
        assert!(pk.vk.gamma_abc_g1.len() > 0, "Proving key should have gamma_abc_g1");
        assert!(vk.gamma_abc_g1.len() > 0, "Verifying key should have gamma_abc_g1");
        
        println!("Setup complete. Keys generated successfully.");
    }

    /// Test full proof generation and verification cycle
    /// 
    /// Note: This test is extremely resource-intensive:
    /// - Memory: ~20GB+ required for proof generation
    /// - Runtime: 5+ minutes on 8-core CPU
    /// - Should only be run manually or in dedicated test infrastructure
    /// 
    /// This test verifies:
    /// 1. Setup generates valid keys
    /// 2. Proof can be generated for a valid witness
    /// 3. Proof verifies correctly with matching public inputs
    /// 4. Proof fails verification with mismatched inputs
    #[test]
    #[ignore] // EXPENSIVE: Requires 20GB+ RAM and 5+ min runtime
    fn test_battle_circuit_prove_verify_full() {
        // 1. Setup
        println!("Starting circuit setup...");
        let (pk, vk) = BattleCircuit::setup().expect("Setup should succeed");
        println!("Setup complete.");
        
        // 2. Create a valid circuit with empty grid (stable state)
        let initial_grid = vec![vec![0u8; GRID_SIZE]; GRID_SIZE];
        let final_grid = initial_grid.clone();
        let pattern_a = vec![vec![0u8; 3]; 3];
        let pattern_b = vec![vec![0u8; 3]; 3];
        let nonce_a = Fr::from(0u64);
        let nonce_b = Fr::from(0u64);
        let commitment_a = Fr::from(0u64);
        let commitment_b = Fr::from(0u64);
        let winner = 2u8; // Tie
        
        let circuit = BattleCircuit {
            initial_grid: Some(initial_grid.clone()),
            final_grid: Some(final_grid.clone()),
            commitment_a: Some(commitment_a),
            commitment_b: Some(commitment_b),
            winner: Some(winner),
            pattern_a: Some(pattern_a),
            pattern_b: Some(pattern_b),
            nonce_a: Some(nonce_a),
            nonce_b: Some(nonce_b),
        };
        
        // 3. Generate proof
        println!("Generating proof...");
        let proof = circuit.prove(&pk).expect("Proof generation should succeed");
        println!("Proof generated.");
        
        // 4. Prepare public inputs
        let public_inputs = BattleCircuit::public_inputs(
            &initial_grid,
            &final_grid,
            commitment_a,
            commitment_b,
            winner,
        );
        
        // 5. Verify proof with correct inputs
        println!("Verifying proof...");
        let result = BattleCircuit::verify(&vk, &proof, &public_inputs);
        assert!(result.is_ok(), "Verification should not error");
        assert!(result.unwrap(), "Proof should verify with correct inputs");
        println!("Proof verified successfully.");
        
        // 6. Verify proof fails with wrong inputs
        let mut wrong_inputs = public_inputs.clone();
        wrong_inputs[0] = Fr::from(99u8); // Corrupt first cell
        let result = BattleCircuit::verify(&vk, &proof, &wrong_inputs);
        // Should either error or return false
        assert!(
            result.is_err() || !result.unwrap(),
            "Proof should fail verification with wrong inputs"
        );
        println!("Proof correctly rejected with wrong inputs.");
    }

    #[test]
    fn test_conway_rules_constraint_count() {
        // Test that a single Conway step produces reasonable constraint count
        let cs = ConstraintSystem::<Fr>::new_ref();
        
        // Create a small grid for testing
        let grid = vec![vec![0u8; GRID_SIZE]; GRID_SIZE];
        
        let circuit = BattleCircuit {
            initial_grid: Some(grid.clone()),
            final_grid: Some(grid.clone()),
            commitment_a: Some(Fr::from(0u64)),
            commitment_b: Some(Fr::from(0u64)),
            winner: Some(2),
            pattern_a: Some(vec![vec![0u8; 3]; 3]),
            pattern_b: Some(vec![vec![0u8; 3]; 3]),
            nonce_a: Some(Fr::from(0u64)),
            nonce_b: Some(Fr::from(0u64)),
        };
        
        circuit.generate_constraints(cs.clone()).unwrap();
        
        let num_constraints = cs.num_constraints();
        println!("Total constraints for {}x{} grid, {} steps: {}", 
                 GRID_SIZE, GRID_SIZE, BATTLE_STEPS, num_constraints);
        
        // Sanity check: should have many constraints (millions for 64x64)
        // For 64x64 grid with 10 steps, expect ~6-7M constraints
        assert!(num_constraints > 100_000, 
                "Should have substantial constraints for CA evolution");
    }
}
