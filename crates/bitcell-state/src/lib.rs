//! State management for BitCell
//!
//! Implements:
//! - Account model (balance, nonce)
//! - Bond management
//! - State Merkle tree
//! - Nullifier set

pub mod account;
pub mod bonds;
pub mod storage;

pub use account::{Account, AccountState};
pub use bonds::{BondState, BondStatus};

use bitcell_crypto::Hash256;
use std::collections::HashMap;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Account not found")]
    AccountNotFound,
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Invalid bond")]
    InvalidBond,
}

/// Global state manager
pub struct StateManager {
    /// Account states
    pub accounts: HashMap<[u8; 33], Account>,
    
    /// Bond states
    pub bonds: HashMap<[u8; 33], BondState>,
    
    /// State root
    pub state_root: Hash256,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            bonds: HashMap::new(),
            state_root: Hash256::zero(),
        }
    }

    /// Get account
    pub fn get_account(&self, pubkey: &[u8; 33]) -> Option<&Account> {
        self.accounts.get(pubkey)
    }

    /// Create or update account
    pub fn update_account(&mut self, pubkey: [u8; 33], account: Account) {
        self.accounts.insert(pubkey, account);
        self.recompute_root();
    }

    /// Get bond state
    pub fn get_bond(&self, pubkey: &[u8; 33]) -> Option<&BondState> {
        self.bonds.get(pubkey)
    }

    /// Update bond state
    pub fn update_bond(&mut self, pubkey: [u8; 33], bond: BondState) {
        self.bonds.insert(pubkey, bond);
        self.recompute_root();
    }

    /// Recompute state root (simplified)
    fn recompute_root(&mut self) {
        // In production: Merkle tree update
        // For v0.1: Simple hash of all data
        let mut data = Vec::new();
        for (k, v) in &self.accounts {
            data.extend_from_slice(k);
            data.extend_from_slice(&v.balance.to_le_bytes());
        }
        self.state_root = Hash256::hash(&data);
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_manager() {
        let mut sm = StateManager::new();
        let pubkey = [1u8; 33];
        
        let account = Account {
            balance: 1000,
            nonce: 0,
        };
        
        sm.update_account(pubkey, account);
        
        let retrieved = sm.get_account(&pubkey).unwrap();
        assert_eq!(retrieved.balance, 1000);
    }
}
