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

class FuncDeclExprHack : HackedExpr {
    FuncDeclStmt stmt;
    this(Stmt _stmt) { stmt = cast(FuncDeclStmt)_stmt; }

    Stmt get_stmt() {
        return stmt;
    }
}

class AccessExpr : Expr {
    string parent;
    string member;

    this(string _parent, string _member) { parent = _parent; member = _member; }
}

class CallExpr : Expr {
    string name;
    Expr[] args;

    this(string _name, Expr[] _args) { name = _name; args = _args; }
}