# BitCell Governance

On-chain governance system for the BitCell blockchain.

## Overview

The governance system enables decentralized decision-making for protocol parameters, treasury spending, and protocol upgrades. It implements token-weighted voting with delegation support and guardian emergency controls.

## Features

- **Proposal System**: Submit and vote on three types of proposals:
  - Parameter changes (e.g., max block size, minimum stake)
  - Treasury spending (allocate funds from protocol treasury)
  - Protocol upgrades (deploy new protocol versions)

- **Voting Methods**:
  - Linear voting: 1 CELL = 1 vote
  - Quadratic voting: sqrt(CELL) = votes (Sybil-resistant)

- **Delegation**: Delegate your voting power to trusted representatives

- **Timelock Protection**:
  - Parameter changes: 2-day delay
  - Protocol upgrades: 2-day delay
  - Treasury spending: 6-hour delay

- **Guardian Controls**: 2-of-3 multi-sig for emergency actions
  - Cancel malicious proposals
  - Execute critical upgrades immediately

## Quick Start

```rust
use bitcell_governance::{GovernanceManager, ProposalType, VotingMethod, GuardianSet};

// Create governance manager
let mut gov = GovernanceManager::new();

// Submit a proposal
let proposer = [1u8; 33];
let proposal_id = gov.submit_proposal(
    proposer,
    ProposalType::ParameterChange {
        parameter: "max_block_size".to_string(),
        new_value: "2000000".to_string(),
    },
    "Increase max block size to 2MB".to_string(),
    current_timestamp,
)?;

// Vote on the proposal
let voter = [2u8; 33];
let voting_power = 10000 * 100_000_000; // 10,000 CELL
gov.vote(proposal_id, voter, true, voting_power, current_timestamp)?;

// Finalize after timelock expires
let passed = gov.finalize_proposal(proposal_id, current_timestamp + (2 * 24 * 60 * 60))?;

if passed {
    println!("Proposal passed!");
}
```

## Delegation

```rust
// Delegate voting power
let delegator = [1u8; 33];
let delegatee = [2u8; 33];
let amount = 5000 * 100_000_000; // 5,000 CELL

gov.delegate(delegator, delegatee, amount)?;

// Delegatee now has additional voting power
let total_power = gov.get_voting_power(&delegatee, base_power);
```

## Guardian Override

```rust
use bitcell_governance::{Guardian, GuardianAction};

// Setup guardians
let guardians = GuardianSet::with_guardians(vec![
    Guardian {
        pubkey: [1u8; 33],
        name: "Guardian 1".to_string(),
        added_at: timestamp,
    },
    Guardian {
        pubkey: [2u8; 33],
        name: "Guardian 2".to_string(),
        added_at: timestamp,
    },
    Guardian {
        pubkey: [3u8; 33],
        name: "Guardian 3".to_string(),
        added_at: timestamp,
    },
]);

// Emergency cancel (requires 2/3 signatures)
let signatures = vec![signature1, signature2]; // 64-byte signatures
gov.guardian_override(proposal_id, GuardianAction::Cancel, signatures)?;
```

## Configuration

```rust
use bitcell_governance::{GovernanceConfig, VotingMethod, GuardianThreshold, TimelockConfig};

let config = GovernanceConfig {
    quorum: 10_000 * 100_000_000, // 10,000 CELL minimum
    voting_method: VotingMethod::Quadratic,
    guardian_threshold: GuardianThreshold {
        required: 2,
        total: 3,
    },
    timelock: TimelockConfig {
        parameter_change_delay: 2 * 24 * 60 * 60,  // 2 days
        treasury_spending_delay: 6 * 60 * 60,       // 6 hours
        protocol_upgrade_delay: 2 * 24 * 60 * 60,   // 2 days
    },
};

let gov = GovernanceManager::with_config(config, guardians);
```

## Security Features

- **Overflow Protection**: All arithmetic uses saturating operations
- **Collision Resistance**: Proposal IDs use SHA-256 hashing
- **Double-Vote Prevention**: Each address can only vote once per proposal
- **Quadratic Voting**: Reduces influence of large token holders
- **Timelock Delays**: Prevent hasty or malicious changes
- **Guardian Multi-sig**: Emergency response capability

## Testing

```bash
cargo test -p bitcell-governance
```

## Benchmarks

```bash
cargo bench -p bitcell-governance
```

## License

MIT OR Apache-2.0
