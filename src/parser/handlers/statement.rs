use crate::{lexer::Token, parser::{ast::{Expr, Stmt, Type}, pratt::BindingPower, Parser}};

use super::{expression::*, types::*};

pub fn parse_stmt(parser: &mut Parser) -> Stmt {
    if let Some(handler) = parser.get_stmt(parser.current()) {
        return handler(parser)
    }

    let expr = parse_expression(parser, BindingPower::Default);
    
    if let Expr::StmtHack(s) = expr {
        return s;
    }

    parser.expect(Token::Semicolon);

    Stmt::Expression(Box::new(expr))
}

pub fn parse_function_declaration(parser: &mut Parser, name: String) -> Stmt {
    parser.expect(Token::OpenParen);
    

    let mut arguments = vec![];

    while parser.has_tokens() && !parser.is_current_kind(Token::CloseParen) {
        match parser.advance() {
            Token::Reference => { 
                let Token::Identifier(ident) = parser.expect(Token::Identifier(String::new())) else {
                    panic!("Only put a '&' symbol behind 'this' to take it as a reference, otherwise use it before type");
                };
                if ident != "this" {
                    panic!("Only put a '&' symbol behind 'this' to take it as a reference, otherwise use it before type");
                }
                arguments.push(Stmt::Field { name: "this".into(), _type: Box::new(Type::Ref(Box::new(Type::Symbol("this".into())))) })
            },
            Token::Identifier(name) => {
                if name == "this" {
                    arguments.push(Stmt::Field { name: "this".into(), _type: Box::new(Type::Symbol("this".into()))});
                } else {
                    arguments.push(Stmt::Field { name: name.into(), _type: Box::new(parse_type(parser, BindingPower::Default)) });
                }
            },
            _ => panic!("Expected identifier or 'this' in fields for function {}", name),
        }

        if !parser.is_current_kind(Token::CloseParen) {
            parser.expect(Token::Comma);
        }
    }

    parser.expect(Token::CloseParen);

    let return_type = if parser.is_current_kind(Token::Identifier(String::new())) {
        parse_type(parser, BindingPower::Default)
    } else {
        Type::Symbol("void".into())
    };

    let mut body = vec![];

    parser.expect(Token::OpenCurly);
    while parser.has_tokens() && !parser.is_current_kind(Token::CloseCurly) {
        body.push(parse_stmt(parser));
    }

    parser.expect(Token::CloseCurly);

    println!("function declaration\nname: {}\nreturn type: {:?}\narguments: {:?}\nbody: {:?}", name, return_type, arguments, body);

    Stmt::FunctionDeclaration {
        name,
        return_type: Box::new(return_type),
        arguments,
        body: Box::new(Stmt::Block(body))
    }
}

pub fn parse_include(parser: &mut Parser) -> Stmt {
    parser.advance();
    let Token::String(val) = parser.expect(Token::String(String::new())) else {
        panic!("Expected string for include path");
    };

    Stmt::Include { path: val.into() }
}