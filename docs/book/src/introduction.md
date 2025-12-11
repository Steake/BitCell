# BitCell Documentation

Welcome to the **BitCell** documentation portal! BitCell is a revolutionary blockchain protocol where consensus is decided by Conway's Game of Life tournaments, combining deterministic cellular automaton battles with zero-knowledge privacy and protocol-local trust.

## What is BitCell?

BitCell replaces traditional proof-of-work mining with something truly unique: **tournament consensus**. Instead of grinding hashes, miners compete by designing creative glider patterns that battle in a 1024×1024 cellular automaton arena. The winner of each tournament earns the right to propose the next block.

### Core Features

- 🎮 **Tournament Consensus**: Miners battle with gliders in Conway's Game of Life
- 🎭 **Ring Signature Anonymity**: Privacy-preserving tournament participation
- 🧠 **Protocol-Local EBSL**: Evidence-based trust without external oracles
- 🔐 **ZK-Everything**: Private smart contracts via Groth16 circuits
- ⚡ **Deterministic Work**: No lottery, no variance, just skill and creativity
- 🌐 **Anti-Cartel Design**: Random pairings prevent miner coordination

## Quick Navigation

### For Users
- **[Quick Start](./getting-started/quick-start.md)** - Get up and running in 5 minutes
- **[Wallet Guide](./wallet/cli-wallet.md)** - Manage your CELL tokens
- **[First Transaction](./getting-started/first-transaction.md)** - Send your first transaction

### For Node Operators
- **[Running a Node](./node/running-node.md)** - Join the network as a validator
- **[Miner Setup](./node/miner-setup.md)** - Participate in tournaments
- **[Network Configuration](./node/network-config.md)** - Configure your node

### For Developers
- **[Smart Contracts](./contracts/zkvm-overview.md)** - Build private dApps
- **[API Reference](./api/overview.md)** - Integrate with BitCell
- **[Contract Examples](./contracts/examples.md)** - Token, NFT, and escrow templates

### For Researchers
- **[Tournament Consensus](./concepts/tournament-consensus.md)** - How the protocol works
- **[EBSL Trust System](./concepts/ebsl-trust.md)** - Protocol-local reputation
- **[ZK Proofs](./concepts/zk-proofs.md)** - Privacy and verification

## Why BitCell?

Traditional blockchains rely on either energy-intensive hash grinding (PoW) or plutocratic voting (PoS). BitCell introduces a third way:

1. **Emergent Complexity**: Tournament outcomes depend on glider pattern creativity
2. **Provable Computation**: All battles are deterministic and zero-knowledge verified
3. **Fair Competition**: Random pairings and ring signatures prevent cartels
4. **Interesting Work**: Designing battle strategies is genuinely engaging

## Key Concepts

### Tournament Protocol

Each block height runs a bracket-style tournament:

1. Eligible miners commit glider patterns (ring-signed)
2. VRF generates random tournament pairings
3. Miners reveal patterns and battle in Conway's Game of Life
4. Winner proves battle validity via ZK-SNARK
5. Tournament champion proposes the next block

### EBSL Trust Scores

Every miner has a trust score computed from evidence:

- **Positive evidence** (r_m): Good blocks, honest participation
- **Negative evidence** (s_m): Invalid blocks, missed reveals

Trust score: `T = r/(r+s+K) + α·K/(r+s+K)`

New miners start below eligibility threshold and must build reputation.

### Private Smart Contracts

BitCell's ZKVM enables private smart contract execution:

- Contract state is encrypted with Pedersen commitments
- Execution happens off-chain with zero-knowledge proofs
- Validators verify proofs without seeing plaintext data
- Full composability with other contracts

## Status & Roadmap

**Current Status**: Release Candidate 1 (RC1)

- ✅ Core consensus and CA engine
- ✅ EBSL trust system
- ✅ Basic ZK circuits
- ✅ CLI and GUI wallets
- ✅ JSON-RPC and WebSocket APIs
- 🚧 Full smart contract SDK
- 🚧 Light clients
- 🚧 Mainnet preparation

See the [Changelog](./appendix/changelog.md) for detailed version history.

## Getting Help

- **Search**: Use the search box above to find what you need
- **GitHub**: [Report issues](https://github.com/Steake/BitCell/issues)
- **Community**: Join our [Discord](https://discord.gg/bitcell) (coming soon)

## License

BitCell is dual-licensed under MIT / Apache 2.0. Choose whichever makes your lawyer happier.

---

Ready to dive in? Start with the [Quick Start Guide](./getting-started/quick-start.md) or explore the sections in the sidebar.
