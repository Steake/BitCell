//! Proposal execution system with timelock and guardian controls

use crate::{Error, Result, ProposalId, ProposalType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Timelock delay in blocks
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimelockDelay {
    /// Number of blocks to wait before execution
    pub blocks: u64,
}

impl TimelockDelay {
    /// Standard timelock delay (e.g., 2 days assuming 12s blocks)
    pub fn standard() -> Self {
        Self { blocks: 14400 } // ~2 days
    }
    
    /// Fast track delay (e.g., 6 hours)
    pub fn fast_track() -> Self {
        Self { blocks: 1800 } // ~6 hours
    }
    
    /// Emergency delay (e.g., 1 hour)
    pub fn emergency() -> Self {
        Self { blocks: 300 } // ~1 hour
    }
}

impl Default for TimelockDelay {
    fn default() -> Self {
        Self::standard()
    }
}

/// Guardian action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuardianAction {
    /// Cancel a proposal
    Cancel(ProposalId),
    
    /// Fast-track a proposal (reduce timelock)
    FastTrack(ProposalId),
    
    /// Veto a proposal execution
    Veto(ProposalId),
}

/// Queued proposal for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedProposal {
    /// Proposal ID
    pub proposal_id: ProposalId,
    
    /// Proposal type
    pub proposal_type: ProposalType,
    
    /// Block when it was queued
    pub queued_block: u64,
    
    /// Timelock delay
    pub timelock: TimelockDelay,
    
    /// Block when it can be executed
    pub execution_block: u64,
}

impl QueuedProposal {
    pub fn new(
        proposal_id: ProposalId,
        proposal_type: ProposalType,
        queued_block: u64,
        timelock: TimelockDelay,
    ) -> Self {
        Self {
            proposal_id,
            proposal_type,
            queued_block,
            timelock,
            execution_block: queued_block + timelock.blocks,
        }
    }
    
    /// Check if proposal is ready for execution
    pub fn is_executable(&self, current_block: u64) -> bool {
        current_block >= self.execution_block
    }
}

/// Execution queue managing timelocked proposals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionQueue {
    /// Queued proposals awaiting execution
    queue: HashMap<ProposalId, QueuedProposal>,
}

impl ExecutionQueue {
    pub fn new() -> Self {
        Self {
            queue: HashMap::new(),
        }
    }
    
    /// Enqueue a proposal for execution after timelock
    pub fn enqueue(
        &mut self,
        proposal_id: ProposalId,
        current_block: u64,
        proposal_type: ProposalType,
    ) {
        let timelock = match &proposal_type {
            ProposalType::ParameterChange { .. } => TimelockDelay::standard(),
            ProposalType::TreasurySpending { .. } => TimelockDelay::fast_track(),
            ProposalType::ProtocolUpgrade { .. } => TimelockDelay::standard(),
        };
        
        let queued = QueuedProposal::new(
            proposal_id,
            proposal_type,
            current_block,
            timelock,
        );
        
        let execution_block = queued.execution_block;
        self.queue.insert(proposal_id, queued);
        
        tracing::info!(
            proposal_id = proposal_id.0,
            execution_block = execution_block,
            "Proposal queued for execution after timelock"
        );
    }
    
    /// Execute a proposal (must be past timelock)
    pub fn execute(
        &mut self,
        proposal_id: ProposalId,
        current_block: u64,
    ) -> Result<()> {
        let queued = self.queue.get(&proposal_id)
            .ok_or(Error::ProposalNotFound)?;
        
        if !queued.is_executable(current_block) {
            return Err(Error::ExecutionLocked);
        }
        
        // Remove from queue
        self.queue.remove(&proposal_id);
        
        tracing::info!(
            proposal_id = proposal_id.0,
            "Proposal executed and removed from queue"
        );
        
        Ok(())
    }
    
    /// Cancel a proposal (guardian action)
    pub fn cancel(&mut self, proposal_id: ProposalId) -> Result<()> {
        self.queue.remove(&proposal_id)
            .ok_or(Error::ProposalNotFound)?;
        
        tracing::warn!(
            proposal_id = proposal_id.0,
            "Proposal cancelled and removed from execution queue"
        );
        
        Ok(())
    }
    
