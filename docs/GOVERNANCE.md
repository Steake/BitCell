# On-Chain Governance System

## Overview

The BitCell On-Chain Governance System (RC3-005) provides a decentralized mechanism for protocol evolution and treasury management. It implements a three-phase governance process: proposal submission, token-weighted voting, and timelocked execution with guardian oversight.

## Architecture

### Core Components

1. **Proposal System** (`proposal.rs`)
   - Parameter changes for protocol configuration
   - Treasury spending proposals
   - Protocol upgrade proposals

2. **Voting Mechanism** (`voting.rs`)
   - Token-weighted voting (1 CELL = 1 vote by default)
   - Optional quadratic voting (power = √tokens)
   - Vote delegation support

3. **Execution System** (`execution.rs`)
   - Timelock delays for proposal execution
   - Multi-sig guardian controls
   - Emergency cancellation capabilities

## Proposal Types

### Parameter Change
```rust
ProposalType::ParameterChange {
    parameter: String,
    new_value: Vec<u8>,
}
```
Used for modifying protocol parameters such as block time, gas limits, or economic parameters.

### Treasury Spending
```rust
ProposalType::TreasurySpending {
    recipient: [u8; 33],
    amount: u64,
    reason: String,
}
```
Used for allocating funds from the protocol treasury to development, grants, or other initiatives.

### Protocol Upgrade
```rust
ProposalType::ProtocolUpgrade {
    version: String,
    code_hash: [u8; 32],
    description: String,
}
```
Used for coordinating protocol upgrades and hard forks.

## Voting Process

### 1. Proposal Submission

```rust
let proposal_id = governance_manager.submit_proposal(
    proposer_pubkey,
    proposal_type,
    "Proposal description",
    voting_period_blocks,  // e.g., 14400 blocks (~2 days)
    current_block,
)?;
```

### 2. Voting

Voters can cast their votes during the voting period:

```rust
governance_manager.vote(
    proposal_id,
    voter_pubkey,
    VoteType::For,  // or Against, or Abstain
    token_balance,
    current_block,
    quadratic: false,  // use quadratic voting if true
)?;
```

#### Voting Options

- **For**: Support the proposal
- **Against**: Oppose the proposal
- **Abstain**: Count toward quorum without affecting the outcome

#### Voting Power Calculation

**Linear Voting (default)**:
```
voting_power = token_balance
```

**Quadratic Voting**:
```
voting_power = sqrt(token_balance)
```

Quadratic voting reduces the influence of large token holders, promoting more egalitarian decision-making.

### 3. Vote Delegation

Token holders can delegate their voting power to trusted representatives:

```rust
governance_manager.delegate(
    delegator_pubkey,
    delegatee_pubkey,
    amount,
)?;
```

The delegatee's effective voting power includes both their own tokens and delegated tokens.

### 4. Proposal Finalization

After the voting period ends, proposals must be finalized:

```rust
governance_manager.finalize_proposal(
    proposal_id,
    current_block,
)?;
```

A proposal passes if:
1. **Quorum is met**: Total votes ≥ quorum threshold (default: 10,000 CELL)
2. **Majority support**: Votes for > Votes against

### 5. Timelocked Execution

Passed proposals enter a timelock period before execution:

| Proposal Type | Timelock Period |
|---------------|-----------------|
| Parameter Change | 14,400 blocks (~2 days) |
| Treasury Spending | 1,800 blocks (~6 hours) |
| Protocol Upgrade | 14,400 blocks (~2 days) |

```rust
governance_manager.execute_proposal(
    proposal_id,
    current_block,
)?;
```

## Guardian Controls

Multi-sig guardians provide emergency oversight with the following capabilities:

### Emergency Cancellation

Guardians can cancel malicious or erroneous proposals:

```rust
governance_manager.emergency_cancel(
    proposal_id,
    guardian_signatures,  // Requires 2/3 majority
)?;
```

### Fast-Track Execution

Guardians can reduce timelock delays for urgent proposals:

```rust
governance_manager.execution_queue.fast_track(
    proposal_id,
    current_block,
)?;
```

## Security Features

### Timelock Protection

All passed proposals must wait through a timelock period, providing:
- Time for community review
- Opportunity for guardian intervention if needed
- Protection against hasty or malicious changes

### Guardian Multi-Sig

Guardian actions require a 2/3 supermajority:
```rust
required_signatures = (total_guardians * 2 + 2) / 3
```

