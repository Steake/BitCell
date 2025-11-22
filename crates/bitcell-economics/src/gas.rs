//! Gas Pricing System (EIP-1559 style)

use crate::params::*;
use serde::{Deserialize, Serialize};

/// Base fee tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseFee {
    current: u64,
}

impl BaseFee {
    pub fn new(initial: u64) -> Self {
        Self { current: initial }
    }
    
    /// Update base fee based on gas usage
    pub fn update(&mut self, gas_used: u64, target_gas: u64) {
        if gas_used > target_gas {
            // Increase base fee
            let delta = self.current * (gas_used - target_gas) / target_gas / BASE_FEE_MAX_CHANGE_DENOMINATOR;
            self.current += delta.max(1);
        } else if gas_used < target_gas {
            // Decrease base fee
            let delta = self.current * (target_gas - gas_used) / target_gas / BASE_FEE_MAX_CHANGE_DENOMINATOR;
            self.current = self.current.saturating_sub(delta);
        }
    }
    
    pub fn current(&self) -> u64 {
        self.current
    }
}

/// Gas price calculator
#[derive(Debug, Clone)]
pub struct GasPrice {
    base_fee: u64,
    priority_fee: u64,
}

impl GasPrice {
    pub fn new(base_fee: u64, priority_fee: u64) -> Self {
        Self {
            base_fee,
            priority_fee,
        }
    }
    
    pub fn total(&self) -> u64 {
        self.base_fee + self.priority_fee
    }
    
    pub fn base_fee(&self) -> u64 {
        self.base_fee
    }
    
    pub fn priority_fee(&self) -> u64 {
        self.priority_fee
    }
}

/// Calculate total gas cost
pub fn calculate_gas_cost(gas_used: u64, base_fee: u64, is_private: bool) -> u64 {
    let multiplier = if is_private {
        PRIVATE_CONTRACT_MULTIPLIER
    } else {
        1
    };
    gas_used * base_fee * multiplier
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_fee_increase() {
        let mut base_fee = BaseFee::new(1000);
        
        // Block is over-full
        base_fee.update(20_000_000, TARGET_GAS_PER_BLOCK);
        
        // Base fee should increase
        assert!(base_fee.current() > 1000);
    }

    #[test]
    fn test_base_fee_decrease() {
        let mut base_fee = BaseFee::new(1000);
        
        // Block is under-full
        base_fee.update(10_000_000, TARGET_GAS_PER_BLOCK);
        
        // Base fee should decrease
        assert!(base_fee.current() < 1000);
    }

    #[test]
    fn test_gas_price() {
        let price = GasPrice::new(100, 20);
        assert_eq!(price.total(), 120);
        assert_eq!(price.base_fee(), 100);
        assert_eq!(price.priority_fee(), 20);
    }

    #[test]
    fn test_privacy_multiplier() {
        let base_fee = 100;
        let gas = 1000;
        
        let cost_public = calculate_gas_cost(gas, base_fee, false);
        let cost_private = calculate_gas_cost(gas, base_fee, true);
        
        assert_eq!(cost_private, cost_public * 2);
    }
}
