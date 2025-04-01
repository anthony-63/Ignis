module parser.lookups;

import std.conv;
import ast.ast;
import ast.expressions;

import lexer.tokens;
import parser.parser;

enum BindingPower {
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

alias StmtHandler = Stmt function(Parser parser);
alias NUDHandler = Expr function(Parser parser);
alias LEDHandler = Expr function(Parser parser, Expr left, BindingPower bp);

alias StmtLookup = StmtHandler[TokenKind];
alias NUDLookup = NUDHandler[TokenKind];
alias LEDLookup = LEDHandler[TokenKind];
alias BPLookup = BindingPower[TokenKind];

BPLookup bp_lu = null;
NUDLookup nud_lu = null;
LEDLookup led_lu = null;
StmtLookup stmt_lu = null;

void led(TokenKind kind, BindingPower bp, LEDHandler handler) {
    bp_lu[kind] = bp;
    led_lu[kind] = handler;
}

void nud(TokenKind kind, BindingPower bp, NUDHandler handler) {
    bp_lu[kind] = bp;
    nud_lu[kind] = handler;
}

void stmt(TokenKind kind, StmtHandler handler) {
    bp_lu[kind] = BindingPower.Default;
    stmt_lu[kind] = handler;
}