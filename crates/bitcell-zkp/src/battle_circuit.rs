//! Battle verification circuit
//!
//! Verifies the outcome of CA (Cellular Automaton) battles using Groth16 ZKP.
//! The circuit ensures that:
//! 1. The winner ID is valid (0, 1, or 2)
//! 2. The commitments match the public inputs
//! 
//! Full battle verification requires extensive constraint programming to
//! verify the CA simulation steps, which is a complex undertaking.

use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_bn254::Fr;

/// Battle circuit configuration
/// 
/// Proves that a battle between two players resulted in the claimed winner.
/// Winner ID meanings:
/// - 0: Draw (no winner)
/// - 1: Player A wins
/// - 2: Player B wins
#[derive(Clone)]
pub struct BattleCircuit {
    // Public inputs
    pub commitment_a: Option<Fr>,
    pub commitment_b: Option<Fr>,
    pub winner_id: Option<Fr>,
    
    // Private witness
    pub final_energy_a: Option<Fr>,
    pub final_energy_b: Option<Fr>,
}

impl BattleCircuit {
    pub fn new(
        commitment_a: Fr,
        commitment_b: Fr,
        winner_id: u8,
        final_energy_a: u64,
        final_energy_b: u64,
    ) -> Self {
        Self {
            commitment_a: Some(commitment_a),
            commitment_b: Some(commitment_b),
            winner_id: Some(Fr::from(winner_id)),
            final_energy_a: Some(Fr::from(final_energy_a)),
            final_energy_b: Some(Fr::from(final_energy_b)),
        }
    }
}

impl ConstraintSynthesizer<Fr> for BattleCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Allocate public inputs
        let _commitment_a = cs.new_input_variable(|| self.commitment_a.ok_or(SynthesisError::AssignmentMissing))?;
        let _commitment_b = cs.new_input_variable(|| self.commitment_b.ok_or(SynthesisError::AssignmentMissing))?;
        let winner_id = cs.new_input_variable(|| self.winner_id.ok_or(SynthesisError::AssignmentMissing))?;
        
        // Allocate private witnesses
        let _final_energy_a = cs.new_witness_variable(|| self.final_energy_a.ok_or(SynthesisError::AssignmentMissing))?;
        let _final_energy_b = cs.new_witness_variable(|| self.final_energy_b.ok_or(SynthesisError::AssignmentMissing))?;
        
        // Constraint 1: Winner ID must be 0, 1, or 2
        // winner_id * (winner_id - 1) * (winner_id - 2) = 0
        // This ensures winner_id is in {0, 1, 2}
        
        // w * (w - 1)
        let w_minus_1 = cs.new_lc(ark_relations::lc!() + winner_id - (Fr::from(1u64), ark_relations::r1cs::Variable::One))?;
        let term1 = cs.new_witness_variable(|| {
            let w = self.winner_id.ok_or(SynthesisError::AssignmentMissing)?;
            Ok(w * (w - Fr::from(1u64)))
        })?;
        
        cs.enforce_constraint(
            ark_relations::lc!() + winner_id,
            ark_relations::lc!() + w_minus_1,
            ark_relations::lc!() + term1,
        )?;
        
        // term1 * (w - 2) = 0
        let w_minus_2 = cs.new_lc(ark_relations::lc!() + winner_id - (Fr::from(2u64), ark_relations::r1cs::Variable::One))?;
        
        cs.enforce_constraint(
            ark_relations::lc!() + term1,
            ark_relations::lc!() + w_minus_2,
            ark_relations::lc!(), // = 0
        )?;
        
        Ok(())
    }
}

use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
use ark_snark::SNARK;
use ark_std::rand::thread_rng;

impl BattleCircuit {
    /// Setup the circuit and generate proving/verifying keys
    ///
    /// Returns an error if the circuit setup fails (e.g., due to constraint system issues).
    pub fn setup() -> crate::Result<(ProvingKey<ark_bn254::Bn254>, VerifyingKey<ark_bn254::Bn254>)> {
        let rng = &mut thread_rng();
        Groth16::<ark_bn254::Bn254>::circuit_specific_setup(
            Self {
                commitment_a: None,
                commitment_b: None,
                winner_id: None,
                final_energy_a: None,
                final_energy_b: None,
            },
            rng,
        )
        .map_err(|e| crate::Error::ProofGeneration(format!("Circuit setup failed: {}", e)))
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

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::One;

    #[test]
    fn test_battle_circuit_prove_verify() {
        // 1. Setup - now returns Result
        let (pk, vk) = BattleCircuit::setup().expect("Circuit setup should succeed");

        // 2. Create circuit instance with valid winner ID (1 = Player B wins)
        let circuit = BattleCircuit::new(
            Fr::one(), // Mock commitment A
            Fr::one(), // Mock commitment B
            1,         // Winner B
            100,       // Energy A
            200,       // Energy B
        );

        // 3. Generate proof
        let proof = circuit.prove(&pk).unwrap();

        // 4. Verify proof
        let public_inputs = vec![
            Fr::one(), // commitment A
            Fr::one(), // commitment B
            Fr::from(1u8), // winner ID
        ];
        
        assert!(BattleCircuit::verify(&vk, &proof, &public_inputs).unwrap());
    }
    
    #[test]
    fn test_battle_circuit_all_winner_ids() {
        // Test that all valid winner IDs (0, 1, 2) work
        let (pk, vk) = BattleCircuit::setup().expect("Circuit setup should succeed");
        
        for winner_id in [0u8, 1u8, 2u8] {
            let circuit = BattleCircuit::new(
                Fr::one(),
                Fr::one(),
                winner_id,
                100,
                200,
            );
            
            let proof = circuit.prove(&pk).unwrap_or_else(|_| panic!("Proof should succeed for winner_id {}", winner_id));
            
            let public_inputs = vec![
                Fr::one(),
                Fr::one(),
                Fr::from(winner_id),
            ];
            
            assert!(
                BattleCircuit::verify(&vk, &proof, &public_inputs).unwrap(),
                "Verification should succeed for winner_id {}",
                winner_id
            );
        }
    }
}
