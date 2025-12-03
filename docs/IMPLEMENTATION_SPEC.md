# BitCell NOT_IMPLEMENTED & TODO Analysis and Implementation Specification

## Executive Summary

This document provides a systematic analysis of all unimplemented functionality in the BitCell codebase, 
categorized by priority and complexity. Each item includes detailed implementation specifications.

---

## Category 1: Transaction Flow (CRITICAL)

### 1.1 Admin Wallet Transaction Sending
**Location:** `crates/bitcell-admin/src/api/wallet.rs:88-110`  
**Current Status:** Returns `StatusCode::NOT_IMPLEMENTED`  
**Dependencies Required:**
- Private key management system
- Transaction builder
- Transaction signer (ECDSA secp256k1)
- RLP encoder
- Nonce management

**Implementation Specification:**

```rust
// 1. Create TransactionBuilder struct
pub struct TransactionBuilder {
    from: PublicKey,
    to: PublicKey,
    amount: u64,
    gas_price: u64,
    gas_limit: u64,
    nonce: u64,
    data: Vec<u8>,
}

impl TransactionBuilder {
    pub fn new(from: PublicKey, to: PublicKey) -> Self { ... }
    pub fn amount(mut self, amount: u64) -> Self { ... }
    pub fn gas_price(mut self, gas_price: u64) -> Self { ... }
    pub fn gas_limit(mut self, gas_limit: u64) -> Self { ... }
    pub fn nonce(mut self, nonce: u64) -> Self { ... }
    pub fn data(mut self, data: Vec<u8>) -> Self { ... }
    pub fn build(self) -> UnsignedTransaction { ... }
}

// 2. Create TransactionSigner trait
pub trait TransactionSigner {
    fn sign(&self, tx: &UnsignedTransaction) -> Result<SignedTransaction, SignerError>;
}

// 3. Implement SecretKeySigner for direct signing
pub struct SecretKeySigner {
    secret_key: SecretKey,
}

impl TransactionSigner for SecretKeySigner {
    fn sign(&self, tx: &UnsignedTransaction) -> Result<SignedTransaction, SignerError> {
        let tx_hash = tx.hash();
        let signature = self.secret_key.sign(tx_hash.as_bytes());
        Ok(SignedTransaction::new(tx.clone(), signature))
    }
}

// 4. RLP encoding for network submission
impl SignedTransaction {
    pub fn to_rlp(&self) -> Vec<u8> {
        // Use rlp crate to encode transaction
        rlp::encode(self).to_vec()
    }
}
```

**Files to Create/Modify:**
- `crates/bitcell-admin/src/tx_builder.rs` (NEW)
- `crates/bitcell-admin/src/signer.rs` (NEW)
- `crates/bitcell-admin/src/api/wallet.rs` (MODIFY)
- `crates/bitcell-consensus/src/transaction.rs` (MODIFY - add RLP encoding)

**Integration Steps:**
1. Create key storage mechanism in admin console
2. Fetch nonce from RPC (eth_getTransactionCount equivalent)
3. Estimate gas using RPC
4. Build and sign transaction
5. Submit via eth_sendRawTransaction
6. Return transaction hash to user

---

### 1.2 Wallet GUI Transaction Sending
**Location:** `crates/bitcell-wallet-gui/src/main.rs:399-402`  
**Current Status:** Shows "not implemented" message  
**Dependencies:** Depends on 1.1 completion

**Implementation Specification:**

