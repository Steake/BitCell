# Finality Gadget Design

## Overview

The BitCell finality gadget implements BFT (Byzantine Fault Tolerant) finality inspired by Tendermint and GRANDPA. It provides fast, deterministic finalization for blocks with the following properties:

- **2/3+ stake agreement** required for finality
- **Blocks become irreversible** after finalization
- **Target finality time < 1 minute**
- **Double-sign slashing** with cryptographic evidence
- **Evidence submission mechanism** for equivocation

## Architecture

### Vote Types

The finality protocol uses two types of votes:

1. **Prevote**: First round vote indicating a validator's preferred block
2. **Precommit**: Second round vote committing to finalize a block

Both vote types require signatures from validators and are tracked separately.

### Finality States

Blocks progress through three finality states:

```
Pending → Prevoted → Finalized
```

- **Pending**: Initial state, insufficient votes
- **Prevoted**: ≥2/3 stake has prevoted
- **Finalized**: ≥2/3 stake has precommitted (irreversible)

### Vote Threshold

The threshold for finality is:

```
threshold = (total_stake * 2) / 3
```

A block reaches the next finality state when the accumulated stake for that vote type exceeds the threshold.

## Protocol Flow

### 1. Block Proposal
```
Proposer → Broadcasts Block
```

### 2. Prevote Phase
```
Validators → Examine Block
           → Sign Prevote
           → Broadcast Prevote

Finality Gadget → Collects Prevotes
                 → Checks Threshold
                 → Block Status: Prevoted (if ≥2/3)
```

### 3. Precommit Phase
```
Validators → See Prevoted Block
           → Sign Precommit
           → Broadcast Precommit

Finality Gadget → Collects Precommits
                 → Checks Threshold
                 → Block Status: Finalized (if ≥2/3)
```

### 4. Finalization
```
Block Finalized → Irreversible
                → Cannot be reverted
                → Safe for downstream systems
```

## Equivocation Detection

### What is Equivocation?

Equivocation (double-signing) occurs when a validator signs conflicting votes:

```
Validator signs Vote A: Block Hash X, Height H, Round R, Type T
Validator signs Vote B: Block Hash Y, Height H, Round R, Type T

Where X ≠ Y → Equivocation!
```

### Detection Mechanism

The finality gadget maintains a vote history:

```rust
vote_history: HashMap<(height, round, vote_type, validator), block_hash>
```

When a new vote arrives:

1. Check if validator already voted at this (height, round, type)
2. If yes, compare block hashes
3. If different → Create equivocation evidence
4. Evidence includes both conflicting votes with signatures

### Evidence Structure

```rust
pub struct EquivocationEvidence {
    pub vote1: FinalityVote,  // First vote
    pub vote2: FinalityVote,  // Conflicting vote
    pub evidence_height: u64,
}
```

Evidence validation ensures:
- Both votes are from same validator
- Both votes are for same height/round/type
- Both votes are for different blocks
- Both signatures are valid

## Slashing Integration

### Evidence Submission

When equivocation is detected:

```rust
let evidence = EquivocationEvidence { ... };
state_manager.submit_evidence(validator, Evidence {
    evidence_type: EvidenceType::Equivocation,
    epoch,
    block_height,
});
```

### Automatic Slashing

Equivocation triggers:

```rust
let action = determine_slashing(
    EvidenceType::Equivocation,
    trust,
    params,
);

// Always returns: SlashingAction::FullAndBan
state_manager.apply_slashing(validator, action);
```

**Consequences:**
- 100% of bonded stake slashed
- Permanent ban from validation
- Evidence recorded on-chain

## Round Progression

The protocol uses rounds to handle network delays:

### Round Structure

```
Round 0: Initial voting
Round 1: Retry after timeout
Round 2: ...
```

### Timeout Handling

If consensus is not reached within the round timeout (<1 minute):

```rust
gadget.advance_round();  // Move to next round
```

Validators can vote again in new rounds without equivocation.

### Important Properties

