mod lexer;
mod parser;

use std::env::args;

use lexer::Token;
use logos::Logos;
use parser::Parser;

fn main() {
    let args = args().skip(1).collect::<Vec<_>>();

    if args.len() != 1 {
        eprintln!("Usage: ignis <input>");
        return;
    }

    let mut iter = args.iter();

    let ouput = String::from("a.out");

    let source = std::fs::read_to_string(iter.next().unwrap()).expect("Failed to find file");
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

    println!("{:?}", tokens);

    let ast = Parser::parse(tokens);
    println!("{:#?}", ast);
}
