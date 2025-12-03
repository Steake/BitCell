# BitCell RC1 Release Notes

**Version:** 0.1.0-rc1  
**Release Date:** December 2025  
**Codename:** "Genesis"

---

## Overview

BitCell RC1 is the first release candidate of the BitCell blockchain platform, featuring a complete implementation of the core consensus mechanism, cryptographic primitives, and networking infrastructure. This release represents a significant milestone in the development of a blockchain system that combines cellular automata-based mining with zero-knowledge proof verification.

---

## Key Features

### 1. Consensus & Block Production

#### VRF-Based Block Proposer Selection
- Implemented Verifiable Random Function (VRF) for fair block proposer selection
- Proper VRF chaining using previous block's VRF output as input
- Cryptographic verification of VRF proofs in block validation
- Deterministic yet unpredictable proposer selection

#### Block Rewards & Economic System
- Bitcoin-style block reward halving mechanism
  - Initial reward: 50 CELL
  - Halving interval: 210,000 blocks
  - Maximum halvings: 64 (defined in `MAX_HALVINGS` constant)
- `credit_account` method with overflow protection using `checked_add`
- Centralized economic constants in `bitcell-economics/src/constants.rs`

### 2. Zero-Knowledge Proofs

#### State Circuit
- Groth16 proof generation and verification using arkworks
- Non-equality constraint enforcement (`old_root != new_root`) via `diff * inv = 1`
- Circuit setup returns `Result` instead of panicking
- Public inputs: old state root, new state root, nullifier

#### Battle Circuit
- Conway's Game of Life evolution verification
- Cell position and state constraints
- Winner determination constraints

#### Merkle Tree Verification (NEW in RC1)
- `MerklePathGadget` for R1CS inclusion proofs
- Configurable tree depth (default: 32 levels = 2^32 leaves)
- Algebraic hash function H(a,b) = a*(b+1) + b^2 with documented security properties
- Collision resistance and one-wayness within R1CS context
- Efficient constraint generation (~5 constraints per tree level)
- Test coverage for various tree depths including collision resistance tests

### 3. Networking

#### libp2p Gossipsub Integration
- Decentralized block and transaction broadcasting
- Topic-based message propagation
- Peer discovery via mDNS

#### DHT Support
- Kademlia DHT for peer discovery
- Consistent logging with `tracing` crate
- Error handling for channel failures

#### Network Metrics
- Message sent/received counters
- Peer connection tracking
- Trust score aggregation

### 4. Storage

#### RocksDB Backend
- Persistent storage for blocks, headers, accounts, bonds
- Column family organization for efficient queries
- State root tracking by height

#### Production Block Pruning (NEW in RC1)
- `prune_old_blocks_production` method with:
  - Atomic batch writes
  - Optional archiving to cold storage
  - Associated data cleanup (transactions, state roots)
  - Database compaction after pruning
  - Detailed `PruningStats` return value

### 5. RPC & API

#### JSON-RPC Methods
| Method | Description |
|--------|-------------|
| `eth_blockNumber` | Get current block height |
| `eth_getBlockByNumber` | Get block by height |
| `eth_getTransactionByHash` | O(1) transaction lookup via hash index |
| `eth_sendRawTransaction` | Submit signed transaction |
| `eth_getTransactionCount` | Get account nonce |
| `eth_gasPrice` | Get current gas price (default: 1 Gwei) |
| `bitcell_getNodeInfo` | Get node ID, version, type |
| `bitcell_getTournamentState` | Get tournament status |
| `bitcell_getBattleReplay` | Get battle replay data |
| `bitcell_getPendingBlockInfo` | Get pending block information |

#### Admin API
- System metrics endpoint (`/api/metrics/system`)
  - CPU usage (average across cores)
  - Memory usage (MB)
  - Disk usage (MB)
  - Process uptime
- Transaction sending (NOT_IMPLEMENTED - security review pending)

### 6. Wallet

#### GUI Features
- Balance display and refresh
- Address QR code generation
- Transaction history
- Tournament visualization
- RPC connection status indicator

#### RPC Client
- `get_balance` - Query account balance
- `get_transaction_count` - Query account nonce
- `send_raw_transaction` - Submit transactions
- `get_gas_price` - Query fee estimation
- `get_tournament_state` - Query tournament data

---

## Breaking Changes

### API Changes
- `StateCircuit::setup()` now returns `Result<(ProvingKey, VerifyingKey), Error>`
- `BattleCircuit::setup()` now returns `Result<(ProvingKey, VerifyingKey), Error>`
- Removed `Serialize`/`Deserialize` derives from circuit structs (incompatible with `Option<Fr>`)
- `credit_account` now returns `Result<Hash256, Error>` instead of `Hash256`

