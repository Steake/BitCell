# BitCell Architecture Overview

## System Design Philosophy

BitCell is designed around three core principles:

1. **Deterministic, Creative Proof-of-Work**: Replace hash lottery with cellular automaton battles
2. **Privacy-First Smart Contracts**: Zero-knowledge execution with Groth16 proofs
3. **Protocol-Local Trust**: Evidence-based reputation without external oracles

## Layer Architecture

### Layer 1: Cryptographic Primitives (`bitcell-crypto`)

**Purpose**: Foundation for all cryptographic operations

**Components**:
- **Hash Functions**: SHA-256 for general use, Poseidon for circuit-friendly operations
- **Digital Signatures**: ECDSA (secp256k1) for standard signing
- **Ring Signatures**: Anonymous participation in tournaments
- **VRF (Verifiable Random Functions)**: Unpredictable but verifiable randomness
- **Commitments**: Pedersen commitments for hiding values
- **Merkle Trees**: State commitments and proofs

**Key Files**:
- `src/hash.rs`: Hash256 wrapper, Hashable trait
- `src/signature.rs`: PublicKey, SecretKey, Signature
- `src/ring.rs`: RingSignature for tournament anonymity
- `src/vrf.rs`: VrfOutput, VrfProof for randomness
- `src/commitment.rs`: PedersenCommitment for value hiding
- `src/merkle.rs`: MerkleTree, MerkleProof

### Layer 2: Cellular Automaton Engine (`bitcell-ca`)

**Purpose**: Tournament battles via Conway-like CA rules

**Components**:
- **Grid**: 1024×1024 toroidal grid with 8-bit cell states (energy)
- **Rules**: Conway-like survival/birth rules with energy inheritance
- **Gliders**: Standard, LWSS, MWSS, HWSS patterns
- **Battles**: Deterministic 1000-step simulations with outcome determination

**Key Files**:
- `src/grid.rs`: Grid, Cell, Position
- `src/rules.rs`: evolve_cell, evolve_grid, parallel evolution
- `src/glider.rs`: GliderPattern enum, Glider struct
- `src/battle.rs`: Battle simulation, BattleOutcome

**Performance**:
- Parallel evolution using Rayon
- Toroidal wrapping for infinite-field behavior
- Energy-based outcome (not just cell count)

### Layer 3: Evidence-Based Subjective Logic (`bitcell-ebsl`)

**Purpose**: Protocol-local miner reputation and trust scoring

**Components**:
- **Evidence Counters**: `r_m` (positive), `s_m` (negative)
- **Subjective Logic Opinion**: Belief, disbelief, uncertainty
- **Trust Score**: Projected probability of honesty
- **Decay**: Asymmetric forgetting (fast positive, slow negative)
- **Slashing**: Deterministic penalties based on violation severity

**Key Files**:
- `src/evidence.rs`: EvidenceType, EvidenceCounters
- `src/trust.rs`: Opinion, TrustScore calculation
- `src/decay.rs`: Decay parameters and application
- `src/slashing.rs`: SlashingAction determination

**Trust Computation**:
```
R = r_m + s_m
belief = r_m / (R + K)
disbelief = s_m / (R + K)
uncertainty = K / (R + K)
trust = belief + α · uncertainty
```

With defaults: `K=2`, `α=0.4`, `T_MIN=0.75`, `T_KILL=0.2`

### Layer 4: Zero-Knowledge Proving (`bitcell-zkp`)

**Purpose**: Verifiable computation without revealing private data

**Circuits** (planned):
1. **Battle Circuit (`C_battle`)**:
   - Verifies: CA evolution, commitment consistency, outcome correctness
   - Public: commitments, winner ID, seed, spawn positions
   - Private: initial grid state, glider patterns, nonce

2. **Execution Circuit (`C_exec`)**:
   - Verifies: ZKVM execution of smart contract
   - Public: old state root, new state root, gas used
   - Private: plaintext state, contract code, witness

