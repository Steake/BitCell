//! ZKVM Interpreter
//!
//! Executes ZKVM instructions and generates execution traces for ZK proving.

use crate::{gas, Instruction, Memory, OpCode};
use serde::{Deserialize, Serialize};

/// Execution trace for ZK proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub steps: Vec<TraceStep>,
    pub gas_used: u64,
}

/// Single step in execution trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    pub pc: usize,
    pub instruction: Instruction,
    pub registers_before: Vec<u64>,
    pub registers_after: Vec<u64>,
    pub memory_reads: Vec<(u32, u64)>,
    pub memory_writes: Vec<(u32, u64)>,
}

#[derive(Debug)]
pub enum InterpreterError {
    OutOfGas,
    InvalidMemoryAccess(String),
    DivisionByZero,
    InvalidJump(usize),
    ProgramTooLarge,
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::OutOfGas => write!(f, "Out of gas"),
            Self::InvalidMemoryAccess(msg) => write!(f, "Invalid memory access: {}", msg),
            Self::DivisionByZero => write!(f, "Division by zero"),
            Self::InvalidJump(addr) => write!(f, "Invalid jump to address {}", addr),
            Self::ProgramTooLarge => write!(f, "Program too large"),
        }
    }
}

impl std::error::Error for InterpreterError {}

/// ZKVM Interpreter with 32 general-purpose registers
pub struct Interpreter {
    registers: [u64; 32],
    memory: Memory,
    pc: usize,
    gas_limit: u64,
    gas_used: u64,
    call_stack: Vec<usize>,
    trace: ExecutionTrace,
}

impl Interpreter {
    /// Create new interpreter with gas limit
    pub fn new(gas_limit: u64) -> Self {
        Self {
            registers: [0; 32],
            memory: Memory::new(1024 * 1024), // 1MB address space
            pc: 0,
            gas_limit,
            gas_used: 0,
            call_stack: Vec::new(),
            trace: ExecutionTrace {
                steps: Vec::new(),
                gas_used: 0,
            },
        }
    }
    
    /// Set register value
    pub fn set_register(&mut self, reg: u8, value: u64) {
        if (reg as usize) < 32 {
            self.registers[reg as usize] = value;
        }
    }
    
    /// Get register value
    pub fn get_register(&self, reg: u8) -> u64 {
        if (reg as usize) < 32 {
            self.registers[reg as usize]
        } else {
            0
        }
    }
    
