#![allow(warnings)]

pub mod CLI {
    pub mod cli;
}

pub mod F {
    pub mod ast;
    pub mod lexer;
    pub mod parser;
}

pub mod B {
    pub mod codegen;
}

use std::env;
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let _ = CLI::cli::parse(args);
}
