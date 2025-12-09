# BitCell Trusted Setup Ceremony

## Overview

This document describes the multi-party computation (MPC) trusted setup ceremony for BitCell's Groth16 zero-knowledge proof circuits. The ceremony generates proving and verification keys while ensuring that no single party can compromise the system's security.

## Background

### Why a Trusted Setup?

Groth16 is a zkSNARK proving system that requires a one-time trusted setup ceremony. This ceremony produces:
- **Proving Key (PK)**: Used by provers to generate proofs
- **Verification Key (VK)**: Used by verifiers to check proofs

The setup process involves generating random "toxic waste" that, if retained by malicious actors, could be used to forge false proofs. The MPC ceremony ensures security as long as **at least one participant** honestly destroys their contribution's randomness.

### Circuits Requiring Setup

BitCell has two main circuits:

1. **BattleCircuit**: Verifies Conway's Game of Life evolution and winner determination
   - Constraints: ~6.7M (for 64x64 grid, 10 steps)
   - Public inputs: `commitment_a`, `commitment_b`, `winner`
   - Private inputs: Grid states, patterns, nonces

2. **StateCircuit**: Verifies Merkle tree state transitions
   - Constraints: ~100K (for depth-32 Merkle trees)
   - Public inputs: `old_root`, `new_root`, `nullifier`, `commitment`
   - Private inputs: Merkle paths, leaf values

## Ceremony Process

### Phase 1: Preparation (Week 1)

#### 1.1 Participant Selection
- Target: **Minimum 5 participants, recommended 10+**
- Diversity criteria:
  - Geographic distribution (3+ continents)
  - Organizational diversity (academia, industry, community)
  - Technical expertise in cryptography/ZK systems
- Public announcement with open call for participation

#### 1.2 Hardware Requirements
Each participant needs:
- **CPU**: 8+ cores (16+ recommended)
- **RAM**: 64GB minimum (128GB recommended for BattleCircuit)
- **Storage**: 100GB free space
- **Network**: Reliable internet connection
- **Time**: 4-8 hours for contribution

#### 1.3 Software Setup
Participants must:
1. Clone BitCell repository
2. Install Rust toolchain (1.70+)
3. Build ceremony tools: `cargo build --release --package bitcell-ceremony`
4. Verify build integrity with provided checksums

### Phase 2: Contribution (Weeks 2-4)

#### 2.1 Sequential Contribution Process

The ceremony runs sequentially. Each participant:

1. **Receives**: Previous participant's contribution (or initial params for first participant)
2. **Contributes**: Adds their own randomness
3. **Transmits**: Updated parameters to next participant
4. **Publishes**: Attestation of their contribution

#### 2.2 Contribution Steps

For each participant `i`:

```bash
# Step 1: Receive previous contribution
./ceremony/receive_contribution.sh <from_participant_i-1>

# Step 2: Generate contribution with randomness
cargo run --release --bin ceremony-contribute -- \
  --circuit battle \
  --input keys/ceremony/battle_params_${i-1}.bin \
  --output keys/ceremony/battle_params_${i}.bin \
  --attestation keys/ceremony/battle_attestation_${i}.json

cargo run --release --bin ceremony-contribute -- \
  --circuit state \
  --input keys/ceremony/state_params_${i-1}.bin \
  --output keys/ceremony/state_params_${i}.bin \
  --attestation keys/ceremony/state_attestation_${i}.json

# Step 3: Verify contribution integrity
cargo run --release --bin ceremony-verify -- \
  --circuit battle \
  --prev keys/ceremony/battle_params_${i-1}.bin \
  --current keys/ceremony/battle_params_${i}.bin \
  --attestation keys/ceremony/battle_attestation_${i}.json

# Step 4: Publish attestation (sign with GPG/PGP)
gpg --clearsign keys/ceremony/battle_attestation_${i}.json
gpg --clearsign keys/ceremony/state_attestation_${i}.json

# Step 5: Transmit to next participant
./ceremony/send_contribution.sh <to_participant_i+1>
```

#### 2.3 Randomness Sources

Participants **must** incorporate multiple entropy sources:
- System entropy (`/dev/urandom`, Windows CryptGenRandom)
- Hardware RNG (if available)
- Physical sources (dice rolls, coin flips)
- Environmental noise (microphone, camera)
- User input (keyboard timing)

**Critical**: All random values must be securely erased after contribution.

#### 2.4 Attestation Format

Each participant publishes a signed attestation:

