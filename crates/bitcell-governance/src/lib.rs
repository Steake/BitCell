//! BitCell On-Chain Governance System
//!
//! This crate implements a comprehensive governance system for BitCell blockchain:
//! - Proposal submission and voting
//! - Token-weighted voting (linear and quadratic)
//! - Vote delegation
//! - Type-specific timelock delays
//! - Multi-sig guardian controls
//!
//! ## Architecture
//!
//! The governance system supports three types of proposals:
//! - **Parameter Changes**: Modify protocol parameters (2-day timelock)
//! - **Treasury Spending**: Allocate treasury funds (6-hour timelock)
//! - **Protocol Upgrades**: Update protocol code (2-day timelock)
//!
//! ## Security Features
//!
//! - Saturating arithmetic for overflow protection
//! - Proposal ID collision resistance using SHA-256
//! - Double-vote prevention
//! - Multi-sig guardian override (2/3 majority)
//! - Quadratic voting for Sybil resistance

pub mod proposal;
pub mod voting;
pub mod delegation;
pub mod guardian;
pub mod timelock;

pub use proposal::{Proposal, ProposalType, ProposalStatus, ProposalId};
pub use voting::{Vote, VotingPower, VotingMethod, VoteRecord};
pub use delegation::{Delegation, DelegationManager};
pub use guardian::{Guardian, GuardianSet, GuardianAction};
pub use timelock::{Timelock, TimelockConfig};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Proposal not found")]
    ProposalNotFound,
    
    #[error("Insufficient voting power: required {required}, have {available}")]
    InsufficientVotingPower { required: u64, available: u64 },
    
    #[error("Proposal already finalized")]
    ProposalFinalized,
    
    #[error("Timelock not expired: {remaining_seconds} seconds remaining")]
    TimelockNotExpired { remaining_seconds: u64 },
    
    #[error("Duplicate vote detected")]
    DuplicateVote,
    
    #[error("Invalid guardian signature")]
    InvalidGuardianSignature,
    
    #[error("Insufficient guardian approvals: required {required}, have {available}")]
    InsufficientGuardianApprovals { required: usize, available: usize },
    
    #[error("Quorum not reached: required {required}, have {available}")]
    QuorumNotReached { required: u64, available: u64 },
    
    #[error("Invalid proposal type")]
    InvalidProposalType,
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Invalid delegation")]
    InvalidDelegation,
}

/// Governance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    /// Minimum quorum (in CELL tokens) required for proposal to pass
    pub quorum: u64,
    
    /// Voting method (Linear or Quadratic)
    pub voting_method: VotingMethod,
    
    /// Guardian threshold (e.g., 2 out of 3)
    pub guardian_threshold: GuardianThreshold,
    
    /// Timelock configuration
    pub timelock: TimelockConfig,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            quorum: 10_000 * 100_000_000, // 10,000 CELL (in smallest units)
            voting_method: VotingMethod::Linear,
            guardian_threshold: GuardianThreshold { required: 2, total: 3 },
            timelock: TimelockConfig::default(),
        }
    }
}

/// Guardian threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuardianThreshold {
    pub required: usize,
    pub total: usize,
}

impl GuardianThreshold {
    pub fn is_satisfied(&self, approvals: usize) -> bool {
        approvals >= self.required
    }
}

/// Main governance manager
pub struct GovernanceManager {
    /// Active proposals indexed by ID
    pub proposals: HashMap<ProposalId, Proposal>,
    
    /// Vote records for each proposal
    pub votes: HashMap<ProposalId, Vec<VoteRecord>>,
    
    /// Delegation manager
    pub delegations: DelegationManager,
    
    /// Guardian set
    pub guardians: GuardianSet,
    
    /// Configuration
    pub config: GovernanceConfig,
}

