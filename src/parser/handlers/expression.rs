use crate::{lexer::Token, parser::{ast::Expr, pratt::BindingPower, Parser}};

use super::statement::*;

pub fn parse_expression(parser: &mut Parser, bp: BindingPower) -> Expr {
    let nud_fn = parser.get_nud(parser.current()).expect(&format!("NUD FUNCTION DOESNT EXIST FOR TOKEN {:?}", parser.current()));
    let mut left = nud_fn(parser);
    loop {
        if let Some(bp_) = parser.get_bp(parser.current()) {
            if *bp_ as usize <= bp as usize {
                break;
            }
            let led_fn = parser.get_led(parser.current()).expect(&format!("LED FUNCTION DOESNT EXIST FOR TOKEN {:?}", parser.current()));
            left = led_fn(parser, left, bp);
            
        } else {
            break;
        }
    }
    left
}

pub fn parse_binary_expression(parser: &mut Parser, left: Expr, bp: BindingPower) -> Expr {
    let op = parser.advance().clone();
    let right = parse_expression(parser, bp);

    Expr::Binary { left: Box::new(left), op, right: Box::new(right) }
}

pub fn parse_primary_expression(parser: &mut Parser) -> Expr {
    let v = match parser.current() {
        Token::Integer(v) => Expr::Int(*v),
        Token::Decimel(v) => Expr::Float(*v),
        Token::String(v) => Expr::String(v.into()),
        Token::Identifier(v) => Expr::Symbol(v.into()),
        _ => panic!("Failed to parse primary expression {:?}", parser.current())
    };



    parser.advance();
    v
}

pub fn parse_arrow_expression(parser: &mut Parser, left: Expr, bp: BindingPower) -> Expr {
    let Expr::Symbol(symbol) = left else {
        panic!("LHS of arrow MUST be an identifer, got {:?}", left);
    }; 

    parser.expect(Token::Arrow);

    match parser.current() {
        Token::Subroutine => {
            parser.advance();
            Expr::StmtHack(parse_function_declaration(parser, symbol))
        }
        Token::Struct => {
            parser.advance();
            Expr::StmtHack(parse_struct_declaration(parser, symbol))
        }
        // Token::Extern => Expr::StmtHack(parse_extern_statement(parser, symbol)),
        _ => panic!("Expected high level declaration with arrow but got {:?}", parser.current()),
    }
}

pub fn parse_grouped_expression(parser: &mut Parser) -> Expr {
    parser.advance();

    let expr = parse_expression(parser, BindingPower::Default);
    parser.expect(Token::CloseParen);

    expr
}

pub fn parse_prefix_expression(parser: &mut Parser) -> Expr {
    let op = parser.advance().clone();

    let nud_fn = parser.get_nud(parser.current()).expect(&format!("NUD FUNCTION DOESNT EXIST FOR TOKEN {:?}", parser.current()));
    let right = nud_fn(parser);

    Expr::Prefix { op, right: Box::new(right) }
}

pub fn parse_struct_create_expression(parser: &mut Parser) -> Expr {
    Expr::Int(0)
}