pub mod lexer {}
use crate::F::lexer::TokenKind;

#[derive(Debug)]
pub struct Program {
    pub module_name: String,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    I32,
    I8,
    I64,
    I128,
    F32,
    F64,
    Str,
    I1,
}

#[derive(Debug)]
pub enum stmt {
    Printf {
        value: Expression,
        line: i32,
    },
    Block(Vec<stmt>),
    Ret {
        value: String,
        line: i32,
    },
    Var {
        heap: bool,
        is_mut: bool,
        name: String,
        r#type: DataType,
        value: String,
        line: i32,
    },
}

#[derive(Debug)]
pub enum Expression {
    Id(String),
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    CharLit(char),
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub is_option: bool,
    pub return_type: DataType,
    pub body: Option<stmt>,
    pub heap: bool,
    pub damtic: bool,
}