3. **State Transition Circuit (`C_state`)**:
   - Verifies: Merkle tree updates, nullifier correctness
   - Public: old root, new root, nullifiers
   - Private: Merkle paths, cleartext values

**Implementation Status**: v0.1 uses placeholder structures

### Layer 5: Consensus Protocol (`bitcell-consensus`)

**Purpose**: Tournament-based block production and fork choice

**Components** (planned):
- **Block Structure**: Header + body with VRF, proofs, transactions
- **Tournament Protocol**: Commit → Reveal → Battle → Propose
- **Fork Choice**: Heaviest chain (sum of deterministic work)
- **Eligibility**: Bonded miners with `trust ≥ T_MIN`

**Tournament Phases**:
1. **Eligibility Snapshot**: Compute active miner set `M_h`
2. **Commit Phase**: Ring-signed glider commitments
3. **Randomness**: VRF-derived tournament seed
4. **Pairing**: Deterministic bracket from seed
5. **Reveal Phase**: Pattern disclosure or forfeit
6. **Battle Phase**: CA simulations + proof generation
7. **Block Assembly**: Winner proposes block with all proofs

**Work Calculation**:
```
work_h = (N_h - 1) · BATTLE_STEPS · GRID_COST
```

Deterministic, not probabilistic.

### Layer 6: State Management (`bitcell-state`)

**Purpose**: Global state tracking for accounts, bonds, contracts

**Components** (planned):
- Account balances (public)
- Bond accounts (locked tokens)
- Contract storage (commitments only)
- Nullifier set (prevent double-spending)
- State root (Merkle tree)

**Privacy Model**:
- Balances: Public (for now)
- Contracts: Private (commitments + proofs only)
- State transitions: Verified via zkSNARKs

### Layer 7: ZKVM Execution (`bitcell-zkvm`)

**Purpose**: Private smart contract execution with zero-knowledge proofs

**Design**:
- RISC-V-inspired instruction set
- Field-friendly arithmetic (BN254 scalar field)
- Off-chain execution by prover
- On-chain verification by validators

**Workflow**:
```
1. User decrypts old state with private key
2. User executes contract function locally
3. User generates execution proof (C_exec)
4. User generates state transition proof (C_state)
5. User submits new commitment + proofs to chain
6. Validators verify proofs (never see plaintext)
```

### Layer 8: Economic Model (`bitcell-economics`)

**Purpose**: Block rewards, fees, treasury management

**Reward Distribution**:
```
Total = base_subsidy(height) + tx_fees + contract_fees

60% → Winner (block proposer)
30% → Participants (weighted by round reached)
10% → Treasury (governance, dev fund)
```

**Deterministic Payout**:
- Computed from tournament bracket
- Validated as part of block verification
- Winner cannot cheat payout schedule

**Fees**:
- Base fee (burned or treasury)
- Tip (goes to proposer)
- Privacy multiplier for contract calls

### Layer 9: Network Protocol (`bitcell-network`)

**Purpose**: P2P communication, gossip, block propagation

**Components** (planned):
- libp2p for transport
- Gossipsub for message propagation
- Kademlia for peer discovery
- Compact blocks for efficiency

**Message Types**:
- `Block`: Full block with proofs
- `GliderCommit`: Ring-signed commitment
- `GliderReveal`: Pattern disclosure
- `Transaction`: User transactions
- `BattleProof`: ZK proof for tournament battle

### Layer 10: Node Implementation (`bitcell-node`)

**Purpose**: Executable node software (miner, validator, light client)

**Node Types**:
1. **Miner Node**:
   - Holds bonded stake
   - Generates glider commitments
   - Participates in tournaments
   - Generates ZK proofs
   - Proposes blocks when winning

2. **Validator Node**:
   - Tracks full chain
   - Verifies all proofs
   - Maintains state tree
   - Relays blocks and transactions

3. **Light Client**:
   - Tracks headers only
   - Requests Merkle proofs on demand
   - Verifies individual proofs
   - Low resource usage

