use std::mem;

use ast::{Expr, Stmt, Type};
use handlers::{expression::*, statement::*, types::*};
use pratt::{BindingPower, LedHandler, NudHandler, PrattLookups, StmtHandler};

use crate::lexer::Token;

pub mod ast;
pub mod pratt;
pub mod lookup;
pub mod handlers;

pub struct Parser {
    lookup: PrattLookups<Expr, Stmt>,
    type_lookup: PrattLookups<Type, Stmt>,

    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut lu = PrattLookups::new();
        let mut tlu = PrattLookups::new();

        lu.led(Token::And, BindingPower::Logical, parse_binary_expression);
        lu.led(Token::Or, BindingPower::Logical, parse_binary_expression);
        lu.led(Token::Range, BindingPower::Logical, parse_binary_expression);

        lu.led(Token::Less, BindingPower::Relational, parse_binary_expression);
        lu.led(Token::LessOrEqual, BindingPower::Relational, parse_binary_expression);
        lu.led(Token::Greater, BindingPower::Relational, parse_binary_expression);
        lu.led(Token::GreaterOrEqual, BindingPower::Relational, parse_binary_expression);
        lu.led(Token::Equals, BindingPower::Relational, parse_binary_expression);
        lu.led(Token::NotEquals, BindingPower::Relational, parse_binary_expression);

        lu.led(Token::Plus, BindingPower::Additive, parse_binary_expression);
        lu.led(Token::Minus, BindingPower::Additive, parse_binary_expression);

        lu.led(Token::Assignment, BindingPower::Assignment, parse_assignment_expression);

        lu.led(Token::Multiply, BindingPower::Multiplicative, parse_binary_expression);
        lu.led(Token::Divide, BindingPower::Multiplicative, parse_binary_expression);
        lu.led(Token::Power, BindingPower::Multiplicative, parse_binary_expression);
        lu.led(Token::Mod, BindingPower::Multiplicative, parse_binary_expression);
        
        lu.led(Token::Arrow, BindingPower::Primary, parse_arrow_expression);

        lu.led(Token::Dot, BindingPower::Member, parse_access_expression);
        lu.led(Token::OpenParen, BindingPower::Call, parse_call_expression);

        lu.led(Token::PlusEquals, BindingPower::Call, parse_op_equals_expression);
        lu.led(Token::MinusEquals, BindingPower::Call, parse_op_equals_expression);

        lu.nud(Token::New, parse_struct_create_expression);

        lu.nud(Token::Integer(0), parse_primary_expression);
        lu.nud(Token::Decimel(0.), parse_primary_expression);
        lu.nud(Token::String(String::new()), parse_primary_expression);
        lu.nud(Token::Identifier(String::new()), parse_primary_expression);

        // lu.nud(Token::OpenBracket, parse_struct_create_expression);
        lu.nud(Token::OpenParen, parse_grouped_expression);

        lu.nud(Token::Minus, parse_prefix_expression);

        lu.stmt(Token::If, parse_if);
        lu.stmt(Token::Mut, parse_var_decl);
        lu.stmt(Token::Immut, parse_var_decl);
        lu.stmt(Token::Return, parse_return);
        lu.stmt(Token::LinkStatic, parse_link_static);
        lu.stmt(Token::LinkLib, parse_link_lib);
        lu.stmt(Token::Include, parse_include);

        tlu.nud(Token::Identifier(String::new()), parse_symbol_type);
        tlu.nud(Token::OpenBracket, parse_array_type);
        tlu.nud(Token::Reference, parse_ref_type);
        
        Self {
            lookup: lu,
            type_lookup: tlu,

            tokens,
            position: 0,
        }
    }

    pub fn parse(tokens: Vec<Token>) -> Stmt {
        let mut body = vec![];

        let mut parser = Parser::new(tokens);
        while parser.has_tokens() {
            body.push(parse_stmt(&mut parser));
        }
                
        Stmt::Block(body)
    }

    pub fn current(&self) -> &Token {
        &self.tokens[self.position]
    }

    pub fn last(&self) -> &Token {
        &self.tokens[self.position-1]
    }

    pub fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.position];
        self.position += 1;
        tok
    }

    pub fn has_tokens(&mut self) -> bool {
        self.position < self.tokens.len() && !self.is_current_kind(Token::EOF)
    }

    pub fn get_stmt(&self, token: &Token) -> Option<&StmtHandler<Stmt>> {
        self.lookup.get_stmt(token)
    }

    pub fn get_nud(&self, token: &Token) -> Option<&NudHandler<Expr>> {
        self.lookup.get_nud(token)
    }

    pub fn get_type_nud(&self, token: &Token) -> Option<&NudHandler<Type>> {
        self.type_lookup.get_nud(token)
    }

    pub fn get_led(&self, token: &Token) -> Option<&LedHandler<Expr>> {
        self.lookup.get_led(token)
    }

    pub fn get_type_led(&self, token: &Token) -> Option<&LedHandler<Type>> {
        self.type_lookup.get_led(token)
    }

    pub fn get_bp(&self, token: &Token) -> Option<&BindingPower> {
        self.lookup.get_bp(token)
    }

    pub fn expect(&mut self, token: Token) -> &Token {
        self.expect_error(token.clone(), &format!("Expected token {:?} but got {:?}", token, self.current()))
    }

    pub fn expect_error(&mut self, token: Token, error: &str) -> &Token {
        if self.is_current_kind(token) {
            return self.advance();
        }
        panic!("{}", error);
    }

    pub fn is_current_kind(&self, token: Token) -> bool {
        let tok = self.current().clone();
        is_kind(token, tok)
    }
}

pub fn is_kind<T>(lhs: T, rhs: T) -> bool {
    mem::discriminant(&lhs) == mem::discriminant(&rhs)
}