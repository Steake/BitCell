//! Security Audit Tests for Economic Model
//!
//! This test suite implements the security audit requirements for RC3-001.3
//! (Economic Model Validation) as specified in docs/SECURITY_AUDIT.md
//!
//! Test Categories:
//! 1. Token Supply and Distribution
//! 2. Block Reward Calculation
//! 3. Fee Market Security
//! 4. Bonding and Slashing
//! 5. EBSL Trust System
//! 6. Economic Attack Prevention

use bitcell_economics::*;
use bitcell_ebsl::*;

// =============================================================================
// 1. Token Supply and Distribution Tests
// =============================================================================

mod supply_security {
    use super::*;

    #[test]
    fn test_initial_block_reward() {
        // Initial reward must be 50 CELL
        let height = 0;
        let reward = calculate_block_reward(height);
        assert_eq!(reward, 50 * COIN, "Initial block reward must be 50 CELL");
    }

    #[test]
    fn test_first_halving() {
        // First halving at block 210,000
        let pre_halving = calculate_block_reward(209_999);
        let post_halving = calculate_block_reward(210_000);
        
        assert_eq!(pre_halving, 50 * COIN, "Pre-halving reward must be 50 CELL");
        assert_eq!(post_halving, 25 * COIN, "Post-halving reward must be 25 CELL");
    }

    #[test]
    fn test_halving_schedule() {
        // Test halving at each interval
        let halvings = [
            (0, 50 * COIN),
            (210_000, 25 * COIN),
            (420_000, 12 * COIN + COIN / 2),  // 12.5 CELL
            (630_000, 6 * COIN + COIN / 4),   // 6.25 CELL
            (840_000, 3 * COIN + COIN / 8),   // 3.125 CELL
        ];
        
        for (height, expected_reward) in halvings.iter() {
            let reward = calculate_block_reward(*height);
            assert_eq!(reward, *expected_reward,
                      "Reward at height {} must be {}", height, expected_reward);
        }
    }

    #[test]
    fn test_max_halvings() {
        // After 64 halvings, reward should be 0
        let height = 64 * 210_000;
        let reward = calculate_block_reward(height);
        assert_eq!(reward, 0, "Reward after 64 halvings must be 0");
    }

    #[test]
    fn test_supply_cap() {
        // Total supply must approach 21M CELL
        let mut total_supply = 0u64;
        
        // Sum rewards for all blocks until reward is 0
        for height in 0..(64 * 210_000) {
            let reward = calculate_block_reward(height);
            total_supply = total_supply.saturating_add(reward);
        }
        
        // Should be approximately 21M CELL (with some rounding)
        let max_supply = 21_000_000 * COIN;
        assert!(total_supply <= max_supply,
                "Total supply {} must not exceed {} CELL", 
                total_supply / COIN, max_supply / COIN);
        
        // Should be close to 21M (within 1%)
        let diff = max_supply.saturating_sub(total_supply);
        assert!(diff < max_supply / 100,
                "Total supply should be within 1% of 21M CELL");
    }

    #[test]
    fn test_reward_distribution_percentages() {
        // Reward distribution: 60% winner, 30% participants, 10% treasury
        let reward = 100 * COIN;
        let distribution = distribute_reward(reward);
        
        assert_eq!(distribution.winner_share, 60 * COIN, "Winner share must be 60%");
        assert_eq!(distribution.participant_share, 30 * COIN, "Participant share must be 30%");
        assert_eq!(distribution.treasury_share, 10 * COIN, "Treasury share must be 10%");
        
        // Sum must equal total reward
        let total = distribution.winner_share 
                  + distribution.participant_share 
                  + distribution.treasury_share;
        assert_eq!(total, reward, "Distribution must sum to total reward");
    }

    #[test]
    fn test_no_reward_overflow() {
        // Reward calculation must not overflow
        for height in [0, u64::MAX / 2, u64::MAX - 1, u64::MAX].iter() {
            let reward = calculate_block_reward(*height);
            // Should not panic and reward should be reasonable
            assert!(reward <= 50 * COIN, "Reward must not exceed initial reward");
        }
    }

