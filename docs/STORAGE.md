# RocksDB Storage Layer Documentation

## Overview

The BitCell storage layer provides production-grade persistent storage using RocksDB with support for:
- Block and header storage with multiple indexes
- Transaction indexing by hash and sender
- State snapshots for fast recovery
- Account and bond state persistence
- Atomic batch operations
- Production-grade pruning with optional archiving

## Architecture

### Column Families

The storage layer uses separate RocksDB column families for different data types:

- `blocks`: Full block data indexed by hash
- `headers`: Block headers indexed by height and hash
- `transactions`: Transaction data indexed by hash
- `tx_by_sender`: Secondary index for transactions by sender address
- `accounts`: Account state data
- `bonds`: Bond state data
- `state_roots`: State root hashes by height
- `chain_index`: Chain metadata (latest height, latest hash, etc.)
- `snapshots`: State snapshots at periodic intervals

### Key Design Decisions

1. **Multiple Indexes**: Blocks and headers are indexed by both height and hash for O(1) lookups
2. **Sender Index**: Transactions use a composite key (sender||height||tx_hash) for efficient range queries
3. **Atomic Writes**: All multi-key operations use `WriteBatch` for atomicity
4. **Snapshots**: Variable-length snapshot format with length prefix for flexibility
5. **Separation of Concerns**: Block data, state data, and indexes are in separate column families

## API Reference

### Basic Operations

#### Creating Storage Manager

```rust
use bitcell_state::StorageManager;
use std::path::Path;

// Create new storage or open existing
let storage = StorageManager::new(Path::new("/path/to/db"))?;
```

#### Storing and Retrieving Blocks

```rust
// Store a block header
let height = 100u64;
let hash = b"block_hash_32_bytes";
let header_data = bincode::serialize(&header)?;

storage.store_header(height, hash, &header_data)?;

// Retrieve by height
let header = storage.get_header_by_height(height)?;

// Retrieve by hash
let header = storage.get_header_by_hash(hash)?;

// Get latest chain height
let latest_height = storage.get_latest_height()?;
```

#### Full Block Storage

```rust
// Store complete block
let block_hash = b"block_hash_32_bytes";
let block_data = bincode::serialize(&block)?;

storage.store_block(block_hash, &block_data)?;

// Retrieve block
let block = storage.get_block(block_hash)?;
```

### Transaction Indexing

#### Storing Transactions

```rust
// Store single transaction
let tx_hash = b"transaction_hash_32_bytes";
let sender = b"sender_public_key_33_bytes";
let tx_data = bincode::serialize(&transaction)?;
let block_height = 100u64;

storage.store_transaction(tx_hash, sender, &tx_data, block_height)?;

// Retrieve by hash (O(1))
let tx = storage.get_transaction(tx_hash)?;
```

#### Batch Transaction Storage

For better performance when storing multiple transactions:

```rust
// Prepare batch
let transactions = vec![
    (tx_hash1, sender1, tx_data1, height1),
    (tx_hash2, sender2, tx_data2, height2),
    // ...
];

// Store atomically
storage.store_transactions_batch(transactions)?;
```

#### Querying by Sender

```rust
// Get all transactions from a sender
let sender = b"sender_public_key_33_bytes";
let txs = storage.get_transactions_by_sender(sender, 0)?;

// Get with limit
let recent_txs = storage.get_transactions_by_sender(sender, 10)?;
```

### State Snapshots

#### Creating Snapshots

```rust
// Create snapshot every N blocks
if height % 10000 == 0 {
    let state_root = compute_state_root(&state);
    let accounts_data = serialize_accounts(&state)?;
    
    storage.create_snapshot(height, &state_root, &accounts_data)?;
}
```

#### Retrieving Snapshots

```rust
// Get most recent snapshot
let snapshot = storage.get_latest_snapshot()?;
if let Some((height, state_root, accounts_data)) = snapshot {
    // Restore state from snapshot
    restore_state(height, &state_root, &accounts_data)?;
}

// Get snapshot at specific height
let snapshot = storage.get_snapshot(50000)?;
```

### Account and Bond State

#### Account Operations

```rust
use bitcell_state::Account;

// Store account
let pubkey = [0u8; 33];
let account = Account {
    balance: 1000,
    nonce: 5,
};

storage.store_account(&pubkey, &account)?;

// Retrieve account
let account = storage.get_account(&pubkey)?;
```

#### Bond Operations

```rust
use bitcell_state::{BondState, BondStatus};

// Store bond
let miner_id = [0u8; 33];
let bond = BondState {
    amount: 5000,
    status: BondStatus::Active,
    locked_epoch: 0,
};

storage.store_bond(&miner_id, &bond)?;

// Retrieve bond
let bond = storage.get_bond(&miner_id)?;
```

### State Roots

```rust
// Store state root for block
let height = 100u64;
let state_root = compute_merkle_root(&state);

storage.store_state_root(height, &state_root)?;

// Retrieve state root
let root = storage.get_state_root(height)?;
```

### Pruning

#### Simple Pruning (Development/Testing)

```rust
// Keep last 1000 blocks
storage.prune_old_blocks(1000)?;
```

#### Production Pruning

For production use with archiving and statistics:

```rust
use std::path::Path;

// Prune with archiving
let archive_path = Path::new("/path/to/archive");
let stats = storage.prune_old_blocks_production(
    1000,  // keep_last
    Some(archive_path)
)?;

println!("Deleted {} blocks", stats.blocks_deleted);
println!("Deleted {} transactions", stats.transactions_deleted);
println!("Archived: {}", stats.archived);
```

## Integration with StateManager

The `StateManager` can use persistent storage:

```rust
use bitcell_state::{StateManager, StorageManager};
use std::sync::Arc;

// Create storage
let storage = Arc::new(StorageManager::new(path)?);

// Create StateManager with storage
let state_manager = StateManager::with_storage(storage)?;

// All state updates are automatically persisted
state_manager.update_account(pubkey, account);

// State survives restarts
// ... restart ...
let state_manager = StateManager::with_storage(storage)?;
// Previous state is automatically loaded
```

## Performance Characteristics

### Time Complexity

| Operation | Complexity | Notes |
|-----------|------------|-------|
| store_header | O(1) | Single write with index updates |
| get_header_by_height | O(1) | Direct key lookup |
| get_header_by_hash | O(1) | Direct key lookup |
| store_transaction | O(1) | Write with sender index |
| get_transaction | O(1) | Direct hash lookup |
| get_transactions_by_sender | O(n) | Range scan over sender's transactions |
| create_snapshot | O(1) | Single write operation |
| get_snapshot | O(1) | Direct key lookup |
| prune_old_blocks | O(n) | Where n is number of blocks to prune |

### Space Complexity

- **Headers**: ~1 KB per block (depends on header size)
- **Blocks**: Variable, depends on transaction count
- **Transactions**: ~500 bytes per transaction (average)
- **Transaction Index**: ~100 bytes per transaction (sender index)
- **Accounts**: ~100 bytes per account
- **Snapshots**: Depends on state size, compressed

### Benchmark Results

Run benchmarks with:
```bash
cargo bench --package bitcell-state
```

Expected performance (on typical hardware):
- Block storage: ~50,000 blocks/second
- Transaction storage: ~100,000 transactions/second
- Transaction batch (100): ~500,000 transactions/second
- Transaction retrieval by hash: ~200,000 ops/second
- Transaction retrieval by sender: ~10,000 ops/second
- Snapshot creation (10KB): ~5,000 ops/second
- Account operations: ~150,000 ops/second

## Best Practices

### 1. Use Batch Operations

When storing multiple items, use batch operations for better performance:

```rust
// Good: Batch
storage.store_transactions_batch(transactions)?;

// Avoid: Loop
for (hash, sender, data, height) in transactions {
    storage.store_transaction(hash, sender, data, height)?;
}
```

### 2. Periodic Snapshots

Create snapshots at regular intervals for fast recovery:

```rust
const SNAPSHOT_INTERVAL: u64 = 10000;

if height % SNAPSHOT_INTERVAL == 0 {
    storage.create_snapshot(height, state_root, accounts_data)?;
}
```

### 3. Pruning Strategy

Balance disk space with recovery capability:

```rust
// Keep enough blocks for reorganization
const KEEP_BLOCKS: u64 = 2000;

// Prune periodically
if height % 1000 == 0 {
    storage.prune_old_blocks_production(KEEP_BLOCKS, archive_path)?;
}
```

### 4. Error Handling

Always handle storage errors appropriately:

```rust
match storage.store_header(height, hash, data) {
    Ok(_) => {
        // Success
    }
    Err(e) => {
        // Log error, possibly retry, or escalate
        tracing::error!("Failed to store header: {}", e);
        return Err(e.into());
    }
}
```

### 5. Fixed-Length Keys

For optimal indexing, use fixed-length addresses/keys:

```rust
// Good: Fixed 33-byte public key
let sender = [0u8; 33];

// Avoid: Variable-length strings
let sender = "variable_length_address".as_bytes();
```

## Testing

### Unit Tests

Run unit tests:
```bash
cargo test --package bitcell-state --lib
```

### Integration Tests

Run integration tests:
```bash
cargo test --package bitcell-state --test storage_persistence_test
```

### Benchmarks

Run benchmarks:
```bash
cargo bench --package bitcell-state
```

## Troubleshooting

### Database Lock Errors

If you encounter "database is locked" errors:
- Ensure only one process accesses the database
- Check for zombie processes holding the lock
- Use proper shutdown procedures

### Performance Issues

If experiencing slow operations:
- Check disk I/O capacity
- Consider SSD instead of HDD
- Increase RocksDB cache size
- Use batch operations
- Profile with benchmarks

### Space Issues

If running out of disk space:
- Implement regular pruning
- Use archiving for old blocks
- Compress snapshots
- Monitor disk usage

### Recovery Issues

If unable to recover state:
- Check latest snapshot availability
- Verify snapshot integrity
- Use snapshot at earlier height
- Replay blocks from snapshot height

## Future Enhancements

Planned improvements (see RC2-005):
- [ ] Compression for snapshots
- [ ] Incremental snapshots
- [ ] State trie integration
- [ ] Better compaction strategies
- [ ] Metrics and monitoring hooks
- [ ] Backup and restore utilities

## References

- [RocksDB Documentation](https://github.com/facebook/rocksdb/wiki)
- [RC2-005 Requirements](/docs/RELEASE_REQUIREMENTS.md#rc2-005-rocksdb-persistence)
- [Storage Benchmarks](../benches/storage_bench.rs)
- [Integration Tests](../tests/storage_persistence_test.rs)