    /// Fast-track a proposal (guardian action)
    pub fn fast_track(
        &mut self,
        proposal_id: ProposalId,
        current_block: u64,
    ) -> Result<()> {
        let queued = self.queue.get_mut(&proposal_id)
            .ok_or(Error::ProposalNotFound)?;
        
        queued.timelock = TimelockDelay::fast_track();
        queued.execution_block = current_block + queued.timelock.blocks;
        
        tracing::info!(
            proposal_id = proposal_id.0,
            new_execution_block = queued.execution_block,
            "Proposal fast-tracked"
        );
        
        Ok(())
    }
    
    /// Get all executable proposals
    pub fn get_executable(&self, current_block: u64) -> Vec<ProposalId> {
        self.queue.values()
            .filter(|p| p.is_executable(current_block))
            .map(|p| p.proposal_id)
            .collect()
    }
    
    /// Get proposal from queue
    pub fn get(&self, proposal_id: ProposalId) -> Option<&QueuedProposal> {
        self.queue.get(&proposal_id)
    }
}

impl Default for ExecutionQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timelock_delays() {
        let standard = TimelockDelay::standard();
        assert_eq!(standard.blocks, 14400);
        
        let fast = TimelockDelay::fast_track();
        assert_eq!(fast.blocks, 1800);
        
        let emergency = TimelockDelay::emergency();
        assert_eq!(emergency.blocks, 300);
    }
    
    #[test]
    fn test_queued_proposal() {
        let proposal = QueuedProposal::new(
            ProposalId(1),
            ProposalType::ParameterChange {
                parameter: "test".to_string(),
                new_value: vec![1],
            },
            100,
            TimelockDelay::fast_track(),
        );
        
        assert_eq!(proposal.execution_block, 1900); // 100 + 1800
        assert!(!proposal.is_executable(1000));
        assert!(proposal.is_executable(1900));
        assert!(proposal.is_executable(2000));
    }
    
    #[test]
    fn test_execution_queue() {
        let mut queue = ExecutionQueue::new();
        
        queue.enqueue(
            ProposalId(1),
            100,
            ProposalType::TreasurySpending {
                recipient: [1u8; 33],
                amount: 1000,
                reason: "Test".to_string(),
            },
        );
        
        let queued = queue.get(ProposalId(1)).unwrap();
        assert_eq!(queued.execution_block, 1900); // Fast track for treasury
        
        // Cannot execute before timelock
        let result = queue.execute(ProposalId(1), 1000);
        assert!(matches!(result, Err(Error::ExecutionLocked)));
        
        // Can execute after timelock
        queue.execute(ProposalId(1), 2000).unwrap();
        assert!(queue.get(ProposalId(1)).is_none());
    }
    
    #[test]
    fn test_cancel() {
        let mut queue = ExecutionQueue::new();
        
        queue.enqueue(
            ProposalId(1),
            100,
            ProposalType::ParameterChange {
                parameter: "test".to_string(),
                new_value: vec![1],
            },
        );
        
        queue.cancel(ProposalId(1)).unwrap();
        assert!(queue.get(ProposalId(1)).is_none());
    }
    
    #[test]
    fn test_fast_track() {
        let mut queue = ExecutionQueue::new();
        
        queue.enqueue(
            ProposalId(1),
            100,
            ProposalType::ParameterChange {
                parameter: "test".to_string(),
                new_value: vec![1],
            },
        );
        
        // Original execution block
        let original = queue.get(ProposalId(1)).unwrap().execution_block;
        assert_eq!(original, 14500); // 100 + 14400 (standard)
        
        // Fast track
        queue.fast_track(ProposalId(1), 200).unwrap();
        
        let new_exec_block = queue.get(ProposalId(1)).unwrap().execution_block;
        assert_eq!(new_exec_block, 2000); // 200 + 1800 (fast track)
    }
    
    #[test]
    fn test_get_executable() {
        let mut queue = ExecutionQueue::new();
        
        queue.enqueue(
            ProposalId(1),
            100,
            ProposalType::TreasurySpending {
                recipient: [1u8; 33],
                amount: 1000,
                reason: "Test".to_string(),
            },
        );
        
        queue.enqueue(
            ProposalId(2),
            100,
            ProposalType::ParameterChange {
                parameter: "test".to_string(),
                new_value: vec![1],
            },
        );
        
        // At block 2000, only proposal 1 is executable (fast track)
        let executable = queue.get_executable(2000);
        assert_eq!(executable.len(), 1);
        assert_eq!(executable[0].0, 1);
        
        // At block 15000, both are executable
        let executable = queue.get_executable(15000);
        assert_eq!(executable.len(), 2);
    }
}
