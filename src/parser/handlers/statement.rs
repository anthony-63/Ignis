use crate::{lexer::Token, parser::{ast::{Expr, Stmt}, pratt::BindingPower, Parser}};

use super::expression::parse_expression;

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