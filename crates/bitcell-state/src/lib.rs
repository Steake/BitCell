//! State management for BitCell
//!
//! Implements:
//! - Account model (balance, nonce)
//! - Bond management
//! - State Merkle tree
//! - Nullifier set
//! - Persistent storage with RocksDB

pub mod account;
pub mod bonds;
pub mod storage;

pub use account::{Account, AccountState};
pub use bonds::{BondState, BondStatus};
pub use storage::{StorageManager, PruningStats};

use bitcell_crypto::Hash256;
use std::collections::HashMap;
use std::sync::Arc;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Account not found")]
    AccountNotFound,
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Invalid bond")]
    InvalidBond,

    #[error("Balance overflow")]
    BalanceOverflow,

    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Global state manager
pub struct StateManager {
    /// Account states (in-memory cache)
    pub accounts: HashMap<[u8; 33], Account>,
    
    /// Bond states (in-memory cache)
    pub bonds: HashMap<[u8; 33], BondState>,
    
    /// State root
    pub state_root: Hash256,
    
    /// Optional persistent storage backend
    storage: Option<Arc<StorageManager>>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            bonds: HashMap::new(),
            state_root: Hash256::zero(),
            storage: None,
        }
    }
    
    /// Create StateManager with persistent storage
    pub fn with_storage(storage: Arc<StorageManager>) -> Result<Self> {
        let mut manager = Self {
            accounts: HashMap::new(),
            bonds: HashMap::new(),
            state_root: Hash256::zero(),
            storage: Some(storage),
        };
        
        // Load existing state from storage if available
        // This is a simplified version - production would iterate all accounts
        manager.recompute_root();
        Ok(manager)
    }

    /// Get account (returns reference to cached value)
    /// 
    /// Note: This only checks the in-memory cache. For guaranteed up-to-date values
    /// that may exist only in storage, use get_account_owned() instead.
    pub fn get_account(&self, pubkey: &[u8; 33]) -> Option<&Account> {
        self.accounts.get(pubkey)
    }
    
    /// Get account with storage fallback (returns owned value)
    /// 
    /// This method checks both the in-memory cache and storage backend,
    /// ensuring that persisted state is accessible even if not yet cached.
    pub fn get_account_owned(&self, pubkey: &[u8; 33]) -> Option<Account> {
        // Check in-memory cache first
        if let Some(account) = self.accounts.get(pubkey) {
            return Some(account.clone());
        }
        
        // Fallback to storage if available
        if let Some(storage) = &self.storage {
            if let Ok(Some(account)) = storage.get_account(pubkey) {
                if tracing::enabled!(tracing::Level::TRACE) {
                    tracing::trace!(
                        pubkey = %hex::encode(&pubkey),
                        "Loaded account from storage (cache miss)"
                    );
                }
                return Some(account);
            }
        }
        
        None
    }

    /// Create or update account
    /// 
    /// Updates the in-memory cache and persists to storage if available.
    /// Storage errors are logged but do not prevent the operation from succeeding
    /// in memory (eventual consistency model).
    pub fn update_account(&mut self, pubkey: [u8; 33], account: Account) {
        self.accounts.insert(pubkey, account.clone());


        // Persist to storage if available
        if let Some(storage) = &self.storage {
            if let Err(e) = storage.store_account(&pubkey, &account) {
                tracing::error!(
                    pubkey = %hex::encode(&pubkey),
                    error = %e,
                    "Failed to persist account to storage. State may be inconsistent on restart."
                );
            }
        }
        
        self.recompute_root();
    }

    /// Get bond state (returns reference to cached value)
    /// 
    /// Note: This only checks the in-memory cache. For guaranteed up-to-date values
    /// that may exist only in storage, use get_bond_owned() instead.
    pub fn get_bond(&self, pubkey: &[u8; 33]) -> Option<&BondState> {
        self.bonds.get(pubkey)
    }
    
    /// Get bond state with storage fallback (returns owned value)
    /// 
    /// This method checks both the in-memory cache and storage backend,
    /// ensuring that persisted state is accessible even if not yet cached.
    pub fn get_bond_owned(&self, pubkey: &[u8; 33]) -> Option<BondState> {
        // Check in-memory cache first
        if let Some(bond) = self.bonds.get(pubkey) {
            return Some(bond.clone());
        }
        
        // Fallback to storage if available
        if let Some(storage) = &self.storage {
            if let Ok(Some(bond)) = storage.get_bond(pubkey) {
                if tracing::enabled!(tracing::Level::TRACE) {
                    tracing::trace!(
                        pubkey = %hex::encode(&pubkey),
                        "Loaded bond from storage (cache miss)"
                    );
                }
                return Some(bond);
            }
        }
        
        None
    }

    /// Update bond state
    ///
    /// Updates the in-memory cache and persists to storage if available.
    /// Storage errors are logged but do not prevent the operation from succeeding
    /// in memory (eventual consistency model).
    pub fn update_bond(&mut self, pubkey: [u8; 33], bond: BondState) {
        self.bonds.insert(pubkey, bond.clone());


        // Persist to storage if available
        if let Some(storage) = &self.storage {
            if let Err(e) = storage.store_bond(&pubkey, &bond) {
                tracing::error!(
                    pubkey = %hex::encode(&pubkey),
                    error = %e,
                    "Failed to persist bond to storage. State may be inconsistent on restart."
                );
            }
        }
        
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
            .ok_or(Error::BalanceOverflow)?;
        
        tracing::debug!(
            pubkey = %hex::encode(&pubkey),
            amount = amount,
            new_balance = account.balance,
            "Credited account"
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
    use tempfile::TempDir;

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

    #[test]
    fn test_state_manager_with_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(StorageManager::new(temp_dir.path()).unwrap());
        let pubkey = [1u8; 33];
        
        // Create state manager with storage and add an account
        {
            let mut sm = StateManager::with_storage(storage.clone()).unwrap();
            let account = Account {
                balance: 1000,
                nonce: 5,
            };
            sm.update_account(pubkey, account);
        }
        
        // Create new state manager with same storage and verify persistence
        {
            let sm = StateManager::with_storage(storage).unwrap();
            let retrieved = sm.get_account_owned(&pubkey).unwrap();
            assert_eq!(retrieved.balance, 1000);
            assert_eq!(retrieved.nonce, 5);
        }
    }

    #[test]
    fn test_bond_persistence_with_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(StorageManager::new(temp_dir.path()).unwrap());
        let miner_id = [42u8; 33];
        
        // Create state manager with storage and add a bond
        {
            let mut sm = StateManager::with_storage(storage.clone()).unwrap();
            let bond = BondState {
                amount: 5000,
                status: BondStatus::Active,
                locked_epoch: 10,
            };
            sm.update_bond(miner_id, bond);
        }
        
        // Create new state manager with same storage and verify persistence
        {
            let sm = StateManager::with_storage(storage).unwrap();
            let retrieved = sm.get_bond_owned(&miner_id).unwrap();
            assert_eq!(retrieved.amount, 5000);
            assert_eq!(retrieved.locked_epoch, 10);
            assert!(retrieved.is_active());
        }
    }

    #[test]
    fn test_state_manager_get_or_create_account() {
        let mut sm = StateManager::new();
        let pubkey = [3u8; 33];
        
        // Account doesn't exist yet
        assert!(sm.get_account(&pubkey).is_none());
        assert!(sm.get_account_owned(&pubkey).is_none());
        
        // Create account
        let account = Account {
            balance: 500,
            nonce: 0,
        };
        sm.update_account(pubkey, account);
        
        // Now it exists
        assert!(sm.get_account(&pubkey).is_some());
        assert_eq!(sm.get_account_owned(&pubkey).unwrap().balance, 500);
    }
}
