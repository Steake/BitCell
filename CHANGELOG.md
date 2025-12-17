# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Governance System (RC3-005)**: Complete on-chain governance implementation
  - Proposal system supporting parameter changes, treasury spending, and protocol upgrades
  - Token-weighted voting with linear (1 CELL = 1 vote) and quadratic (√CELL = votes) methods
  - Vote delegation system for representative democracy
  - Type-specific timelock delays (2 days for params/upgrades, 6 hours for treasury)
  - Multi-sig guardian controls (2/3 threshold) for emergency actions
  - Comprehensive security features:
    - Saturating arithmetic for overflow protection
    - SHA-256-based proposal IDs for collision resistance
    - Double-vote prevention
    - Quorum requirements (default 10,000 CELL)
  - RPC endpoints:
    - `gov_submitProposal` - Submit a new governance proposal
    - `gov_vote` - Vote on an active proposal
    - `gov_getProposal` - Get proposal details
    - `gov_finalizeProposal` - Finalize and execute a passed proposal
    - `gov_delegate` - Delegate voting power
    - `gov_getVotingPower` - Get effective voting power with delegations
  - 20+ unit tests covering all core functionality
  - Integration tests for full proposal lifecycle
  - Performance benchmarks
  - Comprehensive documentation in `docs/GOVERNANCE.md`
  - Quick start guide in `crates/bitcell-governance/README.md`

## [0.1.0] - 2025-01-01

### Added
- Initial release candidate 1 (RC1) features
- Core cryptographic primitives
- Cellular automaton engine
- Zero-knowledge proof architecture
- Consensus protocol
- State management
- P2P networking
- RPC/API layer
- Wallet infrastructure
- Admin console
- Economics system
- EBSL trust system
- ZKVM execution

[Unreleased]: https://github.com/Steake/BitCell/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Steake/BitCell/releases/tag/v0.1.0
