//! # BitCell ZKVM
//!
//! A RISC-like virtual machine for private smart contract execution.
//! Designed to be field-friendly for ZK-SNARK constraint generation.

mod instruction;
mod interpreter;
mod memory;

pub use instruction::{Instruction, OpCode};
pub use interpreter::{Interpreter, ExecutionTrace, InterpreterError};
pub use memory::Memory;

/// Gas costs for each instruction type
pub mod gas {
    pub const ADD: u64 = 1;
    pub const SUB: u64 = 1;
    pub const MUL: u64 = 2;
    pub const DIV: u64 = 4;
    pub const MOD: u64 = 4;
    pub const AND: u64 = 1;
    pub const OR: u64 = 1;
    pub const XOR: u64 = 1;
    pub const NOT: u64 = 1;
    pub const EQ: u64 = 1;
    pub const LT: u64 = 1;
    pub const GT: u64 = 1;
    pub const LOAD: u64 = 3;
    pub const STORE: u64 = 3;
    pub const JMP: u64 = 2;
    pub const JZ: u64 = 2;
    pub const CALL: u64 = 5;
    pub const RET: u64 = 3;
    pub const HASH: u64 = 20;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let mut interp = Interpreter::new(1000);
        
        // ADD r0, r1, r2  (r0 = r1 + r2)
        interp.set_register(1, 10);
        interp.set_register(2, 20);
        
        let program = vec![
            Instruction::new(OpCode::Add, 0, 1, 2),
            Instruction::new(OpCode::Halt, 0, 0, 0),
        ];
        
        interp.execute(&program).expect("execution failed");
        assert_eq!(interp.get_register(0), 30);
    }

    #[test]
    fn test_memory_operations() {
        let mut interp = Interpreter::new(1000);
        
        // STORE r1 to memory address 100, then LOAD back to r3
        interp.set_register(1, 42);
        
        let program = vec![
            Instruction::new(OpCode::Store, 0, 1, 100),  // Store r1 to mem[100]
            Instruction::new(OpCode::Load, 3, 0, 100),   // Load mem[100] to r3
            Instruction::new(OpCode::Halt, 0, 0, 0),
        ];
        
        interp.execute(&program).expect("execution failed");
        assert_eq!(interp.get_register(3), 42);
    }

    #[test]
    fn test_conditional_jump() {
        let mut interp = Interpreter::new(1000);
        
        // JZ: jump if zero
        interp.set_register(1, 0);
        interp.set_register(2, 10);
        interp.set_register(3, 5);
        
        let program = vec![
            Instruction::new(OpCode::Jz, 0, 1, 3),      // If r1 == 0, jump to addr 3
            Instruction::new(OpCode::Add, 0, 0, 2),     // Skip this (add r0 + r2)
            Instruction::new(OpCode::Halt, 0, 0, 0),
            Instruction::new(OpCode::Add, 0, 0, 3),     // Execute this (add r0 + r3)
            Instruction::new(OpCode::Halt, 0, 0, 0),
        ];
        
        interp.execute(&program).expect("execution failed");
        assert_eq!(interp.get_register(0), 5);
    }

    #[test]
    fn test_gas_metering() {
        let mut interp = Interpreter::new(10); // Only 10 gas
        
        let program = vec![
            Instruction::new(OpCode::Add, 0, 1, 2),    // 1 gas
            Instruction::new(OpCode::Mul, 3, 4, 5),    // 2 gas
            Instruction::new(OpCode::Div, 6, 7, 8),    // 4 gas
            Instruction::new(OpCode::Div, 9, 10, 11),  // 4 gas (would exceed)
            Instruction::new(OpCode::Halt, 0, 0, 0),
        ];
        
        let result = interp.execute(&program);
        assert!(result.is_err()); // Should fail due to out of gas
    }
}
