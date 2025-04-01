module ast.statements;

import ast.ast;

class BlockStmt : Stmt {
    Stmt[] body;

    this(Stmt[] _body) { body = _body; }
}

class ExprStmt : Stmt {
    Expr expression;

    this(Expr expr) { expression = expr; }
}

class VarDeclStmt : Stmt {
    string ident;
    bool mutable;
    Expr value;

    Type explicit_type;

    this(string _ident, bool _mut, Expr _value, Type _explicit) { ident = _ident; mutable = _mut; value = _value; explicit_type = _explicit; }
}

class FieldStmt : Stmt {
    string ident;
    Type type;

    this(string _ident, Type _type) { ident = _ident; type = _type; }
}

class ReturnStmt : Stmt {
    Expr ret;

    this(Expr _ret) { ret = _ret; }
}

class StructDeclStmt : Stmt {
    string ident;
    FieldStmt[] fields;
    FuncDeclStmt[] funcs;

    this(string _ident, FieldStmt[] _fields, FuncDeclStmt[] _funcs) { ident = _ident; fields = _fields; funcs = _funcs; }
}

class FuncDeclStmt : Stmt {
    string ident;
    Type return_type;
    FieldStmt[] args;
    Stmt[] body;

    this(string _ident, FieldStmt[] _args, Stmt[] _body, Type _ret_type) { ident = _ident; args = _args; body = _body; return_type = _ret_type; }
}