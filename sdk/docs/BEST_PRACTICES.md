# Smart Contract Best Practices

Security and optimization guidelines for BitCell smart contract development.

## Table of Contents

1. [Security Best Practices](#security-best-practices)
2. [Gas Optimization](#gas-optimization)
3. [Code Quality](#code-quality)
4. [Testing Strategies](#testing-strategies)
5. [Common Pitfalls](#common-pitfalls)
6. [Privacy Considerations](#privacy-considerations)

---

## Security Best Practices

### Input Validation

**Always validate all inputs before processing:**

```zkasm
# Bad - No validation
transfer:
  LOAD r2, r0, 0x40          # amount
  # Process directly - UNSAFE!

# Good - Validate first
transfer:
  LOAD r2, r0, 0x40          # amount
  GT r3, r2, 0               # Check amount > 0
  JZ r3, 0, valid_amount
  HALT                        # Revert if invalid
  
valid_amount:
  # Safe to process
```

### Access Control

**Implement robust authorization checks:**

```zkasm
# Owner-only function pattern
admin_function:
  LOAD r5, r0, 0x100         # Load owner address
  LOAD r6, r0, 0x20          # Load caller address
  EQ r7, r5, r6              # Verify caller == owner
  JZ r7, 0, authorized
  HALT                        # Unauthorized - revert
  
authorized:
  # Execute privileged operation
```

### Arithmetic Safety

**Check for overflow/underflow:**

```zkasm
# Safe addition with overflow check
safe_add:
  # Want: r5 = r3 + r4
  ADD r5, r3, r4
  LT r6, r5, r3              # Check if result < operand
  JZ r6, 0, no_overflow
  HALT                        # Overflow detected
  
no_overflow:
  # r5 contains safe result

# Safe subtraction with underflow check
safe_sub:
  # Want: r5 = r3 - r4
  LT r6, r3, r4              # Check if r3 < r4
  JZ r6, 0, no_underflow
  HALT                        # Would underflow
  
no_underflow:
  SUB r5, r3, r4             # Safe to subtract
```

### Reentrancy Protection

**Protect against reentrancy attacks:**

```zkasm
# Use state flags to prevent reentrancy
withdraw:
  # Check not already in withdrawal
  LOAD r10, r0, 0x150        # Lock flag
  EQ r11, r10, 0
  JZ r11, 0, not_locked
  HALT                        # Already locked - revert
  
not_locked:
  # Set lock
  STORE r0, 1, 0x150
  
  # Perform withdrawal
  # ... withdrawal logic ...
  
  # Release lock
  STORE r0, 0, 0x150
  HALT
```

### State Consistency

**Maintain state consistency:**

```zkasm
# Update all related state atomically
transfer:
  # Load sender balance
  LOAD r5, r0, sender_addr
  
  # Check sufficient balance
  LT r6, r5, amount
  JZ r6, 0, sufficient
  HALT
  
sufficient:
  # Update sender - DO THIS FIRST
  SUB r5, r5, amount
  STORE r0, r5, sender_addr
  
  # Then update recipient
  LOAD r7, r0, recipient_addr
  ADD r7, r7, amount
  STORE r0, r7, recipient_addr
  
  # Both updated or neither (if HALT occurs)
  HALT
```

---

## Gas Optimization

### Instruction Selection

**Choose cheaper instructions when possible:**

```zkasm
# Expensive: Use MUL
MUL r5, r3, 2              # Cost: 2 gas

# Cheaper: Use ADD for powers of 2
ADD r5, r3, r3             # Cost: 1 gas (same result for *2)

# Expensive: Multiple operations
MUL r5, r3, 8              # Cost: 2 gas
DIV r6, r5, 4              # Cost: 4 gas (total: 6)

# Cheaper: Combine when possible
MUL r6, r3, 2              # Cost: 2 gas (same result as *8/4)
```

### Memory Access Patterns

**Minimize memory operations:**

```zkasm
# Bad - Multiple loads of same value
LOAD r5, r0, 0x100
ADD r6, r5, 1
LOAD r5, r0, 0x100         # Redundant load
ADD r7, r5, 2

# Good - Load once, reuse
LOAD r5, r0, 0x100         # Load once
ADD r6, r5, 1              # Use r5
ADD r7, r5, 2              # Reuse r5
```

### Loop Optimization

**Optimize loop structures:**

```zkasm
# Bad - Recalculate in loop
loop_start:
  LOAD r5, r0, 0x100       # Constant - shouldn't be in loop
  MUL r6, r3, r5           # Could be outside
  # ... loop body ...
  ADD r10, r10, 1
  LT r11, r10, r12
  JZ r11, 0, loop_start

# Good - Hoist invariants
LOAD r5, r0, 0x100         # Load once before loop
loop_start:
  MUL r6, r3, r5           # Use preloaded value
  # ... loop body ...
  ADD r10, r10, 1
  LT r11, r10, r12
  JZ r11, 0, loop_start
```

### Batch Operations

**Combine related operations:**

```zkasm
# Bad - Multiple transactions
# Transaction 1: Update balance A
# Transaction 2: Update balance B
# Total cost: 2x transaction overhead

# Good - Single transaction with multiple updates
batch_update:
  # Update balance A
  LOAD r5, r0, addr_a
  ADD r5, r5, delta_a
  STORE r0, r5, addr_a
  
  # Update balance B
  LOAD r6, r0, addr_b
  ADD r6, r6, delta_b
  STORE r0, r6, addr_b
  
  # Single transaction overhead
  HALT
```

---

## Code Quality

### Documentation

**Document your code thoroughly:**

```zkasm
# === TRANSFER FUNCTION ===
# Transfers tokens from sender to recipient
#
# Parameters:
#   memory[0x30] = recipient address
#   memory[0x40] = transfer amount
#
# Memory Layout:
#   0x200 + (address * 8) = balance storage
#
# Returns:
#   Success: HALT with r0 = 1
#   Failure: HALT with r0 = 0
#
# Gas Cost: ~50-100 depending on path

transfer:
  # Load parameters
  LOAD r2, r0, 0x30          # recipient
  LOAD r3, r0, 0x40          # amount
  
  # ... implementation ...
```

### Naming Conventions

**Use clear, consistent naming:**

```zkasm
# Good - Clear purpose
LOAD r5, r0, sender_balance_addr
LOAD r6, r0, recipient_balance_addr
ADD r7, r5, r6             # total_balance

# Bad - Unclear
LOAD r5, r0, 0x200
LOAD r6, r0, 0x208
ADD r7, r5, r6
```

### Code Organization

**Structure your contract logically:**

```zkasm
# 1. Entry point and dispatch
# 2. Public functions
# 3. Internal helpers
# 4. State management utilities

# === ENTRY POINT ===
entry:
  LOAD r1, r0, 0x10
  # ... dispatch logic ...

# === PUBLIC FUNCTIONS ===
transfer:
  # ...

balance_of:
  # ...

# === INTERNAL HELPERS ===
_check_balance:
  # ...

_update_state:
  # ...
```

---

## Testing Strategies

### Unit Testing

**Test individual functions:**

```bash
# Test each function in isolation
./tools/test-contract.sh templates/token.zkasm

# Tests should cover:
# - Normal operation
# - Edge cases (zero amounts, max values)
# - Error conditions (insufficient balance)
# - Access control (unauthorized calls)
```

### Integration Testing

**Test contract interactions:**

```bash
# Test contract-to-contract calls
# Test with various account states
# Test with concurrent operations
```

### Fuzz Testing

**Use random inputs to find edge cases:**

```bash
# Generate random test inputs
# Run many iterations
# Check for unexpected failures
```

### Gas Profiling

**Measure and optimize gas usage:**

```bash
# Profile gas usage for each function
# Identify expensive operations
# Optimize hot paths
```

---

## Common Pitfalls

### 1. Uninitialized State

**Problem:** Reading uninitialized memory returns 0

```zkasm
# Bad - Assumes state exists
LOAD r5, r0, new_account_addr
# r5 = 0 even if account doesn't exist

# Good - Check initialization
LOAD r5, r0, new_account_addr
# Check if account was initialized
LOAD r6, r0, initialized_flag_addr
EQ r7, r6, 1
JZ r7, 0, initialized
HALT  # Account not initialized

initialized:
  # Safe to use r5
```

### 2. Integer Division Truncation

**Problem:** Division truncates, losing precision

```zkasm
# Problem: Want to divide by 3 then multiply by 10
DIV r5, r10, 3             # If r10 = 10, r5 = 3 (not 3.33...)
MUL r6, r5, 10             # r6 = 30 (lost precision)

# Better: Multiply first when possible
MUL r5, r10, 10            # r5 = 100
DIV r6, r5, 3              # r6 = 33 (better)
```

### 3. Division by Zero

**Problem:** DIV by zero halts execution

```zkasm
# Bad - No check
DIV r5, r10, r3            # HALTS if r3 = 0

# Good - Validate divisor
EQ r6, r3, 0
JZ r6, 0, divisor_ok
HALT  # Division by zero error

divisor_ok:
  DIV r5, r10, r3          # Safe
```

### 4. Unbounded Loops

**Problem:** Loops may exceed gas limit

```zkasm
# Bad - Unbounded loop
loop_start:
  # ... process item ...
  ADD r10, r10, 1
  LT r11, r10, array_size  # No gas limit check
  JZ r11, 0, loop_start

# Good - Limit iterations
loop_start:
  # Check iteration limit
  LT r12, r10, max_iterations
  JZ r12, 0, too_many_iterations
  
  # ... process item ...
  ADD r10, r10, 1
  LT r11, r10, array_size
  JZ r11, 0, loop_start

too_many_iterations:
  HALT  # Prevent gas exhaustion
```

### 5. State Race Conditions

**Problem:** Multiple transactions in same block

```zkasm
# Problem: Two transfers in same block
# Both read balance = 100
# Both subtract 60
# Balance should be 100-60-60 = -20 (invalid)
# But both might succeed if not careful

# Solution: Use nonces or locking
transfer:
  LOAD r10, r0, nonce_addr
  ADD r11, r10, 1
  STORE r0, r11, nonce_addr
  
  # Include nonce in state updates
  # Ensures sequential processing
```

---

## Privacy Considerations

### ZK-Friendly Operations

**Use field-friendly operations:**

```zkasm
# Prefer HASH over manual hashing
HASH r5, r3, r4            # Field-friendly Poseidon hash

# Avoid complex bit operations when possible
# They're more expensive in ZK circuits
```

### Private State

**Leverage ZK privacy features:**

```zkasm
# Contract state is private by default
# Only revealed through ZK proofs
# Design contracts to minimize public data

# Public: Function selector, result
# Private: Balance updates, intermediate values
```

### Gas Multipliers

**Benefit from privacy gas bonuses:**

```zkasm
# Private contracts get 2x gas multiplier
# Design for privacy to reduce costs
# User pays less for private operations
```

---

## Checklist

Before deploying:

- [ ] All inputs validated
- [ ] Access control implemented
- [ ] Arithmetic safety checks in place
- [ ] No reentrancy vulnerabilities
- [ ] State consistency maintained
- [ ] Gas optimized for common operations
- [ ] Code well-documented
- [ ] Comprehensive tests written
- [ ] Common pitfalls avoided
- [ ] Privacy features utilized
- [ ] Security audit considered

---

**Last Updated:** December 2025  
**Version:** 1.0
