# BitCell Smart Contract Development Guide

## Overview

BitCell provides a high-level language for writing smart contracts called **BitCell Contract Language (BCL)**. BCL is a Solidity-like language that compiles to ZKVM (Zero-Knowledge Virtual Machine) bytecode, making it easy to write privacy-preserving smart contracts without manually coding assembly instructions.

## Why BCL?

Before BCL, writing smart contracts for BitCell required:
- Manual ZKVM assembly programming
- Deep understanding of the VM instruction set
- Error-prone low-level code management
- Difficult debugging and maintenance

With BCL, you can:
- Write contracts in a familiar, high-level syntax
- Get automatic type checking and validation
- Compile efficiently to optimized ZKVM bytecode
- Focus on business logic instead of VM details

## Quick Start

### 1. Install the Compiler

The BCL compiler (`bclc`) is included in the BitCell repository:

```bash
# Build from source
cargo build --release --package bitcell-compiler --bin bclc

# The binary will be at target/release/bclc
```

### 2. Write Your First Contract

Create a file called `counter.bcl`:

```bcl
contract Counter {
    storage {
        count: uint;
    }
    
    function increment() -> uint {
        count = count + 1;
        return count;
    }
    
    function get() -> uint {
        return count;
    }
}
```

### 3. Compile It

```bash
bclc counter.bcl
# Output: counter.bin (ZKVM bytecode)
```

### 4. Deploy and Execute

```rust
use bitcell_zkvm::Interpreter;
use std::fs;

fn main() {
    // Load compiled bytecode
    let bytecode = fs::read("counter.bin").expect("Failed to read bytecode");
    
    // Create ZKVM interpreter
    let mut vm = Interpreter::new(10000); // 10000 gas limit
    
    // Execute (implementation depends on your deployment setup)
    // ...
}
```

## Language Reference

### Contract Structure

Every BCL contract has two main sections:

```bcl
contract ContractName {
    storage {
        // Persistent storage variables
    }
    
    function functionName(params) -> returnType {
        // Function implementation
    }
}
```

### Data Types

BCL supports the following types:

| Type | Description | Example |
|------|-------------|---------|
| `uint` | 64-bit unsigned integer | `let x: uint = 42;` |
| `bool` | Boolean value | `let active: bool = true;` |
| `address` | Account address | `let owner: address;` |
| `mapping(K => V)` | Key-value storage | `balances: mapping(address => uint);` |

### Storage Variables

Storage variables persist between function calls:

```bcl
storage {
    owner: address;
    balance: uint;
    is_active: bool;
    balances: mapping(address => uint);
}
```

### Functions

Functions define the contract's behavior:

```bcl
function transfer(to: address, amount: uint) -> bool {
    let sender = msg.sender;
    require(balances[sender] >= amount, "Insufficient balance");
    
    balances[sender] = balances[sender] - amount;
    balances[to] = balances[to] + amount;
    
    return true;
}
```

### Variables

Declare local variables with `let`:

```bcl
let x = 42;
let sender = msg.sender;
let has_permission = owner == sender;
```

### Control Flow

**If-Else Statements:**
```bcl
if (balance >= amount) {
    balance = balance - amount;
} else {
    return false;
}
```

**Require (Assertions):**
```bcl
require(msg.sender == owner, "Not authorized");
require(amount > 0, "Amount must be positive");
```

**Return:**
```bcl
return true;
return balance;
```

### Operators

**Arithmetic:**
- `+` Addition
- `-` Subtraction
- `*` Multiplication
- `/` Division
- `%` Modulo

**Comparison:**
- `==` Equal
- `!=` Not equal
- `<` Less than
- `<=` Less than or equal
- `>` Greater than
- `>=` Greater than or equal

**Logical:**
- `&&` AND
- `||` OR
- `!` NOT

### Built-in Variables

BCL provides several built-in variables for contract context:

```bcl
msg.sender     // Address of the caller
msg.value      // Value sent with transaction
block.number   // Current block number
block.timestamp // Current block timestamp
```

## Example Contracts

### ERC20-Like Token

```bcl
contract Token {
    storage {
        balances: mapping(address => uint);
        total_supply: uint;
        owner: address;
    }
    
    function transfer(to: address, amount: uint) -> bool {
        let sender = msg.sender;
        require(balances[sender] >= amount, "Insufficient balance");
        
        balances[sender] = balances[sender] - amount;
        balances[to] = balances[to] + amount;
        
        return true;
    }
    
    function balance_of(account: address) -> uint {
        return balances[account];
    }
    
    function total() -> uint {
        return total_supply;
    }
}
```

### Simple Voting

```bcl
contract Voting {
    storage {
        votes: mapping(address => uint);
        total_votes: uint;
        proposal_id: uint;
    }
    
    function vote(choice: uint) -> bool {
        let voter = msg.sender;
        require(votes[voter] == 0, "Already voted");
        require(choice > 0, "Invalid choice");
        
        votes[voter] = choice;
        total_votes = total_votes + 1;
        
        return true;
    }
    
    function get_vote(voter: address) -> uint {
        return votes[voter];
    }
}
```