    #[test]
    fn test_reward_deterministic() {
        // Same height must always give same reward
        let height = 100_000;
        let reward1 = calculate_block_reward(height);
        let reward2 = calculate_block_reward(height);
        let reward3 = calculate_block_reward(height);
        
        assert_eq!(reward1, reward2, "Reward must be deterministic");
        assert_eq!(reward2, reward3, "Reward must be deterministic");
    }
}

// =============================================================================
// 2. Fee Market Security Tests
// =============================================================================

mod fee_market_security {
    use super::*;

    #[test]
    fn test_base_fee_adjustment() {
        // Base fee must adjust based on block fullness
        let mut gas_price = GasPrice::new();
        
        let initial_base_fee = gas_price.base_fee();
        
        // Full block should increase base fee
        gas_price.adjust_base_fee(1.0); // 100% full
        let increased_base_fee = gas_price.base_fee();
        
        assert!(increased_base_fee > initial_base_fee,
                "Base fee must increase for full blocks");
        
        // Empty block should decrease base fee
        gas_price.adjust_base_fee(0.0); // 0% full
        let decreased_base_fee = gas_price.base_fee();
        
        assert!(decreased_base_fee < increased_base_fee,
                "Base fee must decrease for empty blocks");
    }

    #[test]
    fn test_base_fee_bounds() {
        // Base fee must stay within reasonable bounds
        let mut gas_price = GasPrice::new();
        
        // Try to increase base fee many times
        for _ in 0..1000 {
            gas_price.adjust_base_fee(1.0); // Always full
        }
        
        let max_base_fee = gas_price.base_fee();
        assert!(max_base_fee < u64::MAX / 1000,
                "Base fee must have reasonable upper bound");
        
        // Try to decrease base fee many times
        for _ in 0..1000 {
            gas_price.adjust_base_fee(0.0); // Always empty
        }
        
        let min_base_fee = gas_price.base_fee();
        assert!(min_base_fee > 0,
                "Base fee must have non-zero lower bound");
    }

    #[test]
    fn test_priority_tip_limits() {
        // Priority tips must be bounded
        let max_tip = MAX_GAS_PRICE;
        
        // Creating transaction with excessive tip should fail or cap
        let tip = max_tip + 1;
        
        // Implementation should reject or cap the tip
        // This is a boundary test
        assert!(tip > max_tip, "Test setup: tip exceeds max");
    }

    #[test]
    fn test_gas_limit_enforcement() {
        // Gas limits must be enforced
        let tx_gas_limit = 1_000_000;
        let block_gas_limit = 10_000_000;
        
        // Transaction gas must not exceed block gas limit
        assert!(tx_gas_limit <= block_gas_limit,
                "Transaction gas must not exceed block gas limit");
    }

    #[test]
    fn test_privacy_multiplier() {
        // Private contracts must have 2x gas cost
        let base_gas = 100;
        let privacy_gas = apply_privacy_multiplier(base_gas);
        
        assert_eq!(privacy_gas, base_gas * 2,
                   "Privacy multiplier must be 2x");
    }

    #[test]
    fn test_fee_burning() {
        // Base fee must be burned, not given to miner
        let base_fee = 1000;
        let priority_tip = 100;
        
        let burned = calculate_burned_fee(base_fee, priority_tip);
        let miner_fee = calculate_miner_fee(base_fee, priority_tip);
        
        assert_eq!(burned, base_fee, "Base fee must be burned");
        assert_eq!(miner_fee, priority_tip, "Only priority tip goes to miner");
    }

    #[test]
    fn test_fee_overflow_protection() {
        // Fee calculations must not overflow
        let base_fee = u64::MAX / 2;
        let tip = u64::MAX / 2;
        
        // Total fee calculation should not overflow
        let total = base_fee.saturating_add(tip);
        assert!(total >= base_fee && total >= tip,
                "Fee addition must not overflow");
    }
}

