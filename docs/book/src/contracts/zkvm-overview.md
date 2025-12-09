# ZKVM Overview

BitCell's Zero-Knowledge Virtual Machine (ZKVM) enables private smart contract execution with zero-knowledge proofs.

## What is the ZKVM?

The ZKVM is BitCell's execution environment for smart contracts. Unlike traditional blockchain VMs (like EVM), the ZKVM:

- **Maintains Privacy**: Contract state is encrypted with Pedersen commitments
- **Proves Execution**: All state transitions verified by ZK-SNARKs
- **Supports Multiple Languages**: BCL (high-level) and ZKASM (assembly)
- **Guarantees Determinism**: Same inputs always produce same outputs

## Architecture

```
┌─────────────────────────────────────────────┐
│          Smart Contract Layer               │
│  (BCL Language / ZKASM Assembly)            │
└─────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│          ZKVM Interpreter                   │
│  • 32 registers (r0-r31)                    │
│  • Memory-mapped I/O                        │
│  • Function dispatch                        │
└─────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│       Zero-Knowledge Proof Layer            │
│  • Execution proof (Groth16)                │
│  • State transition proof                   │
│  • Commitment updates                       │
└─────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│          BitCell Blockchain                 │
│  • Stores commitments & proofs only         │
│  • Validators verify without plaintext      │
└─────────────────────────────────────────────┘
```

## Key Features

### 1. Private State

Contract state is never revealed on-chain:

```rust
// On-chain: Only commitment visible
let commitment = Pedersen::commit(state, nonce);
// commitment: 0x1234...abcd

// Off-chain: Prover sees plaintext
let state = {
    "balance": 1000,
    "owner": "0x742d..."
};
```

### 2. Zero-Knowledge Execution

Execution proofs verify correctness without revealing logic:

```rust
// Public inputs (on-chain)
old_state_commitment: 0x1234...
new_state_commitment: 0x5678...
function_selector: 0xabcd

// Private inputs (off-chain)
old_state: { balance: 1000 }
new_state: { balance: 900 }
function_code: [LOAD, SUB, STORE]

// Proof verifies:
// 1. new_state correctly computed from old_state
// 2. function_code executed properly
// 3. commitments match
```

### 3. Composability

Contracts can call other contracts while maintaining privacy:

```zkasm
; Call another contract
LOAD r1, caller_address
LOAD r2, function_selector
LOAD r3, arg1
CALL r1, r2, r3

; Result in r0
STORE result, r0
```

## Instruction Set

The ZKVM implements a RISC-style instruction set:

### Arithmetic

- `ADD rd, rs1, rs2` - Addition
- `SUB rd, rs1, rs2` - Subtraction
- `MUL rd, rs1, rs2` - Multiplication
- `DIV rd, rs1, rs2` - Division (with ZK division circuit)

### Logic

- `AND rd, rs1, rs2` - Bitwise AND
- `OR rd, rs1, rs2` - Bitwise OR
- `XOR rd, rs1, rs2` - Bitwise XOR
- `NOT rd, rs` - Bitwise NOT

### Memory

- `LOAD rd, addr` - Load from memory
- `STORE addr, rs` - Store to memory
- `LOADI rd, imm` - Load immediate value

### Control Flow

- `JZ rs, offset` - Jump if zero
- `JMP offset` - Unconditional jump
- `CALL addr` - Function call
- `RET` - Return from function

### Comparison

- `EQ rd, rs1, rs2` - Set rd = 1 if rs1 == rs2, else 0
- `LT rd, rs1, rs2` - Set rd = 1 if rs1 < rs2, else 0
- `GT rd, rs1, rs2` - Set rd = 1 if rs1 > rs2, else 0

### Special

- `HALT` - Stop execution
- `ASSERT rs` - Assert rs != 0 (aborts if false)
- `EMIT event_id, data` - Emit event log

See [ZKASM Reference](./zkasm-reference.md) for complete instruction details.

## Memory Layout

Standard memory layout for contracts:

```
0x0000 - 0x00FF:  Metadata
  0x0010: Function selector
  0x0014: msg.sender
  0x0018: msg.value
  0x0020: block.number
  0x0028: block.timestamp

0x0100 - 0x01FF:  Configuration
  0x0100: Contract owner
  0x0108: Admin address

0x0200 - 0x0FFF:  Persistent state
  0x0200+: Storage slots

0x1000 - 0xFFFF:  Temporary/stack
  0x1000+: Local variables
  0x2000+: Call stack
```

## Gas Metering

Each instruction consumes gas:

| Instruction | Gas Cost |
|-------------|----------|
| Arithmetic (ADD, SUB, MUL) | 3 |
| Division (DIV) | 10 |
| Memory (LOAD, STORE) | 5 |
| Storage (persistent) | 20 |
| Control flow (JZ, JMP) | 2 |
| Function call (CALL) | 50 |
| ZK proof (verify) | 1000 |

See [Gas Optimization](./gas-optimization.md) for efficiency tips.

## Contract Lifecycle

