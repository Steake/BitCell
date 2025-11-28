//! Tournament orchestration
//!
//! Coordinates the commit-reveal-battle flow for each block height

use crate::{Tournament, TournamentPhase, GliderCommitment, GliderReveal, Error, Result, TournamentMatch};

use bitcell_crypto::{Hash256, PublicKey};
use bitcell_ebsl::{EvidenceCounters, TrustScore, EbslParams, Evidence, EvidenceType};
use serde::{Deserialize, Serialize};
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
    
    /// Global tournament metrics
    pub metrics: TournamentMetrics,
}

/// Behavioural profile for a miner
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MinerBehaviour {
    pub participation_count: u32,
    pub win_rate: f64,
    pub aggression_index: f64, // MII inflicted / MII received
    pub volatility_index: f64, // TED score
    pub stall_tendency: f64,   // Frequency of ties
}

/// Network-level tournament metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TournamentMetrics {
    pub avg_rounds: f64,
    pub bye_frequency: f64,
    pub mii_usage_rate: f64,
    pub tiebreaker_rate: f64,
}

impl TournamentOrchestrator {
    pub fn new(height: u64, eligible_miners: Vec<PublicKey>, seed: Hash256) -> Self {
        Self {
            tournament: Tournament::new(height, eligible_miners, seed),
            ebsl_params: EbslParams::default(),
            miner_evidence: HashMap::new(),
            block_time: 600, // 10 minutes
            metrics: TournamentMetrics::default(),
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

        // 1. Collect participants (reveals) in canonical order
        // We use eligible_miners order to ensure determinism
        let mut participants: Vec<GliderReveal> = Vec::new();
        let reveal_map: HashMap<PublicKey, GliderReveal> = self.tournament.reveals.iter()
            .map(|r| (r.miner, r.clone()))
            .collect();

        for miner in &self.tournament.eligible_miners {
            if let Some(reveal) = reveal_map.get(miner) {
                participants.push(reveal.clone());
            }
        }

        if participants.is_empty() {
             return Err(Error::TournamentError("No participants revealed".to_string()));
        }

        // 2. Run tournament rounds
        let mut round = 0;
        while participants.len() > 1 {
            let mut next_round_participants = Vec::new();
            let mut match_index = 0;
            
            // Pair up participants
            let mut i = 0;
            while i < participants.len() {
                if i + 1 < participants.len() {
                    // Pair found
                    let p_a = &participants[i];
                    let p_b = &participants[i+1];
                    
                    // Derive entropy
                    let entropy = self.derive_match_entropy(round, match_index);
                    
                    // Determine if this is the final battle of the tournament
                    // It is final if this is the only match in this round AND no byes
                    // Or simply: if participants.len() == 2, this is the final match.
                    let is_final = participants.len() == 2;
                    
                    // Configure battle
                    let battle = if is_final {
                        bitcell_ca::Battle::with_history(p_a.glider.clone(), p_b.glider.clone(), 1000, entropy)
                    } else {
                        bitcell_ca::Battle::with_entropy(p_a.glider.clone(), p_b.glider.clone(), 1000, entropy)
                    };
                    
                    // Run simulation
                    let (outcome, history) = if is_final {
                        battle.simulate_with_history()
                    } else {
                        (battle.simulate(), None)
                    };
                    
                    // Determine winner
                    let winner_miner = match outcome {
                        bitcell_ca::BattleOutcome::AWins => p_a.miner,
                        bitcell_ca::BattleOutcome::BWins => p_b.miner,
                        bitcell_ca::BattleOutcome::Tie => p_a.miner, // Should not happen with MII+, but fallback to A
                    };
                    
                    // Record match
                    let match_record = TournamentMatch {
                        round,
                        match_index,
                        participant_a: p_a.miner,
                        participant_b: p_b.miner,
                        winner: winner_miner,
                        entropy_seed: entropy,
                        battle_config: battle,
                        outcome,
                        history,
                        proof_data: vec![], // Placeholder
                    };
                    self.tournament.matches.push(match_record);
                    
                    // Advance winner
                    if winner_miner == p_a.miner {
                        next_round_participants.push(p_a.clone());
                    } else {
                        next_round_participants.push(p_b.clone());
                    }
                    
                    match_index += 1;
                    i += 2;
                } else {
                    // Bye (odd number of participants)
                    next_round_participants.push(participants[i].clone());
                    i += 1;
                }
            }
            
            participants = next_round_participants;
            round += 1;
        }
        
        let winner_miner = participants[0].miner;

        // Now we can mutate
        self.tournament.winner = Some(winner_miner);
        self.tournament.phase = TournamentPhase::Complete;
        
        // Record positive evidence for winner
        self.record_evidence(winner_miner, EvidenceType::GoodBlock);
        
        // 3. Adaptive Strategy: Analyze tournament and update reputation
        self.analyze_tournament();
        
        Ok(winner_miner)
    }

    /// Analyze tournament results and update miner reputation
    fn analyze_tournament(&mut self) {
        let mut miner_stats: HashMap<PublicKey, MinerBehaviour> = HashMap::new();
        let matches = &self.tournament.matches;
        
        // 1. Feature Extraction
        for m in matches {
            // Update participation and win/loss
            let stats_a = miner_stats.entry(m.participant_a).or_default();
            stats_a.participation_count += 1;
            if m.winner == m.participant_a { stats_a.win_rate += 1.0; }
            
            let stats_b = miner_stats.entry(m.participant_b).or_default();
            stats_b.participation_count += 1;
            if m.winner == m.participant_b { stats_b.win_rate += 1.0; }
            
            // Extract Battle Dynamics (MII/TED) if history was tracked
            if let Some(history) = &m.history {
                // Calculate MII/TED for this match
                let (mii_a, mii_b) = m.battle_config.compute_mii(history);
                let (ted_a, ted_b) = m.battle_config.compute_ted(history);
                
                // Update Aggression (MII)
                let stats_a = miner_stats.entry(m.participant_a).or_default();
                stats_a.aggression_index += mii_a as f64;
                
                let stats_b = miner_stats.entry(m.participant_b).or_default();
                stats_b.aggression_index += mii_b as f64;
                
                // Update Volatility (TED)
                let stats_a = miner_stats.entry(m.participant_a).or_default();
                stats_a.volatility_index += ted_a as f64;
                
                let stats_b = miner_stats.entry(m.participant_b).or_default();
                stats_b.volatility_index += ted_b as f64;
            }
            
            // Stall Tendency (Tiebreaker usage)
            // If we had to use tiebreakers (MII/TED/Lex), it means primary energy was tied
            // We can infer this if energy_a == energy_b in the final grid, but we don't store final grid directly in match
            // However, we can check if outcome was determined by tiebreaker.
            // For now, we'll use a simplified heuristic: if track_history was true, it was a high-stakes match.
        }
        
        // Normalize stats
        for stats in miner_stats.values_mut() {
            if stats.participation_count > 0 {
                stats.win_rate /= stats.participation_count as f64;
                stats.aggression_index /= stats.participation_count as f64;
                stats.volatility_index /= stats.participation_count as f64;
            }
        }
        
        // 2. Evidence Mapping
        let mut evidence_updates = Vec::new();
        for (miner, stats) in &miner_stats {
            // Rule 1: High Aggression -> Positive Evidence (Active Play)
            if stats.aggression_index > 1000.0 {
                 evidence_updates.push((*miner, EvidenceType::GoodBlock));
            }
            
            // Rule 2: High Volatility -> Positive Evidence (Complex Strategy)
            if stats.volatility_index > 50.0 {
                evidence_updates.push((*miner, EvidenceType::GoodBlock));
            }
            
            // Rule 3: Consistent Participation (implied by being here)
        }
        
        // 3. Meta Metrics
        let total_matches = matches.len() as f64;
        let history_matches = matches.iter().filter(|m| m.battle_config.track_history).count();
        let avg_rounds = (matches.last().map(|m| m.round).unwrap_or(0) + 1) as f64;
        
        // Apply evidence updates
        for (miner, evidence_type) in evidence_updates {
            self.record_evidence(miner, evidence_type);
        }
        
        if total_matches > 0.0 {
             self.metrics.mii_usage_rate = history_matches as f64 / total_matches;
             self.metrics.avg_rounds = avg_rounds;
        }
    }

    fn derive_match_entropy(&self, round: u32, match_index: u32) -> [u8; 32] {
        let mut data = Vec::new();
        data.extend_from_slice(self.tournament.seed.as_bytes());
        data.extend_from_slice(&round.to_le_bytes());
        data.extend_from_slice(&match_index.to_le_bytes());
        
        *Hash256::hash(&data).as_bytes()
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

    #[test]
    fn test_full_tournament_flow() {
        use bitcell_ca::{Glider, GliderPattern, Position};
        
        // Setup 4 miners
        let sk1 = SecretKey::generate();
        let sk2 = SecretKey::generate();
        let sk3 = SecretKey::generate();
        let sk4 = SecretKey::generate();
        
        let miners = vec![
            sk1.public_key(), 
            sk2.public_key(), 
            sk3.public_key(), 
            sk4.public_key()
        ];
        
        let seed = Hash256::hash(b"tournament_seed");
        let mut orch = TournamentOrchestrator::new(100, miners.clone(), seed);
        
        // Commit phase
        for miner in &miners {
            let commit = GliderCommitment {
                commitment: Hash256::zero(),
                ring_signature: vec![],
                height: 100,
            };
            orch.process_commit(commit).unwrap();
        }
        
        orch.advance_to_reveal().unwrap();
        
        // Reveal phase
        for (i, miner) in miners.iter().enumerate() {
            let glider = Glider::new(GliderPattern::Standard, Position::new(i * 10, i * 10));
            let reveal = GliderReveal {
                glider,
                nonce: vec![],
                miner: *miner,
            };
            orch.process_reveal(reveal).unwrap();
        }
        
        orch.advance_to_battle().unwrap();
        
        // Run battles
        let winner = orch.run_battles().unwrap();
        
        // Verify winner is one of the miners
        assert!(miners.contains(&winner));
        
        // Verify tournament structure
        // 4 participants -> 2 semifinals + 1 final = 3 matches
        assert_eq!(orch.tournament.matches.len(), 3);
        
        // Check rounds
        let round_0_matches = orch.tournament.matches.iter().filter(|m| m.round == 0).count();
        let round_1_matches = orch.tournament.matches.iter().filter(|m| m.round == 1).count();
        
        assert_eq!(round_0_matches, 2);
        assert_eq!(round_1_matches, 1);
        
        // Check finals history tracking
        let final_match = orch.tournament.matches.iter().find(|m| m.round == 1).unwrap();
        assert!(final_match.battle_config.track_history);
        
        // Check semifinal history tracking (should be false)
        let semi_match = orch.tournament.matches.iter().find(|m| m.round == 0).unwrap();
        assert!(!semi_match.battle_config.track_history);
    }

    #[test]
    fn test_adaptive_strategy() {
        use bitcell_ca::{Glider, GliderPattern, Position};
        
        let sk1 = SecretKey::generate();
        let sk2 = SecretKey::generate();
        let miners = vec![sk1.public_key(), sk2.public_key()];
        
        let mut orch = TournamentOrchestrator::new(100, miners.clone(), Hash256::zero());
        
        // Setup tournament that will trigger adaptive strategy
        // Commit
        for miner in &miners {
            orch.process_commit(GliderCommitment {
                commitment: Hash256::zero(),
                ring_signature: vec![],
                height: 100,
            }).unwrap();
        }
        orch.advance_to_reveal().unwrap();
        
        // Reveal
        for (i, miner) in miners.iter().enumerate() {
            orch.process_reveal(GliderReveal {
                glider: Glider::new(GliderPattern::Standard, Position::new(i*10, i*10)),
                nonce: vec![],
                miner: *miner,
            }).unwrap();
        }
        orch.advance_to_battle().unwrap();
        
        // Run battles
        orch.run_battles().unwrap();
        
        // Verify metrics were computed
        assert!(orch.metrics.avg_rounds > 0.0);
        assert!(orch.metrics.mii_usage_rate > 0.0); // Final match uses history
        
        // Verify evidence was recorded
        // Both miners participated, so they should have some evidence (at least implied participation)
        // Winner gets GoodBlock
        let winner = orch.tournament.winner.unwrap();
        let counters = orch.miner_evidence.get(&winner).unwrap();
        assert!(counters.r > 0.0);
    }
}
