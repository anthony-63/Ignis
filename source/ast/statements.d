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

class IfStmt : Stmt {
    Expr cond;
    Stmt[] body;
    Stmt[] _else;

    this(Expr _cond, Stmt[] _body, Stmt[] __else) { cond = _cond; body = _body; _else = __else; }
}

class LinkStmt : Stmt {
    string lib;
    bool _static;

    this(string _lib, bool __static) { lib = _lib; _static = __static; }
}

class FieldStmt : Stmt {
    string ident;
    Type type;

    this(string _ident, Type _type) { ident = _ident; type = _type; }
}

class CallArgumentStmt: Stmt {
    Type type;
    Expr expression;

    this(Type _type, Expr _expr) { expression = _expr; type = _type; }

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

class StructInitFieldStmt : Stmt {
    string name;
    Expr value;

    this(string _name, Expr _val) { name = _name; value = _val; }
}

class FuncDeclStmt : Stmt {
    string ident;
    Type return_type;
    FieldStmt[] args;
    Stmt[] body;

    this(string _ident, FieldStmt[] _args, Stmt[] _body, Type _ret_type) { ident = _ident; args = _args; body = _body; return_type = _ret_type; }
}

class ExternStmt : Stmt {
    string name;
    string symbol;

    Type return_type;
    FieldStmt[] args;

    this(string _name, string _symbol, Type _ret, FieldStmt[] _args) { name = _name; symbol = _symbol; return_type = _ret; args = _args; }
}

class IncludeStmt : Stmt {
    string path;

    this(string _path) { path = _path; }
}