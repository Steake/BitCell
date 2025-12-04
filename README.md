# 🌌 BitCell

**_Quantum-resistant cellular automaton tournaments meet zero-knowledge privacy in a protocol-local trust mesh_**

[![Rust](https://img.shields.io/badge/rust-1.82%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-RC1-green.svg)](https://github.com/Steake/BitCell)

> _"We don't mine blocks. We cultivate them in a Conway garden where only the fittest gliders survive."_

## What Even Is This?

BitCell is a blockchain where consensus is decided by **Conway's Game of Life tournaments**. Yes, really. No SHA-256 lottery. No boring PoS validators clicking buttons. Just pure, deterministic, beautiful cellular automaton combat.

### Core Vibes

- 🎮 **Tournament Consensus**: Miners battle with gliders in a 1024×1024 CA arena
- 🎭 **Ring Signature Anonymity**: Your glider, your battle, not your identity  
- 🧠 **Protocol-Local EBSL**: Reputation that actually means something
- 🔐 **ZK-Everything**: Private smart contracts via modular Groth16 circuits
- ⚡ **Deterministic Work**: No lottery, no variance, just skill and creativity
- 🌐 **Anti-Cartel by Design**: Random pairings + ring sigs = coordination nightmare

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

Three independent circuits:

### 1. Battle Circuit `C_battle`
**Public**: commitments, winner, seed, positions  
**Private**: initial grid, patterns, nonce  
**Verifies**: CA evolution + commitment consistency + outcome

### 2. Execution Circuit `C_exec`
**Public**: old state root, new state root, gas used  
**Private**: plaintext state, contract code, witness  
**Verifies**: ZKVM execution correctness

### 3. State Transition Circuit `C_state`
**Public**: old root, new root, nullifiers  
**Private**: Merkle paths, cleartext values  
**Verifies**: State commitment updates

**Each block** carries `N_h - 1` battle proofs + execution proofs + state proofs.

**v0.1**: Individual Groth16 proofs  
**Future**: Recursive aggregation via Plonk/STARK

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

- Rust 1.82+
- 8GB+ RAM (for large CA grids)
- Linux, macOS, or WSL2

### Build

```bash
cargo build --release
```

### Run Tests

```bash
# All tests
cargo test --all

# With output (see CA evolution)
cargo test --all -- --nocapture

# Specific module
cargo test -p bitcell-ca

# Property tests (slow but thorough)
cargo test --features proptest
```

### Benchmarks

```bash
cargo bench

# Results in target/criterion/
```

## Project Structure

```
BitCell/
├── crates/
│   ├── bitcell-crypto/     # Hash, sigs, VRF, ring sigs, commitments
│   ├── bitcell-ca/         # CA engine, grid, rules, gliders, battles
│   ├── bitcell-ebsl/       # Evidence tracking, trust scores, slashing
│   ├── bitcell-zkp/        # Groth16 circuits (battle, exec, state)
│   ├── bitcell-consensus/  # Blocks, tournament protocol, fork choice
│   ├── bitcell-state/      # State management, bonds, accounts (RocksDB)
│   ├── bitcell-zkvm/       # Private smart contract execution
│   ├── bitcell-economics/  # Rewards, fees, treasury, halving
│   ├── bitcell-network/    # libp2p, gossip, DHT, compact blocks
│   ├── bitcell-node/       # Miner/validator nodes, JSON-RPC, WebSocket
│   ├── bitcell-wallet/     # CLI wallet
│   ├── bitcell-wallet-gui/ # GUI wallet with tournament visualization
│   ├── bitcell-admin/      # Admin console with metrics
│   └── bitcell-simulation/ # Network simulation and testing
├── docs/                   # Architecture, specs, release notes
├── scripts/                # Development and testing scripts
└── tests/                  # Integration tests
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

- [ ] Recursive SNARK aggregation (transition from Groth16)
- [ ] Optimized CA simulation (SIMD, GPU?)
- [ ] Light client implementation
- [ ] Mobile wallet
- [ ] Explorer UI
- [ ] More glider patterns
- [ ] Economic modeling / simulation
- [ ] Formal verification of EBSL properties

### Coding Style

- **No god objects**: Small, composable modules
- **Test everything**: Unit + property + integration
- **Document the why**: Not just the what
- **Benchmarks matter**: Performance is a feature

## Roadmap

### v0.1 ✅ (Alpha)
- [x] Core crypto primitives (ECDSA, VRF, ring sigs, commitments)
- [x] CA engine with battles (1024×1024 grid, Conway rules, energy)
- [x] EBSL trust scores (evidence tracking, decay, slashing)
- [x] ZK circuits (battle verification, execution, state constraints)
- [x] Consensus structures (blocks, tournament, fork choice)
- [x] P2P networking (libp2p-based gossip, DHT)
- [x] Local testnet

### v0.2 ✅ (Beta)
- [x] ZKVM execution framework
- [x] Smart contract deployment (basic)
- [x] State management with RocksDB persistence
- [x] Full validator implementation
- [x] GUI Wallet with tournament visualization
- [x] Admin console with metrics

### v0.3 (Current: Release Candidate 1)
- [x] JSON-RPC and WebSocket APIs
- [x] Block reward halving mechanism (Bitcoin-style economics)
- [x] Transaction processing and mempool
- [x] Comprehensive economic parameters
- [x] Security improvements (DoS protection, gas limits)
- [ ] Light clients
- [ ] Bridge to Ethereum
- [ ] DeFi primitives

### v1.0 (Mainnet)
- [ ] Production-ready zkSNARKs (recursive aggregation)
- [ ] Governance system
- [ ] Security audit
- [ ] Optimized CA performance (SIMD/GPU)
- [ ] Mobile wallets
- [ ] Full documentation
- [ ] 🚀 Launch

## FAQ

**Q: Is this a joke?**  
A: No. We're dead serious about CA tournaments.

**Q: Can I win by just using the biggest glider?**  
A: Maybe initially, but strategy matters. Lightweight gliders can outmaneuver heavier ones.

**Q: What's the TPS?**  
A: ~100 TPS. We're not trying to be Solana. We're trying to be secure and interesting.

**Q: Why not just use PoS?**  
A: Because clicking "stake" buttons is boring. Designing glider strategies is art.

**Q: Is it quantum-resistant?**  
A: CA evolution is fundamentally quantum-resistant. We use classical crypto for signatures, but that's upgradable.

**Q: Can I run this on a Raspberry Pi?**  
A: Validator: probably not (ZK proving is heavy). Light client: yes.

**Q: What's the energy consumption?**  
A: Way less than Bitcoin. CA simulation is deterministic and parallelizable.

## Security

**Status**: Pre-audit alpha. DO NOT use in production.

Found a bug? Email security@bitcell.network or open a private advisory.

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

- **Spec**: See the v1.1 specification document for full protocol details
- **Discord**: https://discord.gg/bitcell (coming soon)
- **Twitter**: https://twitter.com/bitcell_net (coming soon)

---

_Built with 🦀 Rust, ⚡ zkSNARKs, and 🎮 Conway's Game of Life_

_"In a world of hash lotteries, be a glider strategist."_
