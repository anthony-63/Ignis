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

class StructFieldStmt : Stmt {
    string identifier;
    
}

class StructDeclStmt : Stmt {
    string name;
    
}