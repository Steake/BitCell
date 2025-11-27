//! Account model

use serde::{Deserialize, Serialize};

/// Account state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub balance: u64,
    pub nonce: u64,
}

impl Account {
    pub fn new(balance: u64) -> Self {
        Self { balance, nonce: 0 }
    }

    pub fn transfer(&mut self, amount: u64) -> bool {
        if self.balance >= amount {
            self.balance -= amount;
            self.nonce += 1;
            true
        } else {
            false
        }
    }

    pub fn receive(&mut self, amount: u64) {
        self.balance += amount;
    }
}

/// Account state collection
pub type AccountState = std::collections::HashMap<[u8; 33], Account>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_transfer() {
        let mut account = Account::new(1000);
        assert!(account.transfer(500));
        assert_eq!(account.balance, 500);
        assert_eq!(account.nonce, 1);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut account = Account::new(100);
        assert!(!account.transfer(200));
        assert_eq!(account.balance, 100);
    }
}
