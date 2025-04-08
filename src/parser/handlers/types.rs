use crate::{lexer::Token, parser::{ast::Type, pratt::BindingPower, Parser}};

pub fn parse_type(parser: &mut Parser, bp: BindingPower) -> Type {
    let nud_fn = parser.get_type_nud(parser.current()).expect(&format!("NUD TYPE FUNCTION DOESNT EXIST FOR TOKEN {:?}", parser.current()));
    let mut left = nud_fn(parser);
    loop {
        if let Some(bp_) = parser.get_bp(parser.current()) {
            if *bp_ as usize <= bp as usize {
                break;
            }
            let led_fn = parser.get_type_led(parser.current()).expect(&format!("LED TYPE FUNCTION DOESNT EXIST FOR TOKEN {:?}", parser.current()));
            left = led_fn(parser, left, bp);
            
        } else {
            break;
        }
    }
    left
}

pub fn parse_symbol_type(parser: &mut Parser) -> Type {
    let Token::Identifier(ident) = parser.expect(Token::Identifier(String::new())) else {
        panic!("Expected identifier for symbol type, got {:?}", parser.current())
    };

    Type::Symbol(ident.into())
}

pub fn parse_array_type(parser: &mut Parser) -> Type {
    parser.advance();
    parser.expect(Token::CloseBracket);

    Type::Array(Box::new(parse_type(parser, BindingPower::Default)))
}


pub fn parse_ref_type(parser: &mut Parser) -> Type {
    parser.advance();
    Type::Ref(Box::new(parse_type(parser, BindingPower::Default)))
}
