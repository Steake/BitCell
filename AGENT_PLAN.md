# AI Agent Implementation Plan: BitCell RC1 Readiness

This plan outlines a linear, dependency-aware workflow for an AI agent to bring the BitCell codebase to RC1 readiness. Execute these steps in order.

---

## Phase 1: Core Functionality (Transactions & State)

### 1.1. Implement Transaction Building in Wallet GUI
**Goal**: Enable users to create real transactions.
**Files**: `crates/bitcell-wallet-gui/src/main.rs`
**Action**:
1.  Locate `on_send_transaction` callback.
2.  Replace the `mock_tx` string formatting with actual `Transaction` struct construction.
3.  Use `rpc_client.get_nonce(from_address)` to get the correct nonce.
4.  Sign the transaction using the wallet's private key.
5.  Serialize the transaction (bincode/hex).
6.  Call `rpc_client.send_raw_transaction(hex_tx)`.

### 1.2. Implement Raw Transaction Decoding in RPC
**Goal**: Process incoming raw transactions.
**Files**: `crates/bitcell-node/src/rpc.rs`
**Action**:
1.  In `bitcell_sendRawTransaction`:
2.  Decode the hex string into bytes.
3.  Deserialize bytes into `Transaction` struct.
4.  Validate signature and nonce.
5.  Add to `TxPool` (via `node.tx_pool`).
6.  Return the transaction hash.

### 1.3. Implement Balance Fetching in RPC
**Goal**: Return real account balances.
**Files**: `crates/bitcell-node/src/rpc.rs`
**Action**:
1.  In `eth_getBalance`:
2.  Parse the address (PublicKey).
3.  Access `node.blockchain.state`.
4.  Call `state.get_account(address)`.
5.  Return `account.balance` (or 0 if not found).

### 1.4. Implement State Persistence (RocksDB)
**Goal**: Persist state across restarts.
**Files**: `crates/bitcell-state/src/storage.rs`, `crates/bitcell-state/Cargo.toml`
**Action**:
1.  Add `rocksdb` dependency to `bitcell-state`.
2.  Implement `Storage` trait using RocksDB.
3.  Store `Account` and `BondState` data serialized.
4.  Update `StateManager` to use this persistent storage instead of `HashMap`.

---

## Phase 2: Security & Verification

### 2.1. Implement VRF for Block Proposers
**Goal**: Randomize block proposer selection.
**Files**: `crates/bitcell-node/src/blockchain.rs`, `crates/bitcell-crypto/src/vrf.rs` (create if needed)
**Action**:
1.  Implement ECVRF (Elliptic Curve Verifiable Random Function) using `schnorrkel` or similar.
2.  In `produce_block`:
    - Generate VRF output using previous block's VRF output as input.
    - Store `vrf_output` and `vrf_proof` in `BlockHeader`.
3.  In `validate_block`:
    - Verify the `vrf_proof` against the proposer's public key.

### 2.2. Implement ZKP Circuits (Basic Verification)
**Goal**: Verify battle outcomes cryptographically.
**Files**: `crates/bitcell-zkp/src/battle_circuit.rs`, `crates/bitcell-zkp/src/lib.rs`
**Action**:
1.  Update `Groth16Proof` struct to hold real proof data (Bellman/Arkworks).
2.  In `BattleCircuit`:
    - Define constraints for CA evolution (start state -> end state).
    - Even a simplified version checking hash consistency is better than mock.
3.  Update `generate_proof` to actually run the setup and prove.
4.  Update `verify` to run the verifier.

---

## Phase 3: Networking

### 3.1. Integrate libp2p Gossipsub
**Goal**: Efficient block/tx propagation.
**Files**: `crates/bitcell-network/src/transport.rs`, `crates/bitcell-network/Cargo.toml`
**Action**:
1.  Ensure `libp2p` features `gossipsub` are enabled.
2.  Initialize `Gossipsub` behaviour in the swarm.
3.  Subscribe to topics: `blocks`, `transactions`, `consensus`.
4.  Implement `broadcast_block` and `broadcast_transaction` to publish to these topics.
5.  Handle incoming gossip messages in the event loop.

---

## Phase 4: Observability

### 4.1. Real-Time Metrics Collection
**Goal**: Populate admin dashboard with real data.
**Files**: `crates/bitcell-admin/src/api/metrics.rs`
**Action**:
1.  Inject `Arc<Node>` or `Arc<MetricsRegistry>` into the admin API state.
2.  In `get_metrics`:
    - Read `uptime` from node start time.
    - Read `block_height` from blockchain.
    - Read `peer_count` from network manager.
    - Calculate `average_block_time` from recent blocks.

### 4.2. Connect Block Explorer
**Goal**: Show real chain data.
**Files**: `crates/bitcell-admin/src/api/blocks.rs`
**Action**:
1.  In `get_blocks`:
    - Iterate backwards from current height.
    - Fetch blocks from `blockchain`.
    - Map to API response format.
2.  In `get_block_by_hash`:
    - Lookup block in `blockchain`.

---

## Phase 5: Polish & Cleanup

### 5.1. Replace Panic with Result
**Goal**: Robust error handling.
**Files**: `crates/bitcell-ca/src/grid.rs`, `crates/bitcell-state/src/bonds.rs`
**Action**:
1.  Search for `panic!`.
2.  Change function signature to return `Result<T, Error>`.
3.  Propagate errors up the stack.

### 5.2. Expose Node Identity & Reputation
**Goal**: Complete RPC API.
**Files**: `crates/bitcell-node/src/rpc.rs`
**Action**:
1.  In `getNodeInfo`, return actual `local_peer_id`.
2.  Implement `bitcell_getReputation` by querying `TournamentManager`.

### 5.3. Hex Parsing Utils
**Goal**: Clean code.
**Files**: `crates/bitcell-node/src/rpc.rs`
**Action**:
1.  Use `hex::decode` consistently instead of manual string slicing.
2.  Add proper error handling for invalid hex strings.

---

## Execution Strategy

1.  **Sequential Execution**: Follow the phases in order (1 -> 5).
2.  **Verification**: Run `cargo test` after each major component implementation.
3.  **Integration**: Run the full node and wallet to verify end-to-end functionality (e.g., send a tx and see it in the explorer).
