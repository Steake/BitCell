# BitCell ZKVM API Reference

Complete reference for the BitCell Zero-Knowledge Virtual Machine instruction set and contract development API.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Instruction Set](#instruction-set)
3. [Registers](#registers)
4. [Memory Model](#memory-model)
5. [Gas Costs](#gas-costs)
6. [Execution Model](#execution-model)
7. [Contract Interface](#contract-interface)

---

## Architecture Overview

The BitCell ZKVM is a RISC-like virtual machine designed for zero-knowledge smart contract execution:

- **32 general-purpose registers** (r0-r31)
- **1MB sparse memory** address space
- **Field-friendly operations** for efficient ZK proofs
- **Gas metering** for resource control
- **Deterministic execution** for reproducibility

### ZK-SNARK Integration

Every contract execution generates a ZK-SNARK proof:
- **Public inputs:** Function selector, parameters
- **Private inputs:** Contract state, intermediate values
- **Proof:** Groth16 proof of correct execution

---

## Instruction Set

### Arithmetic Instructions

#### ADD - Addition
```zkasm
ADD rd, rs1, rs2
```
- **Operation:** `rd = rs1 + rs2`
- **Gas Cost:** 1
- **Description:** Adds two registers, stores result in destination

**Example:**
```zkasm
ADD r3, r1, r2    # r3 = r1 + r2
```

#### SUB - Subtraction
```zkasm
SUB rd, rs1, rs2
```
- **Operation:** `rd = rs1 - rs2`
- **Gas Cost:** 1
- **Description:** Subtracts rs2 from rs1

**Example:**
```zkasm
SUB r3, r5, r2    # r3 = r5 - r2
```

#### MUL - Multiplication
```zkasm
MUL rd, rs1, rs2
```
- **Operation:** `rd = rs1 * rs2`
- **Gas Cost:** 2
- **Description:** Multiplies two registers

**Example:**
```zkasm
MUL r4, r2, r3    # r4 = r2 * r3
```

#### DIV - Division
```zkasm
DIV rd, rs1, rs2
```
- **Operation:** `rd = rs1 / rs2`
- **Gas Cost:** 4
- **Description:** Integer division, truncates result
- **Error:** Halts if rs2 == 0

**Example:**
```zkasm
DIV r5, r10, r2   # r5 = r10 / r2
```

#### MOD - Modulo
```zkasm
MOD rd, rs1, rs2
```
- **Operation:** `rd = rs1 % rs2`
- **Gas Cost:** 4
- **Description:** Remainder of division

**Example:**
```zkasm
MOD r3, r7, r4    # r3 = r7 % r4
```

---

### Logic Instructions

#### AND - Bitwise AND
```zkasm
AND rd, rs1, rs2
```
- **Operation:** `rd = rs1 & rs2`
- **Gas Cost:** 1

#### OR - Bitwise OR
```zkasm
OR rd, rs1, rs2
```
- **Operation:** `rd = rs1 | rs2`
- **Gas Cost:** 1

#### XOR - Bitwise XOR
```zkasm
XOR rd, rs1, rs2
```
- **Operation:** `rd = rs1 ^ rs2`
- **Gas Cost:** 1

#### NOT - Bitwise NOT
```zkasm
NOT rd, rs1
```
- **Operation:** `rd = ~rs1`
- **Gas Cost:** 1

---

### Comparison Instructions

#### EQ - Equal
```zkasm
EQ rd, rs1, rs2
```
- **Operation:** `rd = (rs1 == rs2) ? 1 : 0`
- **Gas Cost:** 1
- **Returns:** 1 if equal, 0 otherwise

#### LT - Less Than
```zkasm
LT rd, rs1, rs2
```
- **Operation:** `rd = (rs1 < rs2) ? 1 : 0`
- **Gas Cost:** 1

#### GT - Greater Than
```zkasm
GT rd, rs1, rs2
```
- **Operation:** `rd = (rs1 > rs2) ? 1 : 0`
- **Gas Cost:** 1

#### LE - Less Than or Equal
```zkasm
LE rd, rs1, rs2
```
- **Operation:** `rd = (rs1 <= rs2) ? 1 : 0`
- **Gas Cost:** 1

#### GE - Greater Than or Equal
```zkasm
GE rd, rs1, rs2
```
- **Operation:** `rd = (rs1 >= rs2) ? 1 : 0`
- **Gas Cost:** 1

---

### Memory Instructions

#### LOAD - Load from Memory
```zkasm
LOAD rd, rs1, imm
```
- **Operation:** `rd = memory[rs1 + imm]`
- **Gas Cost:** 3
- **Description:** Loads 64-bit value from memory

**Example:**
```zkasm
LOAD r5, r0, 0x100    # Load from address 0x100
LOAD r6, r2, 8        # Load from address (r2 + 8)
```

#### STORE - Store to Memory
```zkasm
STORE rs1, rs2, imm
```
- **Operation:** `memory[rs2 + imm] = rs1`
- **Gas Cost:** 3
- **Description:** Stores 64-bit value to memory

**Example:**
```zkasm
STORE r5, r0, 0x100   # Store r5 to address 0x100
STORE r3, r4, 16      # Store r3 to address (r4 + 16)
```

---

### Control Flow Instructions

#### JMP - Unconditional Jump
```zkasm
JMP imm
```
- **Operation:** `pc = imm`
- **Gas Cost:** 2
- **Description:** Jumps to absolute address

**Example:**
```zkasm
JMP 100               # Jump to instruction 100
```

#### JZ - Jump if Zero
```zkasm
JZ rs1, rs2, imm
```
- **Operation:** `if (rs1 == 0) pc = imm`
- **Gas Cost:** 2
- **Description:** Conditional jump

**Example:**
```zkasm
JZ r5, r0, 50         # If r5 == 0, jump to instruction 50
```

#### CALL - Subroutine Call
```zkasm
CALL imm
```
- **Operation:** Push pc+1 to call stack, jump to imm
- **Gas Cost:** 5
- **Description:** Calls a subroutine

#### RET - Return from Subroutine
```zkasm
RET
```
- **Operation:** Pop return address from call stack, jump to it
- **Gas Cost:** 3
- **Description:** Returns from subroutine

---

### Cryptographic Instructions

#### HASH - Field-Friendly Hash
```zkasm
HASH rd, rs1, rs2
```
- **Operation:** `rd = hash(rs1, rs2)`
- **Gas Cost:** 20
- **Description:** Poseidon hash for ZK-friendly operations

**Example:**
```zkasm
HASH r10, r5, r6      # r10 = hash(r5, r6)
```

---

### System Instructions

#### HALT - Stop Execution
```zkasm
HALT
```
- **Operation:** Stops VM execution
- **Gas Cost:** 0
- **Description:** Normal termination or error

---

## Registers

The ZKVM has 32 general-purpose 64-bit registers:

| Register | Alias | Description |
|----------|-------|-------------|
| r0       | zero  | Always zero (reads as 0, writes ignored) |
| r1-r31   | -     | General purpose registers |

### Register Conventions

While not enforced, these conventions improve code clarity:

| Registers | Purpose |
|-----------|---------|
| r0        | Zero constant |
| r1-r4     | Function arguments |
| r5-r15    | Temporary values |
| r16-r25   | Saved registers (preserved across calls) |
| r26-r30   | Reserved for contract use |
| r31       | Stack pointer (if implementing stack) |

---

## Memory Model

### Address Space

- **Size:** 1MB (1,048,576 bytes)
- **Addressing:** Byte-addressable
- **Values:** 64-bit words (8 bytes)
- **Implementation:** Sparse memory (only used addresses consume space)

### Memory Regions (Recommended Layout)

| Region | Address Range | Purpose |
|--------|---------------|---------|
| Metadata | 0x0000-0x00FF | Contract metadata |
| Config | 0x0100-0x01FF | Owner, parameters |
| State | 0x0200-0x0FFF | Contract state |
| Dynamic | 0x1000+ | Mappings, arrays |

### Memory Operations

Memory is **byte-addressable** but operates on **8-byte words**:

```zkasm
# Store 64-bit value
STORE r5, r0, 0x100       # Store r5 at address 0x100

# Load 64-bit value
LOAD r6, r0, 0x100        # Load from address 0x100 to r6
```

---

## Gas Costs

Gas metering ensures resource constraints:

| Instruction Type | Gas Cost | Examples |
|------------------|----------|----------|
| Arithmetic (simple) | 1 | ADD, SUB, AND, OR, XOR, NOT |
| Arithmetic (complex) | 2 | MUL |
| Arithmetic (heavy) | 4 | DIV, MOD |
| Memory | 3 | LOAD, STORE |
| Control Flow | 2 | JMP, JZ |
| Subroutine | 3-5 | CALL (5), RET (3) |
| Cryptographic | 20 | HASH |
| System | 0 | HALT |

### Gas Limit

Each transaction specifies a gas limit. If exceeded, execution halts with `OutOfGas` error.

---

## Execution Model

### Execution Flow

1. **Initialize:** Load contract bytecode and set gas limit
2. **Execute:** Process instructions sequentially
3. **Terminate:** HALT or error stops execution
4. **Proof:** Generate ZK-SNARK proof of execution

### Execution Trace

The VM generates an execution trace for ZK proof:
- Program counter at each step
- Register states before/after
- Memory reads/writes
- Gas consumed

---

## Contract Interface

### Function Dispatch Pattern

Contracts use a function selector for dispatch:

```zkasm
# Load function selector from memory[0x10]
LOAD r1, r0, 0x10

# Dispatch to function 0
EQ r4, r1, 0
JZ r4, 0, check_function_1
JMP function_0

check_function_1:
# Dispatch to function 1
EQ r4, r1, 1
JZ r4, 0, unknown_function
JMP function_1

function_0:
  # Function logic here
  HALT

function_1:
  # Function logic here
  HALT

unknown_function:
  HALT  # Error: unknown function
```

### Input/Output Convention

| Location | Purpose |
|----------|---------|
| 0x10 | Function selector |
| 0x20 | Caller address |
| 0x30-0x90 | Function parameters |
| r0 | Return value |

### Error Handling

Errors are signaled by HALT:
- **Success:** HALT with r0 = return value
- **Error:** HALT (contract reverts)

---

## Example Contracts

See the `templates/` directory for complete examples:
- **token.zkasm** - Fungible token
- **nft.zkasm** - Non-fungible token
- **escrow.zkasm** - Trustless escrow

---

## Best Practices

1. **Validate all inputs** before processing
2. **Check arithmetic overflow** for critical operations
3. **Use memory regions consistently** across your contract
4. **Minimize gas costs** by optimizing instruction sequences
5. **Document your code** with comments
6. **Test thoroughly** with edge cases

---

**Last Updated:** December 2025  
**Version:** 1.0