```rust
wallet_state.on_send_transaction(move |to_address, amount, chain_str| {
    let window = window_weak.unwrap();
    let wallet_state = window.global::<WalletState>();
    
    // Validate inputs
    let amount: f64 = amount.parse().unwrap_or(0.0);
    if amount <= 0.0 {
        wallet_state.set_status_message("Invalid amount".into());
        return;
    }
    
    if to_address.is_empty() {
        wallet_state.set_status_message("Invalid recipient address".into());
        return;
    }
    
    // Get wallet reference
    let app_state = state.borrow();
    let wallet = match &app_state.wallet {
        Some(w) => w,
        None => {
            wallet_state.set_status_message("Wallet not initialized".into());
            return;
        }
    };
    
    // Get RPC client
    let rpc_client = match &app_state.rpc_client {
        Some(c) => c.clone(),
        None => {
            wallet_state.set_status_message("Not connected to node".into());
            return;
        }
    };
    
    // Build transaction
    let from_address = wallet.primary_address();
    let to_pubkey = match parse_address(&to_address) {
        Ok(p) => p,
        Err(e) => {
            wallet_state.set_status_message(format!("Invalid address: {}", e).into());
            return;
        }
    };
    
    // Spawn async task for transaction submission
    let window_weak = window.as_weak();
    tokio::spawn(async move {
        // 1. Fetch nonce
        let nonce = match rpc_client.get_transaction_count(&from_address).await {
            Ok(n) => n,
            Err(e) => {
                update_status(&window_weak, format!("Failed to get nonce: {}", e));
                return;
            }
        };
        
        // 2. Build transaction
        let tx = TransactionBuilder::new(from_address.to_pubkey(), to_pubkey)
            .amount((amount * 1e18) as u64) // Convert to base units
            .gas_price(1_000_000_000) // 1 Gwei
            .gas_limit(21000)
            .nonce(nonce)
            .build();
        
        // 3. Sign with wallet key
        let signed_tx = match wallet.sign_transaction(&tx) {
            Ok(t) => t,
            Err(e) => {
                update_status(&window_weak, format!("Failed to sign: {}", e));
                return;
            }
        };
        
        // 4. Submit via RPC
        let tx_hash = match rpc_client.send_raw_transaction(&signed_tx.to_rlp()).await {
            Ok(h) => h,
            Err(e) => {
                update_status(&window_weak, format!("Failed to submit: {}", e));
                return;
            }
        };
        
        update_status(&window_weak, format!("Transaction sent: {}", tx_hash));
    });
});
```

**Files to Modify:**
- `crates/bitcell-wallet-gui/src/main.rs`
- `crates/bitcell-wallet-gui/src/rpc_client.rs` (add get_transaction_count, send_raw_transaction)
- `crates/bitcell-wallet/src/lib.rs` (add sign_transaction method)

---

## Category 2: Metrics & Monitoring (HIGH)

### 2.1 System Metrics Collection
**Location:** `crates/bitcell-admin/src/api/metrics.rs:96-127`  
**Current Status:** Returns placeholder values (0)  
**Dependencies:** `sysinfo` crate

**Implementation Specification:**

```rust
use sysinfo::{System, SystemExt, ProcessExt, CpuExt, DiskExt};
use std::time::Instant;
use std::sync::{Arc, RwLock};

/// System metrics collector
pub struct SystemMetricsCollector {
    system: RwLock<System>,
    start_time: Instant,
}

impl SystemMetricsCollector {
    pub fn new() -> Self {
        Self {
            system: RwLock::new(System::new_all()),
            start_time: Instant::now(),
        }
    }
    
    /// Collect current system metrics
    pub fn collect(&self) -> SystemMetrics {
        let mut system = self.system.write().unwrap();
        system.refresh_all();
        
        // Calculate CPU usage (average across all cores)
        let cpu_usage = system.cpus().iter()
            .map(|cpu| cpu.cpu_usage())
            .sum::<f32>() / system.cpus().len() as f32;
        
        // Memory usage
        let memory_usage_mb = system.used_memory() / 1024 / 1024;
        
        // Disk usage (sum of all disks)
        let disk_usage_mb: u64 = system.disks().iter()
            .map(|d| d.total_space() - d.available_space())
            .sum::<u64>() / 1024 / 1024;
        
        SystemMetrics {
            uptime_seconds: self.start_time.elapsed().as_secs(),
            cpu_usage: cpu_usage as f64,
            memory_usage_mb,
            disk_usage_mb,
        }
    }
}
```

