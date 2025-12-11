# BitCell Groth16 Keys

This directory contains the proving and verification keys for BitCell's zero-knowledge proof circuits, generated through a multi-party trusted setup ceremony.

---

## Directory Structure

```
keys/
├── battle/
│   ├── proving_key.bin          # BattleCircuit proving key
│   ├── verification_key.bin     # BattleCircuit verification key
│   └── README.md
└── state/
    ├── proving_key.bin          # StateCircuit proving key
    ├── verification_key.bin     # StateCircuit verification key
    └── README.md
```

---

## Key Hashes

**IMPORTANT:** Always verify these hashes before using the keys!

### BattleCircuit Keys

**Status:** Awaiting Trusted Setup Ceremony (Planned Q1 2026)

```
Proving Key SHA-256:       [Will be filled after ceremony]
Verification Key SHA-256:  [Will be filled after ceremony]
```

### StateCircuit Keys

**Status:** Awaiting Trusted Setup Ceremony (Planned Q1 2026)

```
Proving Key SHA-256:       [Will be filled after ceremony]
Verification Key SHA-256:  [Will be filled after ceremony]
```

---

## Verifying Keys

### Quick Verification

```bash
# Verify BattleCircuit keys
sha256sum keys/battle/proving_key.bin
sha256sum keys/battle/verification_key.bin

# Verify StateCircuit keys
sha256sum keys/state/proving_key.bin
sha256sum keys/state/verification_key.bin

# Compare with published hashes in this README
```

### Full Verification

To fully verify the ceremony was conducted correctly:

1. **Download the ceremony transcript:**
   ```bash
   # Clone repository or download transcript
   git clone https://github.com/Steake/BitCell.git
   cd BitCell
   ```

2. **Run the verification tool:**
   ```bash
   cd ceremony/tools
   cargo run --release --bin ceremony-verify-full \
     --transcript ../../ceremony/transcripts/battle_transcript.json \
     --keys ../../keys/battle/
   ```

3. **Verify random beacon:**
   - Check that the random beacon (Bitcoin block hash) is correct
   - Verify it was chosen fairly (future block at ceremony start)
   - Confirm it matches ceremony announcements

4. **Verify all contributions:**
   - Each participant contribution must be verified
   - Check that proofs are valid
   - Verify the chain of contributions from beacon to final keys

5. **Review attestations:**
   - Read participant attestations in `ceremony/attestations/`
   - Verify participants are independent
   - Check that attestations are properly signed (if using PGP)

---

## Using the Keys

### In Production Code

**Load ceremony keys (PRODUCTION):**

```rust
use bitcell_zkp::{BattleCircuit, StateCircuit};

// Load BattleCircuit keys from ceremony
let (battle_pk, battle_vk) = BattleCircuit::load_ceremony_keys()?;

// Load StateCircuit keys from ceremony
let (state_pk, state_vk) = StateCircuit::load_ceremony_keys()?;

// Generate proof
let proof = battle_circuit.prove(&battle_pk)?;

// Verify proof
let valid = BattleCircuit::verify(&battle_vk, &proof, &public_inputs)?;
```

**DO NOT use `setup()` in production:**

```rust
// ❌ NEVER DO THIS IN PRODUCTION
let (pk, vk) = BattleCircuit::setup()?;  // Insecure test keys!

// ✅ ALWAYS DO THIS IN PRODUCTION
let (pk, vk) = BattleCircuit::load_ceremony_keys()?;  // Secure ceremony keys
```

### For Testing

For tests and development, you can use `setup()`:

```rust
#[test]
fn test_something() {
    // Test keys are fine for testing
    let (pk, vk) = BattleCircuit::setup().unwrap();
    
    // ... test code
}
```

### Key Loading Paths

The keys are loaded from:
- `keys/battle/proving_key.bin` (BattleCircuit)
- `keys/battle/verification_key.bin` (BattleCircuit)
- `keys/state/proving_key.bin` (StateCircuit)
- `keys/state/verification_key.bin` (StateCircuit)

Paths are relative to repository root. If you move the keys, update the paths in:
- `crates/bitcell-zkp/src/battle_circuit.rs`
- `crates/bitcell-zkp/src/state_circuit.rs`

---

## Key Specifications

### BattleCircuit

**Circuit Description:**
- Proves that a Cellular Automaton battle executed correctly
- Verifies Conway's Game of Life rules for all evolution steps
- Ensures winner determination is correct

**Public Inputs:**
- `commitment_a` - Player A's grid commitment
- `commitment_b` - Player B's grid commitment  
- `winner_id` - Winner identifier (0=draw, 1=A wins, 2=B wins)

**Private Inputs:**
- `final_energy_a` - Player A's final energy
- `final_energy_b` - Player B's final energy

**Constraints:** ~6.7M (estimated based on 64x64 grid, 10 steps)