    /// Execute a program
    pub fn execute(&mut self, program: &[Instruction]) -> Result<(), InterpreterError> {
        if program.len() > 100000 {
            return Err(InterpreterError::ProgramTooLarge);
        }
        
        self.pc = 0;
        
        while self.pc < program.len() {
            let inst = program[self.pc];
            
            // Check gas
            let gas_cost = self.gas_cost(&inst.opcode);
            if self.gas_used + gas_cost > self.gas_limit {
                return Err(InterpreterError::OutOfGas);
            }
            self.gas_used += gas_cost;
            
            // Execute instruction
            let registers_before = self.registers.clone();
            let mut memory_reads = Vec::new();
            let mut memory_writes = Vec::new();
            
            match inst.opcode {
                OpCode::Add => {
                    let lhs = self.get_register(inst.rs1);
                    let rhs = self.get_register(inst.rs2());
                    self.set_register(inst.rd, lhs.wrapping_add(rhs));
                    self.pc += 1;
                }
                OpCode::Sub => {
                    let lhs = self.get_register(inst.rs1);
                    let rhs = self.get_register(inst.rs2());
                    self.set_register(inst.rd, lhs.wrapping_sub(rhs));
                    self.pc += 1;
                }
                OpCode::Mul => {
                    let lhs = self.get_register(inst.rs1);
                    let rhs = self.get_register(inst.rs2());
                    self.set_register(inst.rd, lhs.wrapping_mul(rhs));
                    self.pc += 1;
                }
                OpCode::Div => {
                    let lhs = self.get_register(inst.rs1);
                    let rhs = self.get_register(inst.rs2());
                    if rhs == 0 {
                        return Err(InterpreterError::DivisionByZero);
                    }
                    self.set_register(inst.rd, lhs / rhs);
                    self.pc += 1;
                }
                OpCode::Mod => {
                    let lhs = self.get_register(inst.rs1);
                    let rhs = self.get_register(inst.rs2());
                    if rhs == 0 {
                        return Err(InterpreterError::DivisionByZero);
                    }
                    self.set_register(inst.rd, lhs % rhs);
                    self.pc += 1;
                }
                OpCode::And => {
                    let lhs = self.get_register(inst.rs1);
                    let rhs = self.get_register(inst.rs2());
                    self.set_register(inst.rd, lhs & rhs);
                    self.pc += 1;
                }
                OpCode::Or => {
                    let lhs = self.get_register(inst.rs1);
                    let rhs = self.get_register(inst.rs2());
                    self.set_register(inst.rd, lhs | rhs);
                    self.pc += 1;
                }
                OpCode::Xor => {
                    let lhs = self.get_register(inst.rs1);
                    let rhs = self.get_register(inst.rs2());
                    self.set_register(inst.rd, lhs ^ rhs);
                    self.pc += 1;
                }
                OpCode::Not => {
                    let lhs = self.get_register(inst.rs1);
                    self.set_register(inst.rd, !lhs);
                    self.pc += 1;
                }
                OpCode::Eq => {
                    let lhs = self.get_register(inst.rs1);
                    let rhs = self.get_register(inst.rs2());
                    self.set_register(inst.rd, if lhs == rhs { 1 } else { 0 });
                    self.pc += 1;
                }
                OpCode::Lt => {
                    let lhs = self.get_register(inst.rs1);
                    let rhs = self.get_register(inst.rs2());
                    self.set_register(inst.rd, if lhs < rhs { 1 } else { 0 });
                    self.pc += 1;
                }
                OpCode::Gt => {
                    let lhs = self.get_register(inst.rs1);
                    let rhs = self.get_register(inst.rs2());
                    self.set_register(inst.rd, if lhs > rhs { 1 } else { 0 });
                    self.pc += 1;
                }
                OpCode::Le => {
                    let lhs = self.get_register(inst.rs1);
                    let rhs = self.get_register(inst.rs2());
                    self.set_register(inst.rd, if lhs <= rhs { 1 } else { 0 });
                    self.pc += 1;
                }
                OpCode::Ge => {
                    let lhs = self.get_register(inst.rs1);
                    let rhs = self.get_register(inst.rs2());
                    self.set_register(inst.rd, if lhs >= rhs { 1 } else { 0 });
                    self.pc += 1;
                }
                OpCode::Load => {
                    let addr = self.get_register(inst.rs1) as u32 + inst.imm();
                    let value = self.memory.load(addr)
                        .map_err(InterpreterError::InvalidMemoryAccess)?;
                    memory_reads.push((addr, value));
                    self.set_register(inst.rd, value);
                    self.pc += 1;
                }
                OpCode::Store => {
                    let addr = self.get_register(inst.rs2()) as u32 + inst.imm();
                    let value = self.get_register(inst.rs1);
                    self.memory.store(addr, value)
                        .map_err(InterpreterError::InvalidMemoryAccess)?;
                    memory_writes.push((addr, value));
                    self.pc += 1;
                }
                OpCode::Jmp => {
                    let target = inst.imm() as usize;
                    if target >= program.len() {
                        return Err(InterpreterError::InvalidJump(target));
                    }
                    self.pc = target;
                }
                OpCode::Jz => {
                    let cond = self.get_register(inst.rs1);
                    if cond == 0 {
                        let target = inst.imm() as usize;
                        if target >= program.len() {
                            return Err(InterpreterError::InvalidJump(target));
                        }
                        self.pc = target;
                    } else {
                        self.pc += 1;
                    }
                }
                OpCode::Call => {
                    let target = inst.imm() as usize;
                    if target >= program.len() {
                        return Err(InterpreterError::InvalidJump(target));
                    }
                    self.call_stack.push(self.pc + 1);
                    self.pc = target;
                }
                OpCode::Ret => {
                    if let Some(return_addr) = self.call_stack.pop() {
                        self.pc = return_addr;
                    } else {
                        // No return address, halt
                        break;
                    }
                }
                OpCode::Hash => {
                    // Simple hash: XOR and rotate
                    let a = self.get_register(inst.rs1);
                    let b = self.get_register(inst.rs2());
                    let hash = (a ^ b).rotate_left(17);
                    self.set_register(inst.rd, hash);
                    self.pc += 1;
                }
                OpCode::Halt => {
                    break;
                }
            }
            
            // Record trace step
            self.trace.steps.push(TraceStep {
                pc: self.pc,
                instruction: inst,
                registers_before: registers_before.to_vec(),
                registers_after: self.registers.to_vec(),
                memory_reads,
                memory_writes,
            });
        }
        
        self.trace.gas_used = self.gas_used;
        Ok(())
    }
    
    /// Get execution trace
    pub fn trace(&self) -> &ExecutionTrace {
        &self.trace
    }
    
    /// Get gas used
    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }
    
    fn gas_cost(&self, opcode: &OpCode) -> u64 {
        match opcode {
            OpCode::Add => gas::ADD,
            OpCode::Sub => gas::SUB,
            OpCode::Mul => gas::MUL,
            OpCode::Div => gas::DIV,
            OpCode::Mod => gas::MOD,
            OpCode::And => gas::AND,
            OpCode::Or => gas::OR,
            OpCode::Xor => gas::XOR,
            OpCode::Not => gas::NOT,
            OpCode::Eq => gas::EQ,
            OpCode::Lt => gas::LT,
            OpCode::Gt => gas::GT,
            OpCode::Le => gas::LT,  // Same cost as LT
            OpCode::Ge => gas::GT,  // Same cost as GT
            OpCode::Load => gas::LOAD,
            OpCode::Store => gas::STORE,
            OpCode::Jmp => gas::JMP,
            OpCode::Jz => gas::JZ,
            OpCode::Call => gas::CALL,
            OpCode::Ret => gas::RET,
            OpCode::Hash => gas::HASH,
            OpCode::Halt => 0,
        }
    }
}
