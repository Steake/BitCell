///! Transaction pool (mempool) for pending transactions

use bitcell_consensus::Transaction;
use bitcell_crypto::Hash256;
use std::collections::{HashMap, BTreeSet};
use std::sync::{Arc, RwLock};

/// Transaction with priority score for ordering
#[derive(Debug, Clone)]
struct PendingTransaction {
    tx: Transaction,
    received_at: u64,
    priority: u64, // gas_price for now
}

impl PartialEq for PendingTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.tx.hash() == other.tx.hash()
    }
}

impl Eq for PendingTransaction {}

impl PartialOrd for PendingTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PendingTransaction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then older first
        other.priority.cmp(&self.priority)
            .then(self.received_at.cmp(&other.received_at))
    }
}

/// Transaction pool
#[derive(Clone)]
pub struct TransactionPool {
    /// Pending transactions ordered by priority
    pending: Arc<RwLock<BTreeSet<PendingTransaction>>>,
    
    /// Transaction lookup by hash
    tx_map: Arc<RwLock<HashMap<Hash256, Transaction>>>,
    
    /// Maximum pool size
    max_size: usize,
}

impl TransactionPool {
    /// Create a new transaction pool
    pub fn new(max_size: usize) -> Self {
        Self {
            pending: Arc::new(RwLock::new(BTreeSet::new())),
            tx_map: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        }
    }
    
    /// Add a transaction to the pool
    pub fn add_transaction(&self, tx: Transaction) -> Result<(), String> {
        let tx_hash = tx.hash();
        
        // Check if already in pool
        {
            let tx_map = self.tx_map.read().unwrap();
            if tx_map.contains_key(&tx_hash) {
                return Err("Transaction already in pool".to_string());
            }
        }
        
        // Check pool size
        {
            let pending = self.pending.read().unwrap();
            if pending.len() >= self.max_size {
                return Err("Transaction pool full".to_string());
            }
        }
        
        // Create pending transaction
        let pending_tx = PendingTransaction {
            tx: tx.clone(),
            received_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            priority: tx.gas_price,
        };
        
        // Add to pool
        {
            let mut pending = self.pending.write().unwrap();
            pending.insert(pending_tx);
        }
        {
            let mut tx_map = self.tx_map.write().unwrap();
            tx_map.insert(tx_hash, tx);
        }
        
        Ok(())
    }
    
    /// Get top N transactions for block inclusion
    pub fn get_transactions(&self, count: usize) -> Vec<Transaction> {
        let pending = self.pending.read().unwrap();
        pending.iter()
            .take(count)
            .map(|ptx| ptx.tx.clone())
            .collect()
    }
    
    /// Remove transactions (after they've been included in a block)
    pub fn remove_transactions(&self, tx_hashes: &[Hash256]) {
        let mut pending = self.pending.write().unwrap();
        let mut tx_map = self.tx_map.write().unwrap();
        
        for hash in tx_hashes {
            if tx_map.remove(hash).is_some() {
                // Remove from pending set
                pending.retain(|ptx| ptx.tx.hash() != *hash);
            }
        }
    }
    
    /// Get number of pending transactions
    pub fn pending_count(&self) -> usize {
        self.pending.read().unwrap().len()
    }
    
    /// Clear all transactions
    pub fn clear(&self) {
        let mut pending = self.pending.write().unwrap();
        let mut tx_map = self.tx_map.write().unwrap();
        pending.clear();
        tx_map.clear();
    }
}

impl Default for TransactionPool {
    fn default() -> Self {
        Self::new(10000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcell_crypto::SecretKey;
    
    fn create_test_tx(nonce: u64, gas_price: u64) -> Transaction {
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        
        Transaction {
            nonce,
            from: pk,
            to: pk,
            amount: 100,
            gas_limit: 21000,
            gas_price,
            data: vec![],
            signature: sk.sign(b"test"),
        }
    }
    
    #[test]
    fn test_add_transaction() {
        let pool = TransactionPool::new(100);
        let tx = create_test_tx(0, 10);
        
        assert!(pool.add_transaction(tx).is_ok());
        assert_eq!(pool.pending_count(), 1);
    }
    
    #[test]
    fn test_get_transactions_by_priority() {
        let pool = TransactionPool::new(100);
        
        // Add transactions with different gas prices
        let tx1 = create_test_tx(0, 10);
        let tx2 = create_test_tx(1, 30);
        let tx3 = create_test_tx(2, 20);
        
        pool.add_transaction(tx1).unwrap();
        pool.add_transaction(tx2.clone()).unwrap();
        pool.add_transaction(tx3).unwrap();
        
        // Higher gas price should come first
        let txs = pool.get_transactions(2);
        assert_eq!(txs.len(), 2);
        assert_eq!(txs[0].gas_price, 30);
    }
    
    #[test]
    fn test_remove_transactions() {
        let pool = TransactionPool::new(100);
        let tx = create_test_tx(0, 10);
        let tx_hash = tx.hash();
        
        pool.add_transaction(tx).unwrap();
        assert_eq!(pool.pending_count(), 1);
        
        pool.remove_transactions(&[tx_hash]);
        assert_eq!(pool.pending_count(), 0);
    }
}
