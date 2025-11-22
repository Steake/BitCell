//! Protocol-Local EBSL (Evidence-Based Subjective Logic)
//!
//! Implements miner reputation tracking based on on-chain evidence:
//! - Positive/negative evidence counters
//! - Subjective logic opinion calculation
//! - Trust score computation
//! - Decay mechanisms
//! - Slashing and banning logic

pub mod evidence;
pub mod trust;
pub mod decay;
pub mod slashing;

pub use evidence::{Evidence, EvidenceType};
pub use trust::{Opinion, TrustScore};
pub use decay::DecayParams;
pub use slashing::SlashingAction;

/// Result type for EBSL operations
pub type Result<T> = std::result::Result<T, Error>;

/// EBSL errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid evidence value")]
    InvalidEvidence,
    
    #[error("Invalid trust parameters")]
    InvalidParameters,
    
    #[error("Miner not found")]
    MinerNotFound,
}

/// Protocol parameters for EBSL
#[derive(Debug, Clone)]
pub struct EbslParams {
    /// Base K for subjective logic (default: 2)
    pub k: f64,
    
    /// Alpha for expected trust (default: 0.4)
    pub alpha: f64,
    
    /// Minimum trust threshold for eligibility (default: 0.75)
    pub t_min: f64,
    
    /// Kill threshold - miners below this are effectively banned (default: 0.2)
    pub t_kill: f64,
    
    /// Positive evidence decay per epoch (default: 0.99)
    pub pos_decay: f64,
    
    /// Negative evidence decay per epoch (default: 0.999)
    pub neg_decay: f64,
}

impl Default for EbslParams {
    fn default() -> Self {
        Self {
            k: 2.0,
            alpha: 0.4,
            t_min: 0.75,
            t_kill: 0.2,
            pos_decay: 0.99,
            neg_decay: 0.999,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let params = EbslParams::default();
        assert_eq!(params.k, 2.0);
        assert_eq!(params.alpha, 0.4);
        assert!(params.t_min > params.t_kill);
    }
}
