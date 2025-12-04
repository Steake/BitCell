//! Economic Constants
//!
//! All economic parameters for the BitCell protocol centralized in one place.

/// ===== MONETARY POLICY =====

/// 1 Coin = 100,000,000 units (satoshis)
pub const COIN: u64 = 100_000_000;

/// Initial block reward (50 coins)
pub const INITIAL_BLOCK_REWARD: u64 = 50 * COIN;

/// Halving interval (every 210,000 blocks)
pub const HALVING_INTERVAL: u64 = 210_000;

/// Maximum supply (21 million coins)
/// Sum of geometric series: 50 * 210000 * (1 + 1/2 + 1/4 + ... + 1/2^63)
pub const MAX_SUPPLY: u64 = 21_000_000 * COIN;

/// ===== REWARD DISTRIBUTION =====

/// Percentage of block reward to tournament winner
pub const WINNER_SHARE_PCT: u64 = 60;

/// Percentage of block reward to tournament participants
pub const PARTICIPANT_SHARE_PCT: u64 = 30;

/// Percentage of block reward to treasury
pub const TREASURY_SHARE_PCT: u64 = 10;

/// ===== BLOCK TIMING =====

/// Target block time in seconds (10 minutes)
pub const TARGET_BLOCK_TIME_SECS: u64 = 600;

/// Minimum block time in seconds (1 minute)
pub const MIN_BLOCK_TIME_SECS: u64 = 60;

/// Maximum block time in seconds (30 minutes)
pub const MAX_BLOCK_TIME_SECS: u64 = 1800;

/// Development/Testing block time (10 seconds)
pub const DEV_BLOCK_TIME_SECS: u64 = 10;

/// ===== TRANSACTION ECONOMICS =====

/// Default transaction gas limit
pub const DEFAULT_TX_GAS_LIMIT: u64 = 21_000;

/// Default transaction gas price (1 unit per gas)
pub const DEFAULT_GAS_PRICE: u64 = 1;

/// Minimum gas price
pub const MIN_GAS_PRICE: u64 = 1;

/// Base fee (EIP-1559 style)
pub const INITIAL_BASE_FEE: u64 = 1_000;

/// Target gas per block (15M gas)
pub const TARGET_GAS_PER_BLOCK: u64 = 15_000_000;

/// Maximum gas per block (30M gas)
pub const MAX_GAS_PER_BLOCK: u64 = 30_000_000;

/// Base fee max change denominator (12.5% max change per block)
pub const BASE_FEE_MAX_CHANGE_DENOMINATOR: u64 = 8;

/// ===== GAS COSTS =====

/// Gas cost for basic transaction
pub const GAS_TX_BASE: u64 = 21_000;

/// Gas cost per byte of call data
pub const GAS_TX_DATA_NONZERO: u64 = 68;

/// Gas cost per zero byte of call data
pub const GAS_TX_DATA_ZERO: u64 = 4;

/// Privacy multiplier (ZK proofs cost more)
pub const PRIVACY_GAS_MULTIPLIER: u64 = 2;

/// ===== TOURNAMENT ECONOMICS =====

/// Entry deposit for tournaments (prevents spam)
pub const TOURNAMENT_ENTRY_DEPOSIT: u64 = 1 * COIN;

/// Bond period in blocks (how long until you can withdraw)
pub const BOND_PERIOD_BLOCKS: u64 = 2016; // ~2 weeks at 10 min blocks

/// Minimum bond amount
pub const MIN_BOND_AMOUNT: u64 = 100 * COIN;

/// ===== TREASURY =====

/// Treasury minimum balance threshold
pub const TREASURY_MIN_BALANCE: u64 = 1_000_000 * COIN;

/// Maximum treasury withdrawal per period
pub const TREASURY_MAX_WITHDRAWAL_PCT: u64 = 10; // 10% per period

/// ===== PENALTY & SLASHING =====

/// Penalty for missing tournament commitments (% of deposit)
pub const COMMITMENT_MISS_PENALTY_PCT: u64 = 10;

/// Penalty for invalid reveals (% of deposit)
pub const INVALID_REVEAL_PENALTY_PCT: u64 = 25;

/// Slashing for malicious behavior (% of bond)
pub const MALICIOUS_BEHAVIOR_SLASH_PCT: u64 = 100;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coin_denomination() {
        assert_eq!(COIN, 100_000_000);
        assert_eq!(50 * COIN, 5_000_000_000);
    }

    #[test]
    fn test_reward_shares_sum_to_100() {
        assert_eq!(
            WINNER_SHARE_PCT + PARTICIPANT_SHARE_PCT + TREASURY_SHARE_PCT,
            100
        );
    }

    #[test]
    fn test_gas_limits() {
        assert!(DEFAULT_TX_GAS_LIMIT <= TARGET_GAS_PER_BLOCK);
        assert!(TARGET_GAS_PER_BLOCK <= MAX_GAS_PER_BLOCK);
    }

    #[test]
    fn test_block_time_bounds() {
        assert!(MIN_BLOCK_TIME_SECS <= TARGET_BLOCK_TIME_SECS);
        assert!(TARGET_BLOCK_TIME_SECS <= MAX_BLOCK_TIME_SECS);
    }
}
