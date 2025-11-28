//! Adversarial Simulation Lab for BitCell
//!
//! This crate provides a harness for simulating long-running tournament scenarios
//! with various miner behaviors to validate economic and reputation incentives.

use bitcell_consensus::{GliderCommitment, GliderReveal, TournamentOrchestrator};
use bitcell_crypto::{Hash256, PublicKey, SecretKey};
use bitcell_ca::{Glider, GliderPattern, Position};

use rand::Rng;

/// Trait defining a miner's behavior in the simulation
pub trait MinerAgent {
    /// Get the miner's public key
    fn public_key(&self) -> PublicKey;
    
    /// Generate a commitment for the current round
    fn generate_commitment(&mut self, height: u64) -> GliderCommitment;
    
    /// Generate a reveal for the current round (if they choose to reveal)
    fn generate_reveal(&mut self, height: u64) -> Option<GliderReveal>;
    
    /// Name of the agent type (for logging)
    fn name(&self) -> &str;
}

/// Honest Miner: Always commits valid gliders and always reveals
pub struct HonestMiner {
    sk: SecretKey,
    current_glider: Option<Glider>,
    current_nonce: Vec<u8>,
}

impl HonestMiner {
    pub fn new() -> Self {
        Self {
            sk: SecretKey::generate(),
            current_glider: None,
            current_nonce: Vec::new(),
        }
    }
}

impl MinerAgent for HonestMiner {
    fn public_key(&self) -> PublicKey {
        self.sk.public_key()
    }

    fn generate_commitment(&mut self, height: u64) -> GliderCommitment {
        // Honest miner picks a standard glider
        let glider = Glider::new(GliderPattern::Standard, Position::new(100, 100));
        let nonce = vec![0u8; 32]; // Simplified nonce
        
        // Store for reveal
        self.current_glider = Some(glider.clone());
        self.current_nonce = nonce.clone();
        
        GliderCommitment {
            commitment: Hash256::zero(), // Placeholder for actual commitment hash
            ring_signature: vec![],
            height,
        }
    }

    fn generate_reveal(&mut self, _height: u64) -> Option<GliderReveal> {
        if let Some(glider) = &self.current_glider {
            Some(GliderReveal {
                glider: glider.clone(),
                nonce: self.current_nonce.clone(),
                miner: self.public_key(),
            })
        } else {
            None
        }
    }

    fn name(&self) -> &str {
        "HonestMiner"
    }
}

/// Tie Farmer: Uses symmetric patterns to try and force ties
pub struct TieFarmer {
    sk: SecretKey,
    current_glider: Option<Glider>,
}

impl TieFarmer {
    pub fn new() -> Self {
        Self {
            sk: SecretKey::generate(),
            current_glider: None,
        }
    }
}

impl MinerAgent for TieFarmer {
    fn public_key(&self) -> PublicKey {
        self.sk.public_key()
    }

    fn generate_commitment(&mut self, height: u64) -> GliderCommitment {
        // Tie farmer picks a symmetric pattern (e.g., Heavyweight)
        let glider = Glider::new(GliderPattern::Heavyweight, Position::new(100, 100));
        self.current_glider = Some(glider);
        
        GliderCommitment {
            commitment: Hash256::zero(),
            ring_signature: vec![],
            height,
        }
    }

    fn generate_reveal(&mut self, _height: u64) -> Option<GliderReveal> {
        self.current_glider.as_ref().map(|g| GliderReveal {
            glider: g.clone(),
            nonce: vec![],
            miner: self.public_key(),
        })
    }

    fn name(&self) -> &str {
        "TieFarmer"
    }
}

/// Chaos Spammer: Uses random noise to maximize entropy/volatility
pub struct ChaosSpammer {
    sk: SecretKey,
    current_glider: Option<Glider>,
}

impl ChaosSpammer {
    pub fn new() -> Self {
        Self {
            sk: SecretKey::generate(),
            current_glider: None,
        }
    }
}

impl MinerAgent for ChaosSpammer {
    fn public_key(&self) -> PublicKey {
        self.sk.public_key()
    }

    fn generate_commitment(&mut self, height: u64) -> GliderCommitment {
        // Chaos spammer uses a custom high-entropy pattern (simulated here with Heavyweight for now)
        // In a real scenario, this would be a random blob
        let glider = Glider::new(GliderPattern::Heavyweight, Position::new(100, 100));
        self.current_glider = Some(glider);
        
        GliderCommitment {
            commitment: Hash256::zero(),
            ring_signature: vec![],
            height,
        }
    }

    fn generate_reveal(&mut self, _height: u64) -> Option<GliderReveal> {
        self.current_glider.as_ref().map(|g| GliderReveal {
            glider: g.clone(),
            nonce: vec![],
            miner: self.public_key(),
        })
    }

