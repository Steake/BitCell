# BitCell Ceremony Tools

This directory contains tools for conducting and verifying the BitCell trusted setup ceremony.

## Overview

The ceremony tools enable a multi-party trusted setup for Groth16 zkSNARKs. Each tool plays a specific role in the ceremony process.

## Tools

### For Participants

- **`ceremony-contribute`** - Contribute randomness to the ceremony
  - Collects entropy from participant
  - Mixes randomness with current parameters
  - Generates proof of contribution
  - **Status:** Planned for implementation

### For Coordinator

- **`ceremony-coordinator`** - Manage ceremony flow
  - Initialize ceremony with random beacon
  - Sequence participant contributions
  - Generate final keys
  - **Status:** Planned for implementation

### For Everyone

- **`ceremony-verify`** - Verify contributions and keys
  - Verify individual contributions
  - Verify full ceremony transcript
  - Check key derivation
  - **Status:** Planned for implementation

- **`ceremony-audit`** - Generate audit reports
  - Analyze ceremony transcript
  - Generate statistical reports
  - Verify participant independence
  - **Status:** Planned for implementation

## Installation

These tools are built as part of the BitCell repository:

```bash
# Clone repository
git clone https://github.com/Steake/BitCell.git
cd BitCell

# Build ceremony tools
cd ceremony/tools
cargo build --release

# Tools will be in target/release/
```

## Usage

### Participant Contribution

Detailed instructions in [`../participant_instructions.md`](../participant_instructions.md).

Quick start:

```bash
# Download current parameters (provided by coordinator)
curl -o input_params.bin <coordinator_url>

# Run contribution tool
cargo run --release --bin ceremony-contribute \
  --input input_params.bin \
  --output my_contribution.bin \
  --name "Your Name"

# Upload contribution (instructions from coordinator)
```

### Coordinator Operations

Detailed checklist in [`../coordinator_checklist.md`](../coordinator_checklist.md).

Quick reference:

```bash
# Initialize ceremony
cargo run --release --bin ceremony-coordinator init \
  --circuit battle \
  --beacon <bitcoin_block_hash> \
  --output params_round_0.bin

# Verify a contribution
cargo run --release --bin ceremony-verify \
  --input params_round_N.bin \
  --output params_round_N+1.bin \
  --proof contribution_proof.json

# Generate final keys
cargo run --release --bin ceremony-coordinator finalize \
  --circuit battle \
  --input params_round_final.bin \
  --output-dir ../../keys/battle/
```

### Verification

Anyone can verify the ceremony:

```bash
# Verify a single contribution
cargo run --release --bin ceremony-verify \
  --input params_round_N.bin \
  --output params_round_N+1.bin \
  --proof contribution_proof.json

# Verify full ceremony transcript
cargo run --release --bin ceremony-verify \
  --transcript ../transcripts/battle_transcript.json \
  --keys ../../keys/battle/

# Generate audit report
cargo run --release --bin ceremony-audit \
  --transcript ../transcripts/battle_transcript.json \
  --output audit_report.md
```

## Tool Implementation Status

| Tool | Status | Notes |
|------|--------|-------|
| `ceremony-contribute` | 🟡 Planned | Core functionality defined |
| `ceremony-coordinator` | 🟡 Planned | Protocol specified |
| `ceremony-verify` | 🟡 Planned | Verification logic outlined |
| `ceremony-audit` | 🟡 Planned | Reporting format defined |

**Note:** The ceremony is planned for Q1 2026. Tool implementation will be completed before the ceremony begins.

## Architecture

### Contribution Flow

```
┌─────────────────┐
│  Download       │
│  params_N.bin   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Collect        │
│  Entropy        │
│  (dice/coins)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Mix Randomness │
│  with params    │
│  (20-45 min)    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Generate       │
│  params_N+1.bin │
│  + proof        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Upload to      │
│  Coordinator    │
└─────────────────┘
```

### Verification Flow

```
┌─────────────────┐
│  Load           │
│  Transcript     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Verify Random  │
│  Beacon         │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  For Each       │
│  Contribution:  │
│  - Verify proof │
│  - Check hashes │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Verify Final   │
│  Keys Match     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Generate       │
│  Report         │
└─────────────────┘
```

## Security Considerations

### Entropy Collection

The contribution tool collects entropy from multiple sources:
1. Physical sources (dice, coins) - preferred
2. Keyboard timing
3. Camera/microphone noise
4. System /dev/urandom
5. CPU timing variations

**Recommendation:** Use physical sources for highest security.

### Toxic Waste Destruction

After contributing:
1. Securely wipe all parameter files
2. Clear shell history
3. If using VM: delete the VM
4. Attest to destruction

### Coordinator Security

The coordinator should:
1. Use secure infrastructure
2. Verify all contributions
3. Publish everything publicly
4. Maintain audit trail
5. Respond to community questions

## File Formats

### Parameters File

Binary format containing:
- Circuit-specific setup parameters
- Powers of tau
- Encrypted contributions

Size: 2-5 GB per file

### Contribution Proof

JSON format:
```json
{
  "round": 42,
  "participant": "Alice",
  "input_hash": "sha256:...",
  "output_hash": "sha256:...",
  "timestamp": "2025-XX-XX HH:MM:SS UTC",
  "proof_data": {
    "challenge": "...",
    "response": "..."
  }
}
```

### Transcript

JSON format containing:
- Random beacon
- All contributions
- All proofs
- Final key hashes

See [`../transcripts/README.md`](../transcripts/README.md) for schema.

## Testing

Before the ceremony, we'll conduct test runs:

```bash
# Run local test ceremony with 3 participants
./scripts/test_ceremony.sh

# Verify test ceremony
cargo test --package ceremony-tools
```

## Support

**Questions?**
- Email: ceremony@bitcell.org
- Discord: #ceremony-support
- Documentation: [`../../docs/CEREMONY.md`](../../docs/CEREMONY.md)

## References

- **Groth16:** "On the Size of Pairing-based Non-interactive Arguments" (Jens Groth, 2016)
- **Powers of Tau:** Zcash ceremony - https://z.cash/technology/paramgen/
- **MPC Security:** "Scalable Multi-party Computation for zk-SNARK Parameters" (Bowe et al., 2017)

---

**Last Updated:** December 2025  
**Status:** Planning Phase  
**Next Steps:** Implement tools before Q1 2026 ceremony
