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
    
    while(parser.current().kind in bp_lu && bp_lu[parser.current().kind] > bp) {
        kind = parser.current().kind;
        assert(kind in led_lu, format("LED function not existant for (%s)", kind));
        
        auto led_fn = led_lu[kind];
        left = led_fn(parser, left, bp);
    }

    return left;
}

Expr parse_primary_expr(Parser parser) {
    switch(parser.current().kind) {
        case TokenKind.INT: return new IntExpr(to!int(parser.advance().value)); break;
        case TokenKind.DECIMEL: return new FloatExpr(to!float(parser.advance().value)); break;
        case TokenKind.STRING: return new StringExpr(parser.advance().value); break;
        case TokenKind.IDENT: return new SymbolExpr(parser.advance().value); break;
        default: assert(false, format("Failed to parse primary expression (%s)", parser.current().kind));
    }
}

Expr parse_arrow_expr(Parser parser, Expr left, BindingPower bp) {
    assert(!is(left == SymbolExpr), format("LHS of arrow MUST be an identifier, got: %s", left));

    auto symbol = cast(SymbolExpr)left;
    parser.expect(TokenKind.ARROW);

    switch(parser.advance().kind) {
        case TokenKind.SUB: writeln("Function declaration(", symbol.value, ")"); break;
        case TokenKind.STRUCT: writeln("Struct declaration(", symbol.value, ")"); break;
        default: assert(false, format("Expected high level declaration but got %s", parser.current().kind));
    }

    return new SymbolExpr("test");
}

Expr parse_array_expr(Parser parser) {
    parser.advance();

    Expr[] exprs;

    while(true) {
        exprs ~= parse_expr(parser, BindingPower.Default);
        if(parser.current().kind == TokenKind.CLOSE_BRACKET) break;
        else parser.expect(TokenKind.COMMA);
    }

    parser.advance();

    return new ArrayExpr(exprs);
}


Expr parse_binary_expr(Parser parser, Expr left, BindingPower bp) {
    auto op = parser.advance();
    auto right = parse_expr(parser, bp);

    return new BinExpr(left, op, right);
}