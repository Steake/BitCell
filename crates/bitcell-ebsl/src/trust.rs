//! Trust score computation using subjective logic

use crate::evidence::EvidenceCounters;
use crate::EbslParams;
use serde::{Deserialize, Serialize};

/// Subjective logic opinion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opinion {
    /// Belief (certainty in honest behavior)
    pub belief: f64,
    
    /// Disbelief (certainty in dishonest behavior)
    pub disbelief: f64,
    
    /// Uncertainty
    pub uncertainty: f64,
}

impl Opinion {
    /// Create opinion from evidence counters
    pub fn from_evidence(counters: &EvidenceCounters, k: f64) -> Self {
        let r = counters.r;
        let s = counters.s;
        let total = r + s + k;

        let belief = r / total;
        let disbelief = s / total;
        let uncertainty = k / total;

        Opinion {
            belief,
            disbelief,
            uncertainty,
        }
    }

    /// Validate that opinion components sum to 1.0
    pub fn is_valid(&self) -> bool {
        let sum = self.belief + self.disbelief + self.uncertainty;
        (sum - 1.0).abs() < 1e-6
    }

    /// Get expected probability (projection)
    pub fn expected_probability(&self, alpha: f64) -> f64 {
        self.belief + alpha * self.uncertainty
    }
}

/// Trust score (0.0 to 1.0)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TrustScore(f64);

impl TrustScore {
    /// Create a trust score
    pub fn new(score: f64) -> Self {
        Self(score.max(0.0).min(1.0))
    }

    /// Compute trust score from evidence counters
    pub fn from_evidence(counters: &EvidenceCounters, params: &EbslParams) -> Self {
        let opinion = Opinion::from_evidence(counters, params.k);
        let score = opinion.expected_probability(params.alpha);
        Self::new(score)
    }

    /// Get the score value
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Check if miner is eligible (above T_MIN)
    pub fn is_eligible(&self, params: &EbslParams) -> bool {
        self.0 >= params.t_min
    }

    /// Check if miner is effectively dead (below T_KILL)
    pub fn is_killed(&self, params: &EbslParams) -> bool {
        self.0 < params.t_kill
    }

    /// Check if miner is in warning zone (between T_KILL and T_MIN)
    pub fn is_warning(&self, params: &EbslParams) -> bool {
        self.0 >= params.t_kill && self.0 < params.t_min
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::{Evidence, EvidenceType};

    #[test]
    fn test_opinion_from_no_evidence() {
        let counters = EvidenceCounters::new();
        let opinion = Opinion::from_evidence(&counters, 2.0);

        // With no evidence, all uncertainty
        assert_eq!(opinion.belief, 0.0);
        assert_eq!(opinion.disbelief, 0.0);
        assert_eq!(opinion.uncertainty, 1.0);
        assert!(opinion.is_valid());
    }

    #[test]
    fn test_opinion_from_positive_evidence() {
        let mut counters = EvidenceCounters::new();
        for _ in 0..10 {
            counters.add_evidence(Evidence::new(EvidenceType::GoodBlock, 1, 100));
        }

        let opinion = Opinion::from_evidence(&counters, 2.0);

        // Should have high belief
        assert!(opinion.belief > 0.8);
        assert!(opinion.disbelief < 0.1);
        assert!(opinion.is_valid());
    }

    #[test]
    fn test_opinion_from_negative_evidence() {
        let mut counters = EvidenceCounters::new();
        for _ in 0..5 {
            counters.add_evidence(Evidence::new(EvidenceType::InvalidBlock, 1, 100));
        }

        let opinion = Opinion::from_evidence(&counters, 2.0);

        // Should have high disbelief
        assert!(opinion.disbelief > 0.8);
        assert!(opinion.belief < 0.1);
        assert!(opinion.is_valid());
    }

    #[test]
    fn test_opinion_mixed_evidence() {
        let mut counters = EvidenceCounters::new();
        
        // Add some positive
        for _ in 0..5 {
            counters.add_evidence(Evidence::new(EvidenceType::GoodBlock, 1, 100));
        }
        
        // Add some negative
        for _ in 0..2 {
            counters.add_evidence(Evidence::new(EvidenceType::InvalidBlock, 2, 200));
        }

        let opinion = Opinion::from_evidence(&counters, 2.0);
        assert!(opinion.is_valid());
        
        // Should have some belief but also significant disbelief
        assert!(opinion.belief > 0.0);
        assert!(opinion.disbelief > 0.0);
    }

    #[test]
    fn test_trust_score_from_clean_miner() {
        let mut counters = EvidenceCounters::new();
        for _ in 0..20 {
            counters.add_evidence(Evidence::new(EvidenceType::GoodBlock, 1, 100));
        }

        let params = EbslParams::default();
        let trust = TrustScore::from_evidence(&counters, &params);

        // Clean miner should be eligible
        assert!(trust.is_eligible(&params));
        assert!(!trust.is_killed(&params));
        assert!(!trust.is_warning(&params));
    }

    #[test]
    fn test_trust_score_from_bad_miner() {
        let mut counters = EvidenceCounters::new();
        for _ in 0..10 {
            counters.add_evidence(Evidence::new(EvidenceType::InvalidBlock, 1, 100));
        }

        let params = EbslParams::default();
        let trust = TrustScore::from_evidence(&counters, &params);

        // Bad miner should not be eligible
        assert!(!trust.is_eligible(&params));
        assert!(trust.is_killed(&params) || trust.is_warning(&params));
    }

    #[test]
    fn test_trust_score_bounds() {
        let score1 = TrustScore::new(-0.5);
        assert_eq!(score1.value(), 0.0);

        let score2 = TrustScore::new(1.5);
        assert_eq!(score2.value(), 1.0);

        let score3 = TrustScore::new(0.5);
        assert_eq!(score3.value(), 0.5);
    }

    #[test]
    fn test_new_miner_starts_below_threshold() {
        let counters = EvidenceCounters::new();
        let params = EbslParams::default();
        let trust = TrustScore::from_evidence(&counters, &params);

        // New miner with no evidence starts at alpha (0.4) < t_min (0.75)
        assert!(!trust.is_eligible(&params));
        assert_eq!(trust.value(), params.alpha);
    }
}
