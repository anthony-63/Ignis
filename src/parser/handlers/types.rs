use crate::parser::{ast::Type, Parser};

pub fn parse_symbol_type(parser: &mut Parser) -> Type {
    Type::Symbol(String::from("HI"))
}