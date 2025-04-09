
use crate::lexer::Token;
use super::{lookup::LookupTable, Parser};

#[derive(Clone, Copy)]
#[repr(usize)]
pub enum BindingPower {
    Default = 0,
    Comma,
    Assignment,
    Logical,
    Relational,
    Additive,
    Multiplicative,
    Unary,
    Call,
    Member,
    Primary,
}

pub type StmtHandler<S> = fn(parser: &mut Parser) -> S;
pub type NudHandler<E> = fn(parser: &mut Parser) -> E;
pub type LedHandler<E> = fn(parser: &mut Parser, left: E, bp: BindingPower) -> E;

type StmtLookup<S> = LookupTable<StmtHandler<S>>;
type NudLookup<E> = LookupTable<NudHandler<E>>;
type LedLookup<E> = LookupTable<LedHandler<E>>;
type BpLookup = LookupTable<BindingPower>;

pub struct PrattLookups<E, S> {
    bp_lu: BpLookup,
    nud_lu: NudLookup<E>,
    led_lu: LedLookup<E>,
    stmt_lu: StmtLookup<S>,
}

impl<E, S> PrattLookups<E, S> {
    pub fn new() -> Self {
        Self {
            bp_lu: BpLookup::new(),
            nud_lu: NudLookup::new(),
            led_lu: LedLookup::new(),
            stmt_lu: StmtLookup::new(),
        }
    }

    pub fn led(&mut self, token: Token, bp: BindingPower, handler: LedHandler<E>) {
        self.led_lu.insert(token.clone(), handler);
        self.bp_lu.insert(token.clone(), bp);
    }

    pub fn nud(&mut self, token: Token, handler: NudHandler<E>) {
        self.nud_lu.insert(token, handler);
    }

    pub fn stmt(&mut self, token: Token, handler: StmtHandler<S>) {
        self.stmt_lu.insert(token.clone(), handler);
        self.bp_lu.insert(token.clone(), BindingPower::Default);
    }

    pub fn get_stmt(&self, token: &Token) -> Option<&StmtHandler<S>> {
        self.stmt_lu.get(token)
    }

    pub fn get_nud(&self, token: &Token) -> Option<&NudHandler<E>> {
        self.nud_lu.get(token)
    }

    pub fn get_led(&self, token: &Token) -> Option<&LedHandler<E>> {
        self.led_lu.get(token)
    }

    pub fn get_bp(&self, token: &Token) -> Option<&BindingPower> {
        self.bp_lu.get(token)
    }
}