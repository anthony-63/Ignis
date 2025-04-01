module ast.expressions;

import ast.ast;
import lexer.tokens;

class NumberExpr : Expr {
    float fval;
    int ival;

    this(int val) { ival = val; }
    this(float val) { fval = val; }
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