```json
{
  "ceremony": "BitCell Groth16 Trusted Setup",
  "version": "1.0",
  "circuit": "BattleCircuit",
  "participant": {
    "id": 5,
    "name": "Alice Smith",
    "organization": "University of Cryptography",
    "gpg_fingerprint": "ABCD1234...",
    "contact": "alice@example.edu"
  },
  "contribution": {
    "timestamp": "2026-03-15T14:30:00Z",
    "input_hash": "sha256:abc123...",
    "output_hash": "sha256:def456...",
    "duration_seconds": 14400
  },
  "verification": {
    "previous_valid": true,
    "contribution_valid": true,
    "toxic_waste_destroyed": true
  },
  "entropy_sources": [
    "/dev/urandom",
    "hardware_rng",
    "physical_dice",
    "user_input"
  ],
  "system_info": {
    "os": "Linux 5.15",
    "rust_version": "1.78.0",
    "bitcell_commit": "a1b2c3d4..."
  },
  "statement": "I, Alice Smith, attest that I have honestly participated in this trusted setup ceremony. I have contributed randomness from multiple independent sources and have securely destroyed all secret values used in my contribution. I understand that the security of the BitCell network depends on the integrity of this process."
}
```

### Phase 3: Finalization (Week 5)

#### 3.1 Final Key Generation

After all participants contribute:

```bash
# Generate final proving and verification keys
cargo run --release --bin ceremony-finalize -- \
  --circuit battle \
  --params keys/ceremony/battle_params_final.bin \
  --output-pk keys/battle/proving_key.bin \
  --output-vk keys/battle/verification_key.bin

cargo run --release --bin ceremony-finalize -- \
  --circuit state \
  --params keys/ceremony/state_params_final.bin \
  --output-pk keys/state/proving_key.bin \
  --output-vk keys/state/verification_key.bin
```

#### 3.2 Verification

Independent verifiers should:

1. **Verify contribution chain**:
   ```bash
   cargo run --release --bin ceremony-verify-chain -- \
     --circuit battle \
     --contributions keys/ceremony/battle_*.bin
   ```

2. **Test proof generation**:
   ```bash
   cargo test --release --package bitcell-zkp -- \
     --test-threads=1 \
     test_battle_circuit_with_ceremony_keys
   ```

3. **Publish verification report**

#### 3.3 Key Publication

Final keys are published to:
- **GitHub**: `keys/` directory in main repository
- **IPFS**: Content-addressed storage with hashes in README
- **Arweave**: Permanent storage with transaction ID
- **Checksums**: SHA256 hashes signed by ceremony coordinator

```bash
# keys/README.md
BattleCircuit Keys (64x64 grid, 10 steps):
- Proving Key: battle/proving_key.bin
  - Size: ~450MB
  - SHA256: abc123...
  - IPFS: QmXyZ...
  - Arweave: ar://...

- Verification Key: battle/verification_key.bin
  - Size: ~2KB
  - SHA256: def456...
  - IPFS: QmAbc...
  - Arweave: ar://...

StateCircuit Keys (32-level Merkle tree):
- Proving Key: state/proving_key.bin
  - Size: ~50MB
  - SHA256: ghi789...
  - IPFS: QmDef...
  - Arweave: ar://...

- Verification Key: state/verification_key.bin
  - Size: ~1.5KB
  - SHA256: jkl012...
  - IPFS: QmGhi...
  - Arweave: ar://...
```

## Security Considerations

### Threat Model

The ceremony is secure if:
1. **At least one participant is honest**: Properly generates randomness and destroys toxic waste
2. **Computation is correct**: All participants verify previous contributions
3. **Keys are authentic**: Final keys match the contribution chain

### Attack Scenarios

#### Scenario 1: Single Malicious Participant
- **Attack**: Participant retains toxic waste
- **Mitigation**: Security holds if any other participant is honest
- **Probability**: `1 - (n_malicious / n_total)^n_total` approaches 1 with more participants

#### Scenario 2: Contribution Substitution
- **Attack**: Attacker replaces contribution with manipulated version
- **Mitigation**: Public attestations and hash chain verification
- **Detection**: Any verifier can detect hash mismatch

#### Scenario 3: Weak Randomness
- **Attack**: Participant uses predictable randomness
- **Mitigation**: Multiple entropy sources required, public audit of process
- **Detection**: Statistical tests on contribution outputs

### Best Practices

1. **Airgap Machines**: Consider using offline machines for contribution
2. **Memory Sanitization**: Overwrite RAM after contribution (use `mlock`/`memset`)
3. **Disk Encryption**: Encrypt all intermediate files
4. **Secure Deletion**: Use `shred` or similar for file deletion
5. **Public Participation**: Live-stream or record contribution process
6. **Third-Party Audits**: Invite external security researchers

## Coordinator Responsibilities

