module ast.expressions;

import ast.ast;
import lexer.tokens;

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