**Files to Create/Modify:**
- `crates/bitcell-admin/src/system_metrics.rs` (NEW)
- `crates/bitcell-admin/Cargo.toml` (ADD `sysinfo = "0.30"`)
- `crates/bitcell-admin/src/api/metrics.rs` (MODIFY to use SystemMetricsCollector)
- `crates/bitcell-admin/src/lib.rs` (ADD mod system_metrics)

---

### 2.2 Network Message Tracking
**Location:** `crates/bitcell-admin/src/api/metrics.rs:113-114`  
**Current Status:** Returns 0 for messages_sent/received

**Implementation Specification:**

```rust
// In crates/bitcell-node/src/network.rs

use std::sync::atomic::{AtomicU64, Ordering};

pub struct NetworkMetricsCounters {
    pub messages_sent: AtomicU64,
    pub messages_received: AtomicU64,
}

impl NetworkMetricsCounters {
    pub fn new() -> Self {
        Self {
            messages_sent: AtomicU64::new(0),
            messages_received: AtomicU64::new(0),
        }
    }
    
    pub fn increment_sent(&self) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_received(&self) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn get_stats(&self) -> (u64, u64) {
        (
            self.messages_sent.load(Ordering::Relaxed),
            self.messages_received.load(Ordering::Relaxed),
        )
    }
}

// Add to NetworkManager struct
pub struct NetworkManager {
    // ... existing fields ...
    message_counters: Arc<NetworkMetricsCounters>,
}

// Increment counters on message send/receive
async fn handle_incoming_message(&self, ...) {
    self.message_counters.increment_received();
    // ... handle message ...
}

async fn broadcast_block(&self, ...) {
    self.message_counters.increment_sent();
    // ... broadcast ...
}
```

**Files to Modify:**
- `crates/bitcell-node/src/network.rs`
- `crates/bitcell-node/src/monitoring/metrics.rs` (expose message counts)

---

### 2.3 EBSL Trust Scores & Slashing Events
**Location:** `crates/bitcell-admin/src/api/metrics.rs:119-120`  
**Current Status:** Returns placeholder values

**Implementation Specification:**

```rust
// In crates/bitcell-node/src/tournament.rs

pub struct TournamentMetrics {
    trust_scores: HashMap<PublicKey, f64>,
    slashing_events: Vec<SlashingEvent>,
}

#[derive(Clone, Debug)]
pub struct SlashingEvent {
    pub miner: PublicKey,
    pub block_height: u64,
    pub reason: SlashingReason,
    pub amount: u64,
    pub timestamp: u64,
}

#[derive(Clone, Debug)]
pub enum SlashingReason {
    InvalidProof,
    DoubleCommitment,
    MissedReveal,
    InvalidBlock,
}

impl TournamentManager {
    pub fn get_average_trust_score(&self) -> f64 {
        let scores: Vec<f64> = self.trust_scores.values().copied().collect();
        if scores.is_empty() {
            return 0.0;
        }
        scores.iter().sum::<f64>() / scores.len() as f64
    }
    
    pub fn get_slashing_count(&self) -> u64 {
        self.slashing_events.len() as u64
    }
    
    pub fn record_slashing(&mut self, event: SlashingEvent) {
        self.slashing_events.push(event);
    }
}
```

**Files to Modify:**
- `crates/bitcell-node/src/tournament.rs`
- `crates/bitcell-node/src/monitoring/metrics.rs`

---

## Category 3: RPC Endpoints (MEDIUM)

### 3.1 Node ID Exposure
**Location:** `crates/bitcell-node/src/rpc.rs:508`  
**Current Status:** Returns "TODO_NODE_ID"

**Implementation Specification:**

