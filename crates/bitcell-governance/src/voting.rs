//! Voting system with linear and quadratic voting

use serde::{Deserialize, Serialize};
use crate::proposal::ProposalId;

/// Voting power representation
pub type VotingPower = u64;

/// Voting method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VotingMethod {
    /// 1 CELL = 1 vote (linear)
    Linear,
    
    /// sqrt(CELL) = votes (quadratic, Sybil-resistant)
    Quadratic,
}

/// A vote on a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// Proposal being voted on
    pub proposal_id: ProposalId,
    
    /// Address of voter
    pub voter: [u8; 33],
    
    /// Support (true) or oppose (false)
    pub support: bool,
    
    /// Effective voting power used
    pub power: VotingPower,
    
    /// Timestamp of vote
    pub timestamp: u64,
}

/// Record of a vote (stored with proposal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRecord {
    pub voter: [u8; 33],
    pub support: bool,
    pub power: VotingPower,
    pub timestamp: u64,
}

impl VoteRecord {
    pub fn new(voter: [u8; 33], support: bool, power: VotingPower, timestamp: u64) -> Self {
        Self {
            voter,
            support,
            power,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_voting_method() {
        let linear = VotingMethod::Linear;
        let quadratic = VotingMethod::Quadratic;
        
        assert_ne!(linear, quadratic);
        assert_eq!(linear, VotingMethod::Linear);
    }
    
    #[test]
    fn test_vote_record() {
        let voter = [1u8; 33];
        let record = VoteRecord::new(voter, true, 100, 1000);
        
        assert_eq!(record.voter, voter);
        assert!(record.support);
        assert_eq!(record.power, 100);
        assert_eq!(record.timestamp, 1000);
    }
}