## Data Flow

### Block Production Flow

```
1. Epoch starts
   ↓
2. Compute eligible miners M_h (bond + trust check)
   ↓
3. Miners broadcast ring-signed commitments
   ↓
4. Combine VRF outputs → seed_h
   ↓
5. Deterministically pair miners from seed_h
   ↓
6. Miners reveal glider patterns
   ↓
7. Simulate battles (parallel)
   ↓
8. Generate battle proofs (C_battle)
   ↓
9. Tournament winner determined
   ↓
10. Winner executes pending transactions
   ↓
11. Winner generates exec proofs (C_exec) + state proofs (C_state)
   ↓
12. Winner assembles block + all proofs
   ↓
13. Validators verify all proofs
   ↓
14. Block appended if valid
   ↓
15. Update EBSL scores (evidence for all participants)
   ↓
16. Distribute rewards deterministically
   ↓
17. Next epoch
```

### Smart Contract Execution Flow

```
User side (off-chain):
1. Decrypt old state with private key
2. Execute contract function
3. Generate new commitment
4. Create execution proof (C_exec)
5. Create state transition proof (C_state)
6. Submit to mempool

Proposer side:
1. Include tx in block
2. Verify proofs locally
3. Update global state root

Validator side:
1. Receive block
2. Verify all execution proofs
3. Verify all state proofs
4. Accept block if valid
```

## Security Properties

### Consensus Security

- **No Grinding**: VRF seed depends on multiple blocks
- **No Withholding**: Non-reveal = forfeit + negative evidence
- **No Equivocation**: Double-signing detected → full slash + ban
- **Sybil Resistance**: Bond requirement + trust threshold

### Privacy Properties

- **Contract Privacy**: Validators never see plaintext
- **State Privacy**: Only commitments on-chain
- **Execution Privacy**: Proofs reveal nothing about computation
- **Anonymity**: Ring signatures hide tournament participants

### Liveness Properties

- **Guaranteed Progress**: Someone always wins tournament
- **No Stalling**: Missed commits/reveals = evidence penalty
- **Fork Resolution**: Heaviest chain rule (deterministic)

## Performance Characteristics

### Expected Metrics (v0.1 target)

- **Block Time**: 600 seconds (10 minutes)
- **TPS**: ~100 transactions/second
- **CA Simulation**: ~5 seconds (1024² grid, 1000 steps, parallel)
- **Battle Proof**: ~10 seconds generation, ~5ms verification
- **Exec Proof**: ~5 seconds generation, ~5ms verification
- **State Proof**: ~2 seconds generation, ~3ms verification

### Bottlenecks

1. **Proof Generation**: CPU-intensive (Groth16)
2. **CA Simulation**: Memory bandwidth (large grid)
3. **State Updates**: Merkle tree operations

### Optimizations

- Parallel CA evolution (Rayon)
- Batch proof verification
- Compact blocks (don't resend known data)
- Future: Recursive SNARKs for aggregation

## Future Work

### Short Term (v0.2)
- Complete ZK circuit implementations
- Full consensus protocol
- State management
- P2P networking
- Testnet launch

### Medium Term (v0.3)
- ZKVM optimization
- Light client support
- Mobile wallets
- Explorer UI
- Economic simulation

### Long Term (v1.0+)
- Recursive SNARK aggregation
- GPU-accelerated CA
- Cross-chain bridges
- Governance system
- Formal verification

## References

- **Cellular Automata**: Conway, J. H. (1970). "The Game of Life"
- **Subjective Logic**: Jøsang, A. (2016). "Subjective Logic: A Formalism for Reasoning Under Uncertainty"
- **zkSNARKs**: Groth, J. (2016). "On the Size of Pairing-Based Non-Interactive Arguments"
- **VRF**: Micali, S., et al. (1999). "Verifiable Random Functions"
- **Ring Signatures**: Rivest, R., et al. (2001). "How to Leak a Secret"
