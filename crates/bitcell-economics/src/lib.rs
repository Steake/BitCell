//! # BitCell Economics
//!
//! Reward distribution, gas pricing, and treasury management.

mod rewards;
mod gas;
mod treasury;

pub use rewards::{RewardDistribution, RewardSchedule, calculate_block_reward};
pub use gas::{GasPrice, BaseFee, calculate_gas_cost};
pub use treasury::Treasury;

/// Economic parameters
pub mod params {
    /// Initial block subsidy
    pub const INITIAL_SUBSIDY: u64 = 50_000_000_000; // 50 tokens
    
    /// Halving interval (blocks)
    pub const HALVING_INTERVAL: u64 = 210_000;
    
    /// Reward split: 60% winner, 30% participants, 10% treasury
    pub const WINNER_SHARE: u64 = 60;
    pub const PARTICIPANT_SHARE: u64 = 30;
    pub const TREASURY_SHARE: u64 = 10;
    
    /// Base fee parameters (EIP-1559 style)
    pub const TARGET_GAS_PER_BLOCK: u64 = 15_000_000;
    pub const BASE_FEE_MAX_CHANGE_DENOMINATOR: u64 = 8;
    
    /// Privacy multiplier
    pub const PRIVATE_CONTRACT_MULTIPLIER: u64 = 2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_calculation() {
        let height = 0;
        let reward = calculate_block_reward(height);
        assert_eq!(reward, params::INITIAL_SUBSIDY);
        
        // After first halving
        let reward_after_halving = calculate_block_reward(params::HALVING_INTERVAL);
        assert_eq!(reward_after_halving, params::INITIAL_SUBSIDY / 2);
    }

    #[test]
    fn test_reward_distribution() {
        let total_reward = 100_000;
        let dist = RewardDistribution::new(total_reward, 10); // 10 participants
        
        assert_eq!(dist.winner_amount(), 60_000);
        assert_eq!(dist.treasury_amount(), 10_000);
        assert_eq!(dist.total_participant_pool(), 30_000);
    }

    #[test]
    fn test_gas_pricing() {
        let base_fee = 1000;
        let gas_used = 100;
        
        let cost = calculate_gas_cost(gas_used, base_fee, false);
        assert_eq!(cost, 100_000);
        
        // With privacy multiplier
        let cost_private = calculate_gas_cost(gas_used, base_fee, true);
        assert_eq!(cost_private, 200_000);
    }
}

