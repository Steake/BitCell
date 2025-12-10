//! Integration tests for persistent storage
//!
//! These tests verify the production-readiness of the RocksDB storage layer,
//! including persistence across restarts, snapshot functionality, and multi-block scenarios.

use bitcell_state::{Account, StateManager, StorageManager};
use std::sync::Arc;
use tempfile::TempDir;

#[test]
fn test_multi_block_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path()).unwrap();
    
    // Store 100 blocks with transactions and state
    for height in 0..100 {
        let hash = format!("block_hash_{:064}", height);
        let header = format!("block_header_{}", height);
        
        // Store block header
        storage.store_header(height, hash.as_bytes(), header.as_bytes()).unwrap();
        
        // Store transactions for this block
        for tx_idx in 0..10 {
            let tx_hash = format!("tx_{}_{:032}", height, tx_idx);
            let sender = format!("sender_{:033}", height % 10);
            let tx_data = format!("data_{}_{}", height, tx_idx);
            
            storage.store_transaction(
                tx_hash.as_bytes(),
                sender.as_bytes(),
                tx_data.as_bytes(),
                height,
            ).unwrap();
        }
        
        // Store state root
        let state_root = format!("state_root_{:032}", height);
        storage.store_state_root(height, state_root.as_bytes()).unwrap();
        
        // Create snapshot every 10 blocks
        if height % 10 == 0 {
            let accounts_data = format!("snapshot_data_at_{}", height);
            storage.create_snapshot(
                height,
                state_root.as_bytes(),
                accounts_data.as_bytes(),
            ).unwrap();
        }
    }
    
    // Verify all data is retrievable
    assert_eq!(storage.get_latest_height().unwrap(), Some(99));
    
    // Verify blocks
    for height in 0..100 {
        let header = storage.get_header_by_height(height).unwrap();
        assert!(header.is_some(), "Block {} not found", height);
    }
    
    // Verify transactions - check each unique sender once
    for sender_id in 0..10 {
        let sender = format!("sender_{:033}", sender_id);
        let txs = storage.get_transactions_by_sender(sender.as_bytes(), 0).unwrap();
        assert_eq!(txs.len(), 100, "Expected 100 transactions for sender {}", sender_id);
    }
    
    // Verify state roots
    for height in 0..100 {
        let root = storage.get_state_root(height).unwrap();
        assert!(root.is_some(), "State root {} not found", height);
    }
    
    // Verify snapshots
    for height in (0..100).step_by(10) {
        let snapshot = storage.get_snapshot(height).unwrap();
        assert!(snapshot.is_some(), "Snapshot at height {} not found", height);
    }
}

#[test]
fn test_state_recovery_after_restart() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();
    
    // First session: Store data
    {
        let storage = StorageManager::new(&db_path).unwrap();
        
        // Store blocks
        for height in 0..50 {
            let hash = format!("hash_{:064}", height);
            let header = format!("header_{}", height);
            storage.store_header(height, hash.as_bytes(), header.as_bytes()).unwrap();
        }
        
        // Store accounts
        for i in 0u64..100 {
            let address = {
                let mut addr = [0u8; 33];
                addr[0..8].copy_from_slice(&i.to_le_bytes());
                addr
            };
            let account = Account {
                balance: 1000 + i,
                nonce: i,
            };
            storage.store_account(&address, &account).unwrap();
        }
        
        // Store a snapshot
        let state_root = [42u8; 32];
        let accounts_data = b"serialized_accounts_state";
        storage.create_snapshot(50, &state_root, accounts_data).unwrap();
        
        // Storage dropped here, simulating shutdown
    }
    
    // Second session: Verify data persisted
    {
        let storage = StorageManager::new(&db_path).unwrap();
        
        // Verify blocks persisted
        assert_eq!(storage.get_latest_height().unwrap(), Some(49));
        
        for height in 0..50 {
            let header = storage.get_header_by_height(height).unwrap();
            assert!(header.is_some(), "Block {} lost after restart", height);
        }
        
        // Verify accounts persisted
        for i in 0u64..100 {
            let address = {
                let mut addr = [0u8; 33];
                addr[0..8].copy_from_slice(&i.to_le_bytes());
                addr
            };
            let account = storage.get_account(&address).unwrap();
            assert!(account.is_some(), "Account {} lost after restart", i);
            
            let acc = account.unwrap();
            assert_eq!(acc.balance, 1000 + i);
            assert_eq!(acc.nonce, i);
        }
        
        // Verify snapshot persisted
        let snapshot = storage.get_latest_snapshot().unwrap();
        assert!(snapshot.is_some(), "Snapshot lost after restart");
        
        let (height, root, data) = snapshot.unwrap();
        assert_eq!(height, 50);
        assert_eq!(root.as_slice(), &[42u8; 32]);
        assert_eq!(data.as_slice(), b"serialized_accounts_state");
    }
}

