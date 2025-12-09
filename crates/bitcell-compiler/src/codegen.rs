//! Code generator for BCL to ZKVM bytecode

use crate::ast::*;
use crate::{CompilerError, Result};
use bitcell_zkvm::{Instruction, OpCode};
use std::collections::HashMap;

pub fn generate(contract: &Contract) -> Result<Vec<Instruction>> {
    let mut generator = CodeGenerator::new();
    generator.generate_contract(contract)
}

struct CodeGenerator {
    instructions: Vec<Instruction>,
    storage_addrs: HashMap<String, u32>,
    local_regs: HashMap<String, u8>,
    next_storage_addr: u32,
    next_reg: u8,
    label_counter: usize,
}

impl CodeGenerator {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            storage_addrs: HashMap::new(),
            local_regs: HashMap::new(),
            next_storage_addr: 0x200, // Storage starts at 0x200
            next_reg: 10, // Registers 0-9 reserved for special purposes
            label_counter: 0,
        }
    }
    
    fn generate_contract(&mut self, contract: &Contract) -> Result<Vec<Instruction>> {
        // Allocate storage addresses
        for decl in &contract.storage {
            self.storage_addrs.insert(decl.name.clone(), self.next_storage_addr);
            self.next_storage_addr += 8; // 8 bytes per storage slot
        }
        
        // Generate function dispatcher
        self.generate_dispatcher(&contract.functions)?;
        
        // Generate each function
        for func in &contract.functions {
            self.generate_function(func)?;
        }
        
        // Add halt instruction
        self.emit(OpCode::Halt, 0, 0, 0);
        
        Ok(self.instructions.clone())
    }
    
    fn generate_dispatcher(&mut self, functions: &[Function]) -> Result<()> {
        // Load function selector from memory address 0x10 (msg.data[0])
        self.emit(OpCode::Load, 1, 0, 0x10);
        
        // For each function, compare selector and jump to function
        for (i, func) in functions.iter().enumerate() {
            let func_id = self.hash_function_name(&func.name);
            let func_addr = 100 + (i * 200) as u32; // Each function gets 200 instruction slots
            
            // Load function ID into r2
            self.emit_load_immediate(2, func_id);
            
            // Compare r1 with r2, store result in r3
            self.emit(OpCode::Eq, 3, 1, 2);
            
            // If NOT equal (r3 == 0), skip to next check
            // If equal (r3 != 0), jump to function
            let skip_addr = (self.instructions.len() + 2) as u32;
            self.emit(OpCode::Jz, 0, 3, skip_addr);
            self.emit(OpCode::Jmp, 0, 0, func_addr);
        }
        
        // If no function matched, revert
        self.emit(OpCode::Halt, 0, 0, 0);
        
        Ok(())
    }
    
    fn generate_function(&mut self, func: &Function) -> Result<()> {
        self.local_regs.clear();
        self.next_reg = 10;
        
        // Allocate registers for parameters
        for (i, param) in func.params.iter().enumerate() {
            let reg = self.alloc_register();
            self.local_regs.insert(param.name.clone(), reg);
            
            // Load parameter from memory (parameters start at 0x20)
            let param_addr = 0x20 + (i * 8) as u32;
            self.emit(OpCode::Load, reg, 0, param_addr);
        }
        
        // Generate function body
        for stmt in &func.body {
            self.generate_statement(stmt)?;
        }
        
        Ok(())
    }
    
    fn generate_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Let { name, value } => {
                let reg = self.alloc_register();
                self.generate_expression(value, reg)?;
                self.local_regs.insert(name.clone(), reg);
                Ok(())
            }
            Statement::Assign { target, value } => {
                match target {
                    Expression::Identifier(name) => {
                        if let Some(&storage_addr) = self.storage_addrs.get(name) {
                            // Store to storage
                            let value_reg = self.alloc_temp_register();
                            self.generate_expression(value, value_reg)?;
                            self.emit(OpCode::Store, 0, value_reg, storage_addr);
                        } else if let Some(&reg) = self.local_regs.get(name) {
                            // Store to local register
                            self.generate_expression(value, reg)?;
                        } else {
                            return Err(CompilerError::CodeGenError(format!(
                                "Undefined variable: {}",
                                name
                            )));
                        }
                    }
                    Expression::Index { expr, index } => {
                        // For mapping[key] = value
                        // This is simplified - real implementation needs hash-based storage
                        let key_reg = self.alloc_temp_register();
                        self.generate_expression(index, key_reg)?;
                        
                        let value_reg = self.alloc_temp_register();
                        self.generate_expression(value, value_reg)?;
                        
                        // Compute storage address: base + hash(key)
                        if let Expression::Identifier(name) = &**expr {
                            if let Some(&base_addr) = self.storage_addrs.get(name) {
                                // Simple address computation: base + key (should be hash in real impl)
                                let addr_reg = self.alloc_temp_register();
                                self.emit_load_immediate(addr_reg, base_addr as u64);
                                self.emit(OpCode::Add, addr_reg, addr_reg, key_reg as u32);
                                
                                // Store value at computed address (using addr_reg)
                                // Note: ZKVM Store format is: Store rs2, rs1, offset
                                // where mem[rs1 + offset] = rs2
                                // Here we want mem[addr_reg] = value_reg
                                self.emit(OpCode::Store, 0, value_reg, 0);
                                // TODO: This needs proper addressing - currently simplified
                            }
                        }
                    }
                    _ => {
                        return Err(CompilerError::CodeGenError(
                            "Invalid assignment target".to_string(),
                        ));
                    }
                }
                Ok(())
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let cond_reg = self.alloc_temp_register();
                self.generate_expression(condition, cond_reg)?;
                
                let else_label = self.new_label();
                let end_label = self.new_label();
                
                // Jump to else if condition is false (0)
                self.emit(OpCode::Jz, 0, cond_reg, else_label as u32);
                
                // Then block
                for stmt in then_block {
                    self.generate_statement(stmt)?;
                }
                self.emit(OpCode::Jmp, 0, 0, end_label as u32);
                
                // Else block (or empty)
                let _else_addr = self.instructions.len();
                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.generate_statement(stmt)?;
                    }
                }
                
                let _end_addr = self.instructions.len();
                
                // Patch jump addresses
                // (In real implementation, we'd do a two-pass assembly or use labels)
                
                Ok(())
            }
            Statement::Return { value } => {
                if let Some(expr) = value {
                    let result_reg = 0; // Return value in r0
                    self.generate_expression(expr, result_reg)?;
                }
                self.emit(OpCode::Ret, 0, 0, 0);
                Ok(())
            }
            Statement::Require { condition, message: _ } => {
                let cond_reg = self.alloc_temp_register();
                self.generate_expression(condition, cond_reg)?;
                
                // If condition is 0 (false), jump to halt
                let halt_addr = (self.instructions.len() + 2) as u32;
                self.emit(OpCode::Jz, 0, cond_reg, halt_addr);
                
                // Continue execution (skip halt)
                let continue_addr = (self.instructions.len() + 1) as u32;
                self.emit(OpCode::Jmp, 0, 0, continue_addr);
                
                // Halt (revert) - this is the target of the Jz above
                self.emit(OpCode::Halt, 0, 0, 0);
                
                Ok(())
            }
            Statement::Expression(expr) => {
                let temp_reg = self.alloc_temp_register();
                self.generate_expression(expr, temp_reg)?;
                Ok(())
            }
        }
    }
    
    fn generate_expression(&mut self, expr: &Expression, dest_reg: u8) -> Result<()> {
        match expr {
            Expression::Literal(lit) => {
                match lit {
                    Literal::Uint(n) => {
                        self.emit_load_immediate(dest_reg, *n);
                    }
                    Literal::Bool(b) => {
                        self.emit_load_immediate(dest_reg, if *b { 1 } else { 0 });
                    }
                    Literal::Address(_) => {
                        // Simplified: load 0 for addresses
                        self.emit_load_immediate(dest_reg, 0);
                    }
                }
                Ok(())
            }
            Expression::Identifier(name) => {
                if let Some(&storage_addr) = self.storage_addrs.get(name) {
                    // Load from storage
                    self.emit(OpCode::Load, dest_reg, 0, storage_addr);
                } else if let Some(&reg) = self.local_regs.get(name) {
                    // Copy from local register
                    if reg != dest_reg {
                        self.emit(OpCode::Add, dest_reg, reg, 0); // Copy via add with 0
                    }
                } else {
                    return Err(CompilerError::CodeGenError(format!(
                        "Undefined variable: {}",
                        name
                    )));
                }
                Ok(())
            }
            Expression::Binary { left, op, right } => {
                let left_reg = self.alloc_temp_register();
                self.generate_expression(left, left_reg)?;
                
                let right_reg = self.alloc_temp_register();
                self.generate_expression(right, right_reg)?;
                
                let opcode = match op {
                    BinaryOp::Add => OpCode::Add,
                    BinaryOp::Sub => OpCode::Sub,
                    BinaryOp::Mul => OpCode::Mul,
                    BinaryOp::Div => OpCode::Div,
                    BinaryOp::Mod => OpCode::Mod,
                    BinaryOp::Eq => OpCode::Eq,
                    BinaryOp::Lt => OpCode::Lt,
                    BinaryOp::Gt => OpCode::Gt,
                    BinaryOp::Le => OpCode::Le,
                    BinaryOp::Ge => OpCode::Ge,
                    BinaryOp::And => OpCode::And,
                    BinaryOp::Or => OpCode::Or,
                    BinaryOp::Ne => {
                        // Ne is implemented as !(==)
                        self.emit(OpCode::Eq, dest_reg, left_reg, right_reg as u32);
                        self.emit(OpCode::Not, dest_reg, dest_reg, 0);
                        return Ok(());
                    }
                };
                
                self.emit(opcode, dest_reg, left_reg, right_reg as u32);
                Ok(())
            }
            Expression::Unary { op, expr } => {
                self.generate_expression(expr, dest_reg)?;
                match op {
                    UnaryOp::Not => {
                        self.emit(OpCode::Not, dest_reg, dest_reg, 0);
                    }
                    UnaryOp::Neg => {
                        // Negation: 0 - x
                        let zero_reg = self.alloc_temp_register();
                        self.emit_load_immediate(zero_reg, 0);
                        self.emit(OpCode::Sub, dest_reg, zero_reg, dest_reg as u32);
                    }
                }
                Ok(())
            }
            Expression::Call { .. } => {
                // Simplified: function calls not fully implemented
                self.emit_load_immediate(dest_reg, 0);
                Ok(())
            }
            Expression::Index { expr, index } => {
                // Load from mapping
                let key_reg = self.alloc_temp_register();
                self.generate_expression(index, key_reg)?;
                
                if let Expression::Identifier(name) = &**expr {
                    if let Some(&base_addr) = self.storage_addrs.get(name) {
                        // Compute address: base + hash(key)
                        let addr_reg = self.alloc_temp_register();
                        self.emit_load_immediate(addr_reg, base_addr as u64);
                        self.emit(OpCode::Add, addr_reg, addr_reg, key_reg as u32);
                        
                        // Load value from computed address
                        self.emit(OpCode::Load, dest_reg, addr_reg, 0);
                    }
                }
                Ok(())
            }
            Expression::MemberAccess { expr, member } => {
                // Handle msg.sender, msg.value, block.number, etc.
                if let Expression::Identifier(obj) = &**expr {
                    match (obj.as_str(), member.as_str()) {
                        ("msg", "sender") => {
                            self.emit(OpCode::Load, dest_reg, 0, 0x14);  // Updated address
                        }
                        ("msg", "value") => {
                            self.emit(OpCode::Load, dest_reg, 0, 0x18);
                        }
                        ("block", "number") => {
                            self.emit(OpCode::Load, dest_reg, 0, 0x20);
                        }
                        ("block", "timestamp") => {
                            self.emit(OpCode::Load, dest_reg, 0, 0x28);
                        }
                        _ => {
                            // Unknown member access, load 0
                            self.emit_load_immediate(dest_reg, 0);
                        }
                    }
                } else {
                    // Complex member access not fully supported
                    self.emit_load_immediate(dest_reg, 0);
                }
                Ok(())
            }
        }
    }
    
    fn emit(&mut self, opcode: OpCode, rd: u8, rs1: u8, rs2_imm: u32) {
        self.instructions.push(Instruction::new(opcode, rd, rs1, rs2_imm));
    }
    
    fn emit_load_immediate(&mut self, reg: u8, value: u64) {
        // Simple immediate load by using the rs2_imm field
        // Note: This only works for values that fit in 32 bits
        // For larger values, would need multiple instructions
        let value_u32 = (value & 0xFFFFFFFF) as u32;
        
        // Load by adding immediate to register 0 (assuming it's zero)
        // This is a simplification - real implementation would:
        // 1. Use a proper immediate load instruction, or
        // 2. Initialize r0 to 0 and add immediate
        // 3. Or use a two-instruction sequence for full 64-bit values
        self.emit(OpCode::Add, reg, reg, value_u32);
    }
    
    fn alloc_register(&mut self) -> u8 {
        let reg = self.next_reg;
        self.next_reg += 1;
        if self.next_reg >= 32 {
            self.next_reg = 10; // Wrap around (in real impl, we'd do register allocation)
        }
        reg
    }
    
    fn alloc_temp_register(&mut self) -> u8 {
        self.alloc_register()
    }
    
    fn hash_function_name(&self, name: &str) -> u64 {
        // Simple hash for function selector
        let mut hash = 0u64;
        for b in name.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(b as u64);
        }
        hash
    }
    
    fn new_label(&mut self) -> usize {
        let label = self.label_counter;
        self.label_counter += 1;
        label
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;
    use crate::semantic::analyze;

    #[test]
    fn test_codegen_simple() {
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
        
        let tokens = tokenize(source).unwrap();
        let contract = parse(tokens).unwrap();
        analyze(&contract).unwrap();
        let instructions = generate(&contract).unwrap();
        
        assert!(!instructions.is_empty());
    }
}
