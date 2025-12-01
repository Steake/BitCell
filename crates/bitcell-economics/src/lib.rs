//! # BitCell Economics
//!
//! Reward distribution, gas pricing, and treasury management.

pub mod constants;
mod rewards;
mod gas;
mod treasury;

pub use constants::*;
pub use rewards::{RewardDistribution, RewardSchedule, calculate_block_reward};
pub use gas::{GasPrice, BaseFee, calculate_gas_cost};
pub use treasury::Treasury;

/// Legacy params module - use `constants` instead
#[deprecated(since = "0.1.0", note = "Use constants module instead")]
pub mod params {
    pub use crate::constants::*;
    
    /// Initial block subsidy (deprecated - use INITIAL_BLOCK_REWARD)
    pub const INITIAL_SUBSIDY: u64 = crate::constants::INITIAL_BLOCK_REWARD;
    
    /// Reward split percentages
    pub const WINNER_SHARE: u64 = crate::constants::WINNER_SHARE_PCT;
    pub const PARTICIPANT_SHARE: u64 = crate::constants::PARTICIPANT_SHARE_PCT;
    pub const TREASURY_SHARE: u64 = crate::constants::TREASURY_SHARE_PCT;
    
    /// Base fee parameters (EIP-1559 style)
    pub const TARGET_GAS_PER_BLOCK: u64 = crate::constants::TARGET_GAS_PER_BLOCK;
    pub const BASE_FEE_MAX_CHANGE_DENOMINATOR: u64 = crate::constants::BASE_FEE_MAX_CHANGE_DENOMINATOR;
    
    /// Privacy multiplier
    pub const PRIVATE_CONTRACT_MULTIPLIER: u64 = crate::constants::PRIVACY_GAS_MULTIPLIER;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_calculation() {
        let height = 0;
        let reward = calculate_block_reward(height);
        assert_eq!(reward, INITIAL_BLOCK_REWARD);
        
        // After first halving
        let reward_after_halving = calculate_block_reward(HALVING_INTERVAL);
        assert_eq!(reward_after_halving, INITIAL_BLOCK_REWARD / 2);
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

