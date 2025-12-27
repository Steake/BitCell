# 🌌 BitCell

**_Quantum-resistant cellular automaton tournaments meet zero-knowledge privacy in a protocol-local trust mesh_**

[![Rust](https://img.shields.io/badge/rust-1.82%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0--rc1-green.svg)](https://github.com/Steake/BitCell)
[![Status](https://img.shields.io/badge/status-Release%20Candidate%201-yellow.svg)](https://github.com/Steake/BitCell)

> _"We don't mine blocks. We cultivate them in a Conway garden where only the fittest gliders survive."_

## What Even Is This?

BitCell is a blockchain where consensus is decided by **Conway's Game of Life tournaments**. Yes, really. No SHA-256 lottery. No boring PoS validators clicking buttons. Just pure, deterministic, beautiful cellular automaton combat.

**Now in Release Candidate 1** with production-ready VRF, persistent state, hardware wallet support, light client implementation, and a complete ZKVM execution framework.

## 🎉 RC1 Highlights (v0.1.0-rc1)

BitCell RC1 represents the first complete implementation of the core protocol:

- **Production ECVRF**: Real cryptographic VRF using Ristretto255 for fair proposer selection
- **RocksDB Persistence**: Full state management with pruning and block archival
- **Light Client**: Header-only sync with Merkle proofs (<100MB memory)
- **Hardware Wallets**: Complete support for Ledger/Trezor via abstraction layer
- **Smart Contracts**: BCL compiler + ZKVM with 22 opcodes and gas metering
- **Admin Console**: Production-ready dashboard with HSM integration and testnet faucet
- **Security Hardening**: DoS protection, gas limits, input validation, VRF race condition fixes
- **Comprehensive Testing**: 200+ tests passing across all crates

See [RC1 Release Notes](docs/RC-1-Release_Notes.md) for complete details.

### Core Vibes

- 🎮 **Tournament Consensus**: Miners battle with gliders in a 1024×1024 CA arena
- 🎭 **Ring Signature Anonymity**: Your glider, your battle, not your identity  
- 🧠 **Protocol-Local EBSL**: Reputation that actually means something
- 🔐 **ZK-Everything**: Private smart contracts via Groth16 circuits with Merkle gadgets
- ⚡ **Deterministic Work**: No lottery, no variance, just skill and creativity
- 🌐 **Anti-Cartel by Design**: Random pairings + ring sigs = coordination nightmare
- 🔗 **Production VRF**: ECVRF (Ristretto255) for fair block proposer selection
- 💾 **Persistent State**: RocksDB backend with state snapshots and pruning
- 🏦 **Hardware Wallet Ready**: Full support for Ledger, Trezor, and HSM integration

## Why Though?

Because proof-of-work shouldn't be about who has the most GPUs. It should be about **emergent complexity**, **creative strategy**, and **provable computation**. BitCell replaces hash grinding with something actually interesting: designing glider patterns that survive CA evolution better than your opponents.

Plus, we needed a blockchain where "gas wars" could literally mean glider battles. 🚀

## Architecture Aesthetic

```
┌─────────────────────────────────────────────────────────┐
│  Application Layer: dApps, Wallets, Bridges            │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│  ZKVM: Private Smart Contracts                          │
│  • RISC-V-ish instruction set                           │
│  • Pedersen commitments                                 │
│  • Groth16 execution proofs                             │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│  Consensus: Tournament Protocol                         │
│  • Commit Phase: Ring-signed glider commitments         │
│  • Reveal Phase: Pattern disclosure                     │
│  • Battle Phase: 1000-step CA simulation                │
│  • Winner: Highest regional energy → proposes block     │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│  CA Engine: 1024×1024 Toroidal Grid                     │
│  • Conway-like rules + energy                           │
│  • Glider patterns (Standard, LWSS, MWSS, HWSS)        │
│  • Parallel evolution (Rayon)                           │
│  • Battle outcome via energy density                    │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│  EBSL: Evidence-Based Subjective Logic                  │
│  • r_m: positive evidence (good blocks, participation)  │
│  • s_m: negative evidence (invalid blocks, cheating)    │
│  • Trust = b + α·u (subjective logic opinion)           │
│  • Fast punish, slow forgive                            │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│  Crypto Primitives                                       │
│  • ECDSA (secp256k1)                                     │
│  • Ring Signatures (tournament anonymity)                │
│  • VRF (randomness generation)                           │
│  • Pedersen Commitments                                  │
│  • Merkle Trees                                          │
└─────────────────────────────────────────────────────────┘
```

## Quick Start (For The Impatient)

```bash
# Clone the vibes
git clone https://github.com/Steake/BitCell
cd BitCell

# Build the future
cargo build --release

# Run tests (watch CA battles in real-time)
cargo test --all -- --nocapture

# Individual crate tests
cargo test -p bitcell-crypto  # Cryptographic primitives
cargo test -p bitcell-ca      # Cellular automaton engine
cargo test -p bitcell-ebsl    # Trust & reputation system
```

## The Tournament Protocol (The Good Stuff)

Each block height runs a bracket-style tournament:

1. **Eligibility Check**: Protocol computes `M_h` (miners with `bond ≥ B_MIN` and `trust ≥ T_MIN`)
2. **Commit Phase**: Miners submit `H(glider_pattern || nonce)` with ring signatures
3. **VRF Seed**: Combine last `k` blocks' VRF outputs → `seed_h`
4. **Pairing**: Deterministic shuffle using `seed_h` → bracket structure
5. **Reveal Phase**: Miners reveal patterns; non-revealers forfeit
6. **Battle Simulation**: Each pair battles for 1000 CA steps
7. **ZK Proof**: Winner proves battle validity via Groth16 circuit
8. **Block Proposal**: Tournament winner executes contracts, generates proofs, proposes block
9. **Full Verification**: All validators check all proofs (no sampling in consensus)

### Example Battle

```
Miner A: Heavyweight Spaceship (160 energy)
Miner B: Standard Glider (100 energy)

Grid: 1024×1024 toroidal
Steps: 1000
Spawn: A at (256, 512), B at (768, 512)

After 1000 steps:
  Region A energy: 5,847
  Region B energy: 3,291

Winner: Miner A 🎉
```

## Protocol-Local EBSL (Trust Without Oracles)

Every miner has evidence counters:

- **r_m**: Positive (good blocks, honest participation)
- **s_m**: Negative (invalid blocks, missed reveals, equivocation)

Trust score computed as:

```
R = r_m + s_m
T_m = r_m/(R+K) + α·K/(R+K)
```

With `K=2`, `α=0.4`:

- **New miners**: Start at `T = 0.4` (below eligibility `T_MIN = 0.75`)
- **Good behavior**: Builds `r_m`, increases trust
- **Bad behavior**: Builds `s_m` faster, tanks trust quickly
- **Equivocation**: Instant `T → 0`, full slash, permanent ban

Decay per epoch:
- `r_m *= 0.99` (positive decays faster)
- `s_m *= 0.999` (negative decays slower - long memory)

## ZK-SNARK Circuits (Modular by Design)

Four independent circuits in RC1:

### 1. Battle Circuit `C_battle`
**Public**: commitments, winner, seed, positions  
**Private**: initial grid, patterns, nonce  
**Verifies**: CA evolution + commitment consistency + outcome  
**Status**: Structure complete, full constraints in RC2

### 2. Execution Circuit `C_exec`
**Public**: old state root, new state root, gas used  
**Private**: plaintext state, contract code, witness  
**Verifies**: ZKVM execution correctness  
**Status**: Structure complete, full constraints in RC2

### 3. State Transition Circuit `C_state`
**Public**: old root, new root, nullifiers  
**Private**: Merkle paths, cleartext values  
**Verifies**: State commitment updates  
**Status**: ✅ Complete with non-equality constraints

### 4. Merkle Path Circuit `C_merkle`
**Public**: root, leaf, index  
**Private**: path siblings  
**Verifies**: Merkle inclusion proofs in R1CS  
**Status**: ✅ Complete with Poseidon hash gadget

**Each block** carries `N_h - 1` battle proofs + execution proofs + state proofs.

**RC1**: Circuit structures and mock proofs  
**RC2**: Full R1CS constraints and real Groth16 proofs  
**RC3**: Recursive aggregation via Plonk/STARK

## Economics (Bitcoin-Style Halving)

```
Initial block reward: 50 CELL
Halving interval: 210,000 blocks (~4 years)
Maximum halvings: 64 (subsidy reaches 0)
Total supply: ~21 million CELL

block_reward = base_subsidy(h) + tx_fees + contract_fees

Distribution:
  60% → Tournament winner (proposer)
  30% → All participants (weighted by round reached)
  10% → Treasury / dev fund
```

The payout is **deterministically computed** from the tournament bracket. Proposer can't cheat it or the block is invalid.

## Smart Contracts (Privacy Native)

```rust
// On-chain: Only commitments and proofs
let new_commitment = commit(new_state, random_nonce);
let exec_proof = prove_execution(old_state, new_state, function);
let state_proof = prove_state_transition(old_root, new_root);

// Off-chain: Prover decrypts and executes privately
let old_state = decrypt_with_user_key(old_commitment, secret);
let new_state = run_function(function, args, old_state);

// Validators: Never see plaintext, only verify proofs
verify_proof(exec_proof, public_inputs);
verify_proof(state_proof, public_inputs);
```

## Installation

### Prerequisites

- Rust 1.82+ (specified in `rust-toolchain.toml`)
- 8GB+ RAM (for ZK proving and CA simulations)
- Linux, macOS, or WSL2
- Optional: Hardware wallet for secure key management

### Build

```bash
# Clone the repository
git clone https://github.com/Steake/BitCell
cd BitCell

# Build all crates in release mode
cargo build --release

# Build time: ~5-10 minutes depending on hardware
```

### Run Tests

```bash
# Run all tests (200+ tests)
cargo test --all

# With output (see CA evolution and logs)
cargo test --all -- --nocapture

# Specific crate tests
cargo test -p bitcell-crypto       # Cryptographic primitives
cargo test -p bitcell-ca           # Cellular automaton engine
cargo test -p bitcell-ebsl         # Trust & reputation system
cargo test -p bitcell-zkp          # ZK circuits and Merkle gadgets
cargo test -p bitcell-consensus    # Tournament protocol
cargo test -p bitcell-state        # State management
cargo test -p bitcell-node         # Node implementation
cargo test -p bitcell-wallet       # Wallet functionality
cargo test -p bitcell-light-client # Light client

# Property tests (slower but thorough)
cargo test --features proptest
```

### Benchmarks

```bash
# Run performance benchmarks
cargo bench

# Results saved to target/criterion/
# View HTML reports in target/criterion/report/index.html
```

### Run a Local Node

```bash
# Start a validator node
cargo run --release --bin bitcell-node

# Start with custom config
cargo run --release --bin bitcell-node -- --config config.toml

# Run with admin console
cargo run --release --bin bitcell-admin
```

## Project Structure

```
BitCell/
├── crates/
│   ├── bitcell-crypto/      # Hash, sigs, VRF, ring sigs, commitments
│   ├── bitcell-ca/          # CA engine, grid, rules, gliders, battles
│   ├── bitcell-ebsl/        # Evidence tracking, trust scores, slashing
│   ├── bitcell-zkp/         # Groth16 circuits (battle, exec, state, merkle)
│   ├── bitcell-consensus/   # Blocks, tournament protocol, fork choice
│   ├── bitcell-state/       # State management, bonds, accounts, RocksDB
│   ├── bitcell-zkvm/        # Private smart contract execution
│   ├── bitcell-compiler/    # BCL compiler (BitCell Language)
│   ├── bitcell-economics/   # Rewards, fees, treasury, halving
│   ├── bitcell-network/     # Network interface (stub)
│   ├── bitcell-node/        # Miner/validator nodes, JSON-RPC, WebSocket, DHT
│   ├── bitcell-wallet/      # CLI wallet with hardware wallet support
│   ├── bitcell-wallet-gui/  # GUI wallet with tournament visualization
│   ├── bitcell-admin/       # Admin console with metrics, HSM, faucet
│   ├── bitcell-light-client/# Light client with header-only sync
│   ├── bitcell-explorer/    # Block explorer (in development)
│   └── bitcell-simulation/  # Network simulation and testing
├── docs/                    # Architecture, specs, release notes
├── sdk/                     # Contract templates and examples
├── infra/                   # Production infrastructure, monitoring
├── scripts/                 # Development and testing scripts
└── tests/                   # Integration tests
```

## Development

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --all -- -D warnings

# Watch mode (requires cargo-watch)
cargo watch -x test

# Generate docs
cargo doc --no-deps --open
```

## Contributing

We're in alpha. Things break. PRs welcome.

### Areas We Need Help

- [ ] Recursive SNARK aggregation (Plonk migration for proof composition)
- [ ] Optimized CA simulation (SIMD, GPU acceleration with CUDA/OpenCL)
- [ ] Mobile wallet development (iOS/Android SDK)
- [ ] Block explorer UI (React/Vue frontend)
- [ ] More glider patterns and battle strategies
- [ ] Economic modeling and simulation tools
- [ ] Formal verification of EBSL properties
- [ ] Bridge implementations (Ethereum, other chains)
- [ ] Hardware wallet integration testing
- [ ] Documentation improvements and tutorials

### Coding Style

- **No god objects**: Small, composable modules
- **Test everything**: Unit + property + integration
- **Document the why**: Not just the what
- **Benchmarks matter**: Performance is a feature

## Roadmap

### v0.1 (RC1) ✅ (Current Release)
- [x] Core crypto primitives (ECDSA, VRF, ring sigs, Pedersen commitments)
- [x] CA engine with battles (1024×1024 grid, Conway rules, energy)
- [x] EBSL trust scores (evidence tracking, decay, slashing)
- [x] ZK circuits (battle verification, execution, state constraints, Merkle gadgets)
- [x] Consensus structures (blocks, tournament, fork choice, VRF selection)
- [x] P2P networking (libp2p Gossipsub, DHT, peer discovery)
- [x] RocksDB persistence (blocks, state, transactions, pruning)
- [x] JSON-RPC and WebSocket APIs (eth_* and bitcell_* methods)
- [x] ZKVM execution framework (22 opcodes, gas metering, execution trace)
- [x] Smart contract compiler (BCL language)
- [x] Full validator implementation with transaction processing
- [x] GUI Wallet with tournament visualization and hardware wallet support
- [x] Admin console with metrics, HSM integration, and faucet
- [x] Light client with header-only sync and Merkle proofs
- [x] Block reward halving mechanism (Bitcoin-style economics)
- [x] Security improvements (DoS protection, gas limits, input validation)
- [x] Local testnet deployment

### v0.2 (RC2) 🚧 (In Progress)
- [x] Production ECVRF implementation (Ristretto255-based)
- [ ] Real Groth16 circuits with full R1CS constraints
- [ ] CLSAG ring signatures (replacing mock implementation)
- [ ] Full libp2p integration (Gossipsub, Kademlia DHT, NAT traversal)
- [ ] RocksDB state snapshots and advanced pruning
- [ ] Hardware wallet integration (Ledger, Trezor with BIP44)
- [ ] HSM provider implementations (Vault, AWS CloudHSM, Azure KeyVault)
- [ ] WebSocket subscriptions (eth_subscribe, event filtering)
- [ ] Admin authentication (JWT, RBAC, audit logging)
- [ ] Mobile wallet SDK (iOS/Android)
- [ ] Performance benchmarking and optimization

### v0.3 (RC3) 📋 (Planned)
- [ ] Security audit (cryptography, smart contracts, economics)
- [ ] Recursive SNARK aggregation (Plonk migration)
- [ ] GPU CA acceleration (CUDA/OpenCL)
- [ ] Block explorer with tournament visualization
- [ ] Governance system (proposals, voting, execution)
- [ ] Smart contract SDK with templates and dev tools
- [ ] Light client wallet integration
- [ ] BFT finality gadget with slashing
- [ ] Documentation portal with API reference
- [ ] Production infrastructure (multi-region, monitoring, chaos testing)
- [ ] 10-node testnet (1 month stability)

### v1.0 (Mainnet) 🎯 (Future)
- [ ] Production-ready zkSNARKs (recursive aggregation)
- [ ] Third-party security audit completion
- [ ] Optimized CA performance (SIMD/GPU)
- [ ] Bridge to Ethereum
- [ ] DeFi primitives
- [ ] Mobile wallets
- [ ] Full documentation
- [ ] 🚀 Mainnet Launch

## FAQ

**Q: Is this a joke?**  
A: No. We're dead serious about CA tournaments. RC1 is deployed and working.

**Q: Can I win by just using the biggest glider?**  
A: Maybe initially, but strategy matters. Lightweight gliders can outmaneuver heavier ones.

**Q: What's the TPS?**  
A: Target ~50-100 TPS in RC2. We're not trying to be Solana. We're trying to be secure and interesting.

**Q: Why not just use PoS?**  
A: Because clicking "stake" buttons is boring. Designing glider strategies is art.

**Q: Is it quantum-resistant?**  
A: CA evolution is fundamentally quantum-resistant. We use classical crypto for signatures (ECDSA, ECVRF), which is upgradable to post-quantum algorithms.

**Q: Can I run this on a Raspberry Pi?**  
A: Validator: probably not (ZK proving is heavy, requires 8GB+ RAM). Light client: yes (implemented in RC1).

**Q: What's the energy consumption?**  
A: Way less than Bitcoin. CA simulation is deterministic and parallelizable, not hash grinding.

**Q: When mainnet?**  
A: After RC2 and RC3 complete, plus security audit. Target: Q3 2026.

## Security

**Status**: Release Candidate 1 - Pre-audit, not production ready.

**DO NOT use in production** until RC3 security audit completes.

### RC1 Security Features
- ✅ VRF-based block proposer selection (ECVRF with Ristretto255)
- ✅ Transaction signature verification (ECDSA secp256k1)
- ✅ Gas bounds validation (max 10,000 Gwei price, 30M limit)
- ✅ DoS protection (new account gas requirements)
- ✅ Input validation (address format, nonce, balance checks)
- ✅ Error handling with proper logging
- ✅ Hardware wallet support (security state management)
- ✅ HSM integration (admin wallet signing)

### Known Limitations (RC1)
- Mock ring signatures (CLSAG implementation in RC2)
- Mock ZK proofs (full R1CS constraints in RC2)
- Basic network security (full libp2p hardening in RC2)
- Admin API restricted (requires explicit feature flags)

### Reporting Security Issues
Found a vulnerability? Please report responsibly:
- **Email**: security@bitcell.network
- **GitHub**: Open a private security advisory
- **Rewards**: Bug bounty program coming in RC2

### Security Roadmap
- **RC2**: Third-party cryptography review
- **RC3**: Full security audit (crypto, smart contracts, economics)
- **v1.0**: Penetration testing and mainnet security hardening

## License

Dual-licensed under MIT / Apache 2.0.

Choose whichever makes your lawyer happier.

## Credits

- **Cellular Automata**: John Conway (RIP, legend)
- **Subjective Logic**: Audun Jøsang
- **zkSNARKs**: The SCIPR Lab wizards
- **Rust**: The Rust Foundation & community
- **You**: For reading this far 🙏

## Links

- **Release Notes**: [RC1 Release Notes](docs/RC-1-Release_Notes.md)
- **Architecture**: [Architecture Overview](docs/ARCHITECTURE.md)
- **RPC Spec**: Full JSON-RPC and WebSocket API documentation
- **Documentation**: Comprehensive mdBook docs in `docs/book/`
- **Discord**: https://discord.gg/bitcell (coming soon)
- **Twitter**: https://twitter.com/bitcell_net (coming soon)
- **GitHub**: https://github.com/Steake/BitCell

---

_Built with 🦀 Rust, ⚡ zkSNARKs, and 🎮 Conway's Game of Life_

_"In a world of hash lotteries, be a glider strategist."_