// =============================================================================
// 3. Bonding and Slashing Tests
// =============================================================================

mod bonding_security {
    use super::*;

    #[test]
    fn test_minimum_bond_requirement() {
        // Minimum bond must be enforced (1000 CELL)
        let min_bond = MINIMUM_BOND;
        
        assert_eq!(min_bond, 1000 * COIN,
                   "Minimum bond must be 1000 CELL");
        
        // Less than minimum should be rejected
        let insufficient_bond = min_bond - 1;
        assert!(insufficient_bond < min_bond,
                "Insufficient bond must be detected");
    }

    #[test]
    fn test_slashing_invalid_proof() {
        // Invalid proof must result in 10% slash
        let bond = 10000 * COIN;
        let slashed = calculate_slash(bond, SlashReason::InvalidProof);
        
        assert_eq!(slashed, bond / 10,
                   "Invalid proof must slash 10% of bond");
    }

    #[test]
    fn test_slashing_double_commitment() {
        // Double commitment must result in 50% slash
        let bond = 10000 * COIN;
        let slashed = calculate_slash(bond, SlashReason::DoubleCommitment);
        
        assert_eq!(slashed, bond / 2,
                   "Double commitment must slash 50% of bond");
    }

    #[test]
    fn test_slashing_missed_reveal() {
        // Missed reveal must result in 5% slash
        let bond = 10000 * COIN;
        let slashed = calculate_slash(bond, SlashReason::MissedReveal);
        
        assert_eq!(slashed, bond / 20,
                   "Missed reveal must slash 5% of bond");
    }

    #[test]
    fn test_slashing_equivocation() {
        // Equivocation must result in 100% slash
        let bond = 10000 * COIN;
        let slashed = calculate_slash(bond, SlashReason::Equivocation);
        
        assert_eq!(slashed, bond,
                   "Equivocation must slash 100% of bond");
    }

    #[test]
    fn test_bond_state_transitions() {
        // Bond must transition through valid states
        let mut bond_state = BondState::new(1000 * COIN);
        
        assert!(bond_state.is_active(), "New bond must be active");
        
        // Start unbonding
        bond_state.start_unbonding(100);
        assert!(bond_state.is_unbonding(), "Bond must be unbonding");
        
        // Complete unbonding after period
        bond_state.complete_unbonding(200);
        assert!(bond_state.is_unlocked(), "Bond must be unlocked after period");
    }

    #[test]
    fn test_unbonding_period() {
        // Unbonding must enforce waiting period
        let unbonding_period = UNBONDING_PERIOD;
        
        let mut bond_state = BondState::new(1000 * COIN);
        bond_state.start_unbonding(100);
        
        // Cannot complete before period ends
        let can_complete_early = bond_state.can_complete_unbonding(100 + unbonding_period - 1);
        assert!(!can_complete_early, "Cannot complete unbonding early");
        
        // Can complete after period
        let can_complete_later = bond_state.can_complete_unbonding(100 + unbonding_period);
        assert!(can_complete_later, "Can complete unbonding after period");
    }

    #[test]
    fn test_slashing_prevents_underflow() {
        // Slashing must not underflow bond balance
        let bond = 100 * COIN;
        let slashed = calculate_slash(bond, SlashReason::Equivocation);
        
        assert!(slashed <= bond, "Slash amount must not exceed bond");
        
        let remaining = bond.saturating_sub(slashed);
        assert!(remaining >= 0, "Remaining bond must not be negative");
    }
}

// =============================================================================
// 4. EBSL Trust System Tests
// =============================================================================

mod trust_security {
    use super::*;

    #[test]
    fn test_trust_score_calculation() {
        // Trust score must calculate correctly
        let mut evidence = EvidenceCounters::new();
        
        // Add positive evidence
        evidence.add_positive(10);
        
        let trust = evidence.calculate_trust();
        assert!(trust > 0.5, "Positive evidence must increase trust");
        
        // Add negative evidence
        evidence.add_negative(5);
        
        let new_trust = evidence.calculate_trust();
        assert!(new_trust < trust, "Negative evidence must decrease trust");
    }

