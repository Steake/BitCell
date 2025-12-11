# BitCell ZKP Circuit Implementation Guide

This document provides an overview of the Groth16 circuit implementations for BitCell's zero-knowledge proof system.

## Overview

BitCell uses two main circuits for consensus verification:

1. **StateCircuit**: Verifies state transitions, merkle proofs, and nullifier derivation
2. **BattleCircuit**: Verifies Conway's Game of Life evolution and winner determination

Both circuits are implemented with full R1CS constraints using the arkworks library and BN254 curve.

## Architecture

### Two-Tier System

The codebase maintains two implementations for each circuit:

- **Simplified Circuits** (`battle_circuit.rs`, `state_circuit.rs`): Fast, simplified implementations for testing and development
- **Full Constraint Circuits** (`battle_constraints.rs`, `state_constraints.rs`): Production implementations with complete verification

By default, `lib.rs` exports the full constraint implementations. Simplified versions are available as `SimpleBattleCircuit` and `SimpleStateCircuit`.

## StateCircuit (state_constraints.rs)

### Purpose
Verifies that state transitions follow protocol rules and prevent double-spending through nullifier tracking.

### Public Inputs
1. `old_root` - State root before transaction
2. `new_root` - State root after transaction  
3. `nullifier` - Unique identifier derived from spent leaf
4. `commitment` - Commitment to new leaf value

### Private Inputs (Witnesses)
1. `leaf` - Old leaf value being spent
2. `path` - Merkle authentication path (32 siblings)
3. `indices` - Path direction indicators (left/right)
4. `new_leaf` - New leaf value after transaction

### Constraints Enforced

1. **Merkle Path Validation (Old State)**
   - Computes root from `leaf` and `path`
   - Enforces computed root equals `old_root`

2. **Nullifier Derivation**
   - Computes `nullifier = H(leaf)`
   - Ensures nullifier is deterministically derived from spent leaf

3. **Commitment Verification**
   - Computes `commitment = H(new_leaf)`
   - Proves knowledge of new leaf value

4. **Merkle Path Validation (New State)**
   - Computes root from `new_leaf` and `path`
   - Enforces computed root equals `new_root`

### Constraint Count
- Approximately 40K constraints (with simplified hash)
- Would be ~500K constraints with Poseidon hash (production)

### API Usage

```rust
use bitcell_zkp::{StateCircuit, Fr};

// Setup (expensive, done once)
let (pk, vk) = StateCircuit::<Fr>::setup()?;

// Create circuit with public inputs
let circuit = StateCircuit::new(old_root, new_root, nullifier, commitment)
    .with_witnesses(leaf, path, indices, new_leaf);

// Generate proof
let proof = circuit.prove(&pk)?;

// Verify proof
let public_inputs = circuit.public_inputs()?;
let is_valid = StateCircuit::verify(&vk, &proof, &public_inputs)?;
```

## NullifierCircuit (state_constraints.rs)

### Purpose
Proves membership or non-membership of a nullifier in the nullifier set without revealing the merkle path.

### Public Inputs
1. `nullifier` - The nullifier to check
2. `set_root` - Root of nullifier set merkle tree
3. `is_member` - Boolean (1 if member, 0 if not)

### Private Inputs
1. `path` - Merkle authentication path
2. `indices` - Path directions

### Constraints Enforced
- Computes merkle root from nullifier and path
- Enforces: `(computed_root == set_root) ⟺ is_member`

This allows proving a nullifier is used (double-spend check) or unused without revealing the merkle path.

## BattleCircuit (battle_constraints.rs)

### Purpose
Verifies that a Conway's Game of Life battle was executed correctly and the winner was determined fairly.

### Public Inputs
1. `initial_grid` - Flattened NxN grid (N² elements)
2. `final_grid` - Flattened NxN grid after evolution
3. `commitment_a` - Player A's pattern commitment
4. `commitment_b` - Player B's pattern commitment
5. `winner` - Winner ID (0=A, 1=B, 2=tie)

### Private Inputs
1. `pattern_a` - Player A's glider pattern
2. `pattern_b` - Player B's glider pattern
3. `nonce_a` - Random nonce for commitment A
4. `nonce_b` - Random nonce for commitment B

### Constraints Enforced

1. **Commitment Verification**
   - Proves `commitment_a = H(pattern_a || nonce_a)`
   - Proves `commitment_b = H(pattern_b || nonce_b)`

