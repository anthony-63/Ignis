module ast.expressions;

import ast.ast;
import lexer.tokens;

class NumberExpr : Expr {
    float fval;
    int ival;

    this(int val) { ival = val; }
    this(float val) { fval = val; }

    override void expr() {

    }
}

class StringExpr : Expr {
    string value;

    this(string val) { value = val; }

    override void expr() {
        
    }
}

class SymbolExpr : Expr {
    string value;

    this(string val) { value = val; }

    override void expr() {
        
    }
}

class BinExpr : Expr {
    Expr left;
    Token op;
    Expr right;

    this(Expr l, Token o, Expr r) { left = l, op = o; right = r; }

    override void expr() {
        
    }
}

