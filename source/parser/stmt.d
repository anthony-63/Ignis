module parser.stmt;

import std.stdio;

import ast.ast;
import ast.statements;
import ast.expressions;
import parser.expr;
import lexer.tokens;
import parser.parser;
import parser.lookups;

Stmt parse_stmt(Parser parser) {
    if(parser.current().kind in stmt_lu) {
        return stmt_lu[parser.current().kind](parser);
    }

    auto expr = parse_expr(parser, BindingPower.Default);
    parser.expect(TokenKind.SEMICOLON);

    return new ExprStmt(expr);
}

Stmt parse_var_decl_stmt(Parser parser) {
    auto mutable = parser.advance().kind == TokenKind.MUT;

    auto name = parser.expect_error(TokenKind.IDENT, "Expected identifier in variable declaration").value;
    parser.expect(TokenKind.ASSIGNMENT);

    auto val = parse_expr(parser, BindingPower.Assignment);
    parser.expect(TokenKind.SEMICOLON);

    return new VarDeclStmt(name, mutable, val);
}