#[test]
fn test_state_manager_with_storage() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(StorageManager::new(temp_dir.path()).unwrap());
    
    let mut state_manager = StateManager::with_storage(Arc::clone(&storage)).unwrap();
    
    // Create some accounts
    for i in 0u8..10 {
        let mut pubkey = [0u8; 33];
        pubkey[0] = i;
        
        let account = Account {
            balance: 1000 * (i as u64 + 1),
            nonce: 0,
        };
        
        state_manager.update_account(pubkey, account);
    }
    
    // Verify accounts are in memory
    for i in 0u8..10 {
        let mut pubkey = [0u8; 33];
        pubkey[0] = i;
        
        let account = state_manager.get_account(&pubkey);
        assert!(account.is_some());
        assert_eq!(account.unwrap().balance, 1000 * (i as u64 + 1));
    }
    
    // Verify accounts are also persisted to storage
    for i in 0u8..10 {
        let mut pubkey = [0u8; 33];
        pubkey[0] = i;
        
        let account = storage.get_account(&pubkey).unwrap();
        assert!(account.is_some());
        assert_eq!(account.unwrap().balance, 1000 * (i as u64 + 1));
    }
}

#[test]
fn test_snapshot_based_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path()).unwrap();
    
    // Simulate a long chain with periodic snapshots
    for height in 0..1000 {
        let hash = format!("hash_{:064}", height);
        let header = format!("header_{}", height);
        storage.store_header(height, hash.as_bytes(), header.as_bytes()).unwrap();
        
        // Create snapshot every 100 blocks
        if height % 100 == 0 {
            let state_root = format!("root_{:032}", height);
            let accounts_data = format!("snapshot_{}", height);
            storage.create_snapshot(
                height,
                state_root.as_bytes(),
                accounts_data.as_bytes(),
            ).unwrap();
        }
    }
    
    // Prune old blocks, keeping only last 200
    storage.prune_old_blocks(200).unwrap();
    
    // Old blocks should be pruned
    // Latest is 999, prune_until = 999 - 200 = 799, so we prune 0..799
    for height in 0..799 {
        let header = storage.get_header_by_height(height).unwrap();
        assert!(header.is_none(), "Block {} should have been pruned", height);
    }
    
    // Recent blocks should still exist (blocks 799-999 since we stored 0-999)
    for height in 799..1000 {
        let header = storage.get_header_by_height(height).unwrap();
        assert!(header.is_some(), "Block {} should not have been pruned", height);
    }
    
    // All snapshots should still exist (even for pruned blocks)
    for height in (0..1000).step_by(100) {
        let snapshot = storage.get_snapshot(height).unwrap();
        assert!(snapshot.is_some(), "Snapshot at {} should still exist", height);
    }
    
    // Can recover from any snapshot
    let latest_snapshot = storage.get_latest_snapshot().unwrap();
    assert!(latest_snapshot.is_some());
    let (snap_height, _root, _data) = latest_snapshot.unwrap();
    assert_eq!(snap_height, 900);
}