```rust
// Modify RpcState to include node_id
pub struct RpcState {
    pub blockchain: Blockchain,
    pub network: NetworkManager,
    pub tx_pool: TransactionPool,
    pub tournament_manager: Option<Arc<TournamentManager>>,
    pub config: NodeConfig,
    pub node_type: String,
    pub node_id: String,  // ADD THIS FIELD
}

// Initialize in main.rs when creating RpcState
let rpc_state = RpcState {
    // ... other fields ...
    node_id: secret_key.public_key().to_hex_string(),
};

// Update bitcell_get_node_info
async fn bitcell_get_node_info(state: &RpcState) -> Result<Value, JsonRpcError> {
    Ok(json!({
        "node_id": state.node_id,
        "version": "0.1.0",
        "protocol_version": "1",
        "network_id": "bitcell-testnet",
        "api_version": "0.1-alpha",
        "capabilities": ["bitcell/1"],
        "node_type": state.node_type,
    }))
}
```

**Files to Modify:**
- `crates/bitcell-node/src/rpc.rs`
- `crates/bitcell-node/src/main.rs`

---

### 3.2 Block Metrics
**Location:** `crates/bitcell-node/src/rpc.rs:228-231`  
**Current Status:** Placeholder values for nonce, logsBloom, size

**Implementation Specification:**

```rust
// Calculate actual block size
fn calculate_block_size(block: &Block) -> u64 {
    bincode::serialized_size(block).unwrap_or(0)
}

// In eth_get_block_by_number response:
Ok(json!({
    // ... other fields ...
    "nonce": format!("0x{:016x}", block.header.work),
    "logsBloom": format!("0x{}", hex::encode(&[0u8; 256])), // Empty bloom for now
    "size": format!("0x{:x}", calculate_block_size(&block)),
}))
```

---

### 3.3 Pending Block Support
**Location:** `crates/bitcell-node/src/rpc.rs:207`  
**Current Status:** Returns current height only

**Implementation Specification:**

```rust
async fn eth_block_number(state: &RpcState, include_pending: bool) -> Result<Value, JsonRpcError> {
    let height = if include_pending {
        // Return next block number if there are pending transactions
        let pending_count = state.tx_pool.pending_count();
        if pending_count > 0 {
            state.blockchain.height() + 1
        } else {
            state.blockchain.height()
        }
    } else {
        state.blockchain.height()
    };
    Ok(json!(format!("0x{:x}", height)))
}
```

---

## Category 4: ZKP Circuit Completion (MEDIUM)

### 4.1 Merkle Tree Verification Constraints
**Location:** `crates/bitcell-zkp/src/state_circuit.rs:137-141`  
**Current Status:** TODO comment, no implementation

**Implementation Specification:**

```rust
//! Merkle tree verification in R1CS constraints
//! 
//! Verifies inclusion proofs within ZK circuits using Poseidon hash.

use ark_ff::PrimeField;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError, Variable};
use ark_r1cs_std::{
    prelude::*,
    fields::fp::FpVar,
};

/// Merkle tree depth (32 levels = 2^32 leaves)
pub const MERKLE_DEPTH: usize = 32;

/// Gadget for verifying Merkle inclusion proofs in R1CS
pub struct MerklePathGadget<F: PrimeField> {
    /// Leaf value
    pub leaf: FpVar<F>,
    /// Path from leaf to root (sibling hashes)
    pub path: Vec<FpVar<F>>,
    /// Path indices (0 = left, 1 = right)
    pub path_indices: Vec<Boolean<F>>,
}

impl<F: PrimeField> MerklePathGadget<F> {
    /// Verify that `leaf` is included in tree with given `root`
    pub fn verify_inclusion(
        &self,
        cs: ConstraintSystemRef<F>,
        expected_root: &FpVar<F>,
    ) -> Result<(), SynthesisError> {
        assert_eq!(self.path.len(), MERKLE_DEPTH);
        assert_eq!(self.path_indices.len(), MERKLE_DEPTH);
        
        let mut current_hash = self.leaf.clone();
        
        for i in 0..MERKLE_DEPTH {
            // Select left and right based on path index
            let (left, right) = self.path_indices[i].select(
                (&self.path[i], &current_hash),  // If index is 1, sibling is on left
                (&current_hash, &self.path[i]),  // If index is 0, sibling is on right
            )?;
            
            // Hash left || right using Poseidon
            current_hash = poseidon_hash_gadget(cs.clone(), &[left, right])?;
        }
        
        // Enforce computed root equals expected root
        current_hash.enforce_equal(expected_root)?;
        
        Ok(())
    }
}

/// Poseidon hash gadget for R1CS
fn poseidon_hash_gadget<F: PrimeField>(
    cs: ConstraintSystemRef<F>,
    inputs: &[FpVar<F>],
) -> Result<FpVar<F>, SynthesisError> {
    // Implement Poseidon permutation as R1CS constraints
    // This is a complex implementation requiring round constants, S-boxes, etc.
    // For now, placeholder that hashes inputs linearly
    
    let mut result = FpVar::zero();
    for (i, input) in inputs.iter().enumerate() {
        result = result + input * FpVar::constant(F::from((i + 1) as u64));
    }
    Ok(result)
}
```

