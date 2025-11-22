//! Bond management

use serde::{Deserialize, Serialize};

/// Bond status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BondStatus {
    Active,
    Unbonding { unlock_epoch: u64 },
    Slashed { amount: u64 },
}

/// Bond state for a miner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondState {
    pub amount: u64,
    pub status: BondStatus,
    pub locked_epoch: u64,
}

impl BondState {
    pub fn new(amount: u64, epoch: u64) -> Self {
        Self {
            amount,
            status: BondStatus::Active,
            locked_epoch: epoch,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, BondStatus::Active)
    }

    pub fn slash(&mut self, slash_amount: u64) {
        self.amount = self.amount.saturating_sub(slash_amount);
        self.status = BondStatus::Slashed { amount: slash_amount };
    }

    pub fn start_unbonding(&mut self, current_epoch: u64, unbonding_period: u64) {
        self.status = BondStatus::Unbonding {
            unlock_epoch: current_epoch + unbonding_period,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bond_creation() {
        let bond = BondState::new(1000, 0);
        assert_eq!(bond.amount, 1000);
        assert!(bond.is_active());
    }

    #[test]
    fn test_slashing() {
        let mut bond = BondState::new(1000, 0);
        bond.slash(500);
        assert_eq!(bond.amount, 500);
        assert!(!bond.is_active());
    }

    #[test]
    fn test_unbonding() {
        let mut bond = BondState::new(1000, 0);
        bond.start_unbonding(10, 100);
        
        if let BondStatus::Unbonding { unlock_epoch } = bond.status {
            assert_eq!(unlock_epoch, 110);
        } else {
            panic!("Expected unbonding status");
        }
    }
}
