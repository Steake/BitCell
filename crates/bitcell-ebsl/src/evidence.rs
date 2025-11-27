//! Evidence tracking for miner behavior

use serde::{Deserialize, Serialize};

/// Types of evidence (positive and negative events)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceType {
    // Positive evidence
    GoodBlock,              // +1.0
    HonestParticipation,    // +0.25
    
    // Negative evidence
    InvalidBlock,           // +6.0 to negative
    InvalidTournament,      // +10.0 to negative
    ProofFailure,           // +12.0 to negative
    Equivocation,           // +20.0 to negative
    MissedCommitment,       // +2.0 to negative (liveness failure)
    MissedReveal,           // +4.0 to negative (liveness failure, worse)
}

impl EvidenceType {
    /// Get the weight/value of this evidence type
    pub fn weight(&self) -> f64 {
        match self {
            EvidenceType::GoodBlock => 1.0,
            EvidenceType::HonestParticipation => 0.25,
            EvidenceType::InvalidBlock => 6.0,
            EvidenceType::InvalidTournament => 10.0,
            EvidenceType::ProofFailure => 12.0,
            EvidenceType::Equivocation => 20.0,
            EvidenceType::MissedCommitment => 2.0,
            EvidenceType::MissedReveal => 4.0,
        }
    }

    /// Check if this is positive evidence
    pub fn is_positive(&self) -> bool {
        matches!(self, EvidenceType::GoodBlock | EvidenceType::HonestParticipation)
    }

    /// Check if this is negative evidence
    pub fn is_negative(&self) -> bool {
        !self.is_positive()
    }

    /// Check if this is a severe violation (triggers immediate slashing)
    pub fn is_severe(&self) -> bool {
        matches!(
            self,
            EvidenceType::Equivocation | EvidenceType::ProofFailure
        )
    }
}

/// Evidence record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_type: EvidenceType,
    pub epoch: u64,
    pub block_height: u64,
}

impl Evidence {
    pub fn new(evidence_type: EvidenceType, epoch: u64, block_height: u64) -> Self {
        Self {
            evidence_type,
            epoch,
            block_height,
        }
    }
}

/// Miner evidence counters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCounters {
    /// Positive evidence accumulator
    pub r: f64,
    
    /// Negative evidence accumulator
    pub s: f64,
    
    /// History of recent evidence (for auditing)
    pub history: Vec<Evidence>,
}

impl EvidenceCounters {
    pub fn new() -> Self {
        Self {
            r: 0.0,
            s: 0.0,
            history: Vec::new(),
        }
    }

    /// Add evidence to the counters
    pub fn add_evidence(&mut self, evidence: Evidence) {
        let weight = evidence.evidence_type.weight();
        
        if evidence.evidence_type.is_positive() {
            self.r += weight;
        } else {
            self.s += weight;
        }
        
        self.history.push(evidence);
        
        // Keep only recent history (last 1000 events)
        if self.history.len() > 1000 {
            self.history.drain(0..self.history.len() - 1000);
        }
    }

    /// Get total evidence
    pub fn total(&self) -> f64 {
        self.r + self.s
    }

    /// Apply decay factors
    pub fn apply_decay(&mut self, pos_decay: f64, neg_decay: f64) {
        self.r *= pos_decay;
        self.s *= neg_decay;
    }
}

impl Default for EvidenceCounters {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_weight() {
        assert_eq!(EvidenceType::GoodBlock.weight(), 1.0);
        assert_eq!(EvidenceType::Equivocation.weight(), 20.0);
    }

    #[test]
    fn test_evidence_classification() {
        assert!(EvidenceType::GoodBlock.is_positive());
        assert!(!EvidenceType::GoodBlock.is_negative());
        
        assert!(EvidenceType::InvalidBlock.is_negative());
        assert!(!EvidenceType::InvalidBlock.is_positive());
    }

    #[test]
    fn test_evidence_severity() {
        assert!(EvidenceType::Equivocation.is_severe());
        assert!(EvidenceType::ProofFailure.is_severe());
        assert!(!EvidenceType::InvalidBlock.is_severe());
    }

    #[test]
    fn test_counters_addition() {
        let mut counters = EvidenceCounters::new();
        
        counters.add_evidence(Evidence::new(EvidenceType::GoodBlock, 1, 100));
        assert_eq!(counters.r, 1.0);
        assert_eq!(counters.s, 0.0);
        
        counters.add_evidence(Evidence::new(EvidenceType::InvalidBlock, 2, 200));
        assert_eq!(counters.r, 1.0);
        assert_eq!(counters.s, 6.0);
    }

    #[test]
    fn test_counters_decay() {
        let mut counters = EvidenceCounters::new();
        counters.r = 100.0;
        counters.s = 50.0;
        
        counters.apply_decay(0.99, 0.999);
        
        assert_eq!(counters.r, 99.0);
        assert_eq!(counters.s, 49.95);
    }

    #[test]
    fn test_history_pruning() {
        let mut counters = EvidenceCounters::new();
        
        // Add more than 1000 evidence entries
        for i in 0..1100 {
            counters.add_evidence(Evidence::new(
                EvidenceType::GoodBlock,
                i / 10,
                i,
            ));
        }
        
        // Should keep only last 1000
        assert_eq!(counters.history.len(), 1000);
    }
}
