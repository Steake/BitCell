//! Semantic analyzer for BCL

use crate::ast::*;
use crate::{CompilerError, Result};
use std::collections::HashMap;

pub fn analyze(contract: &Contract) -> Result<()> {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_contract(contract)
}

struct SemanticAnalyzer {
    storage_vars: HashMap<String, Type>,
    local_vars: HashMap<String, Type>,
}

impl SemanticAnalyzer {
    fn new() -> Self {
        Self {
            storage_vars: HashMap::new(),
            local_vars: HashMap::new(),
        }
    }
    
    fn analyze_contract(&mut self, contract: &Contract) -> Result<()> {
        // Collect storage variables
        for decl in &contract.storage {
            if self.storage_vars.contains_key(&decl.name) {
                return Err(CompilerError::SemanticError(format!(
                    "Duplicate storage variable: {}",
                    decl.name
                )));
            }
            self.storage_vars.insert(decl.name.clone(), decl.ty.clone());
        }
        
        // Analyze each function
        for func in &contract.functions {
            self.analyze_function(func)?;
        }
        
        Ok(())
    }
    
    fn analyze_function(&mut self, func: &Function) -> Result<()> {
        self.local_vars.clear();
        
        // Add parameters to local scope
        for param in &func.params {
            if self.local_vars.contains_key(&param.name) {
                return Err(CompilerError::SemanticError(format!(
                    "Duplicate parameter: {}",
                    param.name
                )));
            }
            self.local_vars.insert(param.name.clone(), param.ty.clone());
        }
        
        // Analyze function body
        for stmt in &func.body {
            self.analyze_statement(stmt)?;
        }
        
        Ok(())
    }
    
    fn analyze_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Let { name, value } => {
                let ty = self.type_of_expression(value)?;
                self.local_vars.insert(name.clone(), ty);
                Ok(())
            }
            Statement::Assign { target, value } => {
                let target_ty = self.type_of_expression(target)?;
                let value_ty = self.type_of_expression(value)?;
                
                if target_ty != value_ty {
                    return Err(CompilerError::SemanticError(format!(
                        "Type mismatch in assignment: expected {:?}, found {:?}",
                        target_ty, value_ty
                    )));
                }
                
                Ok(())
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let cond_ty = self.type_of_expression(condition)?;
                if cond_ty != Type::Bool {
                    return Err(CompilerError::SemanticError(
                        "If condition must be boolean".to_string(),
                    ));
                }
                
                for stmt in then_block {
                    self.analyze_statement(stmt)?;
                }
                
                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.analyze_statement(stmt)?;
                    }
                }
                
                Ok(())
            }
            Statement::Return { value } => {
                if let Some(expr) = value {
                    self.type_of_expression(expr)?;
                }
                Ok(())
            }
            Statement::Require { condition, .. } => {
                let cond_ty = self.type_of_expression(condition)?;
                if cond_ty != Type::Bool {
                    return Err(CompilerError::SemanticError(
                        "Require condition must be boolean".to_string(),
                    ));
                }
                Ok(())
            }
            Statement::Expression(expr) => {
                self.type_of_expression(expr)?;
                Ok(())
            }
        }
    }
    
    fn type_of_expression(&self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::Literal(lit) => Ok(match lit {
                Literal::Uint(_) => Type::Uint,
                Literal::Bool(_) => Type::Bool,
                Literal::Address(_) => Type::Address,
            }),
            Expression::Identifier(name) => {
                if let Some(ty) = self.local_vars.get(name) {
                    Ok(ty.clone())
                } else if let Some(ty) = self.storage_vars.get(name) {
                    Ok(ty.clone())
                } else {
                    Err(CompilerError::SemanticError(format!(
                        "Undefined variable: {}",
                        name
                    )))
                }
            }
            Expression::Binary { left, op, right } => {
                let left_ty = self.type_of_expression(left)?;
                let right_ty = self.type_of_expression(right)?;
                
                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        if left_ty != Type::Uint || right_ty != Type::Uint {
                            return Err(CompilerError::SemanticError(
                                "Arithmetic operations require uint operands".to_string(),
                            ));
                        }
                        Ok(Type::Uint)
                    }
                    BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                        if left_ty != right_ty {
                            return Err(CompilerError::SemanticError(
                                "Comparison operands must have same type".to_string(),
                            ));
                        }
                        Ok(Type::Bool)
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if left_ty != Type::Bool || right_ty != Type::Bool {
                            return Err(CompilerError::SemanticError(
                                "Logical operations require boolean operands".to_string(),
                            ));
                        }
                        Ok(Type::Bool)
                    }
                }
            }
            Expression::Unary { op, expr } => {
                let ty = self.type_of_expression(expr)?;
                match op {
                    UnaryOp::Not => {
                        if ty != Type::Bool {
                            return Err(CompilerError::SemanticError(
                                "Logical NOT requires boolean operand".to_string(),
                            ));
                        }
                        Ok(Type::Bool)
                    }
                    UnaryOp::Neg => {
                        if ty != Type::Uint {
                            return Err(CompilerError::SemanticError(
                                "Negation requires uint operand".to_string(),
                            ));
                        }
                        Ok(Type::Uint)
                    }
                }
            }
            Expression::Call { name: _, args: _ } => {
                // For now, assume all function calls return uint
                // In a full implementation, we'd look up the function signature
                Ok(Type::Uint)
            }
            Expression::Index { expr, .. } => {
                let ty = self.type_of_expression(expr)?;
                match ty {
                    Type::Mapping(_, value_ty) => Ok(*value_ty),
                    _ => Err(CompilerError::SemanticError(
                        "Index operation requires mapping".to_string(),
                    )),
                }
            }
            Expression::MemberAccess { expr, member } => {
                // Handle common member access patterns
                if let Expression::Identifier(obj) = &**expr {
                    match (obj.as_str(), member.as_str()) {
                        ("msg", "sender") => Ok(Type::Address),
                        ("msg", "value") => Ok(Type::Uint),
                        ("block", "number") => Ok(Type::Uint),
                        ("block", "timestamp") => Ok(Type::Uint),
                        _ => Ok(Type::Uint),  // Default to Uint for unknown members
                    }
                } else {
                    Ok(Type::Uint)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    #[test]
    fn test_semantic_analysis() {
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
        let result = analyze(&contract);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_mismatch() {
        let source = r#"
            contract Test {
                storage {
                    value: uint;
                }
                
                function set(x: uint) -> bool {
                    value = true;
                    return true;
                }
            }
        "#;
        
        let tokens = tokenize(source).unwrap();
        let contract = parse(tokens).unwrap();
        let result = analyze(&contract);
        
        assert!(result.is_err());
    }
}
