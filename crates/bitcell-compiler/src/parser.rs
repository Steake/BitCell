//! Parser for BCL

use crate::ast::*;
use crate::lexer::Token;
use crate::{CompilerError, Result};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }
    
    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }
    
    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }
    
    fn expect(&mut self, token: Token) -> Result<()> {
        if self.current() == &token {
            self.advance();
            Ok(())
        } else {
            Err(CompilerError::ParserError(format!(
                "Expected {:?}, found {:?}",
                token,
                self.current()
            )))
        }
    }
    
    fn parse_contract(&mut self) -> Result<Contract> {
        self.expect(Token::Contract)?;
        
        let name = if let Token::Identifier(n) = self.current() {
            let name = n.clone();
            self.advance();
            name
        } else {
            return Err(CompilerError::ParserError("Expected contract name".to_string()));
        };
        
        self.expect(Token::LBrace)?;
        
        let mut storage = Vec::new();
        let mut functions = Vec::new();
        
        while self.current() != &Token::RBrace && self.current() != &Token::Eof {
            match self.current() {
                Token::Storage => {
                    self.advance();
                    storage = self.parse_storage()?;
                }
                Token::Function => {
                    functions.push(self.parse_function()?);
                }
                _ => {
                    return Err(CompilerError::ParserError(format!(
                        "Unexpected token in contract: {:?}",
                        self.current()
                    )));
                }
            }
        }
        
        self.expect(Token::RBrace)?;
        
        Ok(Contract {
            name,
            storage,
            functions,
        })
    }
    
    fn parse_storage(&mut self) -> Result<Vec<StorageDecl>> {
        self.expect(Token::LBrace)?;
        
        let mut decls = Vec::new();
        
        while self.current() != &Token::RBrace && self.current() != &Token::Eof {
            let name = if let Token::Identifier(n) = self.current() {
                let name = n.clone();
                self.advance();
                name
            } else {
                return Err(CompilerError::ParserError("Expected storage variable name".to_string()));
            };
            
            self.expect(Token::Colon)?;
            let ty = self.parse_type()?;
            self.expect(Token::Semicolon)?;
            
            decls.push(StorageDecl { name, ty });
        }
        
        self.expect(Token::RBrace)?;
        Ok(decls)
    }
    
    fn parse_type(&mut self) -> Result<Type> {
        match self.current() {
            Token::Uint => {
                self.advance();
                Ok(Type::Uint)
            }
            Token::Bool => {
                self.advance();
                Ok(Type::Bool)
            }
            Token::Address => {
                self.advance();
                Ok(Type::Address)
            }
            Token::Mapping => {
                self.advance();
                self.expect(Token::LParen)?;
                let key_type = self.parse_type()?;
                // Accept both => and -> for mapping syntax
                if self.current() == &Token::FatArrow {
                    self.advance();
                } else {
                    self.expect(Token::Arrow)?;
                }
                let value_type = self.parse_type()?;
                self.expect(Token::RParen)?;
                Ok(Type::Mapping(Box::new(key_type), Box::new(value_type)))
            }
            _ => Err(CompilerError::ParserError(format!(
                "Expected type, found {:?}",
                self.current()
            ))),
        }
    }
    
    fn parse_function(&mut self) -> Result<Function> {
        self.expect(Token::Function)?;
        
        let name = if let Token::Identifier(n) = self.current() {
            let name = n.clone();
            self.advance();
            name
        } else {
            return Err(CompilerError::ParserError("Expected function name".to_string()));
        };
        
        self.expect(Token::LParen)?;
        let params = self.parse_parameters()?;
        self.expect(Token::RParen)?;
        
        let return_type = if self.current() == &Token::Arrow {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.expect(Token::LBrace)?;
        let body = self.parse_statements()?;
        self.expect(Token::RBrace)?;
        
        Ok(Function {
            name,
            params,
            return_type,
            body,
        })
    }
    
    fn parse_parameters(&mut self) -> Result<Vec<Parameter>> {
        let mut params = Vec::new();
        
        while self.current() != &Token::RParen && self.current() != &Token::Eof {
            let name = if let Token::Identifier(n) = self.current() {
                let name = n.clone();
                self.advance();
                name
            } else {
                return Err(CompilerError::ParserError("Expected parameter name".to_string()));
            };
            
            self.expect(Token::Colon)?;
            let ty = self.parse_type()?;
            
            params.push(Parameter { name, ty });
            
            if self.current() == &Token::Comma {
                self.advance();
            }
        }
        
        Ok(params)
    }
    
    fn parse_statements(&mut self) -> Result<Vec<Statement>> {
        let mut stmts = Vec::new();
        
        while self.current() != &Token::RBrace && self.current() != &Token::Eof {
            stmts.push(self.parse_statement()?);
        }
        
        Ok(stmts)
    }
    
    fn parse_statement(&mut self) -> Result<Statement> {
        match self.current() {
            Token::Let => {
                self.advance();
                let name = if let Token::Identifier(n) = self.current() {
                    let name = n.clone();
                    self.advance();
                    name
                } else {
                    return Err(CompilerError::ParserError("Expected variable name".to_string()));
                };
                
                self.expect(Token::Assign)?;
                let value = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                
                Ok(Statement::Let { name, value })
            }
            Token::If => {
                self.advance();
                self.expect(Token::LParen)?;
                let condition = self.parse_expression()?;
                self.expect(Token::RParen)?;
                self.expect(Token::LBrace)?;
                let then_block = self.parse_statements()?;
                self.expect(Token::RBrace)?;
                
                let else_block = if self.current() == &Token::Else {
                    self.advance();
                    self.expect(Token::LBrace)?;
                    let block = self.parse_statements()?;
                    self.expect(Token::RBrace)?;
                    Some(block)
                } else {
                    None
                };
                
                Ok(Statement::If {
                    condition,
                    then_block,
                    else_block,
                })
            }
            Token::Return => {
                self.advance();
                let value = if self.current() == &Token::Semicolon {
                    None
                } else {
                    Some(self.parse_expression()?)
                };
                self.expect(Token::Semicolon)?;
                Ok(Statement::Return { value })
            }
            Token::Require => {
                self.advance();
                self.expect(Token::LParen)?;
                let condition = self.parse_expression()?;
                self.expect(Token::Comma)?;
                let message = if let Token::String(s) = self.current() {
                    let msg = s.clone();
                    self.advance();
                    msg
                } else {
                    return Err(CompilerError::ParserError("Expected error message".to_string()));
                };
                self.expect(Token::RParen)?;
                self.expect(Token::Semicolon)?;
                
                Ok(Statement::Require { condition, message })
            }
            Token::Identifier(_) => {
                let expr = self.parse_expression()?;
                
                if self.current() == &Token::Assign {
                    self.advance();
                    let value = self.parse_expression()?;
                    self.expect(Token::Semicolon)?;
                    Ok(Statement::Assign { target: expr, value })
                } else {
                    self.expect(Token::Semicolon)?;
                    Ok(Statement::Expression(expr))
                }
            }
            _ => Err(CompilerError::ParserError(format!(
                "Unexpected token in statement: {:?}",
                self.current()
            ))),
        }
    }
    
    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_logical_or()
    }
    
    fn parse_logical_or(&mut self) -> Result<Expression> {
        let mut left = self.parse_logical_and()?;
        
        while self.current() == &Token::Or {
            self.advance();
            let right = self.parse_logical_and()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_logical_and(&mut self) -> Result<Expression> {
        let mut left = self.parse_comparison()?;
        
        while self.current() == &Token::And {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut left = self.parse_additive()?;
        
        loop {
            let op = match self.current() {
                Token::Eq => BinaryOp::Eq,
                Token::Ne => BinaryOp::Ne,
                Token::Lt => BinaryOp::Lt,
                Token::Le => BinaryOp::Le,
                Token::Gt => BinaryOp::Gt,
                Token::Ge => BinaryOp::Ge,
                _ => break,
            };
            
            self.advance();
            let right = self.parse_additive()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_additive(&mut self) -> Result<Expression> {
        let mut left = self.parse_multiplicative()?;
        
        loop {
            let op = match self.current() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_multiplicative(&mut self) -> Result<Expression> {
        let mut left = self.parse_unary()?;
        
        loop {
            let op = match self.current() {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Mod,
                _ => break,
            };
            
            self.advance();
            let right = self.parse_unary()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_unary(&mut self) -> Result<Expression> {
        match self.current() {
            Token::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_postfix(),
        }
    }
    
    fn parse_postfix(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;
        
        loop {
            match self.current() {
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(Token::RBracket)?;
                    expr = Expression::Index {
                        expr: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Token::Dot => {
                    self.advance();
                    if let Token::Identifier(member) = self.current() {
                        let member = member.clone();
                        self.advance();
                        expr = Expression::MemberAccess {
                            expr: Box::new(expr),
                            member,
                        };
                    } else {
                        return Err(CompilerError::ParserError(
                            "Expected identifier after '.'".to_string(),
                        ));
                    }
                }
                Token::LParen if matches!(expr, Expression::Identifier(_)) => {
                    if let Expression::Identifier(name) = expr {
                        self.advance();
                        let args = self.parse_arguments()?;
                        self.expect(Token::RParen)?;
                        expr = Expression::Call { name, args };
                    }
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }
    
    fn parse_arguments(&mut self) -> Result<Vec<Expression>> {
        let mut args = Vec::new();
        
        while self.current() != &Token::RParen && self.current() != &Token::Eof {
            args.push(self.parse_expression()?);
            
            if self.current() == &Token::Comma {
                self.advance();
            }
        }
        
        Ok(args)
    }
    
    fn parse_primary(&mut self) -> Result<Expression> {
        match self.current().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(Expression::Literal(Literal::Uint(n)))
            }
            Token::True => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(true)))
            }
            Token::False => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(false)))
            }
            Token::Identifier(name) => {
                self.advance();
                Ok(Expression::Identifier(name))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            _ => Err(CompilerError::ParserError(format!(
                "Unexpected token in expression: {:?}",
                self.current()
            ))),
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Contract> {
    let mut parser = Parser::new(tokens);
    parser.parse_contract()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn test_parse_simple_contract() {
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
        
        assert_eq!(contract.name, "Test");
        assert_eq!(contract.storage.len(), 1);
        assert_eq!(contract.functions.len(), 1);
    }
}
