module parser.expr;

import std.conv;
import std.stdio;
import std.format;

import ast.ast;
import lexer.tokens;
import parser.parser;
import parser.lookups;
import ast.expressions;
import parser.stmt;

Expr parse_expr(Parser parser, BindingPower bp) {
    auto kind = parser.current.kind;
    
    assert(kind in nud_lu, format("NUD function not existant for (%s)", kind));

    auto nud_fn = nud_lu[kind];
    auto left = nud_fn(parser);
    
    while(parser.current.kind in bp_lu && bp_lu[parser.current.kind] > bp) {
        kind = parser.current.kind;
        assert(kind in led_lu, format("LED function not existant for (%s)", kind));
        
        auto led_fn = led_lu[kind];
        left = led_fn(parser, left, bp);
    }

    return left;
}

Expr parse_primary_expr(Parser parser) {
    switch(parser.current.kind) {
        case TokenKind.INT: return new IntExpr(to!int(parser.advance().value)); break;
        case TokenKind.DECIMEL: return new FloatExpr(to!float(parser.advance().value)); break;
        case TokenKind.STRING: return new StringExpr(parser.advance().value); break;
        case TokenKind.IDENT: return new SymbolExpr(parser.advance().value); break;
        default: assert(false, format("Failed to parse primary expression (%s)", parser.current.kind));
    }
}

Expr parse_arrow_expr(Parser parser, Expr left, BindingPower bp) {
    assert(!is(left == SymbolExpr), format("LHS of arrow MUST be an identifier, got: %s", left));

    auto symbol = cast(SymbolExpr)left;
    parser.expect(TokenKind.ARROW);

    switch(parser.advance().kind) {
        case TokenKind.SUB: return new FuncDeclExprHack(parse_function_decl(parser, symbol)); break;
        case TokenKind.STRUCT: return new StructDeclExprHack(parse_struct_decl(parser, symbol)); break;
        default: assert(false, format("Expected high level declaration with arrow but got %s", parser.current.kind));
    }

    return new SymbolExpr("test");
}

Expr parse_array_expr(Parser parser) {
    parser.advance();

    Expr[] exprs;

    while(parser.has_tokens() && parser.current.kind != TokenKind.CLOSE_BRACKET) {
        exprs ~= parse_expr(parser, BindingPower.Default);
        if(parser.current.kind != TokenKind.CLOSE_BRACKET) {
            parser.expect(TokenKind.COMMA);
        }
    }

    parser.advance();

    return new ArrayExpr(exprs);
}


Expr parse_binary_expr(Parser parser, Expr left, BindingPower bp) {
    auto op = parser.advance();
    auto right = parse_expr(parser, bp);

    return new BinExpr(left, op, right);
}

Expr parse_dot_expr(Parser parser, Expr left, BindingPower bp) {
    assert(!is(left == SymbolExpr), "Expected Identifier on LHS of access expression");
    parser.advance();
    auto right = parser.expect(TokenKind.IDENT);
    return new AccessExpr((cast(SymbolExpr)left).value, right.value);
}

Expr parse_call_expr(Parser parser, Expr left, BindingPower bp) {
    assert(!is(left == SymbolExpr), "Expected Identifier on LHS of function call");
    parser.advance();

    Expr[] args;
    while(parser.has_tokens() && parser.current.kind != TokenKind.CLOSE_PAREN) {
        args ~= parse_expr(parser, BindingPower.Default);
        if(parser.current.kind != TokenKind.CLOSE_PAREN) {
            parser.expect(TokenKind.COMMA);
        }
    }
    parser.expect(TokenKind.CLOSE_PAREN);

    return new CallExpr((cast(SymbolExpr)left).value, args);
}