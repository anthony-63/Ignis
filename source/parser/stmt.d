module parser.stmt;

import ast.ast;
import ast.statements;
import ast.expressions;
import parser.expr;
import lexer.tokens;
import parser.parser;
import parser.lookups;

Stmt parse_stmt(Parser parser) {
    if(parser.current().kind in stmt_lu) {
        stmt_lu[parser.current().kind](parser);
    }

    auto expr = parse_expr(parser, BindingPower.Default);
    parser.expect(TokenKind.SEMICOLON);

    return new ExprStmt(expr);
}