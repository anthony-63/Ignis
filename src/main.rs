mod lexer;
mod parser;

use std::env::args;

use logos::Logos;

fn main() {
    let mut args = args().skip(1).collect::<Vec<_>>();

    if args.len() != 1 {
        eprintln!("Usage: ignis <input>");
        return;
    }

    let mut iter = args.iter();

    let mut ouput = String::from("a.out");

    let source = std::fs::read_to_string(iter.next().unwrap()).expect("Failed to find file");
    let mut lexer = lexer::Token::lexer(&source);
    let mut tokens = vec![];

    while let Some(t) = lexer.next() {
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

    println!("{:?}", tokens);
}
