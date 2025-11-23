//! ZKVM Instruction Set
//!
//! RISC-like instruction set designed for ZK-SNARK verification.

use serde::{Deserialize, Serialize};

/// Operation codes for the ZKVM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpCode {
    // Arithmetic
    Add,    // rd = rs1 + rs2
    Sub,    // rd = rs1 - rs2
    Mul,    // rd = rs1 * rs2
    Div,    // rd = rs1 / rs2
    Mod,    // rd = rs1 % rs2
    
    // Logic
    And,    // rd = rs1 & rs2
    Or,     // rd = rs1 | rs2
    Xor,    // rd = rs1 ^ rs2
    Not,    // rd = !rs1
    
    // Comparison
    Eq,     // rd = (rs1 == rs2) ? 1 : 0
    Lt,     // rd = (rs1 < rs2) ? 1 : 0
    Gt,     // rd = (rs1 > rs2) ? 1 : 0
    Le,     // rd = (rs1 <= rs2) ? 1 : 0
    Ge,     // rd = (rs1 >= rs2) ? 1 : 0
    
    // Memory
    Load,   // rd = mem[rs1 + imm]
    Store,  // mem[rs2 + imm] = rs1
    
    // Control Flow
    Jmp,    // pc = imm
    Jz,     // if rs1 == 0: pc = imm
    Call,   // call subroutine at imm
    Ret,    // return from subroutine
    
    // Crypto (field-friendly operations)
    Hash,   // rd = hash(rs1, rs2)
    
    // System
    Halt,   // stop execution
}

/// Instruction format: 4 fields (opcode, rd, rs1, rs2/imm)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Instruction {
    pub opcode: OpCode,
    pub rd: u8,     // destination register (0-31)
    pub rs1: u8,    // source register 1
    pub rs2_imm: u32, // source register 2 or immediate value
}

impl Instruction {
    /// Create a new instruction
    pub fn new(opcode: OpCode, rd: u8, rs1: u8, rs2_imm: u32) -> Self {
        Self {
            opcode,
            rd,
            rs1,
            rs2_imm,
        }
    }
    
    /// Get rs2 as a register index
    pub fn rs2(&self) -> u8 {
        (self.rs2_imm & 0xFF) as u8
    }
    
    /// Get immediate value
    pub fn imm(&self) -> u32 {
        self.rs2_imm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_creation() {
        let inst = Instruction::new(OpCode::Add, 1, 2, 3);
        assert_eq!(inst.opcode, OpCode::Add);
        assert_eq!(inst.rd, 1);
        assert_eq!(inst.rs1, 2);
        assert_eq!(inst.rs2(), 3);
    }

    #[test]
    fn test_immediate_value() {
        let inst = Instruction::new(OpCode::Jmp, 0, 0, 1000);
        assert_eq!(inst.imm(), 1000);
    }
}