impl GovernanceManager {
    /// Create a new governance manager with default configuration
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            votes: HashMap::new(),
            delegations: DelegationManager::new(),
            guardians: GuardianSet::new(),
            config: GovernanceConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: GovernanceConfig, guardians: GuardianSet) -> Self {
        Self {
            proposals: HashMap::new(),
            votes: HashMap::new(),
            delegations: DelegationManager::new(),
            guardians,
            config,
        }
    }
    
    /// Submit a new proposal
    pub fn submit_proposal(
        &mut self,
        proposer: [u8; 33],
        proposal_type: ProposalType,
        description: String,
        created_at: u64,
    ) -> Result<ProposalId> {
        let proposal = Proposal::new(proposer, proposal_type, description, created_at);
        let proposal_id = proposal.id;
        
        self.proposals.insert(proposal_id, proposal);
        self.votes.insert(proposal_id, Vec::new());
        
        tracing::info!(
            proposal_id = %hex::encode(&proposal_id.0),
            proposer = %hex::encode(&proposer),
            "Proposal submitted"
        );
        
        Ok(proposal_id)
    }
    
    /// Cast a vote on a proposal
    pub fn vote(
        &mut self,
        proposal_id: ProposalId,
        voter: [u8; 33],
        support: bool,
        voting_power: u64,
        timestamp: u64,
    ) -> Result<()> {
        // Check if proposal exists
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(Error::ProposalNotFound)?;
        
        // Check if proposal is still active
        if proposal.status != ProposalStatus::Active {
            return Err(Error::ProposalFinalized);
        }
        
        // Check for duplicate votes
        let vote_records = self.votes.get(&proposal_id).unwrap();
        if vote_records.iter().any(|v| v.voter == voter) {
            return Err(Error::DuplicateVote);
        }
        
        // Calculate effective voting power based on method
        let effective_power = match self.config.voting_method {
            VotingMethod::Linear => voting_power,
            VotingMethod::Quadratic => integer_sqrt(voting_power),
        };
        
        // Create vote record
        let vote = Vote {
            proposal_id,
            voter,
            support,
            power: effective_power,
            timestamp,
        };
        
        let vote_record = VoteRecord {
            voter,
            support,
            power: effective_power,
            timestamp,
        };
        
        // Update vote counts using saturating arithmetic
        if support {
            proposal.votes_for = proposal.votes_for.saturating_add(effective_power);
        } else {
            proposal.votes_against = proposal.votes_against.saturating_add(effective_power);
        }
        
        // Store vote record
        self.votes.get_mut(&proposal_id).unwrap().push(vote_record);
        
        tracing::info!(
            proposal_id = %hex::encode(&proposal_id.0),
            voter = %hex::encode(&voter),
            support = support,
            power = effective_power,
            "Vote cast"
        );
        
        Ok(())
    }
    
    /// Finalize a proposal (check quorum and timelock)
    pub fn finalize_proposal(
        &mut self,
        proposal_id: ProposalId,
        current_time: u64,
    ) -> Result<bool> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(Error::ProposalNotFound)?;
        
        // Check if already finalized
        if proposal.status != ProposalStatus::Active {
            return Err(Error::ProposalFinalized);
        }
        
        // Check quorum
        let total_votes = proposal.votes_for.saturating_add(proposal.votes_against);
        if total_votes < self.config.quorum {
            proposal.status = ProposalStatus::Rejected;
            return Err(Error::QuorumNotReached {
                required: self.config.quorum,
                available: total_votes,
            });
        }
        
        // Check if passed
        let passed = proposal.votes_for > proposal.votes_against;
        
        if passed {
            // Check timelock
            let timelock_duration = self.config.timelock.get_duration(&proposal.proposal_type);
            let timelock_expiry = proposal.created_at.saturating_add(timelock_duration);
            
            if current_time < timelock_expiry {
                let remaining = timelock_expiry.saturating_sub(current_time);
                return Err(Error::TimelockNotExpired {
                    remaining_seconds: remaining,
                });
            }
            
            proposal.status = ProposalStatus::Passed;
            proposal.executed_at = Some(current_time);
            
            tracing::info!(
                proposal_id = %hex::encode(&proposal_id.0),
                votes_for = proposal.votes_for,
                votes_against = proposal.votes_against,
                "Proposal passed and executed"
            );
            
            Ok(true)
        } else {
            proposal.status = ProposalStatus::Rejected;
            
            tracing::info!(
                proposal_id = %hex::encode(&proposal_id.0),
                votes_for = proposal.votes_for,
                votes_against = proposal.votes_against,
                "Proposal rejected"
            );
            
            Ok(false)
        }
    }
    
    /// Guardian emergency override
    pub fn guardian_override(
        &mut self,
        proposal_id: ProposalId,
        action: GuardianAction,
        signatures: Vec<[u8; 64]>,
    ) -> Result<()> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(Error::ProposalNotFound)?;
        
        // Verify guardian signatures
        let valid_signatures = self.guardians.verify_signatures(&proposal_id, &signatures)?;
        
        // Check threshold
        if !self.config.guardian_threshold.is_satisfied(valid_signatures) {
            return Err(Error::InsufficientGuardianApprovals {
                required: self.config.guardian_threshold.required,
                available: valid_signatures,
            });
        }
        
        // Apply action
        match action {
            GuardianAction::Cancel => {
                proposal.status = ProposalStatus::Cancelled;
                tracing::warn!(
                    proposal_id = %hex::encode(&proposal_id.0),
                    "Proposal cancelled by guardian override"
                );
            }
            GuardianAction::ExecuteImmediately => {
                proposal.status = ProposalStatus::Passed;
                tracing::warn!(
                    proposal_id = %hex::encode(&proposal_id.0),
                    "Proposal executed immediately by guardian override"
                );
            }
        }
        
        Ok(())
    }
    
    /// Get proposal by ID
    pub fn get_proposal(&self, proposal_id: &ProposalId) -> Option<&Proposal> {
        self.proposals.get(proposal_id)
    }
    
    /// Get all votes for a proposal
    pub fn get_votes(&self, proposal_id: &ProposalId) -> Option<&Vec<VoteRecord>> {
        self.votes.get(proposal_id)
    }
    
    /// Delegate voting power
    pub fn delegate(
        &mut self,
        delegator: [u8; 33],
        delegatee: [u8; 33],
        amount: u64,
    ) -> Result<()> {
        self.delegations.delegate(delegator, delegatee, amount)
    }
    
    /// Undelegate voting power
    pub fn undelegate(&mut self, delegator: [u8; 33], delegatee: [u8; 33]) -> Result<()> {
        self.delegations.undelegate(delegator, delegatee)
    }
    
    /// Get effective voting power (including delegations)
    pub fn get_voting_power(&self, voter: &[u8; 33], base_power: u64) -> u64 {
        let delegated_power = self.delegations.get_delegated_power(voter);
        base_power.saturating_add(delegated_power)
    }
}

