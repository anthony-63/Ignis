module ast.ast;

interface Stmt {}
interface Expr {}
interface Type {}
interface HackedExpr : Expr {
    Stmt get_stmt();
}