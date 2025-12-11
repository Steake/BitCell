# Getting Started with BitCell Smart Contracts

This guide will walk you through developing your first smart contract on BitCell.

## Prerequisites

Before you begin, ensure you have:

- **Rust 1.82+** installed (`rustup` recommended)
- **BitCell repository** cloned
- **4GB+ RAM** for local development
- **Basic familiarity** with assembly-like languages

## Step 1: Set Up Your Environment

Clone the BitCell repository and navigate to the SDK:

```bash
git clone https://github.com/Steake/BitCell
cd BitCell/sdk
```

Build the BitCell toolchain:

```bash
cd ..
cargo build --release
```

## Step 2: Explore the Templates

The SDK includes three ready-to-use templates:

### Token Contract (`templates/token.zkasm`)

A fungible token implementation with:
- Transfer tokens between addresses
- Query balances
- Mint/burn capabilities
- Total supply tracking

### NFT Contract (`templates/nft.zkasm`)

A non-fungible token implementation with:
- Mint unique tokens
- Transfer ownership
- Approve transfers
- Query token ownership

### Escrow Contract (`templates/escrow.zkasm`)

A trustless escrow with:
- Create escrow with funds
- Release to beneficiary
- Refund after timeout
- Optional arbiter for disputes

## Step 3: Start a Local Testnet

Launch a single-node testnet for development:

```bash
./tools/start-testnet.sh
```

This starts a local BitCell node at:
- **RPC:** `http://127.0.0.1:8545`
- **P2P:** `http://127.0.0.1:9944`

Leave this running in a separate terminal.

## Step 4: Deploy Your First Contract

Deploy the token template:

```bash
./tools/deploy-contract.sh templates/token.zkasm
```

Output:
```
📦 BitCell Contract Deployment
==============================
Contract: templates/token.zkasm
Network:  local

...

✨ Deployment Complete!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Contract Address: 0xabcd1234...
```

Save the contract address for later use.

## Step 5: Test Your Contract

Run the test suite:

```bash
./tools/test-contract.sh templates/token.zkasm
```

Output:
```
🧪 BitCell Contract Testing
===========================
...
✅ All tests passed!
```

## Step 6: Interact with Your Contract

Use the BitCell RPC to interact with deployed contracts:

```bash
# Query balance
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_call",
    "params": [{
      "to": "0xYOUR_CONTRACT_ADDRESS",
      "data": "0x..."
    }],
    "id": 1
  }'
```

## Step 7: Write Your Own Contract

Create a new ZKASM file:

```bash
nano my_contract.zkasm
```

Basic template structure:

```zkasm
# Load function selector
LOAD r1, r0, 0x10

# Function dispatch
EQ r4, r1, 0               # Check function ID
JZ r4, 0, next_function

my_function:
  # Your logic here
  HALT

next_function:
  # More functions...
  HALT
```

## Step 8: Compile and Deploy

Compile your contract:

```bash
./tools/compile-contract.sh my_contract.zkasm
```

Deploy it:

```bash
./tools/deploy-contract.sh my_contract.zkasm
```

## Understanding ZKVM Instructions

BitCell smart contracts use a RISC-like instruction set. Key instructions:

### Arithmetic
- `ADD rd, rs1, rs2` - Add: rd = rs1 + rs2
- `SUB rd, rs1, rs2` - Subtract: rd = rs1 - rs2
- `MUL rd, rs1, rs2` - Multiply: rd = rs1 * rs2
- `DIV rd, rs1, rs2` - Divide: rd = rs1 / rs2

### Memory
- `LOAD rd, rs1, imm` - Load: rd = mem[rs1 + imm]
- `STORE rs1, rs2, imm` - Store: mem[rs2 + imm] = rs1

### Control Flow
- `JMP imm` - Jump to address
- `JZ rs1, rs2, imm` - Jump if rs1 == 0
- `HALT` - Stop execution

### Logic
- `AND rd, rs1, rs2` - Bitwise AND
- `OR rd, rs1, rs2` - Bitwise OR
- `XOR rd, rs1, rs2` - Bitwise XOR
- `EQ rd, rs1, rs2` - Equals: rd = (rs1 == rs2) ? 1 : 0
- `LT rd, rs1, rs2` - Less than: rd = (rs1 < rs2) ? 1 : 0
- `GT rd, rs1, rs2` - Greater than: rd = (rs1 > rs2) ? 1 : 0

## Memory Layout Best Practices

Organize your contract memory:

```
0x0000-0x00FF: Contract metadata
0x0100-0x01FF: Configuration and owner data
0x0200-0x0FFF: Contract-specific state
0x1000+:       Dynamic data (mappings, arrays)
```

## Gas Costs

Each instruction consumes gas:

| Instruction | Gas Cost |
|-------------|----------|
| ADD, SUB    | 1        |
| MUL         | 2        |
| DIV, MOD    | 4        |
| LOAD, STORE | 3        |
| HASH        | 20       |
| CALL        | 5        |

Optimize your contracts to minimize gas usage!

## Next Steps

- Read the **API Reference** for complete instruction documentation
- Study **Best Practices** for security tips
- Review **Deployment Guide** for production deployment
- Join the BitCell community for support

## Common Patterns

### Input Validation

Always validate inputs:

```zkasm
# Check amount > 0
GT r5, r3, 0
JZ r5, 0, valid_amount
HALT  # Invalid amount

valid_amount:
  # Continue...
```

### Access Control

Implement owner-only functions:

```zkasm
LOAD r5, r0, 0x100     # Load owner
LOAD r6, r0, 0x20      # Load caller
EQ r7, r5, r6          # Check equality
JZ r7, 0, authorized
HALT  # Unauthorized

authorized:
  # Execute privileged operation
```

### State Updates

Use consistent state patterns:

```zkasm
# Load state
LOAD r5, r0, state_addr

# Modify
ADD r5, r5, delta

# Store back
STORE r0, r5, state_addr
```

## Debugging Tips

1. **Use comments** - Document your assembly code
2. **Test incrementally** - Test each function separately
3. **Check gas** - Monitor gas usage during development
4. **Validate state** - Verify state changes after each operation

## Getting Help

- **Documentation:** `docs/API_REFERENCE.md`
- **Examples:** `examples/` directory
- **GitHub Issues:** Report bugs and ask questions
- **Community:** Join the BitCell Discord

Happy coding! 🚀
