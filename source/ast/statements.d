module ast.statements;

import ast.ast;

class BlockStmt : Stmt {
    Stmt[] body;

    this(Stmt[] _body) { body = _body; }

    override void stmt() {

    }
}

class ExprStmt : Stmt {
    Expr expression;

    this(Expr expr) { expression = expr; }

    override void stmt() {}
}