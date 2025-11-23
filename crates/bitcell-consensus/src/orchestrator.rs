//! Tournament orchestration
//!
//! Coordinates the commit-reveal-battle flow for each block height

use crate::{Tournament, TournamentPhase, GliderCommitment, GliderReveal, Error, Result};
use bitcell_crypto::{Hash256, PublicKey};
use bitcell_ebsl::{EvidenceCounters, TrustScore, EbslParams, Evidence, EvidenceType};
use std::collections::HashMap;

/// Tournament orchestrator
pub struct TournamentOrchestrator {
    /// Current tournament state
    pub tournament: Tournament,
    
    /// EBSL parameters
    pub ebsl_params: EbslParams,
    
    /// Miner evidence counters
    pub miner_evidence: HashMap<PublicKey, EvidenceCounters>,
    
    /// Block time in seconds
    pub block_time: u64,
}

impl TournamentOrchestrator {
    pub fn new(height: u64, eligible_miners: Vec<PublicKey>, seed: Hash256) -> Self {
        Self {
            tournament: Tournament::new(height, eligible_miners, seed),
            ebsl_params: EbslParams::default(),
            miner_evidence: HashMap::new(),
            block_time: 600, // 10 minutes
        }
    }

    /// Process commit phase
    pub fn process_commit(&mut self, commitment: GliderCommitment) -> Result<()> {
        if self.tournament.phase != TournamentPhase::Commit {
            return Err(Error::TournamentError("Not in commit phase".to_string()));
        }

        self.tournament.commitments.push(commitment);
        Ok(())
    }

    /// Advance to reveal phase
    pub fn advance_to_reveal(&mut self) -> Result<()> {
        if self.tournament.phase != TournamentPhase::Commit {
            return Err(Error::TournamentError("Not in commit phase".to_string()));
        }

        self.tournament.phase = TournamentPhase::Reveal;
        Ok(())
    }

    /// Process reveal
    pub fn process_reveal(&mut self, reveal: GliderReveal) -> Result<()> {
        if self.tournament.phase != TournamentPhase::Reveal {
            return Err(Error::TournamentError("Not in reveal phase".to_string()));
        }

        // Verify reveal matches commitment (simplified)
        self.tournament.reveals.push(reveal);
        Ok(())
    }

    /// Advance to battle phase
    pub fn advance_to_battle(&mut self) -> Result<()> {
        if self.tournament.phase != TournamentPhase::Reveal {
            return Err(Error::TournamentError("Not in reveal phase".to_string()));
        }

        self.tournament.phase = TournamentPhase::Battle;
        Ok(())
    }

    /// Run all battles
    pub fn run_battles(&mut self) -> Result<PublicKey> {
        if self.tournament.phase != TournamentPhase::Battle {
            return Err(Error::TournamentError("Not in battle phase".to_string()));
        }

        // Get winner miner before mutable borrow
        let winner_miner = self.tournament.reveals.first()
            .map(|r| r.miner)
            .ok_or_else(|| Error::TournamentError("No reveals".to_string()))?;

        // Now we can mutate
        self.tournament.winner = Some(winner_miner);
        self.tournament.phase = TournamentPhase::Complete;
        
        // Record positive evidence for winner
        self.record_evidence(winner_miner, EvidenceType::GoodBlock);
        
        Ok(winner_miner)
    }

    /// Record evidence for a miner
    pub fn record_evidence(&mut self, miner: PublicKey, evidence_type: EvidenceType) {
        let counters = self.miner_evidence.entry(miner).or_insert_with(EvidenceCounters::new);
        counters.add_evidence(Evidence::new(evidence_type, 0, self.tournament.height));
    }

    /// Check if miner is eligible based on EBSL
    pub fn is_eligible(&self, miner: &PublicKey) -> bool {
        if let Some(counters) = self.miner_evidence.get(miner) {
            let trust = TrustScore::from_evidence(counters, &self.ebsl_params);
            trust.is_eligible(&self.ebsl_params)
        } else {
            // New miners start below threshold
            false
        }
    }

    /// Get tournament winner
    pub fn get_winner(&self) -> Option<PublicKey> {
        self.tournament.winner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcell_crypto::SecretKey;

    #[test]
    fn test_orchestrator_phases() {
        let sk = SecretKey::generate();
        let miners = vec![sk.public_key()];
        let mut orch = TournamentOrchestrator::new(1, miners, Hash256::zero());

        assert_eq!(orch.tournament.phase, TournamentPhase::Commit);

        orch.advance_to_reveal().unwrap();
        assert_eq!(orch.tournament.phase, TournamentPhase::Reveal);

        orch.advance_to_battle().unwrap();
        assert_eq!(orch.tournament.phase, TournamentPhase::Battle);
    }

    #[test]
    fn test_evidence_recording() {
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        let miners = vec![pk];
        let mut orch = TournamentOrchestrator::new(1, miners, Hash256::zero());

        orch.record_evidence(pk, EvidenceType::GoodBlock);
        
        let counters = orch.miner_evidence.get(&pk).unwrap();
        assert!(counters.r > 0.0);
    }
}
