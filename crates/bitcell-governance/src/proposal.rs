//! Governance proposal types and logic

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// Unique proposal identifier (SHA-256 hash)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProposalId(pub [u8; 32]);

impl ProposalId {
    /// Generate proposal ID from proposal data
    pub fn generate(
        proposer: &[u8; 33],
        proposal_type: &ProposalType,
        description: &str,
        created_at: u64,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(proposer);
        hasher.update(&bincode::serialize(proposal_type).unwrap_or_default());
        hasher.update(description.as_bytes());
        hasher.update(&created_at.to_le_bytes());
        
        let hash = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&hash);
        
        ProposalId(id)
    }
}

/// Type of governance proposal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalType {
    /// Change a protocol parameter
    ParameterChange {
        parameter: String,
        new_value: String,
    },
    
    /// Spend from treasury
    TreasurySpending {
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

/// Status of a proposal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    /// Proposal is active and accepting votes
    Active,
    
    /// Proposal passed and was executed
    Passed,
    
    /// Proposal was rejected (failed to pass or quorum not met)
    Rejected,
    
    /// Proposal was cancelled by guardians
    Cancelled,
}

/// A governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Unique identifier
    pub id: ProposalId,
    
    /// Address of proposer
    pub proposer: [u8; 33],
    
    /// Type of proposal
    pub proposal_type: ProposalType,
    
    /// Human-readable description
    pub description: String,
    
    /// Timestamp when proposal was created
    pub created_at: u64,
    
    /// Current status
    pub status: ProposalStatus,
    
    /// Total votes in favor (in effective voting power)
    pub votes_for: u64,
    
    /// Total votes against (in effective voting power)
    pub votes_against: u64,
    
    /// Timestamp when proposal was executed (if passed)
    pub executed_at: Option<u64>,
}

impl Proposal {
    /// Create a new proposal
    pub fn new(
        proposer: [u8; 33],
        proposal_type: ProposalType,
        description: String,
        created_at: u64,
    ) -> Self {
        let id = ProposalId::generate(&proposer, &proposal_type, &description, created_at);
        
        Self {
            id,
            proposer,
            proposal_type,
            description,
            created_at,
            status: ProposalStatus::Active,
            votes_for: 0,
            votes_against: 0,
            executed_at: None,
        }
    }
    
    /// Check if proposal is active
    pub fn is_active(&self) -> bool {
        self.status == ProposalStatus::Active
    }
    
    /// Get total votes
    pub fn total_votes(&self) -> u64 {
        self.votes_for.saturating_add(self.votes_against)
    }
    
    /// Get vote percentage for (0-100)
    pub fn vote_percentage_for(&self) -> f64 {
        let total = self.total_votes();
        if total == 0 {
            return 0.0;
        }
        (self.votes_for as f64 / total as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_proposal_id_generation() {
        let proposer = [1u8; 33];
        let proposal_type = ProposalType::ParameterChange {
            parameter: "test".to_string(),
            new_value: "value".to_string(),
        };
        
        let id1 = ProposalId::generate(&proposer, &proposal_type, "Test", 1000);
        let id2 = ProposalId::generate(&proposer, &proposal_type, "Test", 1000);
        
        // Same inputs should produce same ID
        assert_eq!(id1, id2);
        
        // Different timestamp should produce different ID
        let id3 = ProposalId::generate(&proposer, &proposal_type, "Test", 1001);
        assert_ne!(id1, id3);
    }
    
    #[test]
    fn test_proposal_creation() {
        let proposer = [1u8; 33];
        let proposal_type = ProposalType::TreasurySpending {
            recipient: [2u8; 33],
            amount: 1000,
            reason: "Development".to_string(),
        };
        
        let proposal = Proposal::new(proposer, proposal_type, "Fund dev".to_string(), 1000);
        
        assert_eq!(proposal.proposer, proposer);
        assert_eq!(proposal.status, ProposalStatus::Active);
        assert!(proposal.is_active());
        assert_eq!(proposal.votes_for, 0);
        assert_eq!(proposal.votes_against, 0);
    }
    
    #[test]
    fn test_vote_percentage() {
        let mut proposal = Proposal::new(
            [1u8; 33],
            ProposalType::ParameterChange {
                parameter: "test".to_string(),
                new_value: "value".to_string(),
            },
            "Test".to_string(),
            1000,
        );
        
        proposal.votes_for = 75;
        proposal.votes_against = 25;
        
        assert_eq!(proposal.total_votes(), 100);
        assert_eq!(proposal.vote_percentage_for(), 75.0);
    }
    
    #[test]
    fn test_proposal_types() {
        // Test ParameterChange
        let param_change = ProposalType::ParameterChange {
            parameter: "max_block_size".to_string(),
            new_value: "2000000".to_string(),
        };
        assert!(matches!(param_change, ProposalType::ParameterChange { .. }));
        
        // Test TreasurySpending
        let treasury = ProposalType::TreasurySpending {
            recipient: [1u8; 33],
            amount: 5000,
            reason: "Grant".to_string(),
        };
        assert!(matches!(treasury, ProposalType::TreasurySpending { .. }));
        
        // Test ProtocolUpgrade
        let upgrade = ProposalType::ProtocolUpgrade {
            version: "1.1.0".to_string(),
            code_hash: [0u8; 32],
            description: "New features".to_string(),
        };
        assert!(matches!(upgrade, ProposalType::ProtocolUpgrade { .. }));
    }
}