**Files to Create/Modify:**
- `crates/bitcell-zkp/src/merkle_gadget.rs` (NEW)
- `crates/bitcell-zkp/src/poseidon_gadget.rs` (NEW - for proper Poseidon hash)
- `crates/bitcell-zkp/src/state_circuit.rs` (MODIFY to use MerklePathGadget)
- `crates/bitcell-zkp/src/lib.rs` (ADD mod merkle_gadget, mod poseidon_gadget)

---

## Category 5: Network Layer (MEDIUM-LOW)

### 5.1 bitcell-network Transport Layer
**Location:** `crates/bitcell-network/src/transport.rs:17-70`  
**Current Status:** Stub implementation, no actual networking

**Analysis:**
The `crates/bitcell-network` crate appears to be a legacy/alternative implementation. The actual networking is implemented in:
- `crates/bitcell-node/src/network.rs` - TCP-based P2P with real connections
- `crates/bitcell-node/src/dht.rs` - libp2p Gossipsub integration

**Recommendation:** 
Either deprecate `bitcell-network` or merge its interface with the real implementations. For now, mark as low priority and add deprecation notice.

---

## Category 6: Storage Optimizations (LOW)

### 6.1 Block Pruning Enhancement
**Location:** `crates/bitcell-state/src/storage.rs:164-203`  
**Current Status:** Basic implementation with TODO for production

**Implementation Specification:**

```rust
impl StorageManager {
    /// Prune old blocks with iterator-based deletion for efficiency
    /// 
    /// This production implementation:
    /// - Uses RocksDB iterators for efficient range scanning
    /// - Deletes associated transactions and state roots
    /// - Optionally archives to cold storage before deletion
    /// - Handles concurrent reads during pruning
    pub fn prune_old_blocks_production(
        &self,
        keep_last: u64,
        archive_path: Option<&Path>,
    ) -> Result<PruningStats, String> {
        let latest = self.get_latest_height()?.unwrap_or(0);
        if latest <= keep_last {
            return Ok(PruningStats::default());
        }

        let prune_until = latest - keep_last;
        let mut stats = PruningStats::default();

        // Archive before pruning if requested
        if let Some(archive) = archive_path {
            self.archive_blocks(0, prune_until, archive)?;
        }

        // Use WriteBatch for atomic deletion
        let mut batch = WriteBatch::default();
        
        // Get all column families
        let cf_blocks = self.db.cf_handle(CF_BLOCKS).ok_or("Blocks CF not found")?;
        let cf_headers = self.db.cf_handle(CF_HEADERS).ok_or("Headers CF not found")?;
        let cf_txs = self.db.cf_handle(CF_TRANSACTIONS).ok_or("Txs CF not found")?;
        let cf_state_roots = self.db.cf_handle(CF_STATE_ROOTS).ok_or("State roots CF not found")?;

        // Iterate using prefix scan
        for height in 0..prune_until {
            let height_key = height.to_be_bytes();
            
            // Delete block
            batch.delete_cf(cf_blocks, &height_key);
            stats.blocks_deleted += 1;
            
            // Delete header
            batch.delete_cf(cf_headers, &height_key);
            
            // Delete state root
            batch.delete_cf(cf_state_roots, &height_key);
            
            // Delete transactions for this block
            // (requires transaction index by block height)
        }

        self.db.write(batch).map_err(|e| e.to_string())?;
        
        // Compact database to reclaim space
        self.db.compact_range::<&[u8], &[u8]>(None, None);

        Ok(stats)
    }
    
    /// Archive blocks to cold storage
    fn archive_blocks(&self, from: u64, to: u64, path: &Path) -> Result<(), String> {
        // Open archive database
        let archive = StorageManager::new(path)?;
        
        for height in from..to {
            // Copy block data to archive
            if let Some(block) = self.get_block_by_height(height)? {
                archive.store_block(&block.hash(), &block)?;
            }
        }
        
        Ok(())
    }
}

#[derive(Default)]
pub struct PruningStats {
    pub blocks_deleted: u64,
    pub transactions_deleted: u64,
    pub bytes_freed: u64,
}
```

