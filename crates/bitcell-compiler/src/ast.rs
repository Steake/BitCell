//! Abstract Syntax Tree definitions for BCL

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contract {
    pub name: String,
    pub storage: Vec<StorageDecl>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageDecl {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Uint,
    Bool,
    Address,
    Mapping(Box<Type>, Box<Type>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Let {
        name: String,
        value: Expression,
    },
    Assign {
        target: Expression,
        value: Expression,
    },
    If {
        condition: Expression,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
    Return {
        value: Option<Expression>,
    },
    Require {
        condition: Expression,
        message: String,
    },
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
    },
    Call {
        name: String,
        args: Vec<Expression>,
    },
    Index {
        expr: Box<Expression>,
        index: Box<Expression>,
    },
    MemberAccess {
        expr: Box<Expression>,
        member: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Uint(u64),
    Bool(bool),
    Address(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
    Neg,
}
