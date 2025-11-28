///! Tournament manager for coordinating block proposer selection

use crate::{Result, MetricsRegistry};
use bitcell_consensus::{TournamentOrchestrator, TournamentPhase, GliderCommitment, GliderReveal, BattleProof};
use bitcell_crypto::{Hash256, PublicKey};
use bitcell_ebsl::{EvidenceCounters, EvidenceType, EbslParams, TrustScore};
use std::sync::{Arc, RwLock as StdRwLock};
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time;

/// Phase duration in seconds
const COMMIT_PHASE_SECS: u64 = 5;
const REVEAL_PHASE_SECS: u64 = 5;
const BATTLE_PHASE_SECS: u64 = 5;

/// Tournament manager
pub struct TournamentManager {
    /// Current tournament
    tournament: Arc<RwLock<Option<TournamentOrchestrator>>>,
    
    /// Metrics registry
    metrics: MetricsRegistry,
    
    /// Current block height being decided
    current_height: Arc<StdRwLock<u64>>,
    
    /// Miner evidence counters for EBSL
    miner_evidence: Arc<StdRwLock<HashMap<PublicKey, EvidenceCounters>>>,
    
    /// EBSL parameters
    ebsl_params: EbslParams,
}

impl TournamentManager {
    /// Create a new tournament manager
    pub fn new(metrics: MetricsRegistry) -> Self {
        Self {
            tournament: Arc::new(RwLock::new(None)),
            metrics,
            current_height: Arc::new(StdRwLock::new(1)),
            miner_evidence: Arc::new(StdRwLock::new(HashMap::new())),
            ebsl_params: EbslParams::default(),
        }
    }
    
    /// Start a new tournament for the given height
    pub async fn start_tournament(&self, height: u64, eligible_miners: Vec<PublicKey>, seed: Hash256) {
        let mut tournament = self.tournament.write().await;
        *tournament = Some(TournamentOrchestrator::new(height, eligible_miners.clone(), seed));
        
        let mut current_height = self.current_height.write().unwrap();
        *current_height = height;
        
        // Update metrics
        self.metrics.set_active_miners(eligible_miners.len());
        
        println!("Started tournament for height {} with {} miners", height, eligible_miners.len());
    }
    
    /// Add a commitment
    pub async fn add_commitment(&self, commitment: GliderCommitment) -> Result<()> {
        let mut tournament = self.tournament.write().await;
        if let Some(ref mut t) = *tournament {
            t.process_commit(commitment)
                .map_err(|e| crate::Error::Node(format!("Tournament error: {}", e)))
        } else {
            Err(crate::Error::Node("No active tournament".to_string()))
        }
    }
    
    /// Advance to reveal phase
    pub async fn advance_to_reveal(&self) -> Result<()> {
        let mut tournament = self.tournament.write().await;
        if let Some(ref mut t) = *tournament {
            t.advance_to_reveal()
                .map_err(|e| crate::Error::Node(format!("Tournament error: {}", e)))
        } else {
            Err(crate::Error::Node("No active tournament".to_string()))
        }
    }
    
    /// Add a reveal
    pub async fn add_reveal(&self, reveal: GliderReveal) -> Result<()> {
        let mut tournament = self.tournament.write().await;
        if let Some(ref mut t) = *tournament {
            t.process_reveal(reveal)
                .map_err(|e| crate::Error::Node(format!("Tournament error: {}", e)))
        } else {
            Err(crate::Error::Node("No active tournament".to_string()))
        }
    }
    
    /// Advance to battle phase
    pub async fn advance_to_battle(&self) -> Result<()> {
        let mut tournament = self.tournament.write().await;
        if let Some(ref mut t) = *tournament {
            t.advance_to_battle()
                .map_err(|e| crate::Error::Node(format!("Tournament error: {}", e)))
        } else {
            Err(crate::Error::Node("No active tournament".to_string()))
        }
    }
    
    /// Run battles and get winner
    pub async fn run_battles(&self) -> Result<PublicKey> {
        let mut tournament = self.tournament.write().await;
        
        if let Some(ref mut t) = *tournament {
            let winner = t.run_battles()
                .map_err(|e| crate::Error::Node(format!("Tournament error: {}", e)))?;
            
            println!("Tournament winner: {:?}", winner);
            Ok(winner)
        } else {
            Err(crate::Error::Node("No active tournament".to_string()))
        }
    }
    
    /// Get current phase
    pub async fn current_phase(&self) -> Option<TournamentPhase> {
        let tournament = self.tournament.read().await;
        tournament.as_ref().map(|t| t.tournament.phase)
    }
    
    /// Get winner if tournament is complete
    pub async fn get_winner(&self) -> Option<PublicKey> {
        let tournament = self.tournament.read().await;
        tournament.as_ref().and_then(|t| t.get_winner())
    }
    
    /// Check if tournament is complete
    pub async fn is_complete(&self) -> bool {
        let tournament = self.tournament.read().await;
        tournament.as_ref().map_or(false, |t| t.tournament.is_complete())
    }
    
    /// Get battle proofs (simplified - generate placeholder proofs)
    pub async fn get_battle_proofs(&self) -> Vec<BattleProof> {
        let tournament = self.tournament.read().await;
        if let Some(ref t) = *tournament {
            // Generate battle proofs from matches
            t.tournament.matches.iter().map(|match_record| {
                BattleProof {
                    participant_a: match_record.participant_a,
                    participant_b: match_record.participant_b,
                    winner: match_record.winner,
                    proof: match_record.proof_data.clone(),
                    public_inputs: match_record.entropy_seed.to_vec(),
                }
            }).collect()
        } else {
            vec![]
        }
    }
    
