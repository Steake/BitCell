//! Proposal types and management

use serde::{Deserialize, Serialize};

/// Unique proposal identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProposalId(pub u64);

/// Proposal types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    /// Change a protocol parameter
    ParameterChange {
        parameter: String,
        new_value: Vec<u8>,
    },
    
    /// Spend from treasury
    TreasurySpending {
        #[serde(with = "crate::serde_pubkey")]
        recipient: [u8; 33],
        amount: u64,
        reason: String,
    },
    
    /// Protocol upgrade
    ProtocolUpgrade {
        version: String,
        code_hash: [u8; 32],
        description: String,
    },
}

/// Proposal status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    /// Proposal is active and accepting votes
    Active,
    
    /// Voting period ended, proposal passed
    Passed,
    
    /// Voting period ended, proposal rejected
    Rejected,
    
    /// Proposal executed successfully
    Executed,
    
    /// Proposal cancelled by guardians
    Cancelled,
}

/// On-chain governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Unique identifier
    pub id: ProposalId,
    
    /// Proposer's public key
    #[serde(with = "crate::serde_pubkey")]
    pub proposer: [u8; 33],
    
    /// Type of proposal
    pub proposal_type: ProposalType,
    
    /// Human-readable description
    pub description: String,
    
    /// Current status
    pub status: ProposalStatus,
    
    /// Block number when proposal was created
    pub creation_block: u64,
    
    /// Block number when voting ends
    pub voting_end_block: u64,
    
    /// Votes in favor
    pub votes_for: u64,
    
    /// Votes against
    pub votes_against: u64,
    
    /// Abstain votes
    pub votes_abstain: u64,
    
    /// Minimum votes required for validity (quorum)
    pub quorum_threshold: u64,
}

impl Proposal {
    pub fn new(
        id: ProposalId,
        proposer: [u8; 33],
        proposal_type: ProposalType,
        description: String,
        creation_block: u64,
        voting_period_blocks: u64,
    ) -> Self {
        Self {
            id,
            proposer,
            proposal_type,
            description,
            status: ProposalStatus::Active,
            creation_block,
            voting_end_block: creation_block + voting_period_blocks,
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            quorum_threshold: 10000, // Default: 10000 CELL minimum participation
        }
    }
    
    /// Check if proposal is still in voting period
    pub fn is_active(&self, current_block: u64) -> bool {
        self.status == ProposalStatus::Active && current_block <= self.voting_end_block
    }
    
    /// Get total votes cast
    pub fn total_votes(&self) -> u64 {
        self.votes_for + self.votes_against + self.votes_abstain
    }
    
    /// Check if quorum is met
    pub fn quorum_met(&self) -> bool {
        self.total_votes() >= self.quorum_threshold
    }
    
    /// Check if proposal has majority support
    pub fn has_majority(&self) -> bool {
        self.votes_for > self.votes_against
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_creation() {
        let proposal = Proposal::new(
            ProposalId(1),
            [1u8; 33],
            ProposalType::ParameterChange {
                parameter: "block_time".to_string(),
                new_value: vec![10],
            },
            "Reduce block time".to_string(),
            100,
            1000,
        );
        
        assert_eq!(proposal.id.0, 1);
        assert_eq!(proposal.voting_end_block, 1100);
        assert_eq!(proposal.status, ProposalStatus::Active);
    }
    
    #[test]
    fn test_proposal_active() {
        let proposal = Proposal::new(
            ProposalId(1),
            [1u8; 33],
            ProposalType::ParameterChange {
                parameter: "test".to_string(),
                new_value: vec![1],
            },
            "Test".to_string(),
            100,
            1000,
        );
        
        assert!(proposal.is_active(500));
        assert!(proposal.is_active(1100));
        assert!(!proposal.is_active(1101));
    }
    
    #[test]
    fn test_quorum_and_majority() {
        let mut proposal = Proposal::new(
            ProposalId(1),
            [1u8; 33],
            ProposalType::TreasurySpending {
                recipient: [2u8; 33],
                amount: 5000,
                reason: "Test".to_string(),
            },
            "Test".to_string(),
            100,
            1000,
        );
        
        proposal.quorum_threshold = 10000;
        proposal.votes_for = 6000;
        proposal.votes_against = 4000;
        
        assert_eq!(proposal.total_votes(), 10000);
        assert!(proposal.quorum_met());
        assert!(proposal.has_majority());
    }
}
