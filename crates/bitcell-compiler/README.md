# BitCell Contract Language (BCL) Compiler

A Solidity-like compiler for BitCell smart contracts that compiles high-level contract code to ZKVM bytecode.

## Overview

The BitCell Contract Language (BCL) provides a familiar, Solidity-inspired syntax for writing smart contracts that execute on BitCell's Zero-Knowledge Virtual Machine (ZKVM). Instead of writing raw ZKVM assembly instructions, developers can write contracts in BCL and compile them to efficient bytecode.

## Features

- **Solidity-like syntax** - Familiar contract structure and syntax
- **Type safety** - Static type checking with semantic analysis
- **ZKVM integration** - Direct compilation to ZKVM bytecode
- **Standard library** - Common contract patterns included
- **CLI compiler** - Easy-to-use command-line tool

## Language Syntax

### Contract Structure

```bcl
contract MyContract {
    storage {
        // Storage variable declarations
        balance: uint;
        owner: address;
        is_active: bool;
    }
    
    function my_function(param: uint) -> bool {
        // Function body
        return true;
    }
}
```

### Types

- `uint` - Unsigned integer (64-bit)
- `bool` - Boolean value (true/false)
- `address` - Account address
- `mapping(KeyType => ValueType)` - Hash map storage

### Variables

```bcl
// Local variable declaration
let x = 42;
let sender = msg.sender;

// Assignment
value = value + 1;
```

### Control Flow

```bcl
// If-else
if (condition) {
    // then block
} else {
    // else block
}

// Require (assertion)
require(balance >= amount, "Insufficient balance");

// Return
return result;
```

### Operators

- **Arithmetic**: `+`, `-`, `*`, `/`, `%`
- **Comparison**: `==`, `!=`, `<`, `<=`, `>`, `>=`
- **Logical**: `&&`, `||`, `!`

### Built-in Variables

- `msg.sender` - Address of the transaction sender
- `msg.value` - Value sent with transaction
- `block.number` - Current block number
- `block.timestamp` - Current block timestamp

## Example Contracts

### Simple Counter

```bcl
contract Counter {
    storage {
        count: uint;
    }
    
    function increment() -> uint {
        count = count + 1;
        return count;
    }
    
    function decrement() -> uint {
        require(count > 0, "Counter underflow");
        count = count - 1;
        return count;
    }
    
    function get() -> uint {
        return count;
    }
}
```

### Token Contract

```bcl
contract Token {
    storage {
        balances: mapping(address => uint);
        total_supply: uint;
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
}
```

### Escrow Contract

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
        return true;
    }
}
```

## Using the Compiler

### Installation

Build the compiler from source:

```bash
cd crates/bitcell-compiler
cargo build --release --bin bclc
```

The binary will be at `target/release/bclc`.

### CLI Usage

```bash
# Compile a contract
bclc my_contract.bcl

# Specify output file
bclc my_contract.bcl output.bin

# View example contracts
bclc --example token
bclc --example counter
bclc --example escrow
```

### Programmatic Usage

```rust
use bitcell_compiler::compile;

fn main() {
    let source = r#"
        contract Test {
            storage {
                value: uint;
            }
            
            function set(x: uint) -> bool {
                value = x;
                return true;
            }
        }
    "#;
    
    match compile(source) {
        Ok(instructions) => {
            println!("Compiled {} instructions", instructions.len());
            // Use instructions with ZKVM interpreter
        }
        Err(e) => {
            eprintln!("Compilation error: {}", e);
        }
    }
}
```

## Compiler Architecture

The compiler consists of several phases:

1. **Lexical Analysis** (`lexer`) - Tokenizes source code
2. **Parsing** (`parser`) - Builds Abstract Syntax Tree (AST)
3. **Semantic Analysis** (`semantic`) - Type checking and validation
4. **Code Generation** (`codegen`) - Emits ZKVM bytecode

### Pipeline

```
Source Code → Lexer → Tokens → Parser → AST → Semantic Analyzer → 
  Valid AST → Code Generator → ZKVM Instructions
```

## Memory Layout

BCL contracts use a standardized memory layout:

- `0x10` - Function selector
- `0x14` - `msg.sender` (reserved)
- `0x18` - `msg.value` (reserved)
- `0x20` - `block.number` (reserved)
- `0x28` - `block.timestamp` (reserved)
- `0x30-0x1FF` - Function parameters
- `0x200+` - Storage variables
- `0x1000+` - Temporary/stack memory

## Limitations and Future Work

Current implementation is a proof-of-concept with some limitations:

- **No function overloading** - Each function must have a unique name
- **Limited type system** - No user-defined types or structs
- **Simplified storage** - Mapping storage uses simple addressing (not cryptographically secure hashing)
- **No events** - Event emission not yet supported
- **No modifiers** - Function modifiers not implemented
- **Single-pass code generation** - Jump addresses need refinement

Future enhancements:

- [ ] Structs and user-defined types
- [ ] Events and logging
- [ ] Function modifiers (public, private, view, pure)
- [ ] Cryptographic hash-based storage for mappings
- [ ] Optimized bytecode generation
- [ ] Formal verification support
- [ ] Debugging information in bytecode

## Testing

Run the compiler test suite:

```bash
cargo test --package bitcell-compiler
```

## Contributing

Contributions are welcome! Areas needing improvement:

- Additional operators and expressions
- More sophisticated type system
- Optimization passes
- Better error messages
- Standard library expansion
- Documentation and examples

## License

Dual-licensed under MIT / Apache 2.0.
