/// RocksDB persistent storage layer
/// Provides durable storage for blocks, state, and chain data

use rocksdb::{DB, Options, WriteBatch};
use std::path::Path;
use std::sync::Arc;

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
    pub fn store_header(&self, height: u64, hash: &[u8], header: &[u8]) -> Result<(), String> {
        let cf = self.db.cf_handle(CF_HEADERS)
            .ok_or_else(|| "Headers column family not found".to_string())?;

        let mut batch = WriteBatch::default();
        // Store by height
        batch.put_cf(cf, height.to_be_bytes(), header);
        // Store by hash
        batch.put_cf(cf, hash, header);
        // Update chain index
        let index_cf = self.db.cf_handle(CF_CHAIN_INDEX)
            .ok_or_else(|| "Chain index column family not found".to_string())?;
        batch.put_cf(index_cf, b"latest_height", height.to_be_bytes());
        batch.put_cf(index_cf, b"latest_hash", hash);

        self.db.write(batch).map_err(|e| e.to_string())
    }

    /// Store a full block
    pub fn store_block(&self, hash: &[u8], block: &[u8]) -> Result<(), String> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or_else(|| "Blocks column family not found".to_string())?;
        self.db.put_cf(cf, hash, block).map_err(|e| e.to_string())
    }

    /// Get block by hash
    pub fn get_block(&self, hash: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or_else(|| "Blocks column family not found".to_string())?;
        self.db.get_cf(cf, hash).map_err(|e| e.to_string())
    }

    /// Get header by height
    pub fn get_header_by_height(&self, height: u64) -> Result<Option<Vec<u8>>, String> {
        let cf = self.db.cf_handle(CF_HEADERS)
            .ok_or_else(|| "Headers column family not found".to_string())?;
        self.db.get_cf(cf, height.to_be_bytes()).map_err(|e| e.to_string())
    }

    /// Get header by hash
    pub fn get_header_by_hash(&self, hash: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let cf = self.db.cf_handle(CF_HEADERS)
            .ok_or_else(|| "Headers column family not found".to_string())?;
        self.db.get_cf(cf, hash).map_err(|e| e.to_string())
    }

    /// Get latest chain height
    pub fn get_latest_height(&self) -> Result<Option<u64>, String> {
        let cf = self.db.cf_handle(CF_CHAIN_INDEX)
            .ok_or_else(|| "Chain index column family not found".to_string())?;
        if let Some(bytes) = self.db.get_cf(cf, b"latest_height").map_err(|e| e.to_string())? {
            let height = u64::from_be_bytes(
                bytes.as_slice().try_into()
                    .map_err(|_| "Invalid height data".to_string())?
            );
            Ok(Some(height))
        } else {
            Ok(None)
        }
    }

    /// Store account state
    pub fn store_account(&self, address: &[u8], account: &Account) -> Result<(), String> {
        let cf = self.db.cf_handle(CF_ACCOUNTS)
            .ok_or_else(|| "Accounts column family not found".to_string())?;
        let data = bincode::serialize(account)
            .map_err(|e| format!("Serialization error: {}", e))?;
        self.db.put_cf(cf, address, data).map_err(|e| e.to_string())
    }

    /// Get account state
    pub fn get_account(&self, address: &[u8]) -> Result<Option<Account>, String> {
        let cf = self.db.cf_handle(CF_ACCOUNTS)
            .ok_or_else(|| "Accounts column family not found".to_string())?;
        if let Some(data) = self.db.get_cf(cf, address).map_err(|e| e.to_string())? {
            Ok(bincode::deserialize(&data).ok())
        } else {
            Ok(None)
        }
    }

    /// Store bond state
    pub fn store_bond(&self, miner_id: &[u8], bond: &BondState) -> Result<(), String> {
        let cf = self.db.cf_handle(CF_BONDS)
            .ok_or_else(|| "Bonds column family not found".to_string())?;
        let data = bincode::serialize(bond)
            .map_err(|e| format!("Serialization error: {}", e))?;
        self.db.put_cf(cf, miner_id, data).map_err(|e| e.to_string())
    }

    /// Get bond state
    pub fn get_bond(&self, miner_id: &[u8]) -> Result<Option<BondState>, String> {
        let cf = self.db.cf_handle(CF_BONDS)
            .ok_or_else(|| "Bonds column family not found".to_string())?;
        if let Some(data) = self.db.get_cf(cf, miner_id).map_err(|e| e.to_string())? {
            Ok(bincode::deserialize(&data).ok())
        } else {
            Ok(None)
        }
    }

    /// Store state root for a given height
    pub fn store_state_root(&self, height: u64, root: &[u8]) -> Result<(), String> {
        let cf = self.db.cf_handle(CF_STATE_ROOTS)
            .ok_or_else(|| "State roots column family not found".to_string())?;
        self.db.put_cf(cf, height.to_be_bytes(), root).map_err(|e| e.to_string())
    }

    /// Get state root for a given height
    pub fn get_state_root(&self, height: u64) -> Result<Option<Vec<u8>>, String> {
        let cf = self.db.cf_handle(CF_STATE_ROOTS)
            .ok_or_else(|| "State roots column family not found".to_string())?;
        self.db.get_cf(cf, height.to_be_bytes()).map_err(|e| e.to_string())
    }

    /// Prune old blocks (keep last N blocks)
    ///
    /// # TODO: Production Implementation
    /// This is a simplified implementation for development. A production version should:
    /// - Use iterators for efficient range deletion
    /// - Delete associated transactions and state roots
    /// - Handle edge cases (e.g., concurrent reads during pruning)
    /// - Optionally archive pruned blocks to cold storage
    ///
    /// # Arguments
    /// * `keep_last` - Number of recent blocks to retain
    ///
    /// # Returns
    /// * `Ok(())` on success, or error message on failure
    pub fn prune_old_blocks(&self, keep_last: u64) -> Result<(), String> {
        let latest = self.get_latest_height()?.unwrap_or(0);
        if latest <= keep_last {
            return Ok(());
        }

        let prune_until = latest - keep_last;

        // Get column family handles
        let cf_blocks = self.db.cf_handle(CF_BLOCKS)
            .ok_or_else(|| "Blocks column family not found".to_string())?;
        let cf_headers = self.db.cf_handle(CF_HEADERS)
            .ok_or_else(|| "Headers column family not found".to_string())?;

        // Iterate and delete blocks and headers for heights less than prune_until
        for height in 0..prune_until {
            // Delete block by height
            self.db.delete_cf(cf_blocks, height.to_be_bytes())
                .map_err(|e| format!("Failed to delete block at height {}: {}", height, e))?;
            // Delete header by height
            self.db.delete_cf(cf_headers, height.to_be_bytes())
                .map_err(|e| format!("Failed to delete header at height {}: {}", height, e))?;
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