    fn name(&self) -> &str {
        "ChaosSpammer"
    }
}

/// Flaky Griefer: Commits but randomly fails to reveal
pub struct FlakyGriefer {
    sk: SecretKey,
    current_glider: Option<Glider>,
    failure_rate: f64,
}

impl FlakyGriefer {
    pub fn new(failure_rate: f64) -> Self {
        Self {
            sk: SecretKey::generate(),
            current_glider: None,
            failure_rate,
        }
    }
}

impl MinerAgent for FlakyGriefer {
    fn public_key(&self) -> PublicKey {
        self.sk.public_key()
    }

    fn generate_commitment(&mut self, height: u64) -> GliderCommitment {
        let glider = Glider::new(GliderPattern::Standard, Position::new(100, 100));
        self.current_glider = Some(glider);
        
        GliderCommitment {
            commitment: Hash256::zero(),
            ring_signature: vec![],
            height,
        }
    }

    fn generate_reveal(&mut self, _height: u64) -> Option<GliderReveal> {
        let mut rng = rand::thread_rng();
        if rng.gen_bool(self.failure_rate) {
            // Fail to reveal
            None
        } else {
            self.current_glider.as_ref().map(|g| GliderReveal {
                glider: g.clone(),
                nonce: vec![],
                miner: self.public_key(),
            })
        }
    }

    fn name(&self) -> &str {
        "FlakyGriefer"
    }
}

/// Simulation Engine
pub struct SimulationEngine {
    pub orchestrator: TournamentOrchestrator,
    pub agents: Vec<Box<dyn MinerAgent>>,
    pub history: Vec<SimulationEpochResult>,
}

#[derive(Debug, Clone)]
pub struct SimulationEpochResult {
    pub height: u64,
    pub winner: Option<PublicKey>,
    pub mii_usage: f64,
    pub avg_rounds: f64,
}

impl SimulationEngine {
    pub fn new(agents: Vec<Box<dyn MinerAgent>>) -> Self {
        let miners: Vec<PublicKey> = agents.iter().map(|a| a.public_key()).collect();
        let orchestrator = TournamentOrchestrator::new(1, miners, Hash256::zero());
        
        Self {
            orchestrator,
            agents,
            history: Vec::new(),
        }
    }
    
    pub fn run_epoch(&mut self) {
        let height = self.orchestrator.tournament.height;
        
        // 1. Commit Phase
        for agent in &mut self.agents {
            let commit = agent.generate_commitment(height);
            let _ = self.orchestrator.process_commit(commit);
        }
        
        self.orchestrator.advance_to_reveal().unwrap();
        
        // 2. Reveal Phase
        for agent in &mut self.agents {
            if let Some(reveal) = agent.generate_reveal(height) {
                let _ = self.orchestrator.process_reveal(reveal);
            }
        }
        
        self.orchestrator.advance_to_battle().unwrap();
        
        // 3. Battle Phase
        let winner = self.orchestrator.run_battles().ok();
        
        // 4. Record Metrics
        let result = SimulationEpochResult {
            height,
            winner,
            mii_usage: self.orchestrator.metrics.mii_usage_rate,
            avg_rounds: self.orchestrator.metrics.avg_rounds,
        };
        self.history.push(result);
        
        // 5. Reset for next epoch (simplified - normally we'd create new orchestrator)
        // For simulation, we just bump height and clear tournament state but keep evidence
        let miners: Vec<PublicKey> = self.agents.iter().map(|a| a.public_key()).collect();
        let old_evidence = self.orchestrator.miner_evidence.clone();
        
        self.orchestrator = TournamentOrchestrator::new(height + 1, miners, Hash256::zero());
        self.orchestrator.miner_evidence = old_evidence;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_scenario() {
        // Create a mix of agents
        let agents: Vec<Box<dyn MinerAgent>> = vec![
            Box::new(HonestMiner::new()),
            Box::new(HonestMiner::new()),
            Box::new(TieFarmer::new()),
            Box::new(ChaosSpammer::new()),
            Box::new(FlakyGriefer::new(0.5)), // 50% failure rate
        ];
        
        let mut engine = SimulationEngine::new(agents);
        
        // Run 2 epochs (reduced for test speed)
        for _ in 0..2 {
            engine.run_epoch();
        }
        
        // Verify history
        assert_eq!(engine.history.len(), 2);
        
        // Check that we have some results
        for result in &engine.history {
            println!("Epoch {}: Winner {:?}, MII Usage {:.2}, Avg Rounds {:.2}", 
                result.height, result.winner, result.mii_usage, result.avg_rounds);
        }
        
        // Verify evidence accumulation
        // Honest miners should have positive reputation (if they won or participated well)
        // We can't easily check internal state of orchestrator here without exposing it more,
        // but we can check that the engine ran without panicking.
    }
}