### Module Changes
- `bitcell-network` crate deprecated (see deprecation notice)
  - Production networking in `bitcell-node/src/network.rs`
  - DHT implementation in `bitcell-node/src/dht.rs`

---

## Security Improvements

### Error Handling
- Lock poisoning recovery with proper `tracing::error!` logging
- Storage errors logged instead of silently ignored
- Transaction nonce validation allows new accounts (nonce 0)

### Input Validation
- Address format validation in RPC endpoints
- Transaction signature verification
- Balance overflow protection
- **Gas bounds validation** - Max gas price (10,000 Gwei) and gas limit (30M) to prevent overflow attacks

### DoS Protection (NEW in RC1)
- Transactions from new accounts require non-zero gas price and limit
- Upper bounds on gas values prevent resource exhaustion
- Signature verification prevents random spam

### Admin API Security (NEW in RC1)
- Private key transaction signing is disabled by default
- Requires explicit `insecure-tx-signing` feature flag to enable
- Clear warnings about production use and secure alternatives
- Endpoint returns `NOT_IMPLEMENTED` when feature is disabled

### VRF Race Condition Fix (NEW in RC1)
- VRF proof generation now holds the blocks read lock
- Prevents race conditions between reading VRF input and using it
- Ensures consistency in block production

### Logging
- Replaced all `println!`/`eprintln!` with `tracing::{info,debug,error}`
- Structured logging for better filtering and analysis
- Full public key logging for debugging storage issues

---

## Performance Optimizations

### Transaction Lookup
- O(1) transaction lookup via `HashMap<Hash256, TxLocation>` index
- Replaces O(n*m) linear scan of blocks

### Block Metrics
- Static `EMPTY_BLOOM_FILTER` constant (avoids per-request allocation)
- Real block size calculation via `bincode`

---

## Testing

### Test Coverage
- 26+ tests passing across all crates
- ZKP circuit tests (state, battle, merkle)
- Storage tests (creation, header storage, pruning)
- Network tests (peer management)
- RPC client tests (serialization, parsing)

### Test Commands
```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p bitcell-node
cargo test -p bitcell-zkp
cargo test -p bitcell-state
```

---

## Known Issues & Limitations

### Not Yet Implemented
1. **Admin Wallet Transaction Signing** - Disabled by default via feature flag for security
   - Enable with `--features insecure-tx-signing` (testing only)
   - Production use requires HSM or hardware wallet integration
2. **Wallet GUI Transaction Sending** - Displays "coming soon" message
3. **Full Poseidon Hash** - Current algebraic hash is secure for R1CS but Poseidon recommended for maximum security

### Known Bugs
- None critical in RC1

### Platform Support
- Linux (primary)
- macOS (tested)
- Windows (experimental)

---

## Migration Guide

### From Pre-RC1

1. **Update Circuit Calls**
   ```rust
   // Before
   let (pk, vk) = StateCircuit::setup();
   
   // After
   let (pk, vk) = StateCircuit::setup()?;
   ```

2. **Update credit_account Calls**
   ```rust
   // Before
   state_manager.credit_account(pubkey, amount);
   
   // After
   state_manager.credit_account(pubkey, amount)?;
   ```

3. **Update Logging**
   ```rust
   // Before
   println!("Info: {}", msg);
   eprintln!("Error: {}", err);
   
   // After
   tracing::info!("Info: {}", msg);
   tracing::error!("Error: {}", err);
   ```

---

## Dependencies

### Core Dependencies
| Crate | Version | Purpose |
|-------|---------|---------|
| ark-groth16 | 0.4.0 | Groth16 proofs |
| ark-bn254 | 0.4.0 | BN254 curve |
| libp2p | 0.53.2 | P2P networking |
| rocksdb | 0.22.0 | Storage backend |
| tokio | 1.x | Async runtime |
| axum | 0.7.x | HTTP server |
| sysinfo | 0.30.x | System metrics |

---

## Documentation

- [Architecture Overview](./docs/ARCHITECTURE.md)
- [RPC API Specification](./docs/RPC_API_Spec.md)
- [Implementation Specification](./docs/IMPLEMENTATION_SPEC.md)

---

## Contributors

- Core Development Team
- Community Contributors

---

## License

MIT License - See LICENSE file for details.

---

## Next Steps (RC2)

1. Implement full Poseidon hash for production Merkle verification
2. Enable wallet GUI transaction sending with hardware wallet support
3. Add HSM/secure key management integration for admin wallet
4. Performance benchmarking and optimization
5. Third-party security audit
6. Testnet deployment with monitoring

---

## Feedback

Please report issues and feedback via GitHub Issues:
https://github.com/Steake/BitCell/issues
