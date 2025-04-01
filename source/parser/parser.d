module parser.parser;

import std.format;

import lexer.tokens;
import ast.ast;
import ast.statements;
import parser.stmt;
import parser.lookups;
import parser.expr;

void setup_lookup_table() {
    led(TokenKind.AND, BindingPower.Logical, &parse_binary_expr);
    led(TokenKind.OR, BindingPower.Logical, &parse_binary_expr);
    led(TokenKind.RANGE, BindingPower.Logical, &parse_binary_expr);

    led(TokenKind.LESS, BindingPower.Relational, &parse_binary_expr);
    led(TokenKind.LESS_EQUALS, BindingPower.Relational, &parse_binary_expr);
    led(TokenKind.GREATER, BindingPower.Relational, &parse_binary_expr);
    led(TokenKind.GREATER_EQUALS, BindingPower.Relational, &parse_binary_expr);
    led(TokenKind.EQUALS, BindingPower.Relational, &parse_binary_expr);
    led(TokenKind.NOT_EQUALS, BindingPower.Relational, &parse_binary_expr);
    
    led(TokenKind.PLUS, BindingPower.Additive, &parse_binary_expr);
    led(TokenKind.DASH, BindingPower.Additive, &parse_binary_expr);

    led(TokenKind.STAR, BindingPower.Multiplicative, &parse_binary_expr);
    led(TokenKind.SLASH, BindingPower.Multiplicative, &parse_binary_expr);
    led(TokenKind.PERCENT, BindingPower.Multiplicative, &parse_binary_expr);

    nud(TokenKind.INT, BindingPower.Primary, &parse_primary_expr);
    nud(TokenKind.DECIMEL, BindingPower.Primary, &parse_primary_expr);
    nud(TokenKind.STRING, BindingPower.Primary, &parse_primary_expr);
    nud(TokenKind.IDENT, BindingPower.Primary, &parse_primary_expr);

    stmt(TokenKind.MUT, &parse_var_decl_stmt);
    stmt(TokenKind.IMMUT, &parse_var_decl_stmt);
}

class Parser {
    Token[] tokens;
    size_t pos = 0;

    private this(Token[] _tokens) {
        tokens = _tokens;
        setup_lookup_table();
    }

    static BlockStmt parse(Token[] tokens) {
        Stmt[] body;

        auto parser = new Parser(tokens);
        while(parser.has_tokens()) {
            body ~= parse_stmt(parser);
        }

        return new BlockStmt(body);
    }

    Token current() {
        return tokens[pos];
    }

    Token advance() {
        auto t = current();
        pos++;
        return t;
    }

    Token expect_error(TokenKind expected, string err) {
        auto tok = current();
        auto kind = tok.kind;

        if(kind != expected) {
            assert(false, err);
        }

        return advance();
    }

    Token expect(TokenKind expected) {
        return expect_error(expected, format("Expected %s but got %s", expected, tokens[pos-1].kind));
    }

    bool has_tokens() {
        return pos < tokens.length && current().kind != TokenKind.EOF;
    }
}