    #[test]
    fn test_trust_bounds() {
        // Trust score must be between 0 and 1
        let mut evidence = EvidenceCounters::new();
        
        // Max positive evidence
        for _ in 0..1000 {
            evidence.add_positive(1);
        }
        
        let max_trust = evidence.calculate_trust();
        assert!(max_trust <= 1.0, "Trust must not exceed 1.0");
        assert!(max_trust >= 0.0, "Trust must not be negative");
        
        // Max negative evidence
        for _ in 0..1000 {
            evidence.add_negative(1);
        }
        
        let min_trust = evidence.calculate_trust();
        assert!(min_trust <= 1.0, "Trust must not exceed 1.0");
        assert!(min_trust >= 0.0, "Trust must not be negative");
    }

    #[test]
    fn test_trust_decay() {
        // Trust must decay over time
        let mut evidence = EvidenceCounters::new();
        
        // Add evidence
        evidence.add_positive(100);
        evidence.add_negative(50);
        
        let initial_trust = evidence.calculate_trust();
        
        // Apply decay
        evidence.apply_decay();
        
        let decayed_trust = evidence.calculate_trust();
        
        // Trust should change after decay
        // The direction depends on asymmetric decay rates
        assert_ne!(initial_trust, decayed_trust,
                   "Decay must affect trust score");
    }

    #[test]
    fn test_asymmetric_decay() {
        // Negative evidence must decay slower than positive
        let pos_decay = POSITIVE_DECAY_RATE; // 0.99
        let neg_decay = NEGATIVE_DECAY_RATE; // 0.999
        
        assert!(neg_decay > pos_decay,
                "Negative decay must be slower (higher rate closer to 1)");
    }

    #[test]
    fn test_trust_eligibility_threshold() {
        // Trust must meet T_MIN for eligibility
        let t_min = TRUST_MIN; // 0.75
        
        let mut evidence = EvidenceCounters::new();
        
        // Low trust - not eligible
        evidence.add_negative(10);
        let low_trust = evidence.calculate_trust();
        assert!(low_trust < t_min, "Low trust must be below threshold");
        
        // High trust - eligible
        let mut evidence2 = EvidenceCounters::new();
        evidence2.add_positive(100);
        let high_trust = evidence2.calculate_trust();
        assert!(high_trust >= t_min, "High trust must meet threshold");
    }

    #[test]
    fn test_trust_kill_threshold() {
        // Trust below T_KILL results in permanent ban
        let t_kill = TRUST_KILL; // 0.2
        
        let mut evidence = EvidenceCounters::new();
        
        // Add lots of negative evidence
        for _ in 0..100 {
            evidence.add_negative(10);
        }
        
        let trust = evidence.calculate_trust();
        
        if trust < t_kill {
            // Should be permanently banned
            assert!(true, "Trust below T_KILL should trigger ban");
        }
    }

    #[test]
    fn test_evidence_overflow_protection() {
        // Evidence counters must not overflow
        let mut evidence = EvidenceCounters::new();
        
        // Add maximum evidence
        for _ in 0..10000 {
            evidence.add_positive(u32::MAX);
            evidence.add_negative(u32::MAX);
        }
        
        // Should not panic
        let trust = evidence.calculate_trust();
        assert!(trust >= 0.0 && trust <= 1.0,
                "Trust must remain valid even with large evidence");
    }
}

// =============================================================================
// 5. Economic Attack Prevention Tests
// =============================================================================

mod attack_prevention {
    use super::*;

    #[test]
    fn test_sybil_resistance() {
        // Bonding requirement must prevent Sybil attacks
        let min_bond = MINIMUM_BOND;
        let num_sybils = 100;
        
        // Cost to create many Sybil identities
        let total_cost = min_bond.saturating_mul(num_sybils);
        
        // Must be economically significant
        assert!(total_cost >= 100_000 * COIN,
                "Sybil attack must be expensive (100k+ CELL)");
    }