This prevents individual guardians from unilaterally controlling the system.

### Double-Vote Prevention

Each address can only vote once per proposal. Attempted double-voting returns an error:
```rust
Error::AlreadyVoted
```

## Usage Example

### Complete Governance Flow

```rust
use bitcell_governance::*;

// Initialize governance with guardians
let guardians = vec![guardian1_pubkey, guardian2_pubkey, guardian3_pubkey];
let mut governance = GovernanceManager::new(guardians);

// Submit a proposal to adjust block reward
let proposal_id = governance.submit_proposal(
    proposer_pubkey,
    ProposalType::ParameterChange {
        parameter: "block_reward".to_string(),
        new_value: 40u64.to_le_bytes().to_vec(),  // Reduce from 50 to 40
    },
    "Reduce block reward to extend emission schedule".to_string(),
    14400,  // 2-day voting period
    current_block,
)?;

// Community members vote
governance.vote(proposal_id, voter1_pubkey, VoteType::For, 10000, current_block, false)?;
governance.vote(proposal_id, voter2_pubkey, VoteType::For, 8000, current_block, false)?;
governance.vote(proposal_id, voter3_pubkey, VoteType::Against, 3000, current_block, false)?;

// After voting period, finalize the proposal
governance.finalize_proposal(proposal_id, current_block + 14500)?;

// After timelock expires, execute the proposal
governance.execute_proposal(proposal_id, current_block + 14500 + 14400)?;
```

## Proposal Status States

```rust
pub enum ProposalStatus {
    Active,      // Accepting votes
    Passed,      // Passed, waiting for execution
    Rejected,    // Failed to meet quorum or majority
    Executed,    // Successfully executed
    Cancelled,   // Cancelled by guardians
}
```

## Error Handling

The governance system defines comprehensive error types:

```rust
pub enum Error {
    ProposalNotFound,          // Invalid proposal ID
    InvalidProposal,           // Proposal in wrong state
    VotingPeriodEnded,         // Attempted to vote after deadline
    VotingPeriodNotEnded,      // Attempted to finalize too early
    AlreadyVoted,              // Double-vote attempt
    InsufficientVotingPower,   // Not enough tokens
    ExecutionLocked,           // Timelock not expired
    NotAuthorized,             // Insufficient guardian signatures
    InvalidTimelock,           // Invalid timelock configuration
}
```

## Best Practices

### For Proposers

1. **Clear Description**: Provide detailed rationale and expected impact
2. **Appropriate Timelock**: Consider using longer timelocks for major changes
3. **Community Engagement**: Discuss proposals before submission
4. **Parameter Validation**: Ensure proposed values are technically sound

### For Voters

1. **Research**: Review proposal details and community discussion
2. **Consider Delegation**: Delegate to experts if uncertain
3. **Vote Early**: Don't wait until the last minute
4. **Use Quadratic Voting**: Consider using quadratic voting for fairer representation

### For Guardians

1. **Minimal Intervention**: Only use emergency powers for actual emergencies
2. **Transparency**: Document reasons for guardian actions
3. **Community Alignment**: Ensure actions align with community values
4. **Coordination**: Require multiple guardians to agree before acting

## Integration with BitCell

The governance system integrates with other BitCell components:

- **State Management**: Proposals can modify state parameters
- **Economics**: Treasury spending affects token distribution
- **Consensus**: Protocol upgrades coordinate network changes

## Future Enhancements

Potential future improvements include:

1. **On-chain execution**: Automatic parameter changes upon execution
2. **Proposal templates**: Standardized formats for common proposal types
3. **Reputation weighting**: Factor trust scores into voting power
4. **Time-weighted voting**: Give more weight to long-term holders
5. **Conviction voting**: Lock tokens to increase voting power
6. **Liquid democracy**: Transitive delegation chains

## Testing

The governance system includes comprehensive tests covering:

- Proposal submission and lifecycle
- Voting mechanics (linear and quadratic)
- Delegation functionality
- Timelock enforcement
- Guardian controls
- Error conditions

Run tests with:
```bash
cargo test -p bitcell-governance
```

All 20 tests pass successfully, ensuring robust functionality.

## References

- RC3-005: Governance System Requirements
- `docs/RELEASE_REQUIREMENTS.md`: Detailed RC3-005 specification
- Academic references on quadratic voting and liquid democracy