### 1. Development

Write contract in BCL or ZKASM:

```bcl
contract Token {
    mapping(address => uint) balances;
    
    function transfer(address to, uint amount) {
        require(balances[msg.sender] >= amount);
        balances[msg.sender] -= amount;
        balances[to] += amount;
    }
}
```

### 2. Compilation

Compile to ZKVM bytecode:

```bash
bitcell-compiler compile token.bcl -o token.zkvm
```

### 3. Deployment

Deploy to blockchain:

```bash
bitcell-wallet deploy token.zkvm --gas-limit 1000000
```

Returns contract address: `0x1234...`

### 4. Interaction

Call contract functions:

```bash
bitcell-wallet call 0x1234... \
  --function transfer \
  --args '["0x5678...", 100]' \
  --gas-limit 100000
```

### 5. Verification

All executions produce ZK proofs:

- **Execution proof**: Function executed correctly
- **State proof**: State updated properly
- **Commitment proof**: New commitment matches new state

Validators verify proofs without seeing plaintext.

## Security Model

### Guarantees

✓ **Confidentiality**: State never revealed on-chain  
✓ **Integrity**: Invalid executions rejected (proof fails)  
✓ **Authenticity**: Only rightful owner can update state  
✓ **Availability**: State always accessible to owner

### Limitations

⚠ **Computation Cost**: ZK proofs are expensive (seconds per execution)  
⚠ **State Size**: Large states increase proof generation time  
⚠ **No Global State**: Contracts can't easily iterate over all accounts

## Example Contracts

### Simple Storage

```zkasm
; Store a value
FUNCTION store:
    LOAD r1, 0x30        ; Load input value
    STORE 0x200, r1      ; Store to slot 0
    RET

; Retrieve a value
FUNCTION retrieve:
    LOAD r0, 0x200       ; Load from slot 0
    RET
```

### Token Transfer

```zkasm
FUNCTION transfer:
    ; Check sender balance
    LOAD r1, msg.sender
    LOAD r2, [0x200 + r1]     ; sender_balance
    LOAD r3, 0x30              ; amount
    LT r4, r2, r3              ; balance < amount?
    ASSERT r4 == 0             ; Revert if insufficient
    
    ; Deduct from sender
    SUB r2, r2, r3
    STORE [0x200 + r1], r2
    
    ; Add to recipient
    LOAD r5, 0x38              ; recipient address
    LOAD r6, [0x200 + r5]      ; recipient_balance
    ADD r6, r6, r3
    STORE [0x200 + r5], r6
    
    RET
```

See [Contract Examples](./examples.md) for more complete implementations.

## Development Tools

### Compiler

```bash
# Compile BCL to ZKVM
bitcell-compiler compile contract.bcl

# Compile with optimization
bitcell-compiler compile contract.bcl -O2

# Generate assembly listing
bitcell-compiler compile contract.bcl --emit-asm
```

### Simulator

```bash
# Test execution locally
bitcell-simulator run contract.zkvm \
  --function transfer \
  --args '[...]'

# Debug with trace
bitcell-simulator run contract.zkvm \
  --function transfer \
  --args '[...]' \
  --trace
```

### Prover

```bash
# Generate proof locally
bitcell-prover generate contract.zkvm \
  --old-state state1.json \
  --new-state state2.json \
  --witness witness.json

# Verify proof
bitcell-prover verify proof.json
```

## Best Practices

### 1. Minimize State

Keep state small for faster proofs:

```bcl
// Bad: Large state
contract Token {
    mapping(address => uint) balances;
    mapping(address => string) names;  // Unnecessary
    mapping(address => uint[]) history; // Expensive
}

// Good: Minimal state
contract Token {
    mapping(address => uint) balances;
}
```

### 2. Batch Operations

Group multiple operations:

```bcl
function batchTransfer(address[] to, uint[] amounts) {
    for (uint i = 0; i < to.length; i++) {
        transfer(to[i], amounts[i]);
    }
}
```

### 3. Use Events

Emit events for off-chain indexing:

```bcl
event Transfer(address from, address to, uint amount);

function transfer(address to, uint amount) {
    // ... transfer logic ...
    emit Transfer(msg.sender, to, amount);
}
```

### 4. Input Validation

Always validate inputs:

```bcl
function transfer(address to, uint amount) {
    require(to != address(0), "Invalid recipient");
    require(amount > 0, "Invalid amount");
    require(balances[msg.sender] >= amount, "Insufficient balance");
    // ... transfer logic ...
}
```

## Next Steps

- **[BCL Tutorial](./bcl-tutorial.md)** - Learn the BCL language
- **[ZKASM Reference](./zkasm-reference.md)** - Assembly programming
- **[Contract Examples](./examples.md)** - Complete contract templates
- **[Deployment Guide](./deployment.md)** - Deploy your first contract

## Further Reading

- [ZK Proof System](../concepts/zk-proofs.md)
- [Gas Optimization](./gas-optimization.md)
- [Testing Contracts](./testing.md)