### Before Ceremony
- [ ] Select and vet participants
- [ ] Schedule contribution slots
- [ ] Prepare communication channels (Signal/encrypted email)
- [ ] Set up file transfer infrastructure
- [ ] Announce ceremony publicly (blog, Twitter, forums)

### During Ceremony
- [ ] Coordinate participant handoffs
- [ ] Monitor attestation publications
- [ ] Provide technical support
- [ ] Maintain public log of progress
- [ ] Handle contingencies (participant dropouts)

### After Ceremony
- [ ] Verify complete contribution chain
- [ ] Generate and test final keys
- [ ] Publish keys and attestations
- [ ] Write ceremony report
- [ ] Archive all artifacts

## Timeline Example

| Week | Phase | Activities |
|------|-------|------------|
| 1 | Preparation | Participant selection, announcement, setup |
| 2-4 | Contributions | 10 participants × 3-4 days each |
| 5 | Finalization | Key generation, verification, publication |
| 6 | Post-Ceremony | Reports, audits, integration |

## Ceremony Report Template

After completion, publish a comprehensive report:

```markdown
# BitCell Trusted Setup Ceremony Report

## Summary
- **Date**: March 1-31, 2026
- **Participants**: 12 (10 planned + 2 backup)
- **Circuits**: BattleCircuit, StateCircuit
- **Duration**: 28 days
- **Status**: ✅ Successfully Completed

## Participants
1. Alice Smith - University of Cryptography (USA)
2. Bob Johnson - ZK Labs (Switzerland)
3. ... (full list)

## Statistics
- Total contributions: 24 (12 per circuit)
- Average contribution time: 6 hours
- Total data transferred: 15GB
- Verification time: 12 hours

## Attestations
All 12 participants published signed attestations confirming:
- Honest participation
- Multiple entropy sources used
- Toxic waste destroyed
- No retention of secret values

## Verification
Independent verification by:
- Dr. Charlie Brown (Stanford University)
- ZK Security Audit Firm XYZ
- BitCell Core Development Team

All verifications passed ✅

## Key Artifacts
- Ceremony transcript: ceremony/transcript.log
- All attestations: ceremony/attestations/
- Final keys: keys/{battle,state}/
- Verification reports: ceremony/verification/

## Security Statement
We are confident in the security of this ceremony because:
1. 12 geographically distributed participants
2. Diverse organizations and backgrounds
3. Public attestations with verifiable identities
4. Independent verification of contribution chain
5. Multiple entropy sources documented
6. No evidence of compromise

The probability that all participants colluded is negligibly small.
```

## Integration with BitCell

### Using Ceremony Keys

After the ceremony, BitCell circuits use the generated keys:

```rust
// Load proving key for proof generation
let pk = BattleCircuit::load_proving_key("keys/battle/proving_key.bin")?;
let proof = BattleCircuit::prove(&pk, circuit)?;

// Load verification key for proof verification
let vk = BattleCircuit::load_verification_key("keys/battle/verification_key.bin")?;
let valid = BattleCircuit::verify(&vk, &public_inputs, &proof)?;
```

### Key Management in Production

1. **Node Operators**: Only need verification keys (~2KB each)
2. **Provers**: Need full proving keys (~500MB total)
3. **Key Distribution**: Via GitHub releases, IPFS, package managers
4. **Key Updates**: Only required if circuit logic changes (rare)

## FAQ

**Q: What happens if a participant is malicious?**
A: Security holds as long as one participant is honest. The more participants, the higher the confidence.

**Q: Can we re-run the ceremony?**
A: Yes, if circuit logic changes or if ceremony integrity is questioned. Old keys become invalid.

**Q: How long are the keys valid?**
A: Indefinitely, unless circuits change. The ceremony is a one-time process per circuit version.

**Q: Can I verify I participated?**
A: Yes, your contribution hash is in the public chain, and you signed an attestation.

**Q: What if someone finds the toxic waste?**
A: They could forge proofs for that circuit. This is why we need multiple honest participants.

## References

1. [Zcash Powers of Tau Ceremony](https://z.cash/technology/paramgen/)
2. [Ethereum KZG Ceremony](https://ceremony.ethereum.org/)
3. [Groth16 Paper](https://eprint.iacr.org/2016/260.pdf)
4. [MPC for SNARK Setups](https://eprint.iacr.org/2017/1050.pdf)
5. [Trusted Setup Security Analysis](https://vitalik.ca/general/2022/03/14/trustedsetup.html)

## Contact

For questions about the ceremony:
- **Email**: ceremony@bitcell.org
- **GitHub**: https://github.com/Steake/BitCell/issues
- **Discord**: #trusted-setup channel

---

**Last Updated**: December 2025  
**Version**: 1.0  
**Status**: Pre-Ceremony Draft
