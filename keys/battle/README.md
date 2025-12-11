# BattleCircuit Keys

This directory will contain the proving and verification keys for the BattleCircuit, generated through the trusted setup ceremony.

## Files

After the ceremony (planned Q1 2026), this directory will contain:

- `proving_key.bin` - Proving key for BattleCircuit (~2-4 GB)
- `verification_key.bin` - Verification key for BattleCircuit (~1-2 KB)

## Key Hashes

**Status:** Awaiting Trusted Setup Ceremony

```
Proving Key SHA-256:       [Will be filled after ceremony]
Verification Key SHA-256:  [Will be filled after ceremony]
```

## Circuit Information

**BattleCircuit** verifies that Cellular Automaton battles executed correctly.

**Public Inputs:**
- `commitment_a` - Player A's grid commitment
- `commitment_b` - Player B's grid commitment
- `winner_id` - Winner identifier (0=draw, 1=A, 2=B)

**Constraints:** ~6.7M (estimated for 64×64 grid, 10 steps)

**Proving Time:** ~30 seconds target (8-core CPU)  
**Verification Time:** <10ms  
**Proof Size:** 192 bytes

## Usage

```rust
use bitcell_zkp::BattleCircuit;

// Load ceremony keys (production)
let (pk, vk) = BattleCircuit::load_ceremony_keys()?;

// Create circuit instance
let circuit = BattleCircuit::new(
    commitment_a,
    commitment_b,
    winner_id,
    energy_a,
    energy_b,
);

// Generate proof
let proof = circuit.prove(&pk)?;

// Verify proof
let valid = BattleCircuit::verify(&vk, &proof, &public_inputs)?;
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
  --transcript ../transcripts/battle_transcript.json \
  --keys ../../keys/battle/
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
