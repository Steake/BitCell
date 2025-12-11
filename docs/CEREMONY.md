# BitCell Trusted Setup Ceremony

**Version:** 1.0  
**Date:** December 2025  
**Status:** Planning Phase

---

## Table of Contents

1. [Overview](#overview)
2. [Why a Trusted Setup?](#why-a-trusted-setup)
3. [Security Guarantees](#security-guarantees)
4. [Ceremony Timeline](#ceremony-timeline)
5. [Participation Requirements](#participation-requirements)
6. [Ceremony Process](#ceremony-process)
7. [Key Verification](#key-verification)
8. [Audit Trail](#audit-trail)
9. [Post-Ceremony](#post-ceremony)

---

## Overview

The BitCell trusted setup ceremony is a multi-party computation (MPC) protocol used to generate the proving and verification keys for our Groth16 zero-knowledge proof circuits. This ceremony is **critical** for the security of the BitCell blockchain.

### Circuits Requiring Setup

We conduct separate ceremonies for two circuits:

1. **BattleCircuit** - Proves that Cellular Automaton battles executed correctly
2. **StateCircuit** - Proves that state transitions are valid

Each circuit requires its own independent ceremony with separate proving and verification keys.

---

## Why a Trusted Setup?

Groth16 is a zkSNARK scheme that requires a **trusted setup** to generate the cryptographic parameters (proving and verification keys). During this setup:

1. A "toxic waste" secret (τ) is generated
2. This secret is used to create the proving and verification keys
3. The secret **must be destroyed** to ensure security

### The Trust Problem

If the toxic waste is not properly destroyed:
- An adversary with knowledge of τ can create **fake proofs** for invalid statements
- They could prove false battle outcomes or invalid state transitions
- This would completely compromise the blockchain's security

### The Multi-Party Solution

A **multi-party trusted setup** solves this problem:
- Multiple independent parties each contribute randomness
- Each party's contribution updates the toxic waste
- As long as **at least one participant** is honest and destroys their secret, the final keys are secure
- No single party knows the final toxic waste

This is the same approach used by Zcash (Powers of Tau) and other production zkSNARK systems.

---

## Security Guarantees

### What the Ceremony Guarantees

✅ **If at least one participant is honest:**
- The toxic waste is destroyed
- No one can create fake proofs
- The keys are cryptographically secure

✅ **Verifiability:**
- Each contribution can be verified
- The final keys can be verified against the transcript
- Anyone can verify the ceremony was conducted correctly

✅ **Non-interactivity:**
- Participants don't need to be online simultaneously
- Contributions can be made asynchronously
- The coordinator sequences the contributions

### What Could Go Wrong?

❌ **If ALL participants collude or are compromised:**
- The toxic waste could be reconstructed
- Fake proofs could be created
- **This is why we need diverse, independent participants**

❌ **If the coordinator is malicious:**
- They could reject valid contributions
- They could accept invalid contributions
- **This is why we publish full transcripts and verification tools**

---

## Ceremony Timeline

### Phase 1: Planning (2 weeks)
- [x] Design ceremony protocol
- [x] Develop ceremony tools
- [x] Write participant documentation
- [ ] Recruit participants (target: 20+ independent parties)
- [ ] Schedule contribution windows

### Phase 2: Participant Preparation (1 week)
- [ ] Distribute participant instructions
- [ ] Test participant environments
- [ ] Verify identity/independence of participants
- [ ] Establish secure communication channels

### Phase 3: BattleCircuit Ceremony (2-3 weeks)
- [ ] Initialize ceremony with random beacon
- [ ] Accept participant contributions (1-2 days per participant)
- [ ] Verify each contribution
- [ ] Publish transcript
- [ ] Generate final keys

### Phase 4: StateCircuit Ceremony (2-3 weeks)
- [ ] Initialize ceremony with random beacon
- [ ] Accept participant contributions
- [ ] Verify each contribution
- [ ] Publish transcript
- [ ] Generate final keys

### Phase 5: Verification & Publication (1 week)
- [ ] Independent verification of ceremonies
- [ ] Publish keys to repository
- [ ] Publish ceremony transcripts
- [ ] Publish verification attestations
- [ ] Announce completion

**Total Estimated Duration:** 8-10 weeks

---

## Participation Requirements

### Who Can Participate?

We seek participants who are:
- Independent of each other
- Geographically distributed
- From diverse backgrounds (developers, academics, enterprises)
- Committed to blockchain security

### Technical Requirements

Participants need:
- **Hardware:** Modern computer with 16GB+ RAM
- **OS:** Linux, macOS, or Windows
- **Software:** Rust toolchain (provided in instructions)
- **Time:** 2-4 hours for contribution + setup
- **Storage:** ~20GB free disk space

### Security Recommendations

Participants should:
- Use a dedicated/clean machine (VM recommended)
- Generate entropy from physical sources (dice, coin flips)
- Wipe the machine after contributing
- Document their process
- Attest to destroying their toxic waste

### Identity Verification

To ensure independence, we collect:
- Real name or pseudonym (public)
- Email or contact method (private)
- Optional: PGP key for signed attestation
- Optional: Social proof (GitHub, Twitter, LinkedIn)

**Note:** We respect privacy but need to verify participants are independent.

---

## Ceremony Process

### Overview

```
┌─────────────┐
│  Random     │
│  Beacon     │
│  (Block #)  │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│  Participant 1  │
│  Contributes    │
│  Randomness     │
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│  Coordinator    │
│  Verifies       │
│  Contribution   │
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│  Participant 2  │
│  Contributes    │
│  Randomness     │
└──────┬──────────┘
       │
       ⋮
       │
       ▼
┌─────────────────┐
│  Participant N  │
│  Final          │
│  Contribution   │
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│  Final Keys     │
│  Published      │
└─────────────────┘
```

### Coordinator Responsibilities

The ceremony coordinator (BitCell core team) will:

1. **Initialize** the ceremony with a random beacon
2. **Sequence** participant contributions
3. **Verify** each contribution is valid
4. **Publish** intermediate parameters after each contribution
5. **Generate** final keys after all contributions
6. **Document** the entire process

### Participant Responsibilities

Each participant will:

1. **Download** the latest parameters from the coordinator
2. **Generate** local randomness (with physical entropy sources)
3. **Contribute** their randomness to update the parameters
4. **Upload** their contribution to the coordinator
5. **Verify** their contribution was accepted
6. **Destroy** all local files containing secrets
7. **Attest** to destroying their toxic waste

### Step-by-Step Process

Detailed participant instructions are in [`ceremony/participant_instructions.md`](../ceremony/participant_instructions.md).

High-level steps:

1. **Setup Environment**
   ```bash
   # Clone repository
   git clone https://github.com/Steake/BitCell.git
   cd BitCell/ceremony
   
   # Download current parameters
   ./download_params.sh
   ```

2. **Generate Entropy**
   ```bash
   # Use physical sources: dice, coin flips, keyboard timing
   # The ceremony tool will guide you through this
   cargo run --bin ceremony-contribute
   ```

3. **Contribute**
   ```bash
   # Tool will:
   # - Load current parameters
   # - Mix in your randomness
   # - Generate new parameters
   # - Create proof of contribution
   ./ceremony-contribute --input params_round_N.bin --output my_contribution.bin
   ```

4. **Upload**
   ```bash
   # Secure upload to coordinator
   # Details provided via secure channel
   ```

5. **Verify & Destroy**
   ```bash
   # Wait for coordinator verification
   # Securely wipe all files
   shred -vfz -n 10 *
   # Or reimage your VM/machine
   ```

6. **Attest**
   ```bash
   # Sign attestation that you destroyed your secrets
   gpg --sign attestation.txt
   ```

---

## Key Verification

### During Ceremony

After each contribution, the coordinator publishes:
- **Parameters:** Updated τ powers
- **Proof:** Contribution proof from participant
- **Hash:** SHA-256 hash of parameters
- **Attestation:** Participant's signed attestation

Anyone can verify:
```bash
# Verify contribution N was valid
./verify_contribution --params params_round_N.bin --proof proof_N.json

# Check hash matches
sha256sum params_round_N.bin
```

### After Ceremony

Once complete, we publish:
- **Final Keys:** `proving_key.bin` and `verification_key.bin`
- **Transcript:** Complete ceremony log
- **Hashes:** SHA-256 hashes of all keys
- **Attestations:** All participant attestations

Verification:
```bash
# Verify final keys were derived correctly
./verify_ceremony --transcript ceremony_transcript.json

# Check published key hashes
sha256sum keys/battle/proving_key.bin
sha256sum keys/battle/verification_key.bin
```

### Key Commitment

We commit to the key hashes before the ceremony starts:

**Battle Circuit Keys** (TBD):
```
Proving Key:  <will be filled during ceremony>
Verification Key: <will be filled during ceremony>
```

**State Circuit Keys** (TBD):
```
Proving Key:  <will be filled during ceremony>
Verification Key: <will be filled during ceremony>
```

These hashes serve as a public commitment that prevents key substitution.

---

## Audit Trail

### What We Record

For each contribution:
- Participant identifier (name/pseudonym)
- Timestamp
- Input parameters hash
- Output parameters hash
- Contribution proof
- Participant attestation (signed)

### Public Transcript

The ceremony transcript includes:
```json
{
  "ceremony_id": "bitcell-battle-circuit-2025",
  "circuit": "BattleCircuit",
  "start_time": "2025-XX-XX",
  "end_time": "2025-XX-XX",
  "random_beacon": "Bitcoin block #XXXXXX hash",
  "contributions": [
    {
      "round": 1,
      "participant": "Alice (alice@example.com)",
      "timestamp": "2025-XX-XX HH:MM:SS UTC",
      "input_hash": "sha256:...",
      "output_hash": "sha256:...",
      "contribution_proof": {...},
      "attestation": "-----BEGIN PGP SIGNATURE-----..."
    }
  ],
  "final_keys": {
    "proving_key_hash": "sha256:...",
    "verification_key_hash": "sha256:..."
  }
}
```

### Third-Party Verification

We encourage independent auditors to:
- ✅ Verify each contribution proof
- ✅ Verify the random beacon was used correctly
- ✅ Verify final keys match the transcript
- ✅ Verify at least N participants are independent
- ✅ Verify attestations are properly signed

Tools provided:
- `ceremony-verify` - Automated verification
- `ceremony-audit` - Generate audit report

---

## Post-Ceremony

### Key Distribution

After successful ceremony:

1. **Repository Commit**
   - Keys committed to `keys/battle/` and `keys/state/`
   - Transcript committed to `ceremony/transcripts/`
   - Tagged release: `ceremony-v1.0`

2. **Multiple Distribution Channels**
   - GitHub repository
   - IPFS (content-addressed storage)
   - BitTorrent (decentralized distribution)
   - Official website

3. **Verification Checksums**
   ```
   keys/battle/proving_key.bin         SHA256: ...
   keys/battle/verification_key.bin    SHA256: ...
   keys/state/proving_key.bin          SHA256: ...
   keys/state/verification_key.bin     SHA256: ...
   ```

### Using Ceremony Keys

**For Node Operators:**
```rust
// Load keys from ceremony
let (pk, vk) = BattleCircuit::load_ceremony_keys()?;

// Generate proof
let proof = circuit.prove(&pk)?;

// Verify proof
let valid = BattleCircuit::verify(&vk, &proof, &public_inputs)?;
```

**DO NOT** use `BattleCircuit::setup()` in production - this generates insecure test keys!

### Ongoing Verification

We recommend:
- Verify key hashes on first node startup
- Include verification in CI/CD pipelines
- Re-verify periodically
- Report any hash mismatches immediately

### Security Contacts

If you discover issues:
- Email: security@bitcell.org
- PGP Key: [TBD]
- Responsible disclosure: 90 days

---

## Ceremony Tools

All ceremony tools are in `ceremony/tools/`:

- **`ceremony-contribute`** - Participant contribution tool
- **`ceremony-verify`** - Verify contributions and final keys
- **`ceremony-audit`** - Generate audit reports
- **`ceremony-coordinator`** - Coordinator sequencing tool

See [`ceremony/tools/README.md`](../ceremony/tools/README.md) for usage.

---

## Acknowledgments

This ceremony protocol is inspired by:
- Zcash Powers of Tau ceremony
- Ethereum KZG ceremony
- Filecoin trusted setup
- Academic research on secure MPC

Special thanks to all participants who contribute to the security of BitCell.

---

## References

1. **Groth16 Paper:** "On the Size of Pairing-based Non-interactive Arguments" (Jens Groth, 2016)
2. **Powers of Tau:** https://z.cash/technology/paramgen/
3. **MPC Security:** "Scalable Multi-party Computation for zk-SNARK Parameters" (Bowe et al., 2017)
4. **BN254 Curve:** "Pairing-Friendly Elliptic Curves" (Barreto-Naehrig, 2006)

---

**Last Updated:** December 2025  
**Maintainer:** BitCell Core Team  
**Status:** Ready for Participant Recruitment
