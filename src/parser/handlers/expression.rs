use crate::{lexer::Token, parser::{ast::Expr, pratt::BindingPower, Parser}};

pub fn parse_expression(parser: &mut Parser, bp: BindingPower) -> Expr {
    if let Some(nud_fn) = parser.get_nud(parser.current()) {
        let mut left = nud_fn(parser);
        loop {
            if let Some(bp_) = parser.get_bp(parser.current()) {
                if *bp_ as usize <= bp as usize {
                    break;
                }

                if let Some(led_fn) = parser.get_led(parser.current()) {
                    left = led_fn(parser, left, bp);
                } else {
                    panic!("LED FUNCTION DOESNT EXIST FOR TOKEN {:?}", parser.current())
                }
            } else {
                break;
            }
        }
        left
    } else {
        panic!("NUD FUNCTION DOESNT EXIST FOR TOKEN {:?}", parser.current());
    }
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
        Token::String(v) => Expr::String(v.clone()),
        Token::Identifier(v) => Expr::Symbol(v.clone()),
        _ => panic!("Failed to parse primary expression {:?}", parser.current())
    };

    parser.advance();
    return v;
}

pub fn parse_struct_create_expression(parser: &mut Parser) -> Expr {
    Expr::Int(0)
}