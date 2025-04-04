module ast.expressions;

import ast.ast;
import lexer.tokens;
import ast.statements;


class IntExpr : Expr {
    int val;

    this(int _val) { val = _val; }
}

class FloatExpr : Expr {
    float val;

    this(float _val) { val = _val; }
}


class StringExpr : Expr {
    string value;

    this(string val) { value = val; }
}

class SymbolExpr : Expr {
    string value;

    this(string val) { value = val; }
}

class BinExpr : Expr {
    Expr left;
    Token op;
    Expr right;

    this(Expr l, Token o, Expr r) { left = l, op = o; right = r; }
}

class PrefixExpr : Expr {
    Token op;
    Expr right;

    this(Token o, Expr r) { op = o; right = r; }
}

class ArrayExpr : Expr {
    Expr[] data;

    this(Expr[] _data) { data = _data; }
}

class StructDeclExprHack : HackedExpr {
    StructDeclStmt stmt;

    this(Stmt _stmt) { stmt = cast(StructDeclStmt)_stmt; }

    Stmt get_stmt() {
        return stmt;
    }
}

class ExternExprHack : HackedExpr {
    ExternStmt stmt;

    this(Stmt _stmt) { stmt = cast(ExternStmt)_stmt; }

    Stmt get_stmt() {
        return stmt;
    }
}

class FuncDeclExprHack : HackedExpr {
    FuncDeclStmt stmt;
    this(Stmt _stmt) { stmt = cast(FuncDeclStmt)_stmt; }

    Stmt get_stmt() {
        return stmt;
    }
}

class CallExpr : Expr {
    string name;
    Expr[] args;

    this(string _name, Expr[] _args) { name = _name; args = _args; }
}

class AssignmentExpr : Expr {
    string left;
    Expr right;

    this(string l, Expr r) { left = l, right = r; }
}

class StructCreateExpr : Expr {
    string name;

    StructInitFieldStmt[] initializers;

    this(string _name, StructInitFieldStmt[] _inits) { name = _name; initializers = _inits; }
}