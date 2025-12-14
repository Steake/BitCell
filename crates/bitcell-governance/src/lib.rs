//! On-Chain Governance System for BitCell
//!
//! Implements RC3-005 requirements:
//! - Proposal System for parameter changes, treasury spending, and protocol upgrades
//! - Voting Mechanism with token-weighted voting, delegation, and optional quadratic voting
//! - Execution with timelock delay, emergency cancel, and multi-sig guardian

mod serde_pubkey;
pub mod proposal;
pub mod voting;
pub mod execution;

pub use proposal::{Proposal, ProposalType, ProposalStatus, ProposalId};
pub use voting::{Vote, VoteType, VotingPower, Delegation};
pub use execution::{ExecutionQueue, TimelockDelay, GuardianAction};

use std::collections::HashMap;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Proposal not found")]
    ProposalNotFound,
    
    #[error("Invalid proposal")]
    InvalidProposal,
    
    #[error("Voting period ended")]
    VotingPeriodEnded,
    
    #[error("Voting period not ended")]
    VotingPeriodNotEnded,
    
    #[error("Already voted")]
    AlreadyVoted,
    
    #[error("Insufficient voting power")]
    InsufficientVotingPower,
    
    #[error("Execution locked")]
    ExecutionLocked,
    
    #[error("Not authorized")]
    NotAuthorized,
    
    #[error("Invalid timelock")]
    InvalidTimelock,
}

/// Governance system manager
pub struct GovernanceManager {
    /// Active proposals
    pub proposals: HashMap<ProposalId, Proposal>,
    
    /// Vote records
    pub votes: HashMap<ProposalId, HashMap<[u8; 33], Vote>>,
    
    /// Delegations
    pub delegations: HashMap<[u8; 33], Delegation>,
    
    /// Execution queue
    pub execution_queue: ExecutionQueue,
    
    /// Multi-sig guardians
    pub guardians: Vec<[u8; 33]>,
    
    /// Next proposal ID
    next_proposal_id: u64,
}

impl GovernanceManager {
    pub fn new(guardians: Vec<[u8; 33]>) -> Self {
        Self {
            proposals: HashMap::new(),
            votes: HashMap::new(),
            delegations: HashMap::new(),
            execution_queue: ExecutionQueue::new(),
            guardians,
            next_proposal_id: 1,
        }
    }
    
    /// Submit a new proposal
    pub fn submit_proposal(
        &mut self,
        proposer: [u8; 33],
        proposal_type: ProposalType,
        description: String,
        voting_period_blocks: u64,
        current_block: u64,
    ) -> Result<ProposalId> {
        let proposal_id = ProposalId(self.next_proposal_id);
        self.next_proposal_id += 1;
        
        let proposal = Proposal::new(
            proposal_id,
            proposer,
            proposal_type,
            description,
            current_block,
            voting_period_blocks,
        );
        
        self.proposals.insert(proposal_id, proposal);
        self.votes.insert(proposal_id, HashMap::new());
        
        tracing::info!(
            proposal_id = proposal_id.0,
            proposer = %hex::encode(&proposer),
            "New proposal submitted"
        );
        
        Ok(proposal_id)
    }
    
    /// Cast a vote on a proposal
    pub fn vote(
        &mut self,
        proposal_id: ProposalId,
        voter: [u8; 33],
        vote_type: VoteType,
        voting_power: u64,
        current_block: u64,
        quadratic: bool,
    ) -> Result<()> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(Error::ProposalNotFound)?;
        
        // Check if voting period is still active
        if current_block > proposal.voting_end_block {
            return Err(Error::VotingPeriodEnded);
        }
        
        // Check if already voted
        let votes = self.votes.get_mut(&proposal_id).unwrap();
        if votes.contains_key(&voter) {
            return Err(Error::AlreadyVoted);
        }
        
        // Calculate effective voting power (quadratic if enabled)
        let effective_power = if quadratic {
            VotingPower::quadratic(voting_power)
        } else {
            VotingPower::linear(voting_power)
        };
        
        // Record vote
        let vote = Vote {
            voter,
            vote_type: vote_type.clone(),
            voting_power: effective_power,
            block_number: current_block,
        };
        
