use crate::{lexer::Token, parser::{ast::{Expr, Stmt}, pratt::BindingPower, Parser}};

use super::statement::*;

pub fn parse_expression(parser: &mut Parser, bp: BindingPower) -> Expr {
    let nud_fn = parser.get_nud(parser.current()).unwrap_or_else(|| panic!("NUD FUNCTION DOESNT EXIST FOR TOKEN {:?}", parser.current()));
    let mut left = nud_fn(parser);
    loop {
        if let Some(bp_) = parser.get_bp(parser.current()) {
            if *bp_ as usize <= bp as usize {
                break;
            }
            let led_fn = parser.get_led(parser.current()).unwrap_or_else(|| panic!("LED FUNCTION DOESNT EXIST FOR TOKEN {:?}", parser.current()));
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

pub fn parse_bool_expression(parser: &mut Parser) -> Expr {
    let v = Expr::Bool(parser.is_current_kind(Token::True));
    parser.advance();
    v
}

pub fn parse_arrow_expression(parser: &mut Parser, left: Expr, _bp: BindingPower) -> Expr {
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
        Token::Extern => {
            parser.advance();
            Expr::StmtHack(parse_extern(parser, symbol))
        }
        _ => panic!("Expected high level declaration with arrow but got {:?}", parser.current()),
    }
}

pub fn parse_grouped_expression(parser: &mut Parser) -> Expr {
    parser.advance();

    let expr = parse_expression(parser, BindingPower::Default);
    parser.expect(Token::CloseParen);

    expr
}

pub fn parse_assignment_expression(parser: &mut Parser, left: Expr, _bp: BindingPower) -> Expr {
    parser.advance();

    Expr::Assignment { assignee: Box::new(left), right: Box::new(parse_expression(parser, BindingPower::Default)) }
}

pub fn parse_prefix_expression(parser: &mut Parser) -> Expr {
    let op = parser.advance().clone();

    let nud_fn = parser.get_nud(parser.current()).unwrap_or_else(|| panic!("NUD FUNCTION DOESNT EXIST FOR TOKEN {:?}", parser.current()));
    let right = nud_fn(parser);

    Expr::Prefix { op, right: Box::new(right) }
}

pub fn parse_call_expression(parser: &mut Parser, left: Expr, bp: BindingPower) -> Expr {
    let Expr::Symbol(callee) = left else {
        panic!("Expected symbol on the left of a function call but got {:?}", left);
    }; 
    parser.advance();

    let mut arguments = vec![];

    while parser.has_tokens() && !parser.is_current_kind(Token::CloseParen) {
        arguments.push(parse_expression(parser, BindingPower::Default));

        if !parser.is_current_kind(Token::CloseParen) {
            parser.expect(Token::Comma);
        }
    }
    parser.expect(Token::CloseParen);

    Expr::Call { name: callee, args: arguments }
}

pub fn parse_op_equals_expression(parser: &mut Parser, left: Expr, bp: BindingPower) -> Expr {
    let Expr::Symbol(lhs) = left.clone() else {
        panic!("Expected symbol on LHS of 'OPERATOR=' expression, got {:?}", left);
    };

    let real_op = match parser.current() {
        Token::PlusEquals => Token::Plus,
        Token::MinusEquals => Token::Minus,
        _ => panic!("Invalid 'OPERATOR=' expression {:?}", left),
    };

    parser.advance();

    Expr::Assignment { 
        assignee: Box::new(left.clone()),
        right: Box::new(Expr::Binary {
            left: Box::new(left),
            op: real_op,
            right: Box::new(parse_expression(parser, BindingPower::Default)),
        }),
    }
}

pub fn parse_access_expression(parser: &mut Parser, left: Expr, bp: BindingPower) -> Expr {
    parser.advance();

    let Token::Identifier(right) = parser.advance() else {
        panic!("Expected identifier on rhs of access expression");
    };
    Expr::Access { lhs: Box::new(left), rhs: Box::new(Expr::Symbol(right.clone())) }
}

pub fn parse_struct_create_expression(parser: &mut Parser) -> Expr {
    parser.advance();

    let Token::Identifier(name) = parser.advance() else {
        panic!("Expected identifier for stuct initialization");
    };

    let newname = name.clone();

    let mut fields = vec![];
    parser.expect(Token::OpenCurly);
    while parser.has_tokens() && !parser.is_current_kind(Token::CloseCurly) {
        let Token::Identifier(name) = parser.advance() else {
            panic!("Expected identifier in struct initialization, but got {:?}", parser.current());
        };

        let nname = name.clone();

        parser.expect(Token::Colon);
        fields.push(Stmt::StructInitField { name: nname, value: Box::new(parse_expression(parser, BindingPower::Default)) });

        if !parser.is_current_kind(Token::CloseCurly) {
            parser.expect(Token::Comma);
        }
    }

    parser.expect(Token::CloseCurly);

    Expr::StructInitialize { name: newname, fields: fields }
}