**Proving Key Size:** ~2-4 GB (compressed)  
**Verification Key Size:** ~1-2 KB  
**Proof Size:** ~192 bytes

### StateCircuit

**Circuit Description:**
- Proves that a state transition is valid
- Verifies Merkle tree updates
- Prevents double-spending via nullifiers

**Public Inputs:**
- `old_state_root` - Previous state root
- `new_state_root` - New state root
- `nullifier` - Prevents double-spending

**Private Inputs:**
- `leaf_index` - Index in Merkle tree

**Constraints:** ~1M (estimated)

**Proving Key Size:** ~500 MB - 1 GB (compressed)  
**Verification Key Size:** ~1 KB  
**Proof Size:** ~192 bytes

---

## Ceremony Information

### Ceremony Schedule

| Phase | Circuit | Dates | Status |
|-------|---------|-------|--------|
| Preparation | - | Dec 2025 | ✅ Complete |
| BattleCircuit Ceremony | BattleCircuit | Q1 2026 | 📅 Planned |
| StateCircuit Ceremony | StateCircuit | Q1 2026 | 📅 Planned |
| Verification | Both | Q1 2026 | 📅 Planned |

### Ceremony Details

**Random Beacon:** [Bitcoin block #XXXXXX - TBD]  
**Participants:** [20-30 expected]  
**Coordinator:** BitCell Core Team  
**Contact:** ceremony@bitcell.org

### Ceremony Transcript

After ceremony completion, the full transcript will be available at:
- `ceremony/transcripts/battle_transcript.json`
- `ceremony/transcripts/state_transcript.json`

The transcript includes:
- Random beacon block hash
- All participant contributions
- All verification proofs
- All participant attestations
- Final key hashes

### Participant List

After ceremony completion, participants will be listed in:
- `ceremony/participants.md`
- `docs/CEREMONY.md`
- BitCell website: https://bitcell.org/ceremony

---

## Security Notes

### Trust Model

The security of these keys relies on **at least one honest participant** in the ceremony:

- ✅ If ≥1 participant destroyed their toxic waste → Keys are secure
- ❌ If ALL participants colluded → Keys could be compromised

With 20+ independent participants from diverse backgrounds and locations, the probability that ALL collude is negligible.

### What If Keys Are Compromised?

If the keys are compromised (e.g., toxic waste not destroyed):
- Attackers can create fake proofs
- Invalid battles/transactions could be proven valid
- Blockchain security is compromised

**Prevention:**
- Multiple independent participants (20-30+)
- Geographic diversity (5+ countries)
- Background diversity (developers, academics, enterprises)
- Public attestations and verification
- Open-source ceremony code

**Detection:**
- Monitor for suspicious proofs
- Watch for invalid state transitions
- Community vigilance

**Mitigation:**
- If compromise detected: Re-run ceremony
- Network can fork to reject old keys
- New keys deployed via network upgrade

### Ceremony Best Practices

Our ceremony follows industry best practices:

1. **Multiple Independent Participants** (20-30+)
2. **Public Random Beacon** (Bitcoin block hash)
3. **Verifiable Contributions** (Each contribution has proof)
4. **Public Transcript** (Full audit trail)
5. **Participant Attestations** (Public accountability)
6. **Open Source Tools** (Anyone can verify)
7. **Multiple Distribution Channels** (GitHub, IPFS, BitTorrent)

These practices are based on successful ceremonies by:
- Zcash (Powers of Tau)
- Ethereum (KZG ceremony)
- Filecoin
- Semaphore

---

## Distribution Channels

The keys are distributed via multiple channels for redundancy:

### Primary: GitHub

```bash
git clone https://github.com/Steake/BitCell.git
cd BitCell/keys
```

### IPFS (Content-Addressed)

```bash
# After ceremony, IPFS hashes will be:
ipfs get /ipfs/<battle_keys_cid>
ipfs get /ipfs/<state_keys_cid>
```

### BitTorrent

```bash
# After ceremony, magnet links will be:
# Battle keys: magnet:?xt=urn:btih:<hash>
# State keys:  magnet:?xt=urn:btih:<hash>
```

### Official Website

Download from: https://bitcell.org/keys

---

## Support

### Questions?

- **Documentation:** See `docs/CEREMONY.md`
- **Email:** ceremony@bitcell.org
- **Discord:** #ceremony channel
- **Forum:** https://forum.bitcell.org

### Report Issues

If you discover any issues with the keys:
- **Security issues:** security@bitcell.org (PGP key available)
- **Other issues:** GitHub Issues

### Stay Updated

- **Website:** https://bitcell.org
- **Twitter:** @BitCellNetwork
- **Blog:** https://blog.bitcell.org

---

## License

The keys themselves are public data and can be freely used. The ceremony code and tools are licensed under the same license as BitCell (see repository LICENSE file).

---

**Last Updated:** December 2025  
**Key Status:** Awaiting Ceremony (Q1 2026)  
**Next Update:** After ceremony completion