        // Update proposal tallies
        match vote_type {
            VoteType::For => proposal.votes_for += effective_power.value,
            VoteType::Against => proposal.votes_against += effective_power.value,
            VoteType::Abstain => proposal.votes_abstain += effective_power.value,
        }
        
        votes.insert(voter, vote);
        
        tracing::info!(
            proposal_id = proposal_id.0,
            voter = %hex::encode(&voter),
            vote_type = ?vote_type,
            power = effective_power.value,
            "Vote cast"
        );
        
        Ok(())
    }
    
    /// Finalize a proposal after voting period ends
    pub fn finalize_proposal(
        &mut self,
        proposal_id: ProposalId,
        current_block: u64,
    ) -> Result<()> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(Error::ProposalNotFound)?;
        
        // Check if voting period has ended
        if current_block <= proposal.voting_end_block {
            return Err(Error::VotingPeriodNotEnded);
        }
        
        // Determine outcome
        let total_votes = proposal.votes_for + proposal.votes_against + proposal.votes_abstain;
        let quorum_met = total_votes >= proposal.quorum_threshold;
        let majority_met = proposal.votes_for > proposal.votes_against;
        
        if quorum_met && majority_met {
            proposal.status = ProposalStatus::Passed;
            
            // Queue for execution with timelock
            self.execution_queue.enqueue(
                proposal_id,
                current_block,
                proposal.proposal_type.clone(),
            );
            
            tracing::info!(
                proposal_id = proposal_id.0,
                votes_for = proposal.votes_for,
                votes_against = proposal.votes_against,
                "Proposal passed and queued for execution"
            );
        } else {
            proposal.status = ProposalStatus::Rejected;
            
            tracing::info!(
                proposal_id = proposal_id.0,
                votes_for = proposal.votes_for,
                votes_against = proposal.votes_against,
                quorum_met = quorum_met,
                majority_met = majority_met,
                "Proposal rejected"
            );
        }
        
        Ok(())
    }
    
    /// Execute a proposal after timelock expires
    pub fn execute_proposal(
        &mut self,
        proposal_id: ProposalId,
        current_block: u64,
    ) -> Result<()> {
        let proposal = self.proposals.get(&proposal_id)
            .ok_or(Error::ProposalNotFound)?;
        
        if proposal.status != ProposalStatus::Passed {
            return Err(Error::InvalidProposal);
        }
        
        self.execution_queue.execute(proposal_id, current_block)?;
        
        // Mark as executed
        if let Some(p) = self.proposals.get_mut(&proposal_id) {
            p.status = ProposalStatus::Executed;
        }
        
        tracing::info!(
            proposal_id = proposal_id.0,
            "Proposal executed"
        );
        
        Ok(())
    }
    
    /// Emergency cancel by guardians
    pub fn emergency_cancel(
        &mut self,
        proposal_id: ProposalId,
        guardian_signatures: Vec<[u8; 33]>,
    ) -> Result<()> {
        // Verify sufficient guardian signatures (require 2/3 majority)
        let required = (self.guardians.len() * 2 + 2) / 3;
        let valid_signatures = guardian_signatures.iter()
            .filter(|sig| self.guardians.contains(sig))
            .count();
        
        if valid_signatures < required {
            return Err(Error::NotAuthorized);
        }
        
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(Error::ProposalNotFound)?;
        
        proposal.status = ProposalStatus::Cancelled;
        
        // Try to cancel from execution queue (may not be queued yet)
        let _ = self.execution_queue.cancel(proposal_id);
        
        tracing::warn!(
            proposal_id = proposal_id.0,
            guardian_signatures = valid_signatures,
            "Proposal emergency cancelled by guardians"
        );
        
        Ok(())
    }
    
    /// Delegate voting power
    pub fn delegate(
        &mut self,
        delegator: [u8; 33],
        delegatee: [u8; 33],
        amount: u64,
    ) -> Result<()> {
        let delegation = Delegation {
            delegator,
            delegatee,
            amount,
        };
        
        self.delegations.insert(delegator, delegation);
        
        tracing::info!(
            delegator = %hex::encode(&delegator),
            delegatee = %hex::encode(&delegatee),
            amount = amount,
            "Voting power delegated"
        );
        
        Ok(())
    }
    
    /// Get effective voting power (including delegations)
    pub fn get_voting_power(&self, voter: &[u8; 33], token_balance: u64) -> u64 {
        let mut power = token_balance;
        
        // Add delegated power
        for delegation in self.delegations.values() {
            if delegation.delegatee == *voter {
                power += delegation.amount;
            }
        }
        
        power
    }
}

