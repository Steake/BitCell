//! Slashing and banning logic for severe violations

use crate::evidence::EvidenceType;
use crate::trust::TrustScore;
use crate::EbslParams;
use serde::{Deserialize, Serialize};

/// Slashing action to take
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlashingAction {
    /// No action
    None,
    
    /// Partial slash (percentage of bond)
    Partial(u8), // 0-100
    
    /// Full slash and permanent ban
    FullAndBan,
    
    /// Temporary ban (number of epochs)
    TemporaryBan(u64),
}

/// Determine slashing action based on evidence and trust
pub fn determine_slashing(
    evidence_type: EvidenceType,
    trust: TrustScore,
    params: &EbslParams,
) -> SlashingAction {
    match evidence_type {
        EvidenceType::Equivocation => {
            // Equivocation is always full slash + permanent ban
            SlashingAction::FullAndBan
        }
        
        EvidenceType::ProofFailure => {
            // Proof failures are very serious
            if trust.is_killed(params) {
                SlashingAction::FullAndBan
            } else {
                SlashingAction::Partial(75) // 75% slash
            }
        }
        
        EvidenceType::InvalidTournament => {
            if trust.is_killed(params) {
                SlashingAction::Partial(50)
            } else {
                SlashingAction::Partial(25)
            }
        }
        
        EvidenceType::InvalidBlock => {
            if trust.is_killed(params) {
                SlashingAction::TemporaryBan(10) // 10 epochs
            } else {
                SlashingAction::Partial(15)
            }
        }
        
        EvidenceType::MissedReveal => {
            if trust.is_killed(params) {
                SlashingAction::TemporaryBan(5)
            } else {
                SlashingAction::None // Just trust penalty
            }
        }
        
        EvidenceType::MissedCommitment => {
            // Mild liveness failure - just trust penalty
            SlashingAction::None
        }
        
        EvidenceType::GoodBlock | EvidenceType::HonestParticipation => {
            // Positive evidence - no slashing
            SlashingAction::None
        }
    }
}

/// Calculate ban duration based on trust score
pub fn calculate_ban_duration(trust: TrustScore, params: &EbslParams) -> Option<u64> {
    if trust.is_killed(params) {
        // Very low trust - long ban
        Some(100)
    } else if trust.is_warning(params) {
        // Warning zone - moderate ban
        Some(20)
    } else {
        // Above threshold - no ban
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equivocation_always_full_ban() {
        let params = EbslParams::default();
        let trust = TrustScore::new(0.9); // Even high trust

        let action = determine_slashing(EvidenceType::Equivocation, trust, &params);
        assert_eq!(action, SlashingAction::FullAndBan);
    }

    #[test]
    fn test_proof_failure_high_trust() {
        let params = EbslParams::default();
        let trust = TrustScore::new(0.8);

        let action = determine_slashing(EvidenceType::ProofFailure, trust, &params);
        assert_eq!(action, SlashingAction::Partial(75));
    }

    #[test]
    fn test_proof_failure_low_trust() {
        let params = EbslParams::default();
        let trust = TrustScore::new(0.1); // Below T_KILL

        let action = determine_slashing(EvidenceType::ProofFailure, trust, &params);
        assert_eq!(action, SlashingAction::FullAndBan);
    }

    #[test]
    fn test_missed_commitment_no_slash() {
        let params = EbslParams::default();
        let trust = TrustScore::new(0.5);

        let action = determine_slashing(EvidenceType::MissedCommitment, trust, &params);
        assert_eq!(action, SlashingAction::None);
    }

    #[test]
    fn test_positive_evidence_no_slash() {
        let params = EbslParams::default();
        let trust = TrustScore::new(0.5);

        let action = determine_slashing(EvidenceType::GoodBlock, trust, &params);
        assert_eq!(action, SlashingAction::None);
    }

    #[test]
    fn test_ban_duration_killed() {
        let params = EbslParams::default();
        let trust = TrustScore::new(0.1); // Below T_KILL (0.2)

        let duration = calculate_ban_duration(trust, &params);
        assert_eq!(duration, Some(100));
    }

    #[test]
    fn test_ban_duration_warning() {
        let params = EbslParams::default();
        let trust = TrustScore::new(0.5); // Between T_KILL and T_MIN

        let duration = calculate_ban_duration(trust, &params);
        assert_eq!(duration, Some(20));
    }

    #[test]
    fn test_ban_duration_eligible() {
        let params = EbslParams::default();
        let trust = TrustScore::new(0.8); // Above T_MIN

        let duration = calculate_ban_duration(trust, &params);
        assert_eq!(duration, None);
    }
}
