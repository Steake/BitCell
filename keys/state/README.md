# StateCircuit Keys

This directory will contain the proving and verification keys for the StateCircuit, generated through the trusted setup ceremony.

## Files

After the ceremony (planned Q1 2026), this directory will contain:

- `proving_key.bin` - Proving key for StateCircuit (~500 MB - 1 GB)
- `verification_key.bin` - Verification key for StateCircuit (~1 KB)

## Key Hashes

**Status:** Awaiting Trusted Setup Ceremony

```
Proving Key SHA-256:       [Will be filled after ceremony]
Verification Key SHA-256:  [Will be filled after ceremony]
```

## Circuit Information

**StateCircuit** verifies that state transitions are valid.

**Public Inputs:**
- `old_state_root` - Previous state root
- `new_state_root` - New state root (must differ from old)
- `nullifier` - Prevents double-spending

**Constraints:** ~1M (estimated)

**Proving Time:** ~20 seconds target (8-core CPU)  
**Verification Time:** <10ms  
**Proof Size:** 192 bytes

## Usage

```rust
use bitcell_zkp::StateCircuit;

// Load ceremony keys (production)
let (pk, vk) = StateCircuit::load_ceremony_keys()?;

// Create circuit instance
let circuit = StateCircuit::new(
    old_root,
    new_root,
    nullifier,
    leaf_index,
);

// Generate proof
let proof = circuit.prove(&pk)?;

// Verify proof
let valid = StateCircuit::verify(&vk, &proof, &public_inputs)?;
```

## Ceremony Details

**Random Beacon:** [Bitcoin block #XXXXXX - TBD]  
**Expected Participants:** 20-30  
**Expected Duration:** 2-3 weeks  
**Coordinator:** BitCell Core Team

## Verification

After ceremony completion, verify keys:

```bash
# Check hashes
sha256sum proving_key.bin
sha256sum verification_key.bin

# Full verification
cd ../../ceremony/tools
cargo run --release --bin ceremony-verify \
  --transcript ../transcripts/state_transcript.json \
  --keys ../../keys/state/
```

## Distribution

Keys will be distributed via:
- GitHub (this repository)
- IPFS: `ipfs://[CID]` (TBD)
- BitTorrent: `magnet:?xt=urn:btih:[hash]` (TBD)
- Website: https://bitcell.org/keys

---

**Last Updated:** December 2025  
**Status:** Awaiting Ceremony  
**Next Update:** After Q1 2026 ceremony
