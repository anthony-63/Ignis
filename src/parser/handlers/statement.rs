use std::env::var;

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

pub fn parse_if(parser: &mut Parser) -> Stmt {
    parser.advance();

    let condition = parse_expression(parser, BindingPower::Default);
    let mut body = vec![];
    let mut _else = None;

    parser.expect(Token::OpenCurly);
    while parser.has_tokens() && !parser.is_current_kind(Token::CloseCurly) {
        body.push(parse_stmt(parser));
    }

    parser.expect(Token::CloseCurly);

    if parser.is_current_kind(Token::Else) {
        parser.advance();
        if parser.is_current_kind(Token::If) {
            _else = Some(Box::new(parse_if(parser)));
        } else {
            let mut _else_body = vec![];
            parser.expect(Token::OpenCurly);
            while parser.has_tokens() && !parser.is_current_kind(Token::CloseCurly) {
                _else_body.push(parse_stmt(parser));
            }
        
            parser.expect(Token::CloseCurly);

            _else = Some(Box::new(Stmt::Block(_else_body)));
        }
    }

    Stmt::If { condition: Box::new(condition), body: Box::new(Stmt::Block(body)), _else }
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

    Stmt::FunctionDeclaration {
        name,
        return_type: Box::new(return_type),
        arguments,
        body: Box::new(Stmt::Block(body))
    }
}

pub fn parse_struct_declaration(parser: &mut Parser, name: String) -> Stmt {
    parser.expect(Token::OpenCurly);

    let mut fields = vec![];
    let mut functions = vec![];

    while parser.has_tokens() && !parser.is_current_kind(Token::CloseCurly) {
        let Token::Identifier(field_name) = parser.advance() else {
            panic!("Expected identifier for struct field name");
        };
        let name = field_name.clone();

        if parser.is_current_kind(Token::Arrow) {
            parser.advance();
            parser.expect(Token::Subroutine);

            functions.push(parse_function_declaration(parser, name));
            continue;
        }
        
        let _type = parse_type(parser, BindingPower::Default);

        fields.push(Stmt::Field { name: name, _type: Box::new(_type) });
        if !parser.is_current_kind(Token::CloseCurly) {
            parser.expect(Token::Comma);
        }
    }

    parser.advance();

    Stmt::StructDeclaration { name, fields, functions }
}

pub fn parse_var_decl(parser: &mut Parser) -> Stmt {
    let mutable = parser.is_current_kind(Token::Mut);
    parser.advance();

    let Token::Identifier(var_name) = parser.advance() else {
        panic!("Expected identifier for variable name, got {:?}", parser.last())
    };

    let name = var_name.clone();

    let mut explicit_type = None;
    if parser.is_current_kind(Token::Colon) {
        parser.advance();
        explicit_type = Some(Box::new(parse_type(parser, BindingPower::Default)));
    }

    parser.expect(Token::Assignment);
    let val = parse_expression(parser, BindingPower::Default);

    parser.expect(Token::Semicolon);

    Stmt::VariableDeclaration { name: name.into(), mutable, explicit_type: explicit_type, value: Box::new(val) }
}

pub fn parse_extern(parser: &mut Parser, name: String) -> Stmt {
    let mut symbol = name.clone();
    
    if parser.is_current_kind(Token::OpenBracket) {
        parser.advance();
        let Token::Identifier(s) = parser.advance() else {
            panic!("Expected identifier, example: [SYMBOL]");
        };
        symbol = s.into();
    }

    parser.expect(Token::OpenParen);

    let mut arguments = vec![];

    while parser.has_tokens() && !parser.is_current_kind(Token::CloseParen) {
        match parser.advance() {
            Token::Identifier(name) => arguments.push(Stmt::Field { name: name.into(), _type: Box::new(parse_type(parser, BindingPower::Default)) }),
            _ => panic!("Expected identifier in fields for extern function {}", name),
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

    parser.expect(Token::Semicolon);

    Stmt::Extern { name, symbol, return_type: Box::new(return_type), arguments }
}

pub fn parse_link_lib(parser: &mut Parser) -> Stmt {
    parser.advance();
    let Token::String(libname) = parser.advance() else {
        panic!("Expected string for linklib argbument")
    };

    Stmt::Link { library: libname.into(), _static: false }
}

pub fn parse_link_static(parser: &mut Parser) -> Stmt {
    parser.advance();
    let Token::String(libname) = parser.advance() else {
        panic!("Expected string for linkstatic argbument")
    };

    Stmt::Link { library: libname.into(), _static: true }
}

pub fn parse_return(parser: &mut Parser) -> Stmt {
    parser.advance();

    let val = parse_expression(parser, BindingPower::Default);
    parser.expect(Token::Semicolon);

    Stmt::Return { value: Box::new(val) }
}

pub fn parse_include(parser: &mut Parser) -> Stmt {
    parser.advance();
    let Token::String(val) = parser.expect(Token::String(String::new())) else {
        panic!("Expected string for include path");
    };

    Stmt::Include { path: val.into() }
}