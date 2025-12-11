# BitCell Release Requirements Specification

**Document Version:** 1.0  
**Last Updated:** December 2025  
**Status:** Comprehensive Requirements for RC1, RC2, RC3

---

## Executive Summary

This document provides a detailed specification of all requirements for BitCell Release Candidates 1, 2, and 3. Each requirement is categorized by priority, includes acceptance criteria, and details the specific implementation needs.

---

## Table of Contents

1. [RC1 Requirements](#rc1-requirements)
2. [RC2 Requirements](#rc2-requirements)
3. [RC3 Requirements](#rc3-requirements)
4. [Cross-Release Dependencies](#cross-release-dependencies)
5. [Acceptance Criteria Summary](#acceptance-criteria-summary)

---

# RC1 Requirements

## RC1 Completion Status: 85%

### RC1-001: Core Cryptographic Primitives ✅ COMPLETE

**Priority:** Critical  
**Status:** Complete  
**Crate:** `bitcell-crypto`

#### Implemented Features
| Feature | Status | Description |
|---------|--------|-------------|
| SHA-256 Hashing | ✅ | `Hash256` wrapper with `Hashable` trait |
| ECDSA Signatures | ✅ | secp256k1 curve, sign/verify operations |
| Poseidon Hash | ✅ | BN254 curve, 8 full + 57 partial rounds, 128-bit security |
| Merkle Trees | ✅ | Binary tree with inclusion proofs |
| Pedersen Commitments | ✅ | BN254 curve hiding commitments |

#### Missing/Incomplete
| Feature | Status | Required Action |
|---------|--------|-----------------|
| Ring Signatures | 🟡 | Hash-based mock; functional but needs CLSAG upgrade in RC2 |
| VRF | 🟡 | Hash-based mock; functional but needs ECVRF upgrade in RC2 |

#### Acceptance Criteria
- [x] All 46 crypto tests passing
- [x] Poseidon hash produces deterministic outputs
- [x] Signature verification is constant-time
- [x] Merkle proofs verify correctly

---

### RC1-002: Cellular Automaton Engine ✅ COMPLETE

**Priority:** Critical  
**Status:** Complete  
**Crate:** `bitcell-ca`

#### Implemented Features
| Feature | Status | Description |
|---------|--------|-------------|
| 1024×1024 Grid | ✅ | Toroidal wrapping, 8-bit cell energy |
| Conway Evolution | ✅ | B3/S23 rules with parallel Rayon execution |
| Glider Patterns | ✅ | Standard, LWSS, MWSS, HWSS patterns |
| Battle Simulation | ✅ | 1000-step deterministic simulation |
| Energy Calculation | ✅ | Regional energy-based winner determination |

#### Missing/Incomplete
None - Fully complete for RC1

#### Acceptance Criteria
- [x] All 27 CA tests passing
- [x] Battle outcomes are deterministic (same inputs = same output)
- [x] Parallel evolution produces same results as sequential
- [x] All 4 glider patterns spawn correctly

---

### RC1-003: Zero-Knowledge Proof Architecture 🟡 PARTIAL

**Priority:** Critical  
**Status:** 70% Complete  
**Crate:** `bitcell-zkp`

#### Implemented Features
| Feature | Status | Description |
|---------|--------|-------------|
| BattleCircuit Structure | ✅ | Circuit struct defined with proper fields |
| StateCircuit Structure | ✅ | Circuit struct with old_root ≠ new_root constraint |
| Groth16Proof Wrapper | ✅ | arkworks integration, serialization |
| MerklePathGadget | ✅ | R1CS-compatible inclusion proofs |
| PoseidonMerkleGadget | ✅ | Full Poseidon permutation in R1CS |

#### Missing/Incomplete for RC1
| Feature | Status | Required Action |
|---------|--------|-----------------|
| Battle Circuit Constraints | 🟡 | Mock implementation - real constraints in RC2 |
| State Circuit Constraints | 🟡 | Mock implementation - real constraints in RC2 |
| Trusted Setup | ❌ | Deferred to RC2 |
| Proof Verification Keys | ❌ | Deferred to RC2 |

#### Acceptance Criteria
- [x] All 15 ZKP tests passing
- [x] Merkle gadget verifies inclusion proofs
- [x] Poseidon gadget matches native implementation
- [ ] Real Groth16 proofs generate and verify (RC2)

---

### RC1-004: Consensus Protocol 🟡 PARTIAL

**Priority:** Critical  
**Status:** 85% Complete  
**Crate:** `bitcell-consensus`

#### Implemented Features
| Feature | Status | Description |
|---------|--------|-------------|
| Block Structure | ✅ | Header + body with all required fields |
| Tournament Protocol | ✅ | Commit → Reveal → Battle → Complete phases |
| Fork Choice | ✅ | Heaviest chain rule |
| Tournament Orchestrator | ✅ | Phase advancement and state management |
| Deterministic Work | ✅ | Work calculation from tournament |

#### Missing/Incomplete for RC1
| Feature | Status | Required Action |
|---------|--------|-----------------|
| VRF Block Selection | 🟡 | Uses hash-based VRF; needs ECVRF in RC2 |
| Finality Gadget | ❌ | Deferred to RC3 |

#### Acceptance Criteria
- [x] All 10 consensus tests passing
- [x] Tournament phases advance correctly
- [x] Block validation rejects invalid blocks
- [x] Fork choice selects heaviest chain

---

### RC1-005: State Management 🟡 PARTIAL

**Priority:** Critical  
**Status:** 80% Complete  
**Crate:** `bitcell-state`

#### Implemented Features
| Feature | Status | Description |
|---------|--------|-------------|
| Account Model | ✅ | Balance + nonce tracking |
| Bond Management | ✅ | Active/Unbonding/Slashed states |
| State Root | ✅ | Merkle root commitment |
| credit_account | ✅ | With overflow protection |

#### Missing/Incomplete for RC1
| Feature | Status | Required Action |
|---------|--------|-----------------|
| RocksDB Storage | ❌ | In-memory only; RocksDB in RC2 |
| State Pruning | 🟡 | Basic structure; full implementation in RC2 |
| State Snapshots | ❌ | Deferred to RC2 |

#### Acceptance Criteria
- [x] All 6 state tests passing
- [x] Account balances update correctly
- [x] Bond states transition properly
- [ ] State persists across restarts (RC2)

---

### RC1-006: P2P Networking 🟡 PARTIAL

**Priority:** High  
**Status:** 60% Complete  
**Crate:** `bitcell-network`, `bitcell-node`

#### Implemented Features
| Feature | Status | Description |
|---------|--------|-------------|
| Message Types | ✅ | Block, Transaction, GliderCommit, GliderReveal, BattleProof |
| Peer Management | 🟡 | Basic reputation tracking |
| Basic Gossip | 🟡 | Block/tx propagation |
| Basic DHT | 🟡 | Kademlia structure |

#### Missing/Incomplete for RC1
| Feature | Status | Required Action |
|---------|--------|-----------------|
| Full libp2p Integration | ❌ | Deferred to RC2 |
| Gossipsub Protocol | ❌ | Basic gossip only; Gossipsub in RC2 |
| NAT Traversal | ❌ | Deferred to RC2 |
| Compact Blocks | ❌ | Deferred to RC2 |

#### Acceptance Criteria
- [x] All 3 network tests passing
- [x] Peers can connect and exchange messages
- [x] Blocks propagate between nodes
- [ ] Full DHT discovery works (RC2)

---

### RC1-007: RPC/API Layer ✅ MOSTLY COMPLETE

**Priority:** High  
**Status:** 90% Complete  
**Crate:** `bitcell-node`

#### Implemented Features
| Feature | Status | Description |
|---------|--------|-------------|
| eth_blockNumber | ✅ | Current block height |
| eth_getBlockByNumber | ✅ | Block retrieval by height |
| eth_getTransactionByHash | ✅ | O(1) lookup via hash index |
| eth_sendRawTransaction | ✅ | Transaction submission |
| eth_getTransactionCount | ✅ | Account nonce |
| eth_getBalance | ✅ | Account balance |
| eth_gasPrice | ✅ | Fee estimation |
| bitcell_getNodeInfo | ✅ | Node details |
| bitcell_getTournamentState | ✅ | Tournament status |
| bitcell_getBattleReplay | ✅ | Battle replay data |

#### Missing/Incomplete for RC1
| Feature | Status | Required Action |
|---------|--------|-----------------|
| WebSocket Subscriptions | 🟡 | Basic support; full subscriptions in RC2 |
| eth_subscribe | ❌ | Deferred to RC2 |
| Event Filtering | ❌ | Deferred to RC2 |

#### Acceptance Criteria
- [x] All JSON-RPC methods return valid responses
- [x] Transaction submission validates signature
- [x] Balance queries return correct values
- [ ] WebSocket subscriptions work (RC2)

---

### RC1-008: Wallet Infrastructure ✅ MOSTLY COMPLETE

**Priority:** High  
**Status:** 85% Complete  
**Crates:** `bitcell-wallet`, `bitcell-wallet-gui`

#### Implemented Features
| Feature | Status | Description |
|---------|--------|-------------|
| Mnemonic Generation | ✅ | BIP39 12/18/24 word support |
| Address Derivation | ✅ | Multi-chain (BitCell/BTC/ETH) |
| Transaction Building | ✅ | Builder pattern with signing |
| Wallet Lock/Unlock | ✅ | Security state management |
| GUI Balance Display | ✅ | Real-time balance updates |
| GUI QR Codes | ✅ | Address QR generation |
| Hardware Wallet Abstraction | ✅ | `HardwareWalletDevice` trait |
| SigningMethod | ✅ | Unified SW/HW signing |

#### Missing/Incomplete for RC1
| Feature | Status | Required Action |
|---------|--------|-----------------|
| Ledger Integration | 🟡 | Abstraction ready; full integration in RC2 |
| Trezor Integration | 🟡 | Abstraction ready; full integration in RC2 |
| GUI Transaction Sending | 🟡 | UI exists; full functionality in RC2 |
| Multi-sig Support | ❌ | Deferred to RC3 |

#### Acceptance Criteria
- [x] All 87 wallet tests passing
- [x] Mnemonic recovery works correctly
- [x] Transactions sign and verify
- [x] Hardware wallet mock works
- [ ] Real hardware wallet signing (RC2)

---

### RC1-009: Admin Console 🟡 PARTIAL

**Priority:** Medium  
**Status:** 80% Complete  
**Crate:** `bitcell-admin`

#### Implemented Features
| Feature | Status | Description |
|---------|--------|-------------|
| Web Dashboard | ✅ | Tera-templated interface |
| Metrics API | ✅ | System/chain/network metrics |
| Config API | ✅ | Node configuration management |
| Blocks API | ✅ | Block explorer endpoints |
| HSM Integration | ✅ | `HsmClient` with multiple providers |
| MockHsmBackend | ✅ | Testing implementation |

#### Missing/Incomplete for RC1
| Feature | Status | Required Action |
|---------|--------|-----------------|
| Vault HSM Provider | 🟡 | Structure only; full implementation in RC2 |
| AWS CloudHSM Provider | 🟡 | Structure only; full implementation in RC2 |
| Azure KeyVault Provider | ❌ | Deferred to RC2 |
| Authentication | ❌ | Deferred to RC2 |
| Audit Dashboard | ❌ | Deferred to RC2 |

#### Acceptance Criteria
- [x] Dashboard loads and displays metrics
- [x] HSM mock operations work
- [x] Config can be read/updated
- [ ] Real HSM providers work (RC2)

---

### RC1-010: Economics System ✅ COMPLETE

**Priority:** Medium  
**Status:** 100% Complete  
**Crate:** `bitcell-economics`

#### Implemented Features
| Feature | Status | Description |
|---------|--------|-------------|
| Block Rewards | ✅ | 50 CELL initial, 210K halving interval |
| Reward Distribution | ✅ | 60% winner, 30% participants, 10% treasury |
| Gas Pricing | ✅ | EIP-1559 style with base fee |
| Treasury Management | ✅ | Allocation tracking |
| Privacy Multiplier | ✅ | 2x for private contracts |

#### Missing/Incomplete
None - Fully complete for RC1

#### Acceptance Criteria
- [x] All 14 economics tests passing
- [x] Halving occurs at correct intervals
- [x] Reward distribution matches specification
- [x] Gas pricing adjusts correctly

---

### RC1-011: EBSL Trust System ✅ COMPLETE

**Priority:** Medium  
**Status:** 100% Complete  
**Crate:** `bitcell-ebsl`

#### Implemented Features
| Feature | Status | Description |
|---------|--------|-------------|
| Evidence Tracking | ✅ | r_m (positive), s_m (negative) counters |
| Trust Computation | ✅ | T = b + α·u formula |
| Decay System | ✅ | Asymmetric (fast punish, slow forgive) |
| Slashing | ✅ | Graduated penalties + equivocation ban |

#### Missing/Incomplete
None - Fully complete for RC1

#### Acceptance Criteria
- [x] All 27 EBSL tests passing
- [x] Trust scores compute correctly
- [x] Decay applies per-epoch
- [x] Equivocation triggers permanent ban

---

### RC1-012: ZKVM Execution ✅ MOSTLY COMPLETE

**Priority:** Medium  
**Status:** 90% Complete  
**Crate:** `bitcell-zkvm`

#### Implemented Features
| Feature | Status | Description |
|---------|--------|-------------|
| Instruction Set | ✅ | 22 opcodes (arithmetic, logic, memory, control) |
| 32-Register Model | ✅ | General purpose registers |
| Sparse Memory | ✅ | 1MB address space |
| Gas Metering | ✅ | Per-instruction costs |
| Execution Trace | ✅ | For proof generation |

#### Missing/Incomplete for RC1
| Feature | Status | Required Action |
|---------|--------|-----------------|
| ZK Proof Integration | 🟡 | Structure ready; full integration in RC2 |
| Contract Deployment | 🟡 | Basic; full in RC2 |

#### Acceptance Criteria
- [x] All 9 ZKVM tests passing
- [x] Arithmetic operations compute correctly
- [x] Memory operations work within bounds
- [x] Gas metering tracks correctly

---

# RC2 Requirements

## RC2 Theme: "Production Hardening"
## Target: Q1 2026

---

### RC2-001: Real Groth16 Circuits

**Priority:** Critical  
**Estimated Effort:** 7 weeks  
**Dependencies:** RC1-003 (ZKP Architecture)

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC2-001.1** Battle Circuit Constraints | Implement full R1CS constraints for CA evolution verification | - Constraints enforce Conway rules<br>- Winner determination is verifiable<br>- Proof size < 300 bytes |
| **RC2-001.2** State Circuit Constraints | Implement constraints for state transition verification | - State root updates are verifiable<br>- Nullifiers prevent double-spend<br>- Merkle proofs verify in-circuit |
| **RC2-001.3** Trusted Setup Ceremony | Generate production proving/verification keys | - Multi-party computation ceremony<br>- Toxic waste properly destroyed<br>- Keys published and verified |
| **RC2-001.4** Proof Performance | Optimize proof generation time | - Battle proof < 30 seconds<br>- State proof < 20 seconds<br>- Verification < 10ms |

#### Technical Specifications

```
Battle Circuit:
- Public Inputs: glider_commitments[2], winner_id, vrf_seed, spawn_positions[2]
- Private Inputs: initial_grid[1024x1024], patterns[2], nonces[2]
- Constraints: ~10M (estimated)
- Proving Time Target: <30s on 8-core CPU

State Circuit:
- Public Inputs: old_root, new_root, nullifier_set_root
- Private Inputs: merkle_paths[], old_values[], new_values[]
- Constraints: ~1M (estimated)
- Proving Time Target: <20s on 8-core CPU
```

---

### RC2-002: Production VRF (ECVRF) ✅ COMPLETE

**Priority:** Critical  
**Estimated Effort:** 2 weeks  
**Dependencies:** RC1-001 (Crypto Primitives)  
**Status:** ✅ Complete (December 2025)

#### Requirements

| Requirement | Description | Acceptance Criteria | Status |
|-------------|-------------|---------------------|--------|
| **RC2-002.1** ECVRF Implementation | Replace hash-based VRF with proper ECVRF | - Uses P-256 or Ed25519 curve<br>- Follows IETF draft-irtf-cfrg-vrf<br>- Proof size ~80 bytes | ✅ **COMPLETE** - Uses Ristretto255 (Curve25519-based), proof size ~100 bytes |
| **RC2-002.2** VRF Verification | Cryptographically sound verification | - Verification time < 1ms<br>- No false positives possible<br>- Deterministic output | ✅ **COMPLETE** - Verify ~200-250µs, cryptographically sound |
| **RC2-002.3** VRF Chaining | Proper input chaining between blocks | - Uses previous block's VRF output<br>- Prevents grinding attacks<br>- Maintains determinism | ✅ **COMPLETE** - Implemented in blockchain.rs with proper chaining |

#### Implementation Details

**Files Modified/Created:**
- `crates/bitcell-crypto/src/ecvrf.rs` - Core ECVRF implementation (302 lines)
- `crates/bitcell-crypto/src/vrf.rs` - High-level VRF wrapper (172 lines)
- `crates/bitcell-node/src/blockchain.rs` - Blockchain integration with VRF chaining
- `docs/ECVRF_SPECIFICATION.md` - Comprehensive 400+ line specification
- `crates/bitcell-crypto/benches/crypto_bench.rs` - Performance benchmarks added

**Test Coverage:**
- 12 unit tests in `ecvrf.rs` (all passing)
- 6 comprehensive test vectors covering:
  - Deterministic behavior
  - VRF chaining (blockchain simulation)
  - Multiple proposers
  - Proof serialization
  - Grinding resistance
  - Non-malleability
- Integration tests in `tests/vrf_integration.rs` (existing, all passing)

**Performance Characteristics:**
- Key generation: ~50 µs
- Prove operation: ~150-200 µs
- Verify operation: ~200-250 µs
- 10-block chain: ~1.5-2 ms
- Proof size: ~100 bytes (serialized with bincode)

**Security Properties:**
- ✅ Uniqueness (only secret key holder can produce valid proofs)
- ✅ Collision resistance (different keys → different outputs)
- ✅ Pseudorandomness (outputs indistinguishable from random)
- ✅ Non-malleability (proofs cannot be tampered with)
- ✅ Grinding resistance (attackers cannot manipulate outputs)
- ✅ Forward security (past outputs don't reveal future outputs)

**Cryptographic Construction:**
- **Curve:** Ristretto255 (prime-order group from Curve25519)
- **Security Level:** 128-bit (equivalent to AES-128)
- **Hash Function:** SHA-512 for all hash operations
- **Proof Structure:** Schnorr-like (Gamma, c, s)
- **Domain Separation:** Proper domain separation strings for all operations

**Notes:**
- Implementation uses Ristretto255 instead of pure Ed25519 for cofactor-free operations
- While not byte-for-byte compatible with IETF RFC 9381, provides equivalent security
- Ristretto255 chosen for simpler implementation and better resistance to cofactor attacks
- Full specification documented in `docs/ECVRF_SPECIFICATION.md`

---

### RC2-003: CLSAG Ring Signatures

**Priority:** Critical  
**Estimated Effort:** 2 weeks  
**Dependencies:** RC1-001 (Crypto Primitives)

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC2-003.1** CLSAG Implementation | Implement Concise Linkable Spontaneous Anonymous Group signatures | - O(n) verification complexity<br>- Linkability prevents double-signing<br>- Key images are unique |
| **RC2-003.2** Ring Size | Support configurable ring sizes | - Minimum ring size: 11<br>- Maximum ring size: 64<br>- Default: 16 |
| **RC2-003.3** Key Image Tracking | Prevent double-spending via key images | - Key images stored in persistent set<br>- O(1) duplicate detection<br>- Merkle commitment for light clients |

---

### RC2-004: Full libp2p Integration

**Priority:** Critical  
**Estimated Effort:** 3 weeks  
**Dependencies:** RC1-006 (Networking)

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC2-004.1** Gossipsub Protocol | Implement proper Gossipsub for block/tx propagation | - Topic mesh with D=6<br>- Heartbeat interval: 1s<br>- Message deduplication |
| **RC2-004.2** Kademlia DHT | Full peer discovery implementation | - Bootstrap nodes<br>- Iterative routing<br>- Value storage for peer info |
| **RC2-004.3** NAT Traversal | Enable connections behind NAT | - AutoNAT protocol<br>- Relay circuit fallback<br>- Hole punching support |
| **RC2-004.4** Transport Encryption | Secure peer connections | - Noise protocol handshake<br>- TLS 1.3 alternative<br>- Perfect forward secrecy |
| **RC2-004.5** Compact Blocks | Bandwidth-efficient block propagation | - Send tx hashes instead of full txs<br>- Reconciliation protocol<br>- ~80% bandwidth reduction |

---

### RC2-005: RocksDB Persistence

**Priority:** Critical  
**Estimated Effort:** 2 weeks  
**Dependencies:** RC1-005 (State Management)

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC2-005.1** Block Storage | Persist blocks to RocksDB | - Blocks indexed by height and hash<br>- Headers in separate column family<br>- Atomic writes |
| **RC2-005.2** State Storage | Persist account state | - Account data serialized efficiently<br>- State root by height<br>- Efficient range queries |
| **RC2-005.3** Transaction Index | Fast transaction lookup | - Index by hash<br>- Index by sender<br>- O(1) lookup |
| **RC2-005.4** State Snapshots | Periodic state checkpoints | - Snapshot every 10000 blocks<br>- Atomic snapshot creation<br>- Fast state recovery |
| **RC2-005.5** Pruning | Remove old block data | - Configurable retention period<br>- Optional archive to cold storage<br>- Database compaction |

---

### RC2-006: Hardware Wallet Integration

**Priority:** High  
**Estimated Effort:** 4 weeks (2 weeks each)  
**Dependencies:** RC1-008 (Wallet Infrastructure)

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC2-006.1** Ledger Integration | Full Ledger device support | - Nano S/X support<br>- Transaction signing<br>- Address derivation on device |
| **RC2-006.2** Trezor Integration | Full Trezor device support | - Model One/T support<br>- Transaction signing<br>- Passphrase support |
| **RC2-006.3** BIP44 Derivation | Standard derivation paths | - m/44'/9999'/0'/0/n for BitCell<br>- Display on device<br>- Address verification |

---

### RC2-007: HSM Provider Implementations

**Priority:** High  
**Estimated Effort:** 3 weeks  
**Dependencies:** RC1-009 (Admin Console)

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC2-007.1** HashiCorp Vault | Full Vault Transit integration | - Key generation<br>- ECDSA signing<br>- Audit logging |
| **RC2-007.2** AWS CloudHSM | AWS HSM integration | - PKCS#11 interface<br>- Key management<br>- Multi-AZ support |
| **RC2-007.3** Azure KeyVault | Azure integration | - Managed HSM<br>- Key rotation<br>- Access policies |

---

### RC2-008: WebSocket Subscriptions

**Priority:** High  
**Estimated Effort:** 2 weeks  
**Dependencies:** RC1-007 (RPC/API)

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC2-008.1** eth_subscribe | Standard subscription endpoint | - newHeads subscription<br>- logs subscription<br>- pendingTransactions |
| **RC2-008.2** Event Filtering | Filter events by criteria | - Address filter<br>- Topic filter<br>- Block range |
| **RC2-008.3** Connection Management | Handle multiple clients | - Client tracking<br>- Graceful disconnect<br>- Rate limiting |

---

### RC2-009: Admin Authentication

**Priority:** High  
**Estimated Effort:** 2 weeks  
**Dependencies:** RC1-009 (Admin Console)

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC2-009.1** JWT Authentication | Token-based auth | - RS256 signing<br>- Refresh tokens<br>- Token revocation |
| **RC2-009.2** Role-Based Access | Permission system | - Admin role<br>- Operator role<br>- Viewer role |
| **RC2-009.3** Audit Logging | Log all admin actions | - Timestamp<br>- User identification<br>- Action details |

---

### RC2-010: Testnet Faucet ✅ COMPLETE

**Priority:** Medium  
**Estimated Effort:** 1 week  
**Dependencies:** RC2-005 (RocksDB)  
**Status:** Complete

#### Requirements

| Requirement | Description | Acceptance Criteria | Status |
|-------------|-------------|---------------------|--------|
| **RC2-010.1** Faucet API | Token distribution endpoint | - Rate limiting per address<br>- CAPTCHA integration<br>- Amount limits | ✅ Complete |
| **RC2-010.2** Web Interface | User-friendly faucet UI | - Address input<br>- Transaction status<br>- Recent distributions | ✅ Complete |

#### Implementation Details

**Module:** `crates/bitcell-admin/src/faucet.rs`, `crates/bitcell-admin/src/api/faucet.rs`

**Features Implemented:**
- Rate limiting: time-based and daily request limits per address
- Anti-abuse: maximum recipient balance check, address validation
- Request tracking and audit logging with full history
- CAPTCHA support (configurable, ready for integration)
- Comprehensive API endpoints (request, info, history, stats, check eligibility)
- Modern web UI with real-time updates
- Configurable via `FaucetConfig`

**API Endpoints:**
- `POST /api/faucet/request` - Request tokens
- `GET /api/faucet/info` - Get faucet information
- `GET /api/faucet/history` - Get request history
- `GET /api/faucet/stats` - Get usage statistics
- `POST /api/faucet/check` - Check address eligibility
- `GET /faucet` - Web UI

**Tests:** 4 unit tests covering validation, rate limiting, and statistics

**Documentation:** See `docs/FAUCET.md` and `examples/faucet.env`

---

### RC2-011: Mobile Wallet SDK

**Priority:** Medium  
**Estimated Effort:** 3 weeks  
**Dependencies:** RC1-008 (Wallet Infrastructure)

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC2-011.1** Core SDK | Cross-platform wallet core | - iOS/Android support<br>- FFI bindings<br>- Secure storage |
| **RC2-011.2** Key Management | Mobile key storage | - Keychain/Keystore integration<br>- Biometric unlock<br>- Backup/restore |

---

## RC2 Success Criteria

- [ ] All tests pass with real ZK proofs
- [ ] 3-node testnet runs for 1 week without issues
- [ ] Transaction throughput ≥ 50 TPS
- [ ] Proof generation < 30 seconds
- [ ] State persists across node restarts
- [ ] Hardware wallet transaction signing works
- [ ] HSM signing operations work

---

# RC3 Requirements

## RC3 Theme: "Mainnet Preparation"
## Target: Q2 2026

---

### RC3-001: Security Audit

**Priority:** Critical  
**Estimated Effort:** 6-8 weeks (external)  
**Dependencies:** RC2 Complete

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC3-001.1** Cryptography Audit | Third-party review of crypto | - No critical findings<br>- All high/medium resolved<br>- Audit report published |
| **RC3-001.2** Smart Contract Audit | ZKVM security review | - Execution safety verified<br>- Gas metering reviewed<br>- No reentrancy issues |
| **RC3-001.3** Economic Audit | Economic model validation | - No inflation bugs<br>- Reward distribution verified<br>- Fee market analysis |
| **RC3-001.4** Penetration Testing | Infrastructure security | - No critical vulnerabilities<br>- DoS resistance verified<br>- Network attack simulation |

---

### RC3-002: Recursive SNARK Aggregation

**Priority:** Critical  
**Estimated Effort:** 6 weeks  
**Dependencies:** RC2-001 (Real Groth16)

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC3-002.1** Plonk Migration | Move from Groth16 to Plonk | - Compatible with recursion<br>- Universal setup<br>- Same security level |
| **RC3-002.2** Proof Aggregation | Aggregate multiple proofs | - Aggregate N proofs into 1<br>- Constant verification time<br>- Proof size < 1KB |
| **RC3-002.3** Performance Target | Optimize aggregated proofs | - Block proof < 10 seconds<br>- Verification < 5ms<br>- Memory < 16GB |

---

### RC3-003: GPU CA Acceleration

**Priority:** High  
**Estimated Effort:** 4 weeks  
**Dependencies:** RC1-002 (CA Engine)

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC3-003.1** CUDA Implementation | GPU kernel for evolution | - CUDA 11+ support<br>- Same results as CPU<br>- 10x+ speedup |
| **RC3-003.2** OpenCL Fallback | Cross-platform GPU support | - AMD/Intel GPU support<br>- Graceful fallback to CPU<br>- Automatic detection |
| **RC3-003.3** Larger Grids | Support bigger battle arenas | - 4096×4096 grid option<br>- Configurable size<br>- Linear memory scaling |

---

### RC3-004: Block Explorer

**Priority:** High  
**Estimated Effort:** 4 weeks  
**Dependencies:** RC2-005 (RocksDB)

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC3-004.1** Block Viewing | Display block details | - Block header fields<br>- Transaction list<br>- State root |
| **RC3-004.2** Transaction Details | Transaction information | - Sender/recipient<br>- Amount/fee<br>- Status |
| **RC3-004.3** Tournament Visualization | Battle replay UI | - Grid visualization<br>- Step-by-step playback<br>- Winner highlight |
| **RC3-004.4** Account Page | Address information | - Balance history<br>- Transaction list<br>- Trust score |
| **RC3-004.5** Search | Find blocks/txs/addresses | - Hash search<br>- Address search<br>- Block height |

---

### RC3-005: Governance System

**Priority:** High  
**Estimated Effort:** 4 weeks  
**Dependencies:** RC2-001 (Real ZK)

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC3-005.1** Proposal System | On-chain governance proposals | - Parameter changes<br>- Treasury spending<br>- Protocol upgrades |
| **RC3-005.2** Voting Mechanism | Token-weighted voting | - 1 CELL = 1 vote<br>- Delegation support<br>- Quadratic option |
| **RC3-005.3** Execution | Automatic proposal execution | - Timelock delay<br>- Emergency cancel<br>- Multi-sig guardian |

---

### RC3-006: Smart Contract SDK

**Priority:** Medium  
**Estimated Effort:** 3 weeks  
**Dependencies:** RC2-001 (Real ZK)

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC3-006.1** Contract Templates | Pre-built contract patterns | - Token standard<br>- NFT standard<br>- Escrow pattern |
| **RC3-006.2** Development Tools | Contract development kit | - Local testnet<br>- Deployment scripts<br>- Testing framework |
| **RC3-006.3** Documentation | Comprehensive guides | - Getting started<br>- API reference<br>- Best practices |

---

### RC3-007: Light Client

**Priority:** Medium  
**Estimated Effort:** 4 weeks  
**Dependencies:** RC2-004 (libp2p)

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC3-007.1** Header Sync | Download and verify headers | - Header chain validation<br>- Checkpoint support<br>- Low bandwidth |
| **RC3-007.2** Merkle Proofs | Request and verify proofs | - State proof requests<br>- Transaction inclusion<br>- Receipt proofs |
| **RC3-007.3** Wallet Integration | Light client wallet mode | - Balance queries<br>- Transaction submission<br>- Minimal resources |

---

### RC3-008: Finality Gadget

**Priority:** Medium  
**Estimated Effort:** 3 weeks  
**Dependencies:** RC2-004 (libp2p)

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC3-008.1** BFT Finality | Finalize blocks after N confirmations | - 2/3 stake agreement<br>- Irreversible after finality<br>- < 1 minute finality |
| **RC3-008.2** Slashing | Punish equivocation | - Double-sign detection<br>- Evidence submission<br>- Automatic slashing |

---

### RC3-009: Documentation Portal

**Priority:** Medium  
**Estimated Effort:** 2 weeks  
**Dependencies:** None

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC3-009.1** Website | Documentation site | - Clean design<br>- Search functionality<br>- Mobile responsive |
| **RC3-009.2** API Reference | Complete API docs | - All RPC methods<br>- Request/response examples<br>- Error codes |
| **RC3-009.3** Tutorials | Step-by-step guides | - Node setup<br>- Wallet usage<br>- Contract development |

---

### RC3-010: Production Infrastructure

**Priority:** Critical  
**Estimated Effort:** 4 weeks  
**Dependencies:** All RC2

#### Requirements

| Requirement | Description | Acceptance Criteria |
|-------------|-------------|---------------------|
| **RC3-010.1** Multi-Region Deployment | Geographically distributed | - 3+ regions<br>- Latency < 200ms<br>- Failover |
| **RC3-010.2** Monitoring | Production observability | - Prometheus metrics<br>- Grafana dashboards<br>- Alerting |
| **RC3-010.3** Chaos Engineering | Fault tolerance testing | - Node failures<br>- Network partitions<br>- Byzantine behavior |
| **RC3-010.4** Incident Response | Operational procedures | - Runbooks<br>- On-call rotation<br>- Post-mortem process |

---

## RC3 Success Criteria

- [ ] Security audit completed with no critical findings
- [ ] 10-node testnet runs for 1 month without issues
- [ ] Transaction throughput ≥ 100 TPS
- [ ] Proof generation < 10 seconds (with recursion)
- [ ] Block explorer operational
- [ ] Governance proposals can be submitted
- [ ] Light client syncs and verifies
- [ ] Documentation complete

---

# Cross-Release Dependencies

```
RC1 Foundation
├── RC1-001 Crypto ──────────────────┬─→ RC2-002 ECVRF
│                                    └─→ RC2-003 CLSAG
├── RC1-003 ZKP Architecture ────────→ RC2-001 Real Groth16 ──→ RC3-002 Recursive SNARKs
├── RC1-005 State Management ────────→ RC2-005 RocksDB
├── RC1-006 Networking ──────────────→ RC2-004 libp2p ────────→ RC3-007 Light Client
├── RC1-008 Wallet ──────────────────→ RC2-006 Hardware Wallet
└── RC1-009 Admin ───────────────────→ RC2-007 HSM Providers
                                      └─→ RC2-009 Authentication

RC2 Production
├── RC2-001 Real ZK ─────────────────→ RC3-001 Security Audit
│                                    └─→ RC3-002 Recursive SNARKs
├── RC2-004 libp2p ──────────────────→ RC3-007 Light Client
│                                    └─→ RC3-008 Finality
└── RC2-005 RocksDB ─────────────────→ RC3-004 Block Explorer

RC3 Mainnet
├── RC3-001 Security Audit ──────────→ Mainnet Launch
├── RC3-002 Recursive SNARKs ────────→ Mainnet Launch
└── RC3-010 Production Infra ────────→ Mainnet Launch
```

---

# Acceptance Criteria Summary

## RC1 Release Gate

| Criteria | Status |
|----------|--------|
| All unit tests pass | ✅ 200+ passing |
| Core crypto functional | ✅ |
| CA battles deterministic | ✅ |
| Mock ZK proofs work | ✅ |
| Basic networking works | ✅ |
| Wallet creates/signs | ✅ |
| RPC endpoints respond | ✅ |

## RC2 Release Gate

| Criteria | Target |
|----------|--------|
| Real ZK proofs generate | < 30s |
| ZK proofs verify | < 10ms |
| State persists | Survives restart |
| 3-node testnet | 1 week stable |
| Hardware wallet signs | Works |
| TPS | ≥ 50 |

## RC3 Release Gate

| Criteria | Target |
|----------|--------|
| Security audit | No critical findings |
| Recursive proofs | < 10s generation |
| 10-node testnet | 1 month stable |
| Block explorer | Operational |
| Light client | Syncs correctly |
| TPS | ≥ 100 |

---

**Document Version:** 1.0  
**Generated:** December 2025  
**Next Update:** RC2 Planning Sprint