- Same validator can vote in different rounds (not equivocation)
- Cannot vote twice in same round (is equivocation)
- Round number is part of vote signature

## Vote Signature

### Signature Message

```rust
fn sign_message(&self) -> Vec<u8> {
    let mut msg = Vec::new();
    msg.extend_from_slice(self.block_hash.as_bytes());
    msg.extend_from_slice(&self.block_height.to_le_bytes());
    msg.push(vote_type);  // 0 = Prevote, 1 = Precommit
    msg.extend_from_slice(&self.round.to_le_bytes());
    msg
}
```

### Verification

```rust
signature.verify(&validator, &message)
```

All votes are cryptographically verified before being counted.

## Weighted Stake

The finality gadget supports validators with different stake amounts:

```rust
validator_stakes: HashMap<PublicKey, u64>
```

Example:
```
Validator A: 500 stake (50%)
Validator B: 300 stake (30%)  
Validator C: 200 stake (20%)
Total: 1000 stake

Finality requires: 667+ stake (66.7%)
```

A + B (800 stake) can finalize
A + C (700 stake) can finalize
B + C (500 stake) cannot finalize

## Security Properties

### Byzantine Fault Tolerance

The 2/3+ threshold ensures safety even with up to 1/3 Byzantine validators:

- Honest validators: > 2/3 stake
- Byzantine validators: < 1/3 stake
- Byzantine validators cannot:
  - Finalize conflicting blocks
  - Prevent finalization indefinitely
  - Avoid detection if they equivocate

### Finality Guarantees

Once a block is finalized:

1. **Safety**: No conflicting block can be finalized
2. **Liveness**: New blocks can always be finalized (if >2/3 honest)
3. **Accountability**: Any Byzantine behavior is provably attributable

### Evidence Cryptography

Equivocation evidence provides:

- **Non-repudiation**: Validator cannot deny signing
- **Verifiability**: Anyone can verify the evidence
- **Completeness**: Both conflicting votes with full signatures

## Performance Characteristics

### Expected Timings

Under normal conditions:

```
Prevote Phase:     10-20 seconds
Precommit Phase:   10-20 seconds
Total Finality:    20-40 seconds
```

With network delays:

```
Round 0:           ~30 seconds
Round 1 (timeout): ~30 seconds  
Round 2 (timeout): ~30 seconds
Max timeout:       < 1 minute per spec
```

### Scalability

Vote collection scales with validator set size:

- O(1) vote verification per vote
- O(N) votes per block (N = validator count)
- O(1) finality status check

Recommended validator set sizes:
- Minimum: 4 validators
- Optimal: 10-100 validators
- Maximum: 1000+ validators (tested)

## Integration Example

```rust
use bitcell_consensus::{FinalityGadget, FinalityVote, VoteType};

// Initialize gadget with validator set
let mut gadget = FinalityGadget::new(validator_stakes);

// Collect votes
for vote in incoming_votes {
    match gadget.add_vote(vote) {
        Ok(()) => {
            // Vote accepted
        }
        Err(evidence) => {
            // Equivocation detected!
            submit_evidence_to_chain(evidence);
            slash_validator(evidence.vote1.validator);
        }
    }
}

// Check finality
if gadget.is_finalized(&block_hash) {
    mark_block_irreversible(block_hash);
}
```

## Future Enhancements

### Optimizations

1. **Vote Aggregation**: Combine multiple votes into single messages
2. **Signature Aggregation**: BLS signatures for smaller proofs
3. **Light Client Support**: Merkle proofs for finality status

### Protocol Extensions

1. **Fast Finality**: Optimistic finality in <10 seconds
2. **Checkpointing**: Periodic finality checkpoints
3. **Cross-Chain**: Finality proofs for bridges

## References

- **Tendermint**: Two-phase BFT consensus
- **GRANDPA**: Chain finality gadget (Polkadot)
- **PBFT**: Original BFT consensus algorithm
- **Casper FFG**: Ethereum finality gadget