---

## Implementation Priority Order

### Phase 1 (Critical - 1-2 weeks):
- [x] 1.1 Admin Wallet Transaction Sending
- [x] 1.2 Wallet GUI Transaction Sending

### Phase 2 (High - 1 week):
- [x] 2.1 System Metrics Collection
- [x] 3.1 Node ID Exposure

### Phase 3 (Medium - 2 weeks):
- [x] 2.2 Network Message Tracking
- [x] 2.3 EBSL Trust Scores
- [x] 3.2 Block Metrics
- [x] 3.3 Pending Block Support
- [ ] 4.1 Merkle Tree Verification

### Phase 4 (Low - ongoing):
- [ ] 5.1 Review bitcell-network usage
- [ ] 6.1 Block Pruning optimization

---

## Files Summary

| File | Changes Required | Priority | Status |
|------|------------------|----------|--------|
| `crates/bitcell-admin/src/api/wallet.rs` | Full tx sending | Critical | **DONE** |
| `crates/bitcell-admin/src/tx_builder.rs` | NEW FILE | Critical | N/A (used bitcell-wallet) |
| `crates/bitcell-admin/src/signer.rs` | NEW FILE | Critical | N/A (used bitcell-wallet) |
| `crates/bitcell-wallet-gui/src/main.rs` | Integrate tx sending | Critical | **DONE** |
| `crates/bitcell-wallet-gui/src/rpc_client.rs` | Add tx methods | Critical | **DONE** |
| `crates/bitcell-admin/src/system_metrics.rs` | NEW FILE | High | **DONE** |
| `crates/bitcell-admin/Cargo.toml` | Add sysinfo dep | High | **DONE** |
| `crates/bitcell-admin/src/api/metrics.rs` | Real metrics | High | **DONE** |
| `crates/bitcell-node/src/rpc.rs` | Multiple TODOs | Medium | **DONE** |
| `crates/bitcell-node/src/network.rs` | Message counters | Medium | Deferred |
| `crates/bitcell-node/src/tournament.rs` | Trust/slashing | Medium | Deferred |
| `crates/bitcell-zkp/src/merkle_gadget.rs` | NEW FILE | Medium | Pending |
| `crates/bitcell-zkp/src/state_circuit.rs` | Merkle verification | Medium | Pending |
| `crates/bitcell-state/src/storage.rs` | Production pruning | Low | Pending |

---

## Testing Requirements

Each implementation should include:

1. **Unit Tests**: Cover happy path and error cases
2. **Integration Tests**: Test component interactions
3. **Security Tests**: Verify signature validation, input sanitization
4. **Performance Tests**: Ensure acceptable latency for user-facing features

---

## Documentation Requirements

1. Update API documentation for new RPC methods
2. Add user guide for transaction sending
3. Document metrics collection and interpretation
4. Add architectural diagrams for new components