impl Default for GovernanceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Integer square root for quadratic voting
/// Uses binary search for efficiency
pub fn integer_sqrt(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    
    let mut left = 1u64;
    let mut right = n;
    let mut result = 0u64;
    
    while left <= right {
        let mid = left + (right - left) / 2;
        
        // Check if mid * mid <= n using division to avoid overflow
        if mid <= n / mid {
            result = mid;
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guardian::Guardian;
    
    #[test]
    fn test_integer_sqrt() {
        assert_eq!(integer_sqrt(0), 0);
        assert_eq!(integer_sqrt(1), 1);
        assert_eq!(integer_sqrt(4), 2);
        assert_eq!(integer_sqrt(9), 3);
        assert_eq!(integer_sqrt(16), 4);
        assert_eq!(integer_sqrt(100), 10);
        assert_eq!(integer_sqrt(10000), 100);
        assert_eq!(integer_sqrt(99), 9);
        assert_eq!(integer_sqrt(101), 10);
    }
    
    #[test]
    fn test_governance_config_default() {
        let config = GovernanceConfig::default();
        assert_eq!(config.quorum, 10_000 * 100_000_000);
        assert_eq!(config.voting_method, VotingMethod::Linear);
        assert_eq!(config.guardian_threshold.required, 2);
        assert_eq!(config.guardian_threshold.total, 3);
    }
    
    #[test]
    fn test_submit_proposal() {
        let mut gov = GovernanceManager::new();
        let proposer = [1u8; 33];
        
        let proposal_id = gov.submit_proposal(
            proposer,
            ProposalType::ParameterChange {
                parameter: "max_block_size".to_string(),
                new_value: "2000000".to_string(),
            },
            "Increase max block size to 2MB".to_string(),
            1000,
        ).unwrap();
        
        let proposal = gov.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.proposer, proposer);
        assert_eq!(proposal.status, ProposalStatus::Active);
    }
    
    #[test]
    fn test_vote_linear() {
        let mut gov = GovernanceManager::new();
        let proposer = [1u8; 33];
        let voter = [2u8; 33];
        
        let proposal_id = gov.submit_proposal(
            proposer,
            ProposalType::TreasurySpending {
                recipient: [3u8; 33],
                amount: 1000,
                reason: "Development grant".to_string(),
            },
            "Fund development".to_string(),
            1000,
        ).unwrap();
        
        // Vote with 100 power
        gov.vote(proposal_id, voter, true, 100, 1100).unwrap();
        
        let proposal = gov.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.votes_for, 100);
        assert_eq!(proposal.votes_against, 0);
    }
    
    #[test]
    fn test_vote_quadratic() {
        let mut config = GovernanceConfig::default();
        config.voting_method = VotingMethod::Quadratic;
        
        let mut gov = GovernanceManager::with_config(config, GuardianSet::new());
        let proposer = [1u8; 33];
        let voter = [2u8; 33];
        
        let proposal_id = gov.submit_proposal(
            proposer,
            ProposalType::ParameterChange {
                parameter: "min_stake".to_string(),
                new_value: "1000".to_string(),
            },
            "Reduce min stake".to_string(),
            1000,
        ).unwrap();
        
        // Vote with 100 power -> sqrt(100) = 10 effective power
        gov.vote(proposal_id, voter, true, 100, 1100).unwrap();
        
        let proposal = gov.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.votes_for, 10);
    }
    
    #[test]
    fn test_duplicate_vote_prevention() {
        let mut gov = GovernanceManager::new();
        let proposer = [1u8; 33];
        let voter = [2u8; 33];
        
        let proposal_id = gov.submit_proposal(
            proposer,
            ProposalType::ParameterChange {
                parameter: "test".to_string(),
                new_value: "value".to_string(),
            },
            "Test".to_string(),
            1000,
        ).unwrap();
        
        // First vote succeeds
        gov.vote(proposal_id, voter, true, 100, 1100).unwrap();
        
        // Second vote fails
        let result = gov.vote(proposal_id, voter, false, 50, 1200);
        assert!(matches!(result, Err(Error::DuplicateVote)));
    }
    
    #[test]
    fn test_quorum_not_reached() {
        let mut gov = GovernanceManager::new();
        let proposer = [1u8; 33];
        
        let proposal_id = gov.submit_proposal(
            proposer,
            ProposalType::ParameterChange {
                parameter: "test".to_string(),
                new_value: "value".to_string(),
            },
            "Test".to_string(),
            1000,
        ).unwrap();
        
        // Vote with insufficient power (quorum is 10,000 CELL)
        gov.vote(proposal_id, [2u8; 33], true, 100, 1100).unwrap();
        
        // Finalization fails due to quorum
        let result = gov.finalize_proposal(proposal_id, 2000);
        assert!(matches!(result, Err(Error::QuorumNotReached { .. })));
    }
    
    #[test]
    fn test_delegation() {
        let mut gov = GovernanceManager::new();
        let delegator = [1u8; 33];
        let delegatee = [2u8; 33];
        
        // Delegate 1000 power
        gov.delegate(delegator, delegatee, 1000).unwrap();
        
        // Check effective voting power
        let power = gov.get_voting_power(&delegatee, 500);
        assert_eq!(power, 1500); // 500 base + 1000 delegated
    }
}
