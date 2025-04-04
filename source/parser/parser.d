module parser.parser;

import std.stdio;
import std.format;

import lexer.tokens;
import ast.ast;
import ast.statements;
import ast.expressions;
import parser.stmt;
import parser.lookups;
import parser.types;
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

    led(TokenKind.ASSIGNMENT, BindingPower.Assignment, &parse_assignment_expr);

    led(TokenKind.STAR, BindingPower.Multiplicative, &parse_binary_expr);
    led(TokenKind.POW, BindingPower.Multiplicative, &parse_binary_expr);
    led(TokenKind.SLASH, BindingPower.Multiplicative, &parse_binary_expr);
    led(TokenKind.PERCENT, BindingPower.Multiplicative, &parse_binary_expr);

    led(TokenKind.ARROW, BindingPower.Primary, &parse_arrow_expr);

    led(TokenKind.DOT, BindingPower.Member, &parse_dot_expr);
    led(TokenKind.OPEN_PAREN, BindingPower.Call, &parse_call_expr);

    led(TokenKind.PLUS_EQUALS, BindingPower.Call, &parse_op_equals_expr);
    led(TokenKind.MINUS_EQUALS, BindingPower.Call, &parse_op_equals_expr);

    nud(TokenKind.NEW, &parse_struct_create_expr);

    nud(TokenKind.INT, &parse_primary_expr);
    nud(TokenKind.DECIMEL, &parse_primary_expr);
    nud(TokenKind.STRING, &parse_primary_expr);
    nud(TokenKind.IDENT, &parse_primary_expr);

    nud(TokenKind.OPEN_BRACKET, &parse_array_expr);
    nud(TokenKind.OPEN_PAREN, &parse_grouped_expr);    

    nud(TokenKind.DASH, &parse_prefix_expr);

    stmt(TokenKind.IF, &parse_if_stmt);
    stmt(TokenKind.MUT, &parse_var_decl_stmt);
    stmt(TokenKind.IMMUT, &parse_var_decl_stmt);
    stmt(TokenKind.RETURN, &parse_ret_stmt);
    stmt(TokenKind.LINKSTATIC, &parse_link_static_stmt);
    stmt(TokenKind.LINKLIB, &parse_link_lib_stmt);
    stmt(TokenKind.INCLUDE, &parse_include_stmt);
}

void setup_type_lookup_table() {
    type_nud(TokenKind.IDENT, &parse_symbol_type);
    type_nud(TokenKind.OPEN_BRACKET, &parse_array_type);
    type_nud(TokenKind.REF, &parse_ref_type);
}

class Parser {
    Token[] tokens;
    size_t pos = 0;

    private this(Token[] _tokens) {
        tokens = _tokens;
        setup_lookup_table();
        setup_type_lookup_table();
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
        auto t = current;
        pos++;
        return t;
    }

    Token peek() {
        return tokens[pos + 1];
    }

    Token expect_error(TokenKind expected, string err) {
        auto tok = current;
        auto kind = tok.kind;

        if(kind != expected) {
            assert(false, err);
        }

        return advance();
    }

    Token expect(TokenKind expected) {
        return expect_error(expected, format("Expected %s but got %s", expected, tokens[pos]));
    }

    bool has_tokens() {
        return pos < tokens.length && current.kind != TokenKind.EOF;
    }
}