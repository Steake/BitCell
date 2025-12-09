# BitCell Trusted Setup Keys

This directory contains proving and verification keys generated from the BitCell trusted setup ceremony.

## Directory Structure

```
keys/
├── README.md              # This file
├── battle/               # BattleCircuit keys
│   ├── proving_key.bin   # Proving key for battle proofs
│   ├── verification_key.bin  # Verification key for battle proofs
│   └── metadata.json     # Key metadata and hashes
├── state/                # StateCircuit keys
│   ├── proving_key.bin   # Proving key for state proofs
│   ├── verification_key.bin  # Verification key for state proofs
│   └── metadata.json     # Key metadata and hashes
└── ceremony/             # Ceremony artifacts
    ├── transcript.log    # Full ceremony transcript
    ├── attestations/     # Participant attestations
    └── verification/     # Independent verification reports
```

## Key Information

### BattleCircuit Keys (64x64 grid, 10 steps)

**Status**: 🚧 Pending Ceremony

These keys are for the production BattleCircuit that verifies Conway's Game of Life evolution.

- **Circuit Parameters**:
  - Grid Size: 64×64 cells
  - Battle Steps: 10 generations
  - Constraints: ~6.7M R1CS constraints
  - Public Inputs: `commitment_a`, `commitment_b`, `winner`

- **Expected Key Sizes**:
  - Proving Key: ~450MB
  - Verification Key: ~2KB

- **Proving Performance**:
  - Generation Time: <30s (on 8-core CPU)
  - Memory Required: 16GB+ RAM
  - GPU Acceleration: Optional (recommended)

**Checksums** (after ceremony):
```
Proving Key:
  SHA256: [to be generated]
  IPFS: [to be published]
  Arweave: [to be published]

Verification Key:
  SHA256: [to be generated]
  IPFS: [to be published]
  Arweave: [to be published]
```

### StateCircuit Keys (32-level Merkle tree)

**Status**: 🚧 Pending Ceremony

These keys are for the StateCircuit that verifies Merkle tree state transitions.

- **Circuit Parameters**:
  - Merkle Tree Depth: 32 levels
  - Constraints: ~100K R1CS constraints
  - Public Inputs: `old_root`, `new_root`, `nullifier`, `commitment`

- **Expected Key Sizes**:
  - Proving Key: ~50MB
  - Verification Key: ~1.5KB

- **Proving Performance**:
  - Generation Time: <20s (on 8-core CPU)
  - Memory Required: 8GB+ RAM

**Checksums** (after ceremony):
```
Proving Key:
  SHA256: [to be generated]
  IPFS: [to be published]
  Arweave: [to be published]

Verification Key:
  SHA256: [to be generated]
  IPFS: [to be published]
  Arweave: [to be published]
```

## Trusted Setup Ceremony

### Status: 🚧 Planned for Q1 2026

The keys in this directory will be generated through a multi-party computation (MPC) trusted setup ceremony. See [CEREMONY.md](../docs/CEREMONY.md) for full details.

### Ceremony Overview

- **Participants**: Minimum 5, target 10+ independent contributors
- **Duration**: ~4 weeks (including preparation and finalization)
- **Security**: Safe as long as at least one participant is honest
- **Transparency**: All contributions publicly verifiable

### Ceremony Phases

1. **Preparation** (Week 1)
   - Participant selection and vetting
   - Hardware/software setup verification
   - Public announcement

2. **Contributions** (Weeks 2-4)
   - Sequential participant contributions
   - Each adds randomness and publishes attestation
   - Public verification of contribution chain

3. **Finalization** (Week 5)
   - Generate final proving and verification keys
   - Independent third-party verification
   - Publication to multiple platforms (GitHub, IPFS, Arweave)

## Using the Keys

### For Node Operators

Node operators only need verification keys to validate proofs:

```rust
use bitcell_zkp::key_management::{load_verification_key, KeyType};

// Load verification key
let vk = load_verification_key("keys/battle/verification_key.bin")?;

// Verify a proof
let valid = BattleCircuit::verify(&vk, &public_inputs, &proof)?;
```

**Download**: Verification keys are ~3.5KB total and can be downloaded from:
- GitHub: This repository
- IPFS: [CID to be published]
- npm/cargo packages (bundled)

