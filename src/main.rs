mod lexer;
mod parser;
pub mod compiler;

use std::{env::args, path::Path};

use compiler::Compiler;
use lexer::Token;
use logos::Logos;
use parser::Parser;

fn main() {
    let args = args().skip(1).collect::<Vec<_>>();

    if args.len() != 2 {
        eprintln!("Usage: ignis <input>");
        return;
    }

    let mut iter = args.iter();

    
    let source = std::fs::read_to_string(iter.next().unwrap()).expect("Failed to find file");
    let output = iter.next().unwrap();

    let lexer = lexer::Token::lexer(&source);
    let mut tokens = vec![];

    for t in lexer {
        match t {
            Ok(tok) => tokens.push(tok),
            Err(e) => {
                if !e.is_empty() {
                    println!("Invalid token: {}", e);
                    return;
                }
            },
        }
    }

    tokens.push(Token::EOF);
    let ast = Parser::parse(tokens);
    println!("{:?}", ast);
    Compiler::compile(Path::new(output), ast, vec![], None, false);
}

fn mnt_to_string(bytes: &[i8]) -> String {
    unsafe { std::str::from_utf8_unchecked(std::mem::transmute(bytes)) }.to_string()
 }