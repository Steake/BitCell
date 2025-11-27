//! Decay mechanisms for evidence over time

use crate::evidence::EvidenceCounters;
use serde::{Deserialize, Serialize};

/// Decay parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayParams {
    /// Positive evidence decay factor (applied per epoch)
    pub pos_decay: f64,
    
    /// Negative evidence decay factor (applied per epoch)
    pub neg_decay: f64,
}

impl Default for DecayParams {
    fn default() -> Self {
        Self {
            pos_decay: 0.99,   // Positive evidence decays faster
            neg_decay: 0.999,  // Negative evidence decays slower (forgive slowly)
        }
    }
}

/// Apply decay to evidence counters
pub fn apply_decay(counters: &mut EvidenceCounters, params: &DecayParams) {
    counters.apply_decay(params.pos_decay, params.neg_decay);
}

/// Apply decay for multiple epochs at once
pub fn apply_decay_epochs(counters: &mut EvidenceCounters, params: &DecayParams, epochs: u64) {
    for _ in 0..epochs {
        apply_decay(counters, params);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::{Evidence, EvidenceType};

    #[test]
    fn test_decay_application() {
        let mut counters = EvidenceCounters::new();
        counters.r = 100.0;
        counters.s = 50.0;

        let params = DecayParams::default();
        apply_decay(&mut counters, &params);

        assert_eq!(counters.r, 99.0);
        assert_eq!(counters.s, 49.95);
    }

    #[test]
    fn test_decay_over_many_epochs() {
        let mut counters = EvidenceCounters::new();
        counters.r = 100.0;
        counters.s = 100.0;

        let params = DecayParams::default();
        
        // Apply decay for 100 epochs
        apply_decay_epochs(&mut counters, &params, 100);

        // Positive should decay more than negative
        assert!(counters.r < counters.s);
        
        // Both should be significantly reduced
        assert!(counters.r < 50.0);
        assert!(counters.s > 90.0); // Decays much slower
    }

    #[test]
    fn test_decay_asymmetry() {
        let mut counters_pos = EvidenceCounters::new();
        counters_pos.r = 100.0;

        let mut counters_neg = EvidenceCounters::new();
        counters_neg.s = 100.0;

        let params = DecayParams::default();

        // Apply same number of epochs
        apply_decay_epochs(&mut counters_pos, &params, 50);
        apply_decay_epochs(&mut counters_neg, &params, 50);

        // Negative evidence should decay slower (retain more value)
        assert!(counters_pos.r < counters_neg.s);
    }

    #[test]
    fn test_zero_decay_stable() {
        let mut counters = EvidenceCounters::new();
        counters.r = 0.0;
        counters.s = 0.0;

        let params = DecayParams::default();
        apply_decay(&mut counters, &params);

        assert_eq!(counters.r, 0.0);
        assert_eq!(counters.s, 0.0);
    }
}