### For Provers

Provers need proving keys to generate proofs:

```rust
use bitcell_zkp::key_management::{load_proving_key, KeyType};

// Load proving key
let pk = load_proving_key("keys/battle/proving_key.bin")?;

// Generate a proof
let proof = BattleCircuit::prove(&pk, circuit)?;
```

**Download**: Proving keys are ~500MB total and can be downloaded from:
- GitHub Releases: [Link to be published]
- IPFS: [CID to be published]
- Arweave: [Transaction ID to be published]
- Torrent: [Magnet link to be published]

### Key Verification

Always verify key integrity after downloading:

```rust
use bitcell_zkp::key_management::{KeyMetadata, verify_proving_key_hash};

// Load metadata
let metadata = KeyMetadata::load("keys/battle/metadata.json")?;

// Verify key integrity
metadata.verify_keys(
    "keys/battle/proving_key.bin",
    "keys/battle/verification_key.bin"
)?;
```

## Security Considerations

### Key Authenticity

To ensure you're using authentic ceremony keys:

1. **Verify checksums**: Compare SHA256 hashes with published values
2. **Check signatures**: Verify GPG signatures from ceremony coordinator
3. **Cross-reference**: Compare with multiple distribution channels (GitHub, IPFS, Arweave)
4. **Review ceremony**: Read ceremony report and attestations

### Key Storage

- **Verification Keys**: Can be publicly distributed, no security requirements
- **Proving Keys**: Can be publicly distributed, but handle with care:
  - Verify integrity before use
  - Store on secure, backed-up storage
  - Consider local caching for performance

### Key Updates

Keys only need to be updated if:
- Circuit logic changes (breaking change)
- Ceremony is compromised (extremely unlikely with proper MPC)
- Security vulnerability discovered in key generation

## Development and Testing

### Using Test Keys

For development and testing, circuits use `circuit_specific_setup()` to generate ephemeral keys:

```rust
// Generate test keys (NOT for production)
let rng = &mut ark_std::test_rng();
let (pk, vk) = BattleCircuit::setup()?;
```

⚠️ **Warning**: Test keys are NOT secure for production use. They are generated with predictable randomness and do not undergo a trusted setup ceremony.

### Generating Test Keys

To generate test keys for local development:

```bash
cargo run --release --bin generate-test-keys -- \
  --circuit battle \
  --output-pk keys/test/battle_pk.bin \
  --output-vk keys/test/battle_vk.bin
```

## Distribution Channels

After the ceremony, keys will be available on:

### Primary Distribution
- **GitHub**: This repository (`keys/` directory)
- **GitHub Releases**: Packaged downloads with checksums

### Decentralized Storage
- **IPFS**: Content-addressed, verifiable storage
- **Arweave**: Permanent, immutable storage
- **BitTorrent**: Distributed, redundant downloads

### Package Managers
- **npm**: `@bitcell/zkp-keys` (verification keys only)
- **crates.io**: `bitcell-zkp-keys` (verification keys only)

### Mirrors
- Official website: https://bitcell.org/keys
- Documentation site: https://docs.bitcell.org/keys
- Community mirrors (to be announced)

## Ceremony Participants

### Confirmed Participants (After Ceremony)

| # | Name | Organization | Location | Attestation |
|---|------|--------------|----------|-------------|
| 1 | [Name] | [Org] | [Country] | [Link] |
| 2 | [Name] | [Org] | [Country] | [Link] |
| ... | ... | ... | ... | ... |

## Verification Reports

### Independent Verifications

After the ceremony, this section will list independent third-party verifications:

- [ ] [Organization 1] - Verification Report ([link])
- [ ] [Organization 2] - Verification Report ([link])
- [ ] [Organization 3] - Verification Report ([link])

## Support

For questions about the keys or ceremony:

- **Documentation**: See [CEREMONY.md](../docs/CEREMONY.md)
- **Issues**: https://github.com/Steake/BitCell/issues
- **Email**: ceremony@bitcell.org
- **Discord**: #trusted-setup channel

---

**Last Updated**: December 2025  
**Status**: Pre-Ceremony  
**Next Update**: After ceremony completion (Q1 2026)
