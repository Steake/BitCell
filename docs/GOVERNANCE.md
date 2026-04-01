# BitCell Governance System

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Proposal Types](#proposal-types)
4. [Voting Mechanisms](#voting-mechanisms)
5. [Delegation System](#delegation-system)
6. [Timelock Protection](#timelock-protection)
7. [Guardian Controls](#guardian-controls)
8. [Security Considerations](#security-considerations)
9. [API Reference](#api-reference)
10. [Examples](#examples)

## Overview

The BitCell governance system enables decentralized protocol management through on-chain proposals and token-weighted voting. Token holders can propose changes, vote on proposals, delegate their voting power, and execute approved proposals after a timelock period.

### Key Features

- **Three Proposal Types**: Parameter changes, treasury spending, and protocol upgrades
- **Flexible Voting**: Linear (1 CELL = 1 vote) or quadratic (√CELL = votes) methods
- **Vote Delegation**: Delegate voting power to trusted representatives
- **Time-Delayed Execution**: Type-specific timelock delays for security
- **Emergency Controls**: Multi-sig guardian system for critical situations

## Architecture

### Components

```
GovernanceManager
├── Proposals (HashMap<ProposalId, Proposal>)
├── Votes (HashMap<ProposalId, Vec<VoteRecord>>)
├── DelegationManager
├── GuardianSet
└── GovernanceConfig
```

### Proposal Lifecycle

```
Submit → Active → Voting → Timelock → Execute/Reject
                            ↓
                     Guardian Override (optional)
```

1. **Submit**: Proposer creates a new proposal
2. **Active**: Proposal accepts votes
3. **Voting**: Token holders vote for/against
4. **Timelock**: Waiting period after vote passes
5. **Execute**: Proposal is finalized and executed

Guardians can intervene at any stage for emergency actions.

## Proposal Types

### Parameter Change

Modify protocol parameters such as:
- Maximum block size
- Minimum stake amount
- Fee parameters
- Timeout values

**Timelock**: 2 days

```rust
ProposalType::ParameterChange {
    parameter: "max_block_size".to_string(),
    new_value: "2000000".to_string(),
}
```

### Treasury Spending

Allocate funds from the protocol treasury for:
- Development grants
- Marketing campaigns
- Infrastructure costs
- Ecosystem support

**Timelock**: 6 hours

```rust
ProposalType::TreasurySpending {
    recipient: recipient_address,
    amount: 100_000 * 100_000_000, // 100,000 CELL
    reason: "Development grant Q1 2026".to_string(),
}
```

### Protocol Upgrade

Deploy new protocol versions:
- Bug fixes
- Feature additions
- Performance improvements
- Security patches

**Timelock**: 2 days

```rust
ProposalType::ProtocolUpgrade {
    version: "1.1.0".to_string(),
    code_hash: upgrade_hash,
    description: "Add finality gadget".to_string(),
}
```

## Voting Mechanisms

### Linear Voting

Default voting method where each CELL token equals one vote.

**Formula**: `voting_power = token_balance`

**Pros**:
- Simple and intuitive
- Transparent weighting

**Cons**:
- Susceptible to whale control
- Less Sybil-resistant

### Quadratic Voting

Alternative method where voting power is the square root of token balance.

**Formula**: `voting_power = √token_balance`

**Pros**:
- Reduces influence of large holders
- More Sybil-resistant
- Encourages broader participation

**Cons**:
- More complex calculation
- Lower voting power for small holders

### Example Comparison

| Token Balance | Linear Votes | Quadratic Votes |
|--------------|--------------|-----------------|
| 100 CELL     | 100          | 10              |
| 10,000 CELL  | 10,000       | 100             |
| 1,000,000 CELL | 1,000,000  | 1,000           |

A holder with 10,000x more tokens only gets 100x more votes in quadratic voting.

## Delegation System

### Overview

Token holders can delegate their voting power to representatives without transferring tokens.

### Features

- **Non-custodial**: Tokens never leave your wallet
- **Flexible**: Delegate to multiple addresses
- **Revocable**: Remove delegations at any time
- **Accumulative**: Multiple delegators can delegate to one address

### Delegation Example

```rust
// Alice delegates 5,000 CELL to Bob
gov.delegate(alice_address, bob_address, 5000 * 100_000_000)?;

// Bob now has additional voting power
let bob_power = gov.get_voting_power(&bob_address, bob_balance);
// bob_power = bob_balance + 5000 CELL

// Carol also delegates to Bob
gov.delegate(carol_address, bob_address, 3000 * 100_000_000)?;

// Bob's power increases
let bob_power = gov.get_voting_power(&bob_address, bob_balance);
// bob_power = bob_balance + 5000 + 3000 CELL

// Alice can revoke her delegation
gov.undelegate(alice_address, bob_address)?;
```

### Use Cases

1. **Representative Democracy**: Delegate to active community members
2. **Expert Voting**: Delegate to technical experts for protocol decisions
3. **Time Constraints**: Delegate when unable to actively participate

## Timelock Protection

### Purpose

Timelock delays prevent:
- Hasty decisions
- Flash loan attacks
- Malicious proposals
- Insufficient review time

### Delay Periods

| Proposal Type | Delay |
|--------------|-------|
| Parameter Change | 2 days |
| Protocol Upgrade | 2 days |
| Treasury Spending | 6 hours |

Treasury spending has a shorter delay to allow quick operational funding while still providing review time.

### Implementation

```rust
// After vote passes, check timelock
let timelock_duration = config.timelock.get_duration(&proposal.proposal_type);
let expiry = proposal.created_at + timelock_duration;

if current_time < expiry {
    return Err(Error::TimelockNotExpired {
        remaining_seconds: expiry - current_time,
    });
}
```

## Guardian Controls

### Purpose

Guardians provide emergency response capability for:
- Malicious proposals
- Security vulnerabilities
- Critical bugs
- Time-sensitive situations

### Guardian Threshold

**Default**: 2-of-3 multi-sig
- Prevents single point of failure
- Requires majority agreement
- Balances speed and security

### Guardian Actions

#### Cancel Proposal

Immediately cancel a malicious or flawed proposal:

```rust
gov.guardian_override(
    proposal_id,
    GuardianAction::Cancel,
    vec![signature1, signature2],
)?;
```

#### Execute Immediately

Bypass timelock for critical fixes:

```rust
gov.guardian_override(
    proposal_id,
    GuardianAction::ExecuteImmediately,
    vec![signature1, signature2],
)?;
```

### Guardian Selection

Guardians should be:
- Reputable community members
- Geographically distributed
- Technically competent
- Available for emergencies

## Security Considerations

### Overflow Protection

All arithmetic uses saturating operations:

```rust
proposal.votes_for = proposal.votes_for.saturating_add(power);
```

This prevents:
- Integer overflow attacks
- Underflow vulnerabilities
- Vote count manipulation

### Proposal ID Collision Resistance

Proposal IDs use SHA-256 hashing:

```rust
let id = SHA256(proposer || proposal_type || description || timestamp);
```

This ensures:
- Unique proposal identifiers
- Collision resistance (2^128 security)
- Deterministic ID generation

### Double-Vote Prevention

Each address can vote only once per proposal:

```rust
if vote_records.iter().any(|v| v.voter == voter) {
    return Err(Error::DuplicateVote);
}
```

### Quorum Requirements

Default quorum: 10,000 CELL

Prevents:
- Low-participation attacks
- Unrepresentative decisions
- Governance capture

### Attack Vectors and Mitigations

| Attack | Mitigation |
|--------|-----------|
| Vote buying | Quadratic voting reduces incentive |
| Whale control | Quadratic voting + delegation |
| Flash loans | Timelock delays |
| Sybil attacks | Quadratic voting |
| Spam proposals | Quorum requirements |
| Emergency exploit | Guardian override |

## API Reference

### GovernanceManager

```rust
impl GovernanceManager {
    // Create new instance
    pub fn new() -> Self;
    pub fn with_config(config: GovernanceConfig, guardians: GuardianSet) -> Self;
    
    // Proposal management
    pub fn submit_proposal(
        &mut self,
        proposer: [u8; 33],
        proposal_type: ProposalType,
        description: String,
        created_at: u64,
    ) -> Result<ProposalId>;
    
    pub fn get_proposal(&self, proposal_id: &ProposalId) -> Option<&Proposal>;
    
    // Voting
    pub fn vote(
        &mut self,
        proposal_id: ProposalId,
        voter: [u8; 33],
        support: bool,
        voting_power: u64,
        timestamp: u64,
    ) -> Result<()>;
    
    pub fn get_votes(&self, proposal_id: &ProposalId) -> Option<&Vec<VoteRecord>>;
    
    // Finalization
    pub fn finalize_proposal(
        &mut self,
        proposal_id: ProposalId,
        current_time: u64,
    ) -> Result<bool>;
    
    // Delegation
    pub fn delegate(
        &mut self,
        delegator: [u8; 33],
        delegatee: [u8; 33],
        amount: u64,
    ) -> Result<()>;
    
    pub fn undelegate(
        &mut self,
        delegator: [u8; 33],
        delegatee: [u8; 33],
    ) -> Result<()>;
    
    pub fn get_voting_power(&self, voter: &[u8; 33], base_power: u64) -> u64;
    
    // Guardian controls
    pub fn guardian_override(
        &mut self,
        proposal_id: ProposalId,
        action: GuardianAction,
        signatures: Vec<[u8; 64]>,
    ) -> Result<()>;
}
```

## Examples

### Complete Proposal Flow

```rust
use bitcell_governance::*;

fn main() -> Result<()> {
    // Setup
    let mut gov = GovernanceManager::new();
    let current_time = 1704067200; // Jan 1, 2024
    
    // Submit proposal
    let proposer = [1u8; 33];
    let proposal_id = gov.submit_proposal(
        proposer,
        ProposalType::ParameterChange {
            parameter: "max_block_size".to_string(),
            new_value: "2000000".to_string(),
        },
        "Increase block size for better throughput".to_string(),
        current_time,
    )?;
    
    println!("Proposal submitted: {}", hex::encode(&proposal_id.0));
    
    // Multiple voters vote
    let voters = vec![
        ([2u8; 33], 15000 * 100_000_000, true),   // 15K CELL - for
        ([3u8; 33], 8000 * 100_000_000, true),    // 8K CELL - for
        ([4u8; 33], 2000 * 100_000_000, false),   // 2K CELL - against
    ];
    
    for (voter, power, support) in voters {
        gov.vote(proposal_id, voter, support, power, current_time + 3600)?;
        println!("Vote cast: {} with {} power, support: {}",
            hex::encode(&voter), power, support);
    }
    
    // Check proposal status
    let proposal = gov.get_proposal(&proposal_id).unwrap();
    println!("Votes for: {}, against: {}", proposal.votes_for, proposal.votes_against);
    println!("Vote percentage: {:.2}%", proposal.vote_percentage_for());
    
    // Wait for timelock (2 days for parameter change)
    let after_timelock = current_time + (2 * 24 * 60 * 60) + 1;
    
    // Finalize
    let passed = gov.finalize_proposal(proposal_id, after_timelock)?;
    
    if passed {
        println!("Proposal passed and executed!");
    } else {
        println!("Proposal rejected");
    }
    
    Ok(())
}
```

### Using Quadratic Voting

```rust
use bitcell_governance::*;

let config = GovernanceConfig {
    quorum: 10_000 * 100_000_000,
    voting_method: VotingMethod::Quadratic,
    ..Default::default()
};

let mut gov = GovernanceManager::with_config(config, GuardianSet::new());

// With 10,000 CELL, get sqrt(10,000) = 100 votes
let voter = [1u8; 33];
let power = 10_000 * 100_000_000;

gov.vote(proposal_id, voter, true, power, timestamp)?;

// Effective voting power is 100 (sqrt of 10,000)
let proposal = gov.get_proposal(&proposal_id).unwrap();
assert_eq!(proposal.votes_for, 100);
```

### Delegation Workflow

```rust
use bitcell_governance::*;

let mut gov = GovernanceManager::new();

// Alice delegates to Bob
let alice = [1u8; 33];
let bob = [2u8; 33];
let delegation_amount = 5000 * 100_000_000; // 5K CELL

gov.delegate(alice, bob, delegation_amount)?;

// Bob votes with his balance + delegated power
let bob_balance = 3000 * 100_000_000; // 3K CELL
let total_power = gov.get_voting_power(&bob, bob_balance);
// total_power = 8K CELL (3K + 5K delegated)

gov.vote(proposal_id, bob, true, total_power, timestamp)?;

// Later, Alice revokes delegation
gov.undelegate(alice, bob)?;
```

### Guardian Emergency Response

```rust
use bitcell_governance::*;
use bitcell_crypto::{SecretKey, PublicKey};

// Setup guardians
let guardian_keys: Vec<SecretKey> = vec![
    SecretKey::generate(),
    SecretKey::generate(),
    SecretKey::generate(),
];

let guardians = GuardianSet::with_guardians(vec![
    Guardian {
        pubkey: guardian_keys[0].public_key().to_bytes(),
        name: "Guardian Alpha".to_string(),
        added_at: timestamp,
    },
    Guardian {
        pubkey: guardian_keys[1].public_key().to_bytes(),
        name: "Guardian Beta".to_string(),
        added_at: timestamp,
    },
    Guardian {
        pubkey: guardian_keys[2].public_key().to_bytes(),
        name: "Guardian Gamma".to_string(),
        added_at: timestamp,
    },
]);

let mut gov = GovernanceManager::with_config(
    GovernanceConfig::default(),
    guardians,
);

// Malicious proposal detected
// Get 2 guardian signatures
let message = &proposal_id.0;
let sig1 = guardian_keys[0].sign(message).to_bytes();
let sig2 = guardian_keys[1].sign(message).to_bytes();

// Cancel the proposal
gov.guardian_override(
    proposal_id,
    GuardianAction::Cancel,
    vec![sig1, sig2],
)?;

println!("Malicious proposal cancelled by guardian override");
```

## Best Practices

1. **Proposal Descriptions**: Write clear, detailed descriptions
2. **Reasonable Quorum**: Set quorum appropriate to token distribution
3. **Guardian Selection**: Choose diverse, trusted guardians
4. **Vote Participation**: Actively participate or delegate
5. **Review Period**: Use timelock period to review proposals
6. **Test Changes**: Test parameter changes in testnet first
7. **Monitor Proposals**: Track active proposals regularly

## Future Enhancements

- [ ] Proposal deposits (stake to submit)
- [ ] Vote commit-reveal scheme
- [ ] Conviction voting
- [ ] Futarchy integration
- [ ] Cross-chain governance
- [ ] Snapshot voting
- [ ] Reputation-weighted voting

## References

- [Quadratic Voting](https://en.wikipedia.org/wiki/Quadratic_voting)
- [On-Chain Governance](https://vitalik.ca/general/2021/08/16/voting3.html)
- [Governance Attacks](https://blog.openzeppelin.com/on-chain-governance-analysis)

## License

MIT OR Apache-2.0