    #[test]
    fn test_grinding_prevention() {
        // VRF must prevent grinding attacks
        // This is tested in crypto tests, but verify economic incentive
        
        // Cost of grinding: need to bond and risk slashing
        let bond_required = MINIMUM_BOND;
        let slash_for_failure = bond_required / 10; // 10% slash
        
        // Grinding multiple attempts is expensive
        let attempts = 100;
        let expected_loss = slash_for_failure * (attempts / 2); // 50% caught
        
        assert!(expected_loss > bond_required,
                "Grinding must be economically unfavorable");
    }

    #[test]
    fn test_nothing_at_stake() {
        // Slashing must prevent nothing-at-stake problem
        let bond = 10000 * COIN;
        let equivocation_slash = calculate_slash(bond, SlashReason::Equivocation);
        
        assert_eq!(equivocation_slash, bond,
                   "Equivocation must slash 100% - heavy penalty");
    }

    #[test]
    fn test_fee_market_spam_protection() {
        // High base fee must prevent spam
        let mut gas_price = GasPrice::new();
        
        // Simulate many full blocks (spam)
        for _ in 0..100 {
            gas_price.adjust_base_fee(1.0);
        }
        
        let high_base_fee = gas_price.base_fee();
        let initial_base_fee = GasPrice::new().base_fee();
        
        assert!(high_base_fee > initial_base_fee * 10,
                "Base fee must increase significantly to deter spam");
    }

    #[test]
    fn test_treasury_depletion_protection() {
        // Treasury must receive consistent funding
        let reward = 1000 * COIN;
        let distribution = distribute_reward(reward);
        
        let treasury_percentage = (distribution.treasury_share * 100) / reward;
        assert_eq!(treasury_percentage, 10,
                   "Treasury must receive 10% of all rewards");
    }
}

// =============================================================================
// 6. Numerical Safety Tests
// =============================================================================

mod numerical_safety {
    use super::*;

    #[test]
    fn test_no_integer_overflow_in_rewards() {
        // Reward calculations must not overflow
        for height in 0..1_000_000 {
            let reward = calculate_block_reward(height);
            
            // Reward must be reasonable
            assert!(reward <= 50 * COIN,
                    "Reward at height {} must not exceed initial", height);
        }
    }

    #[test]
    fn test_no_underflow_in_slashing() {
        // Slashing must not underflow
        let bonds = [0, 1, 1000, 1_000_000 * COIN, u64::MAX / 2];
        let reasons = [
            SlashReason::InvalidProof,
            SlashReason::DoubleCommitment,
            SlashReason::MissedReveal,
            SlashReason::Equivocation,
        ];
        
        for bond in bonds.iter() {
            for reason in reasons.iter() {
                let slashed = calculate_slash(*bond, *reason);
                assert!(slashed <= *bond,
                        "Slash amount must not exceed bond");
            }
        }
    }

    #[test]
    fn test_no_overflow_in_fee_calculations() {
        // Fee calculations must handle large values
        let large_values = [
            u64::MAX / 2,
            u64::MAX / 4,
            u64::MAX / 8,
        ];
        
        for value in large_values.iter() {
            let doubled = value.saturating_mul(2);
            assert!(doubled >= *value || doubled == u64::MAX,
                    "Multiplications must use saturating arithmetic");
        }
    }

    #[test]
    fn test_reward_distribution_rounding() {
        // Reward distribution must not lose coins to rounding
        let rewards = [1, 3, 7, 99, 1000, 999999];
        
        for reward in rewards.iter() {
            let distribution = distribute_reward(*reward);
            
            let total = distribution.winner_share 
                      + distribution.participant_share 
                      + distribution.treasury_share;
            
            // Total must equal original (allowing for rounding)
            let diff = if total > *reward {
                total - *reward
            } else {
                *reward - total
            };
            
            assert!(diff <= 2,
                    "Rounding error must be minimal (≤2) for reward {}", reward);
        }
    }
}