#[test]
fn test_concurrent_storage_operations() {
    use std::thread;
    
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(StorageManager::new(temp_dir.path()).unwrap());
    
    let mut handles = vec![];
    
    // Spawn threads for concurrent operations
    for thread_id in 0..5 {
        let storage_clone = Arc::clone(&storage);
        
        let handle = thread::spawn(move || {
            // Each thread stores its own blocks
            for i in 0..20 {
                let height = thread_id * 1000 + i;
                let hash = format!("hash_{}_{:032}", thread_id, i);
                let header = format!("header_{}_{}", thread_id, i);
                
                storage_clone.store_header(
                    height,
                    hash.as_bytes(),
                    header.as_bytes()
                ).unwrap();
            }
            
            // Each thread stores accounts
            for i in 0u64..20 {
                let address = {
                    let mut addr = [0u8; 33];
                    addr[0] = thread_id as u8;
                    addr[1..9].copy_from_slice(&i.to_le_bytes());
                    addr
                };
                let account = Account {
                    balance: (thread_id * 1000 + i) as u64,
                    nonce: i,
                };
                storage_clone.store_account(&address, &account).unwrap();
            }
            
            // Each thread stores transactions
            for i in 0..20 {
                let tx_hash = format!("tx_{}_{:032}", thread_id, i);
                let sender = format!("sender_{:033}", thread_id);
                let tx_data = format!("data_{}_{}", thread_id, i);
                
                storage_clone.store_transaction(
                    tx_hash.as_bytes(),
                    sender.as_bytes(),
                    tx_data.as_bytes(),
                    (thread_id * 1000 + i) as u64,
                ).unwrap();
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify all data was stored correctly
    for thread_id in 0..5 {
        // Verify blocks
        for i in 0..20 {
            let height = thread_id * 1000 + i;
            let header = storage.get_header_by_height(height).unwrap();
            assert!(header.is_some(), "Block from thread {} not found", thread_id);
        }
        
        // Verify accounts
        for i in 0u64..20 {
            let address = {
                let mut addr = [0u8; 33];
                addr[0] = thread_id as u8;
                addr[1..9].copy_from_slice(&i.to_le_bytes());
                addr
            };
            let account = storage.get_account(&address).unwrap();
            assert!(account.is_some(), "Account from thread {} not found", thread_id);
        }
        
        // Verify transactions
        let sender = format!("sender_{:033}", thread_id);
        let txs = storage.get_transactions_by_sender(sender.as_bytes(), 0).unwrap();
        assert_eq!(txs.len(), 20, "Transactions from thread {} not all found", thread_id);
    }
}

#[test]
fn test_production_pruning_with_archive() {
    let temp_dir = TempDir::new().unwrap();
    let archive_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path()).unwrap();
    
    // Store blocks
    for height in 0..500 {
        let hash = format!("hash_{:064}", height);
        let header = format!("header_{}", height);
        storage.store_header(height, hash.as_bytes(), header.as_bytes()).unwrap();
    }
    
    // Prune with archiving
    let stats = storage.prune_old_blocks_production(100, Some(archive_dir.path())).unwrap();
    
    // Verify stats - should delete blocks 0 to 398 (399 blocks)
    // Latest is 499, prune_until = 499 - 100 = 399, so we prune 0..399
    assert_eq!(stats.blocks_deleted, 399);
    assert!(stats.archived);
    
    // Verify pruning worked
    for height in 0..399 {
        let header = storage.get_header_by_height(height).unwrap();
        assert!(header.is_none(), "Block {} should be pruned", height);
    }
    
    for height in 399..500 {
        let header = storage.get_header_by_height(height).unwrap();
        assert!(header.is_some(), "Block {} should exist", height);
    }
    
    // Verify archive was created (archive has its own database)
    let archive_storage = StorageManager::new(archive_dir.path()).unwrap();
    // Archive should contain the archived blocks (implementation detail)
    // This is a basic check that the archive database was created
    assert!(archive_storage.get_stats().is_ok());
}

#[test]
fn test_large_transaction_batch() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path()).unwrap();
    
    // Create a large batch of transactions
    let batch_size = 1000;
    let mut batch_data = Vec::new();
    
    for i in 0..batch_size {
        let tx_hash = format!("tx_hash_{:032}", i);
        let sender = format!("sender_{:033}", i % 100);
        let tx_data = format!("data_{}", i);
        batch_data.push((tx_hash, sender, tx_data));
    }
    
    // Convert to references for the batch operation
    let batch_refs: Vec<(&[u8], &[u8], &[u8], u64)> = batch_data
        .iter()
        .enumerate()
        .map(|(i, (h, s, d))| (h.as_bytes(), s.as_bytes(), d.as_bytes(), i as u64))
        .collect();
    
    // Store batch atomically
    storage.store_transactions_batch(batch_refs).unwrap();
    
    // Verify all transactions are retrievable
    for i in 0..batch_size {
        let tx_hash = format!("tx_hash_{:032}", i);
        let tx = storage.get_transaction(tx_hash.as_bytes()).unwrap();
        assert!(tx.is_some(), "Transaction {} not found", i);
    }
    
    // Verify sender indexes
    for sender_id in 0..100 {
        let sender = format!("sender_{:033}", sender_id);
        let txs = storage.get_transactions_by_sender(sender.as_bytes(), 0).unwrap();
        assert_eq!(txs.len(), 10, "Expected 10 transactions for sender {}", sender_id);
    }
}
