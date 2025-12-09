//! Lexical analyzer for BCL

use crate::{CompilerError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Contract,
    Storage,
    Function,
    Let,
    If,
    Else,
    Return,
    Require,
    Mapping,
    
    // Types
    Uint,
    Bool,
    Address,
    
    // Literals
    Number(u64),
    True,
    False,
    String(String),
    Identifier(String),
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    Assign,
    Arrow,
    FatArrow,  // =>
    
    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semicolon,
    Dot,
    
    // Special
    Eof,
}

pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    let mut line = 1;
    let mut col = 1;
    
    while let Some(&ch) = chars.peek() {
        match ch {
            // Whitespace
            ' ' | '\t' | '\r' => {
                chars.next();
                col += 1;
            }
            '\n' => {
                chars.next();
                line += 1;
                col = 1;
            }
            
            // Comments
            '/' if chars.clone().nth(1) == Some('/') => {
                chars.next();
                chars.next();
                while let Some(&ch) = chars.peek() {
                    chars.next();
                    if ch == '\n' {
                        line += 1;
                        col = 1;
                        break;
                    }
                }
            }
            
            // Single-character tokens
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
                col += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
                col += 1;
            }
            '{' => {
                tokens.push(Token::LBrace);
                chars.next();
                col += 1;
            }
            '}' => {
                tokens.push(Token::RBrace);
                chars.next();
                col += 1;
            }
            '[' => {
                tokens.push(Token::LBracket);
                chars.next();
                col += 1;
            }
            ']' => {
                tokens.push(Token::RBracket);
                chars.next();
                col += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
                col += 1;
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
                col += 1;
            }
            ';' => {
                tokens.push(Token::Semicolon);
                chars.next();
                col += 1;
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
                col += 1;
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
                col += 1;
            }
            '%' => {
                tokens.push(Token::Percent);
                chars.next();
                col += 1;
            }
            '.' => {
                tokens.push(Token::Dot);
                chars.next();
                col += 1;
            }
            
            // Multi-character operators
            '-' => {
                chars.next();
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token::Arrow);
                    col += 2;
                } else {
                    tokens.push(Token::Minus);
                    col += 1;
                }
            }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Eq);
                    col += 2;
                } else if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token::FatArrow);
                    col += 2;
                } else {
                    tokens.push(Token::Assign);
                    col += 1;
                }
            }
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Ne);
                    col += 2;
                } else {
                    tokens.push(Token::Not);
                    col += 1;
                }
            }
            '<' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Le);
                    col += 2;
                } else {
                    tokens.push(Token::Lt);
                    col += 1;
                }
            }
            '>' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Ge);
                    col += 2;
                } else {
                    tokens.push(Token::Gt);
                    col += 1;
                }
            }
            '&' => {
                chars.next();
                if chars.peek() == Some(&'&') {
                    chars.next();
                    tokens.push(Token::And);
                    col += 2;
                } else {
                    return Err(CompilerError::LexerError {
                        line,
                        col,
                        message: "Expected '&&'".to_string(),
                    });
                }
            }
            '|' => {
                chars.next();
                if chars.peek() == Some(&'|') {
                    chars.next();
                    tokens.push(Token::Or);
                    col += 2;
                } else {
                    return Err(CompilerError::LexerError {
                        line,
                        col,
                        message: "Expected '||'".to_string(),
                    });
                }
            }
            
            // String literals
            '"' => {
                chars.next();
                col += 1;
                let mut string = String::new();
                while let Some(&ch) = chars.peek() {
                    chars.next();
                    col += 1;
                    if ch == '"' {
                        break;
                    }
                    string.push(ch);
                }
                tokens.push(Token::String(string));
            }
            
            // Numbers
            '0'..='9' => {
                let mut num = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() {
                        num.push(ch);
                        chars.next();
                        col += 1;
                    } else {
                        break;
                    }
                }
                let value = num.parse::<u64>().map_err(|_| CompilerError::LexerError {
                    line,
                    col,
                    message: format!("Invalid number: {}", num),
                })?;
                tokens.push(Token::Number(value));
            }
            
            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        chars.next();
                        col += 1;
                    } else {
                        break;
                    }
                }
                
                let token = match ident.as_str() {
                    "contract" => Token::Contract,
                    "storage" => Token::Storage,
                    "function" => Token::Function,
                    "let" => Token::Let,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "return" => Token::Return,
                    "require" => Token::Require,
                    "mapping" => Token::Mapping,
                    "uint" => Token::Uint,
                    "bool" => Token::Bool,
                    "address" => Token::Address,
                    "true" => Token::True,
                    "false" => Token::False,
                    _ => Token::Identifier(ident),
                };
                tokens.push(token);
            }
            
            _ => {
                return Err(CompilerError::LexerError {
                    line,
                    col,
                    message: format!("Unexpected character: {}", ch),
                });
            }
        }
    }
    
    tokens.push(Token::Eof);
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_keywords() {
        let tokens = tokenize("contract function storage").unwrap();
        assert_eq!(tokens[0], Token::Contract);
        assert_eq!(tokens[1], Token::Function);
        assert_eq!(tokens[2], Token::Storage);
    }

    #[test]
    fn test_tokenize_operators() {
        let tokens = tokenize("+ - * == != <= >= && ||").unwrap();
        assert_eq!(tokens[0], Token::Plus);
        assert_eq!(tokens[1], Token::Minus);
        assert_eq!(tokens[2], Token::Star);
        assert_eq!(tokens[3], Token::Eq);
        assert_eq!(tokens[4], Token::Ne);
        assert_eq!(tokens[5], Token::Le);
        assert_eq!(tokens[6], Token::Ge);
        assert_eq!(tokens[7], Token::And);
        assert_eq!(tokens[8], Token::Or);
    }

    #[test]
    fn test_tokenize_literals() {
        let tokens = tokenize(r#"42 true false "hello""#).unwrap();
        assert_eq!(tokens[0], Token::Number(42));
        assert_eq!(tokens[1], Token::True);
        assert_eq!(tokens[2], Token::False);
        assert_eq!(tokens[3], Token::String("hello".to_string()));
    }
}
