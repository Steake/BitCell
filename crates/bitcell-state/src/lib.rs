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

    /// Recompute state root using Merkle tree
    fn recompute_root(&mut self) {
        // Build Merkle tree from account data
        let mut leaves = Vec::new();
        
        for (pubkey, account) in &self.accounts {
            // Create leaf: hash(pubkey || balance || nonce)
            let mut data = Vec::new();
            data.extend_from_slice(pubkey);
            data.extend_from_slice(&account.balance.to_le_bytes());
            data.extend_from_slice(&account.nonce.to_le_bytes());
            leaves.push(Hash256::hash(&data));
        }
        
        // If no accounts, use zero hash
        if leaves.is_empty() {
            self.state_root = Hash256::zero();
            return;
        }
        
        // Build Merkle tree and get root
        let tree = bitcell_crypto::MerkleTree::new(leaves);
        self.state_root = tree.root();
    }
    
    /// Apply a transaction (returns updated state root)
    pub fn apply_transaction(
        &mut self,
        from: [u8; 33],
        to: [u8; 33],
        amount: u64,
        nonce: u64,
    ) -> Result<Hash256> {
        // Get sender account
        let from_account = self.accounts.get(&from)
            .ok_or(Error::AccountNotFound)?;
        
        // Verify nonce
        if from_account.nonce != nonce {
            return Err(Error::InvalidBond); // Reusing error type
        }
        
        // Verify balance
        if from_account.balance < amount {
            return Err(Error::InsufficientBalance);
        }
        
        // Update sender
        let mut updated_from = from_account.clone();
        updated_from.balance -= amount;
        updated_from.nonce += 1;
        self.accounts.insert(from, updated_from);
        
        // Update receiver (create if doesn't exist)
        let mut to_account = self.accounts.get(&to)
            .cloned()
            .unwrap_or(Account { balance: 0, nonce: 0 });
        to_account.balance += amount;
        self.accounts.insert(to, to_account);
        
        // Recompute and return new state root
        self.recompute_root();
        Ok(self.state_root)
    }

    /// Credit an account (minting/coinbase)
    /// Returns the new state root on success, or an error if overflow would occur.
    /// Note: This method should only be called by blockchain core during block processing.
    pub fn credit_account(&mut self, pubkey: [u8; 33], amount: u64) -> Result<Hash256> {
        let mut account = self.accounts.get(&pubkey)
            .cloned()
            .unwrap_or(Account { balance: 0, nonce: 0 });
        
        account.balance = account.balance.checked_add(amount)
            .ok_or(Error::InsufficientBalance)?; // Reuse error type for overflow
        
        tracing::debug!(
            "Credited account {:?} with {} units (new balance: {})",
            hex::encode(&pubkey[..8]), // Log first 8 bytes of pubkey
            amount,
            account.balance
        );
        
        self.accounts.insert(pubkey, account);
        
        self.recompute_root();
        Ok(self.state_root)
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
