//! BitCell Contract Language (BCL) Compiler CLI
//!
//! Compiles .bcl files to ZKVM bytecode

use bitcell_compiler::{compile, CompilerError};
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <input.bcl> [output.bin]", args[0]);
        eprintln!("       {} --example <name>", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} contract.bcl", args[0]);
        eprintln!("  {} contract.bcl output.bin", args[0]);
        eprintln!("  {} --example token", args[0]);
        eprintln!("  {} --example counter", args[0]);
        std::process::exit(1);
    }
    
    // Handle example contracts
    if args[1] == "--example" {
        if args.len() < 3 {
            eprintln!("Error: Please specify an example name");
            eprintln!("Available examples: token, counter, escrow");
            std::process::exit(1);
        }
        
        let example_source = match args[2].as_str() {
            "token" => bitcell_compiler::stdlib::patterns::TOKEN_CONTRACT,
            "counter" => bitcell_compiler::stdlib::patterns::COUNTER_CONTRACT,
            "escrow" => bitcell_compiler::stdlib::patterns::ESCROW_CONTRACT,
            _ => {
                eprintln!("Error: Unknown example '{}'", args[2]);
                eprintln!("Available examples: token, counter, escrow");
                std::process::exit(1);
            }
        };
        
        println!("{}", example_source);
        
        println!("\nCompiling example...");
        match compile_source(example_source) {
            Ok(instructions) => {
                println!("✓ Compilation successful!");
                println!("Generated {} instructions", instructions.len());
            }
            Err(e) => {
                eprintln!("✗ Compilation failed: {}", e);
                std::process::exit(1);
            }
        }
        
        return;
    }
    
    let input_path = PathBuf::from(&args[1]);
    let output_path = if args.len() >= 3 {
        PathBuf::from(&args[2])
    } else {
        input_path.with_extension("bin")
    };
    
    // Read source file
    let source = match fs::read_to_string(&input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", input_path.display(), e);
            std::process::exit(1);
        }
    };
    
    println!("Compiling {}...", input_path.display());
    
    // Compile
    match compile_source(&source) {
        Ok(instructions) => {
            println!("✓ Compilation successful!");
            println!("Generated {} instructions", instructions.len());
            
            // Serialize instructions to binary format
            let bytecode = serialize_instructions(&instructions);
            
            // Write output
            if let Err(e) = fs::write(&output_path, bytecode) {
                eprintln!("Error writing output file '{}': {}", output_path.display(), e);
                std::process::exit(1);
            }
            
            println!("Output written to {}", output_path.display());
        }
        Err(e) => {
            eprintln!("✗ Compilation failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn compile_source(source: &str) -> Result<Vec<bitcell_zkvm::Instruction>, CompilerError> {
    compile(source)
}

fn serialize_instructions(instructions: &[bitcell_zkvm::Instruction]) -> Vec<u8> {
    // Simple binary serialization
    // Format: [count: u32][instruction1][instruction2]...
    // Each instruction: [opcode: u8][rd: u8][rs1: u8][rs2_imm: u32]
    
    let mut bytes = Vec::new();
    
    // Write instruction count
    let count = instructions.len() as u32;
    bytes.extend_from_slice(&count.to_le_bytes());
    
    // Write each instruction
    for inst in instructions {
        // Opcode as u8
        bytes.push(opcode_to_byte(&inst.opcode));
        bytes.push(inst.rd);
        bytes.push(inst.rs1);
        bytes.extend_from_slice(&inst.rs2_imm.to_le_bytes());
    }
    
    bytes
}

fn opcode_to_byte(opcode: &bitcell_zkvm::OpCode) -> u8 {
    use bitcell_zkvm::OpCode;
    match opcode {
        OpCode::Add => 0,
        OpCode::Sub => 1,
        OpCode::Mul => 2,
        OpCode::Div => 3,
        OpCode::Mod => 4,
        OpCode::And => 5,
        OpCode::Or => 6,
        OpCode::Xor => 7,
        OpCode::Not => 8,
        OpCode::Eq => 9,
        OpCode::Lt => 10,
        OpCode::Gt => 11,
        OpCode::Le => 12,
        OpCode::Ge => 13,
        OpCode::Load => 14,
        OpCode::Store => 15,
        OpCode::Jmp => 16,
        OpCode::Jz => 17,
        OpCode::Call => 18,
        OpCode::Ret => 19,
        OpCode::Hash => 20,
        OpCode::Halt => 21,
    }
}
