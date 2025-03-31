module parser.expr;

import std.conv;
import std.stdio;
import std.format;

import ast.ast;
import lexer.tokens;
import parser.parser;
import parser.lookups;
import ast.expressions;

Expr parse_expr(Parser parser, BindingPower bp) {
    auto kind = parser.current().kind;
    assert(kind in nud_lu, format("NUD function not existant for (%s)", kind));

    auto nud_fn = nud_lu[kind];

    auto left = nud_fn(parser);
    while(bp_lu[parser.current().kind] > bp) {
        kind = parser.current().kind;
        assert(kind in led_lu, format("LED function not existant for (%s)", kind));
        
        auto led_fn = led_lu[kind];
        left = led_fn(parser, left, bp);
    }

    return left;
}

Expr parse_primary_expr(Parser parser) {
    writeln("primary: ", parser.current());
    switch(parser.current().kind) {
        case TokenKind.INT: return new NumberExpr(to!int(parser.advance().value)); break;
        case TokenKind.DECIMEL: return new NumberExpr(to!float(parser.advance().value)); break;
        case TokenKind.STRING: return new StringExpr(parser.advance().value); break;
        case TokenKind.IDENT: return new SymbolExpr(parser.advance().value); break;
        default: assert(false, format("Failed to parse primary expression (%s)", parser.current()));
    }
}

Expr parse_binary_expr(Parser parser, Expr left, BindingPower bp) {
    writeln("binary: ", parser.current());
    auto op = parser.advance();
    auto right = parse_expr(parser, bp);

    return new BinExpr(left, op, right);
}