impl Default for GovernanceManager {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_proposal() {
        let mut gov = GovernanceManager::new(vec![]);
        let proposer = [1u8; 33];
        
        let proposal_id = gov.submit_proposal(
            proposer,
            ProposalType::ParameterChange {
                parameter: "block_time".to_string(),
                new_value: vec![10],
            },
            "Reduce block time to 10s".to_string(),
            1000,
            100,
        ).unwrap();
        
        assert_eq!(proposal_id.0, 1);
        assert!(gov.proposals.contains_key(&proposal_id));
    }
    
    #[test]
    fn test_voting() {
        let mut gov = GovernanceManager::new(vec![]);
        let proposer = [1u8; 33];
        let voter = [2u8; 33];
        
        let proposal_id = gov.submit_proposal(
            proposer,
            ProposalType::ParameterChange {
                parameter: "block_time".to_string(),
                new_value: vec![10],
            },
            "Reduce block time".to_string(),
            1000,
            100,
        ).unwrap();
        
        // Cast vote
        gov.vote(proposal_id, voter, VoteType::For, 1000, 150, false).unwrap();
        
        let proposal = gov.proposals.get(&proposal_id).unwrap();
        assert_eq!(proposal.votes_for, 1000);
    }
    
    #[test]
    fn test_double_vote_rejected() {
        let mut gov = GovernanceManager::new(vec![]);
        let proposer = [1u8; 33];
        let voter = [2u8; 33];
        
        let proposal_id = gov.submit_proposal(
            proposer,
            ProposalType::ParameterChange {
                parameter: "block_time".to_string(),
                new_value: vec![10],
            },
            "Test".to_string(),
            1000,
            100,
        ).unwrap();
        
        gov.vote(proposal_id, voter, VoteType::For, 1000, 150, false).unwrap();
        
        // Try to vote again
        let result = gov.vote(proposal_id, voter, VoteType::Against, 500, 200, false);
        assert!(matches!(result, Err(Error::AlreadyVoted)));
    }
    
    #[test]
    fn test_finalize_proposal() {
        let mut gov = GovernanceManager::new(vec![]);
        let proposer = [1u8; 33];
        
        let proposal_id = gov.submit_proposal(
            proposer,
            ProposalType::TreasurySpending {
                recipient: [5u8; 33],
                amount: 10000,
                reason: "Development grant".to_string(),
            },
            "Fund development".to_string(),
            1000,
            100,
        ).unwrap();
        
        // Cast votes (total must meet quorum of 10000)
        gov.vote(proposal_id, [2u8; 33], VoteType::For, 7000, 150, false).unwrap();
        gov.vote(proposal_id, [3u8; 33], VoteType::Against, 3000, 160, false).unwrap();
        
        // Finalize after voting period
        gov.finalize_proposal(proposal_id, 1200).unwrap();
        
        let proposal = gov.proposals.get(&proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Passed);
    }
    
    #[test]
    fn test_emergency_cancel() {
        let guardians = vec![[10u8; 33], [11u8; 33], [12u8; 33]];
        let mut gov = GovernanceManager::new(guardians.clone());
        
        let proposal_id = gov.submit_proposal(
            [1u8; 33],
            ProposalType::ParameterChange {
                parameter: "test".to_string(),
                new_value: vec![1],
            },
            "Test".to_string(),
            1000,
            100,
        ).unwrap();
        
        // Emergency cancel with 2 of 3 guardians
        gov.emergency_cancel(proposal_id, vec![guardians[0], guardians[1]]).unwrap();
        
        let proposal = gov.proposals.get(&proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Cancelled);
    }
    
    #[test]
    fn test_delegation() {
        let mut gov = GovernanceManager::new(vec![]);
        let delegator = [1u8; 33];
        let delegatee = [2u8; 33];
        
        gov.delegate(delegator, delegatee, 5000).unwrap();
        
        let power = gov.get_voting_power(&delegatee, 1000);
        assert_eq!(power, 6000); // 1000 own + 5000 delegated
    }
}
