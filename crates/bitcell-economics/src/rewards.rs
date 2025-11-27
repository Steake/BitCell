//! Reward Distribution System

use crate::params::*;
use serde::{Deserialize, Serialize};

/// Calculate block reward based on height
pub fn calculate_block_reward(height: u64) -> u64 {
    let halvings = height / HALVING_INTERVAL;
    if halvings >= 64 {
        return 0; // No more rewards after 64 halvings
    }
    INITIAL_SUBSIDY >> halvings
}

/// Reward distribution for a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardDistribution {
    pub total_reward: u64,
    pub winner_amount: u64,
    pub participant_pool: u64,
    pub treasury_amount: u64,
    pub num_participants: usize,
}

impl RewardDistribution {
    /// Create new reward distribution
    pub fn new(total_reward: u64, num_participants: usize) -> Self {
        let winner_amount = (total_reward * WINNER_SHARE) / 100;
        let participant_pool = (total_reward * PARTICIPANT_SHARE) / 100;
        let treasury_amount = (total_reward * TREASURY_SHARE) / 100;
        
        Self {
            total_reward,
            winner_amount,
            participant_pool,
            treasury_amount,
            num_participants,
        }
    }
    
    /// Get winner payout
    pub fn winner_amount(&self) -> u64 {
        self.winner_amount
    }
    
    /// Get treasury allocation
    pub fn treasury_amount(&self) -> u64 {
        self.treasury_amount
    }
    
    /// Get total participant pool
    pub fn total_participant_pool(&self) -> u64 {
        self.participant_pool
    }
    
    /// Calculate payout for a participant based on round reached
    /// Later rounds get exponentially more
    pub fn participant_payout(&self, rounds_reached: u32) -> u64 {
        if self.num_participants <= 1 {
            return 0;
        }
        
        // Weight by 2^rounds_reached
        let weight = 1u64 << rounds_reached;
        
        // Total weight sum: sum of 2^i for all participants
        // For simplicity, assume equal distribution for now
        self.participant_pool / self.num_participants as u64
    }
}

/// Reward schedule tracking
#[derive(Debug, Clone)]
pub struct RewardSchedule {
    current_height: u64,
}

impl RewardSchedule {
    pub fn new() -> Self {
        Self { current_height: 0 }
    }
    
    pub fn current_reward(&self) -> u64 {
        calculate_block_reward(self.current_height)
    }
    
    pub fn advance(&mut self) {
        self.current_height += 1;
    }
    
    pub fn next_halving_height(&self) -> u64 {
        ((self.current_height / HALVING_INTERVAL) + 1) * HALVING_INTERVAL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_halving_schedule() {
        assert_eq!(calculate_block_reward(0), INITIAL_SUBSIDY);
        assert_eq!(calculate_block_reward(HALVING_INTERVAL - 1), INITIAL_SUBSIDY);
        assert_eq!(calculate_block_reward(HALVING_INTERVAL), INITIAL_SUBSIDY / 2);
        assert_eq!(calculate_block_reward(HALVING_INTERVAL * 2), INITIAL_SUBSIDY / 4);
    }

    #[test]
    fn test_participant_payouts() {
        let dist = RewardDistribution::new(1_000_000, 4);
        
        // Each participant gets 1/4 of the 30% pool
        let payout = dist.participant_payout(0);
        assert_eq!(payout, 75_000); // 300_000 / 4
    }

    #[test]
    fn test_reward_schedule() {
        let mut schedule = RewardSchedule::new();
        
        assert_eq!(schedule.current_reward(), INITIAL_SUBSIDY);
        assert_eq!(schedule.next_halving_height(), HALVING_INTERVAL);
        
        schedule.current_height = HALVING_INTERVAL;
        assert_eq!(schedule.current_reward(), INITIAL_SUBSIDY / 2);
    }
}