### Escrow Service

```bcl
contract Escrow {
    storage {
        depositor: address;
        beneficiary: address;
        amount: uint;
        released: bool;
    }
    
    function deposit(to: address, value: uint) -> bool {
        require(amount == 0, "Already deposited");
        
        depositor = msg.sender;
        beneficiary = to;
        amount = value;
        released = false;
        
        return true;
    }
    
    function release() -> bool {
        require(msg.sender == depositor, "Only depositor can release");
        require(!released, "Already released");
        
        released = true;
        // Transfer amount to beneficiary...
        
        return true;
    }
    
    function refund() -> bool {
        require(msg.sender == depositor, "Only depositor can refund");
        require(!released, "Already released");
        
        released = true;
        // Return amount to depositor...
        
        return true;
    }
}
```

## Compiler CLI Reference

### View Example Contracts

```bash
# Show built-in token example
bclc --example token

# Show counter example
bclc --example counter

# Show escrow example
bclc --example escrow
```

### Compile Contracts

```bash
# Compile to default output (contract.bin)
bclc contract.bcl

# Specify custom output file
bclc contract.bcl my_contract.bin

# View compilation results
bclc contract.bcl
# ✓ Compilation successful!
# Generated 45 instructions
# Output written to contract.bin
```

## Integration with ZKVM

Compiled BCL contracts produce ZKVM bytecode that can be executed by the BitCell interpreter:

```rust
use bitcell_compiler::compile;
use bitcell_zkvm::Interpreter;

// Compile contract from source
let source = std::fs::read_to_string("contract.bcl")?;
let instructions = compile(&source)?;

// Execute with ZKVM
let mut vm = Interpreter::new(100000); // gas limit

// Set up function selector and parameters in memory
vm.memory_mut().store(0x10, function_selector)?;
vm.memory_mut().store(0x20, param1)?;

// Execute
vm.execute(&instructions)?;

// Read return value
let result = vm.get_register(0);
```

## Memory Layout

BCL contracts use a standardized memory layout:

| Address Range | Purpose |
|---------------|---------|
| `0x10` | Function selector (which function to call) |
| `0x20-0x1FF` | Function parameters |
| `0x200+` | Persistent storage variables |
| `0x1000+` | Temporary computation space |

## Best Practices

### 1. Check Preconditions Early

```bcl
function transfer(to: address, amount: uint) -> bool {
    require(amount > 0, "Amount must be positive");
    require(balances[msg.sender] >= amount, "Insufficient balance");
    // ... rest of function
}
```

### 2. Use Descriptive Variable Names

```bcl
// Good
let sender_balance = balances[msg.sender];
let recipient_balance = balances[to];

// Avoid
let x = balances[msg.sender];
let y = balances[to];
```

### 3. Document Complex Logic

```bcl
// Calculate new balance after fee deduction
let fee = amount / 100;  // 1% fee
let net_amount = amount - fee;
```

### 4. Initialize Storage Variables

```bcl
storage {
    owner: address;      // Will be set during deployment
    is_active: bool;     // Defaults to false
    counter: uint;       // Defaults to 0
}
```

## Limitations

Current BCL implementation has some limitations:

- **No loops** - `for` and `while` loops not yet supported
- **No structs** - Only primitive types and mappings
- **No events** - Event emission not implemented
- **Single return value** - Functions return one value
- **No modifiers** - Function modifiers not supported
- **Limited standard library** - Expanding over time

These limitations will be addressed in future releases.

## Troubleshooting

### Common Errors

**"Type mismatch in assignment"**
```
Solution: Ensure variable types match. Use type conversion if needed.
```

**"Undefined variable"**
```
Solution: Check variable is declared in storage or as local variable.
```

**"Expected token"**
```
Solution: Check syntax - missing semicolon, brace, or parenthesis.
```

### Getting Help

- Check the [compiler README](../crates/bitcell-compiler/README.md)
- Review example contracts with `bclc --example <name>`
- Run tests: `cargo test --package bitcell-compiler`

## Contributing

BCL is evolving! We welcome contributions:

- **Language features** - Loops, structs, events
- **Optimizations** - Better code generation
- **Standard library** - More contract patterns
- **Tooling** - IDE support, debugger, profiler
- **Documentation** - Tutorials, guides, examples

See the [main README](../README.md) for contribution guidelines.

## Next Steps

1. ✅ Write your first contract
2. ✅ Compile with `bclc`
3. ✅ Test with ZKVM interpreter
4. 📖 Read about [ZK-SNARK integration](./ARCHITECTURE.md)
5. 🚀 Deploy to BitCell testnet

Happy coding! 🦀⚡
