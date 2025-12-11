# BitCell Light Client

A lightweight client implementation for BitCell blockchain, designed for resource-constrained devices.

## Features

- **Header-Only Sync**: Downloads and validates only block headers, reducing bandwidth and storage requirements
- **Checkpoint Support**: Fast sync using trusted checkpoints
- **Merkle Proof Verification**: Verifies state queries via Merkle proofs from full nodes
- **Wallet Mode**: Support for balance queries and transaction submission without full state
- **Resource Efficient**: Designed to use <100MB of memory

## Architecture

The light client consists of several key components:

### Header Chain (`header_chain.rs`)
Maintains a chain of verified block headers without full block data. Implements:
- Header validation (parent links, timestamps, VRF)
- Fork choice (heaviest chain rule)
- Memory-efficient pruning
- Tip height tracking

### Checkpoint Manager (`checkpoints.rs`)
Enables fast sync by allowing the client to skip validation of ancient blocks:
- Hardcoded trusted checkpoints
- Dynamic checkpoint addition
- Checkpoint-based sync

### Sync Protocol (`sync.rs`)
Manages the synchronization process:
- Syncs from latest checkpoint
- Batch header downloads
- Progress tracking
- Status reporting

### Merkle Proof System (`proofs.rs`)
Verifies state queries without full state:
- Account balance proofs
- Account nonce proofs
- Transaction inclusion proofs
- Storage slot proofs

### Network Protocol (`protocol.rs`)
Defines messages for light client <-> full node communication:
- Header requests/responses
- State proof requests/responses
- Chain tip queries
- Transaction submission

### Light Wallet (`wallet.rs`)
Provides wallet functionality using only headers and proofs:
- Read-only mode (balance queries)
- Full mode (can sign and submit transactions)
- Account info caching
- Pending transaction tracking

## Usage

### Creating a Light Client

```rust
use bitcell_light_client::*;
use bitcell_crypto::SecretKey;
use parking_lot::RwLock;
use std::sync::Arc;

// Create header chain with genesis
let genesis = /* get genesis header */;
let config = HeaderChainConfig::default();
let header_chain = Arc::new(HeaderChain::new(genesis, config));

// Create checkpoint manager
let checkpoint_manager = Arc::new(RwLock::new(CheckpointManager::new()));

// Create sync manager
let sync = HeaderSync::new(header_chain.clone(), checkpoint_manager);

// Start syncing
sync.sync_to(target_height).await?;
```

### Creating a Light Wallet

```rust
use bitcell_light_client::*;

// Read-only wallet (balance queries only)
let wallet = LightWallet::read_only(
    public_key,
    header_chain.clone(),
    protocol.clone()
);

// Full wallet (can sign transactions)
let secret_key = Arc::new(SecretKey::generate());
let wallet = LightWallet::full(
    secret_key,
    header_chain.clone(),
    protocol.clone()
);

// Query balance (requires network connection to full node)
let balance = wallet.get_balance().await?;

// Create and submit transaction (full mode only)
let tx = wallet.create_transaction(to, amount, nonce, gas_limit, gas_price)?;
let tx_hash = wallet.submit_transaction(tx).await?;
```

### Verifying State Proofs

```rust
use bitcell_light_client::*;

// Request state proof from full node
let proof_req = StateProofRequest::balance(block_height, account_address);

// Verify proof against header chain
let header = header_chain.get_header(block_height).unwrap();
proof.verify(&header.state_root)?;

// Extract balance from proof (only if verification succeeded)
let balance = proof.extract_balance()?;
```

## Resource Usage

The light client is designed for minimal resource usage:

- **Memory**: <100MB typical usage
  - ~500 bytes per header
  - Configurable header cache (default: 10,000 headers)
  - Account info caching
  - Automatic pruning of old headers

- **Bandwidth**: Minimal
  - Only downloads headers (~500 bytes each)
  - State queries via compact Merkle proofs
  - No full block downloads

- **Storage**: Optional
  - Can operate entirely in memory
  - Optional persistent storage for headers
  - No need for full blockchain data

## Checkpoints

Checkpoints are hardcoded trusted block headers that allow fast sync. They are updated with each software release and can be dynamically added by the user.

```rust
let checkpoint = Checkpoint::new(header, "Checkpoint at height 100000".to_string());
checkpoint_manager.write().add_checkpoint(checkpoint)?;
```

## Network Protocol

The light client communicates with full nodes using the following message types:

- `GetHeaders`: Request headers in a range
- `Headers`: Response with requested headers
- `GetStateProof`: Request a Merkle proof for state
- `StateProof`: Response with proof
- `GetChainTip`: Query current tip
- `ChainTip`: Response with tip info
- `SubmitTransaction`: Submit a signed transaction
- `TransactionResult`: Result of submission

## Security

The light client maintains security through:

1. **Header Validation**: All headers are currently validated for:
   - Correct parent hash linkage
   - Increasing timestamps

   > **Warning:** Validation of VRF proofs and work calculations is **not yet implemented**. Until these checks are added, the light client is vulnerable to malicious peers providing invalid headers and state roots. Do **not** use in production or trust state proofs from untrusted sources.

2. **Merkle Proof Verification**: All state queries are verified against the state root in validated headers

3. **Checkpoint Trust**: Only trusted checkpoints are used for fast sync

4. **Fork Choice**: Follows the heaviest chain rule

## Future Enhancements

- Persistent storage for headers
- P2P networking for multi-peer sync
- Fraud proof system
- More efficient proof formats (e.g., Patricia trie proofs)
- Mobile device optimizations

## Testing

Run tests:
```bash
cargo test -p bitcell-light-client
```

All tests should pass, validating:
- Header chain management
- Checkpoint functionality
- Proof verification
- Wallet operations
- Network protocol encoding/decoding

## Compatibility

- Works on Raspberry Pi and similar resource-constrained devices
- Compatible with full BitCell nodes for proof requests
- Supports both read-only and full wallet modes
