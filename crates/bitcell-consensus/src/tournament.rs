//! Tournament protocol structures

use bitcell_ca::{Battle, BattleOutcome, Glider, BattleHistory};
use bitcell_crypto::{Hash256, PublicKey};
use serde::{Deserialize, Serialize};

/// Tournament phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TournamentPhase {
    /// Waiting for commitments
    Commit,
    
    /// Waiting for reveals
    Reveal,
    
    /// Running battles
    Battle,
    
    /// Complete
    Complete,
}

/// Glider commitment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GliderCommitment {
    /// Hash of (glider_pattern || nonce)
    pub commitment: Hash256,
    
    /// Ring signature (anonymous)
    pub ring_signature: Vec<u8>,
    
    /// Block height
    pub height: u64,
}

/// Glider reveal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GliderReveal {
    /// The actual glider
    pub glider: Glider,
    
    /// Nonce used in commitment
    pub nonce: Vec<u8>,
    
    /// Miner identity (revealed)
    pub miner: PublicKey,
}

/// A single match in the tournament
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentMatch {
    /// Round number (0-based)
    pub round: u32,
    
    /// Match index within round (0-based)
    pub match_index: u32,
    
    /// Participant A
    pub participant_a: PublicKey,
    
    /// Participant B
    pub participant_b: PublicKey,
    
    /// Winner
    pub winner: PublicKey,
    
    /// Entropy seed used
    pub entropy_seed: [u8; 32],
    
    /// Battle configuration
    pub battle_config: Battle,
    
    /// Battle outcome
    pub outcome: BattleOutcome,
    
    /// Battle history (only for finals/tracked battles)
    pub history: Option<BattleHistory>,
    
    /// Proof data (placeholder for ZK proof)
    pub proof_data: Vec<u8>,
}

/// Tournament state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tournament {
    /// Block height
    pub height: u64,
    
    /// Eligible miners
    pub eligible_miners: Vec<PublicKey>,
    
    /// Tournament seed (from VRF)
    pub seed: Hash256,
    
    /// Current phase
    pub phase: TournamentPhase,
    
    /// Commitments received
    pub commitments: Vec<GliderCommitment>,
    
    /// Reveals received
    pub reveals: Vec<GliderReveal>,
    
    /// Matches executed
    pub matches: Vec<TournamentMatch>,
    
    /// Winner
    pub winner: Option<PublicKey>,
}

impl Tournament {
    /// Create a new tournament
    pub fn new(height: u64, eligible_miners: Vec<PublicKey>, seed: Hash256) -> Self {
        Self {
            height,
            eligible_miners,
            seed,
            phase: TournamentPhase::Commit,
            commitments: Vec::new(),
            reveals: Vec::new(),
            matches: Vec::new(),
            winner: None,
        }
    }

    /// Check if tournament is complete
    pub fn is_complete(&self) -> bool {
        self.phase == TournamentPhase::Complete
    }

    /// Get winner
    pub fn get_winner(&self) -> Option<PublicKey> {
        self.winner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcell_crypto::SecretKey;

    #[test]
    fn test_tournament_creation() {
        let sk1 = SecretKey::generate();
        let sk2 = SecretKey::generate();
        
        let miners = vec![sk1.public_key(), sk2.public_key()];
        let seed = Hash256::hash(b"test_seed");
        
        let tournament = Tournament::new(100, miners, seed);
        
        assert_eq!(tournament.height, 100);
        assert_eq!(tournament.phase, TournamentPhase::Commit);
        assert!(!tournament.is_complete());
    }

    #[test]
    fn test_tournament_phases() {
        let mut tournament = Tournament::new(1, vec![], Hash256::zero());
        
        assert_eq!(tournament.phase, TournamentPhase::Commit);
        
        tournament.phase = TournamentPhase::Reveal;
        assert_eq!(tournament.phase, TournamentPhase::Reveal);
        
        tournament.phase = TournamentPhase::Battle;
        assert_eq!(tournament.phase, TournamentPhase::Battle);
        
        tournament.phase = TournamentPhase::Complete;
        assert!(tournament.is_complete());
    }
}
