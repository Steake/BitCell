//! # BitCell Compiler
//!
//! A Solidity-like compiler for BitCell smart contracts.
//! Compiles high-level contract code to ZKVM bytecode.
//!
//! ## Language: BitCell Contract Language (BCL)
//!
//! BCL is a simplified Solidity-like language designed for ZK-friendly smart contracts.
//!
//! ### Example:
//! ```text
//! contract SimpleToken {
//!     storage {
//!         balances: mapping(address => uint);
//!         total_supply: uint;
//!     }
//!
//!     function transfer(to: address, amount: uint) -> bool {
//!         let sender = msg.sender;
//!         require(balances[sender] >= amount, "Insufficient balance");
//!         
//!         balances[sender] = balances[sender] - amount;
//!         balances[to] = balances[to] + amount;
//!         
//!         return true;
//!     }
//! }
//! ```

pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod stdlib;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("Lexer error at line {line}, column {col}: {message}")]
    LexerError {
        line: usize,
        col: usize,
        message: String,
    },
    #[error("Parser error: {0}")]
    ParserError(String),
    #[error("Semantic error: {0}")]
    SemanticError(String),
    #[error("Code generation error: {0}")]
    CodeGenError(String),
}

pub type Result<T> = std::result::Result<T, CompilerError>;

/// Compile BCL source code to ZKVM bytecode
pub fn compile(source: &str) -> Result<Vec<bitcell_zkvm::Instruction>> {
    // Lexical analysis
    let tokens = lexer::tokenize(source)?;
    
    // Parsing
    let ast = parser::parse(tokens)?;
    
    // Semantic analysis
    semantic::analyze(&ast)?;
    
    // Code generation
    let instructions = codegen::generate(&ast)?;
    
    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_contract() {
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
        
        let result = compile(source);
        assert!(result.is_ok());
    }
}
