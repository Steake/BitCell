# bitcell-governance

On-chain governance system for the BitCell blockchain, implementing RC3-005 requirements.

## Features

- **Proposal System**: Submit proposals for parameter changes, treasury spending, and protocol upgrades
- **Token-Weighted Voting**: Democratic voting with 1 CELL = 1 vote (linear) or quadratic voting option
- **Vote Delegation**: Delegate voting power to trusted representatives
- **Timelock Execution**: Mandatory waiting period before proposal execution
- **Guardian Controls**: Multi-sig emergency cancellation and fast-track capabilities
- **Comprehensive Testing**: 20 unit tests covering all functionality

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
bitcell-governance = { path = "crates/bitcell-governance" }
```

## Usage

```rust
use bitcell_governance::*;

// Create governance manager with guardians
let guardians = vec![guardian1, guardian2, guardian3];
let mut governance = GovernanceManager::new(guardians);

// Submit a proposal
let proposal_id = governance.submit_proposal(
    proposer_pubkey,
    ProposalType::ParameterChange {
        parameter: "block_time".to_string(),
        new_value: vec![10],
    },
    "Reduce block time to 10s".to_string(),
    14400,  // voting period in blocks
    current_block,
)?;

// Vote on the proposal
governance.vote(
    proposal_id,
    voter_pubkey,
    VoteType::For,
    token_balance,
    current_block,
    false,  // quadratic voting
)?;

// Finalize after voting period
governance.finalize_proposal(proposal_id, current_block + 15000)?;

// Execute after timelock
governance.execute_proposal(proposal_id, current_block + 30000)?;
```

## Documentation

See `docs/GOVERNANCE.md` for comprehensive documentation including:

- Architecture overview
- Proposal types
- Voting process
- Guardian controls
- Security features
- Best practices
- Integration guide

## Testing

```bash
cargo test -p bitcell-governance
```

All tests pass:
- 20 unit tests
- Coverage of all major functionality
- Edge case testing

## License

MIT OR Apache-2.0
