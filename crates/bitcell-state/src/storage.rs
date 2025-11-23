/// RocksDB persistent storage layer
/// Provides durable storage for blocks, state, and chain data

use rocksdb::{DB, Options, WriteBatch, IteratorMode};
use std::path::Path;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

use crate::{Account, BondState};

/// Database column families
const CF_BLOCKS: &str = "blocks";
const CF_HEADERS: &str = "headers";
const CF_TRANSACTIONS: &str = "transactions";
const CF_ACCOUNTS: &str = "accounts";
const CF_BONDS: &str = "bonds";
const CF_STATE_ROOTS: &str = "state_roots";
const CF_CHAIN_INDEX: &str = "chain_index";

/// Persistent storage manager
pub struct StorageManager {
    db: Arc<DB>,
}

impl StorageManager {
    /// Open or create a database
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, rocksdb::Error> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        
        let cfs = vec![
            CF_BLOCKS,
            CF_HEADERS,
            CF_TRANSACTIONS,
            CF_ACCOUNTS,
            CF_BONDS,
            CF_STATE_ROOTS,
            CF_CHAIN_INDEX,
        ];
        
        let db = DB::open_cf(&opts, path, cfs)?;
        
        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// Store a block header
    pub fn store_header(&self, height: u64, hash: &[u8], header: &[u8]) -> Result<(), rocksdb::Error> {
        let cf = self.db.cf_handle(CF_HEADERS).unwrap();
        
        let mut batch = WriteBatch::default();
        // Store by height
        batch.put_cf(cf, height.to_be_bytes(), header);
        // Store by hash
        batch.put_cf(cf, hash, header);
        // Update chain index
        let index_cf = self.db.cf_handle(CF_CHAIN_INDEX).unwrap();
        batch.put_cf(index_cf, b"latest_height", height.to_be_bytes());
        batch.put_cf(index_cf, b"latest_hash", hash);
        
        self.db.write(batch)
    }

    /// Store a full block
    pub fn store_block(&self, hash: &[u8], block: &[u8]) -> Result<(), rocksdb::Error> {
        let cf = self.db.cf_handle(CF_BLOCKS).unwrap();
        self.db.put_cf(cf, hash, block)
    }

    /// Get block by hash
    pub fn get_block(&self, hash: &[u8]) -> Result<Option<Vec<u8>>, rocksdb::Error> {
        let cf = self.db.cf_handle(CF_BLOCKS).unwrap();
        self.db.get_cf(cf, hash)
    }

    /// Get header by height
    pub fn get_header_by_height(&self, height: u64) -> Result<Option<Vec<u8>>, rocksdb::Error> {
        let cf = self.db.cf_handle(CF_HEADERS).unwrap();
        self.db.get_cf(cf, height.to_be_bytes())
    }

    /// Get header by hash
    pub fn get_header_by_hash(&self, hash: &[u8]) -> Result<Option<Vec<u8>>, rocksdb::Error> {
        let cf = self.db.cf_handle(CF_HEADERS).unwrap();
        self.db.get_cf(cf, hash)
    }

    /// Get latest chain height
    pub fn get_latest_height(&self) -> Result<Option<u64>, rocksdb::Error> {
        let cf = self.db.cf_handle(CF_CHAIN_INDEX).unwrap();
        if let Some(bytes) = self.db.get_cf(cf, b"latest_height")? {
            let height = u64::from_be_bytes(bytes.as_slice().try_into().unwrap());
            Ok(Some(height))
        } else {
            Ok(None)
        }
    }

    /// Store account state
    pub fn store_account(&self, address: &[u8], account: &Account) -> Result<(), rocksdb::Error> {
        let cf = self.db.cf_handle(CF_ACCOUNTS).unwrap();
        let data = bincode::serialize(account).unwrap();
        self.db.put_cf(cf, address, data)
    }

    /// Get account state
    pub fn get_account(&self, address: &[u8]) -> Result<Option<Account>, rocksdb::Error> {
        let cf = self.db.cf_handle(CF_ACCOUNTS).unwrap();
        if let Some(data) = self.db.get_cf(cf, address)? {
            Ok(bincode::deserialize(&data).ok())
        } else {
            Ok(None)
        }
    }

    /// Store bond state
    pub fn store_bond(&self, miner_id: &[u8], bond: &BondState) -> Result<(), rocksdb::Error> {
        let cf = self.db.cf_handle(CF_BONDS).unwrap();
        let data = bincode::serialize(bond).unwrap();
        self.db.put_cf(cf, miner_id, data)
    }

    /// Get bond state
    pub fn get_bond(&self, miner_id: &[u8]) -> Result<Option<BondState>, rocksdb::Error> {
        let cf = self.db.cf_handle(CF_BONDS).unwrap();
        if let Some(data) = self.db.get_cf(cf, miner_id)? {
            Ok(bincode::deserialize(&data).ok())
        } else {
            Ok(None)
        }
    }

    /// Store state root for a given height
    pub fn store_state_root(&self, height: u64, root: &[u8]) -> Result<(), rocksdb::Error> {
        let cf = self.db.cf_handle(CF_STATE_ROOTS).unwrap();
        self.db.put_cf(cf, height.to_be_bytes(), root)
    }

    /// Get state root for a given height
    pub fn get_state_root(&self, height: u64) -> Result<Option<Vec<u8>>, rocksdb::Error> {
        let cf = self.db.cf_handle(CF_STATE_ROOTS).unwrap();
        self.db.get_cf(cf, height.to_be_bytes())
    }

    /// Prune old blocks (keep last N blocks)
    pub fn prune_old_blocks(&self, keep_last: u64) -> Result<(), rocksdb::Error> {
        let latest = self.get_latest_height()?.unwrap_or(0);
        if latest <= keep_last {
            return Ok(());
        }
        
        let prune_until = latest - keep_last;
        let cf = self.db.cf_handle(CF_BLOCKS).unwrap();
        
        // This is a simplified version - in production would iterate and delete
        for height in 0..prune_until {
            if let Some(header_data) = self.get_header_by_height(height)? {
                // Extract hash and delete block
                // (Simplified - would need proper header deserialization)
                let _ = header_data;
            }
        }
        
        Ok(())
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<String, rocksdb::Error> {
        self.db.property_value("rocksdb.stats")
            .map(|v| v.unwrap_or_else(|| "No stats available".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let result = StorageManager::new(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_store_and_retrieve_header() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageManager::new(temp_dir.path()).unwrap();
        
        let height = 100u64;
        let hash = b"test_hash_12345678";
        let header = b"test_header_data";
        
        storage.store_header(height, hash, header).unwrap();
        
        let retrieved = storage.get_header_by_height(height).unwrap();
        assert_eq!(retrieved.as_deref(), Some(header.as_slice()));
        
        let by_hash = storage.get_header_by_hash(hash).unwrap();
        assert_eq!(by_hash.as_deref(), Some(header.as_slice()));
    }

    #[test]
    fn test_latest_height() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageManager::new(temp_dir.path()).unwrap();
        
        assert_eq!(storage.get_latest_height().unwrap(), None);
        
        storage.store_header(42, b"hash", b"header").unwrap();
        assert_eq!(storage.get_latest_height().unwrap(), Some(42));
    }
}
