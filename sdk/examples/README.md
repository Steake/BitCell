# BitCell Smart Contract Examples

This directory contains working examples demonstrating various contract patterns and features.

## Available Examples

### 1. Simple Counter (`counter.zkasm`)
A basic counter contract demonstrating:
- State storage
- State updates
- Access control

### 2. Token Swap (`token_swap.zkasm`)
Atomic token swap between two parties:
- Escrow pattern
- Atomic operations
- Timeouts

### 3. Voting Contract (`voting.zkasm`)
Simple voting system:
- Proposal creation
- Vote casting
- Result tallying
- Deadline enforcement

### 4. Multi-signature Wallet (`multisig.zkasm`)
Multi-party approval system:
- Multiple owners
- Threshold signatures
- Transaction proposals
- Approval tracking

## Running Examples

Each example can be deployed and tested:

```bash
# Compile
../tools/compile-contract.sh counter.zkasm

# Deploy to local testnet
../tools/deploy-contract.sh counter.zkasm local

# Test
../tools/test-contract.sh counter.zkasm
```

## Learning Path

1. **Start with Counter** - Basic state management
2. **Study Token Swap** - Escrow pattern
3. **Review Voting** - Complex state and logic
4. **Advanced: Multisig** - Multiple parties and thresholds

## Integration Patterns

Examples demonstrate:
- Contract-to-contract calls
- State management strategies
- Event emission (via state changes)
- Access control patterns
- Gas optimization techniques

## Contributing

Have an interesting contract pattern? Submit a PR with:
- Contract source (.zkasm)
- Documentation
- Test cases
- Gas analysis