    /// Record evidence for a miner
    pub fn record_evidence(&self, miner: PublicKey, evidence_type: EvidenceType) {
        {
            let mut evidence_map = self.miner_evidence.write().unwrap();
            let counters = evidence_map.entry(miner).or_insert_with(EvidenceCounters::new);
            
            // Add evidence with current block height
            let height = *self.current_height.read().unwrap();
            counters.add_evidence(bitcell_ebsl::Evidence::new(evidence_type, 0, height));
        } // Drop write lock here
        
        // Update metrics (acquires read lock)
        self.update_ebsl_metrics();
    }
    
    /// Check if a miner is eligible based on EBSL trust score
    pub fn is_miner_eligible(&self, miner: &PublicKey) -> bool {
        let evidence_map = self.miner_evidence.read().unwrap();
        
        if let Some(counters) = evidence_map.get(miner) {
            let trust = TrustScore::from_evidence(counters, &self.ebsl_params);
            trust.is_eligible(&self.ebsl_params)
        } else {
            // New miners start below threshold, need to build reputation
            false
        }
    }
    
    /// Get all eligible miners from a set of candidates
    pub fn filter_eligible_miners(&self, candidates: Vec<PublicKey>) -> Vec<PublicKey> {
        candidates.into_iter()
            .filter(|miner| self.is_miner_eligible(miner))
            .collect()
    }
    
    /// Get trust score for a miner
    pub fn get_trust_score(&self, miner: &PublicKey) -> f64 {
        let evidence_map = self.miner_evidence.read().unwrap();
        
        if let Some(counters) = evidence_map.get(miner) {
            let trust = TrustScore::from_evidence(counters, &self.ebsl_params);
            trust.value()
        } else {
            0.0
        }
    }
    
    /// Update EBSL metrics (active/banned miners)
    fn update_ebsl_metrics(&self) {
        let evidence_map = self.miner_evidence.read().unwrap();
        
        let mut active_count = 0;
        let mut banned_count = 0;
        
        for (_miner, counters) in evidence_map.iter() {
            let trust = TrustScore::from_evidence(counters, &self.ebsl_params);
            
            if trust.is_eligible(&self.ebsl_params) {
                active_count += 1;
            } else if trust.value() < self.ebsl_params.t_kill {
                banned_count += 1;
            }
        }
        
        self.metrics.set_active_miners(active_count);
        self.metrics.set_banned_miners(banned_count);
    }
}

/// Run a full tournament cycle (for simplified single-node testing)
pub async fn run_tournament_cycle(
    manager: Arc<TournamentManager>,
    height: u64,
    eligible_miners: Vec<PublicKey>,
    seed: Hash256,
) -> Result<PublicKey> {
    use bitcell_ca::{Glider, GliderPattern};
    use bitcell_ca::grid::Position;
    
    // Start tournament
    manager.start_tournament(height, eligible_miners.clone(), seed).await;
    
    // For single-node testing, we'll submit commitments/reveals ourselves
    // In production, miners would do this over the network
    
    // Commit phase - submit a dummy commitment for each miner
    println!("Tournament: Commit phase ({}s)", COMMIT_PHASE_SECS);
    
    // Submit commitments for all miners
    for _miner_pk in &eligible_miners {
        // Create dummy commitment
        let commitment_data = format!("{:?}{}", height, seed);
        let commitment_hash = bitcell_crypto::Hash256::hash(commitment_data.as_bytes());
        
        let commitment = bitcell_consensus::GliderCommitment {
            commitment: commitment_hash,
            ring_signature: vec![0u8; 64], // Dummy signature
            height,
        };
        
        let _ = manager.add_commitment(commitment).await;
    }
    
    time::sleep(Duration::from_secs(COMMIT_PHASE_SECS)).await;
    
    // Advance to reveal
    manager.advance_to_reveal().await?;
    
    // Reveal phase - reveal the gliders
    println!("Tournament: Reveal phase ({}s)", REVEAL_PHASE_SECS);
    
    // Submit reveals for all miners
    for miner_pk in &eligible_miners {
        // Create a simple glider for testing
        let glider = Glider::new(
            GliderPattern::Standard,
            Position::new(100, 100),
        );
        
        // Dummy nonce (in production this would be random)  
        let nonce = vec![height as u8];
        
        let reveal = bitcell_consensus::GliderReveal {
            glider,
            nonce,
            miner: *miner_pk,
        };
        
        let _ = manager.add_reveal(reveal).await;
    }
    
    time::sleep(Duration::from_secs(REVEAL_PHASE_SECS)).await;
    
    // Advance to battle
    manager.advance_to_battle().await?;
    
    // Battle phase
    println!("Tournament: Battle phase ({}s)", BATTLE_PHASE_SECS);
    
    // Run battles - now async, no need for spawn_blocking
    let winner = manager.run_battles().await?;
    
    time::sleep(Duration::from_secs(BATTLE_PHASE_SECS)).await;
    
    println!("Tournament complete for height {}, winner: {:?}", height, winner);
    Ok(winner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcell_crypto::SecretKey;
    
    #[tokio::test]
    async fn test_tournament_creation() {
        let metrics = MetricsRegistry::new();
        let manager = TournamentManager::new(metrics);
        
        let sk = SecretKey::generate();
        let miners = vec![sk.public_key()];
        let seed = Hash256::zero();
        
        manager.start_tournament(1, miners, seed).await;
        assert_eq!(manager.current_phase().await, Some(TournamentPhase::Commit));
    }
}