2. **Initial Placement**
   - Verifies patterns were placed correctly in initial grid

3. **Conway's Rules Evolution**
   - Simulates N steps of Conway's Game of Life
   - Enforces B3/S23 rules with toroidal wrapping
   - Each cell's next state is determined by neighbor count

4. **Winner Determination**
   - Computes regional energy for each player
   - Verifies winner matches actual outcome

### Circuit Configuration

#### Test Configuration (Current)
- Grid Size: 64×64
- Evolution Steps: 10
- Constraints: ~100K
- Proving Time: ~30 seconds

#### Production Configuration (Target)
- Grid Size: 1024×1024
- Evolution Steps: 1000
- Constraints: ~10M (estimated)
- Proving Time: <30 seconds (requires optimization)

### API Usage

```rust
use bitcell_zkp::{BattleCircuit, Fr};

// Setup (very expensive, done once)
let (pk, vk) = BattleCircuit::<Fr>::setup()?;

// Create circuit
let circuit = BattleCircuit::new(
    initial_grid,
    final_grid,
    commitment_a,
    commitment_b,
    winner
).with_witnesses(pattern_a, pattern_b, nonce_a, nonce_b);

// Generate proof (expensive)
let proof = circuit.prove(&pk)?;

// Verify proof (fast)
let public_inputs = circuit.public_inputs()?;
let is_valid = BattleCircuit::verify(&vk, &proof, &public_inputs)?;
```

## Performance Targets (RC2-001.4)

### StateCircuit
- Proving Time: <20 seconds on 8-core CPU
- Verification: <10ms
- Proof Size: ~200 bytes

### BattleCircuit
- Proving Time: <30 seconds on 8-core CPU
- Verification: <10ms
- Proof Size: ~200 bytes

## Production Readiness Checklist

### Completed ✅
- [x] Full R1CS constraint implementations
- [x] Setup/prove/verify API for all circuits
- [x] Public input helpers
- [x] Comprehensive test coverage
- [x] Double-spend prevention constraints
- [x] Merkle proof validation
- [x] Conway's rule enforcement

### TODO for RC2 🔄
- [ ] Replace simplified hash with Poseidon in state_constraints
- [ ] Optimize BattleCircuit for 1024×1024 grid
- [ ] Trusted setup ceremony
- [ ] Publish verification keys
- [ ] GPU acceleration for proof generation
- [ ] Benchmark on production hardware
- [ ] Integration tests with consensus layer

### Security Considerations ⚠️

1. **Hash Function**: Current implementation uses a simplified hash (x² + x + 1) for testing. Production MUST use Poseidon hash from `poseidon_merkle.rs`.

2. **Trusted Setup**: Circuit-specific trusted setup is required. Toxic waste must be properly destroyed. Consider multi-party computation ceremony.

3. **Constraint Completeness**: All protocol invariants must be enforced in-circuit. The consensus layer should NOT rely on off-circuit validation.

4. **Nullifier Tracking**: The consensus layer must maintain a persistent set of used nullifiers and reject duplicate nullifiers.

## Testing

### Fast Tests (Always Run)
```bash
cargo test --package bitcell-zkp --lib
```
Runs 16 tests in ~15 seconds, including constraint satisfaction tests.

### Expensive Tests (Manual)
```bash
cargo test --package bitcell-zkp --lib -- --ignored
```
Runs proof generation tests. Requires significant time and memory:
- StateCircuit setup: ~5 minutes
- StateCircuit prove: ~20 seconds per proof
- BattleCircuit setup: ~5 minutes (64×64 grid)
- BattleCircuit prove: ~30 seconds per proof (64×64 grid)

Production configuration (1024×1024) requires 20GB+ RAM and is not suitable for CI.

## References

- [arkworks](https://github.com/arkworks-rs): ZK-SNARK library
- [Groth16](https://eprint.iacr.org/2016/260.pdf): Efficient pairing-based ZK-SNARK
- [Poseidon](https://eprint.iacr.org/2019/458.pdf): SNARK-friendly hash function
- [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life): Cellular automaton rules

## Support

For questions about circuit implementation, see:
- `crates/bitcell-zkp/src/state_constraints.rs` - State circuit implementation
- `crates/bitcell-zkp/src/battle_constraints.rs` - Battle circuit implementation  
- `crates/bitcell-zkp/src/poseidon_merkle.rs` - Production Poseidon hash
- `docs/RELEASE_REQUIREMENTS.md` - RC2 requirements and acceptance criteria
