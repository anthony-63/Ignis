use crate::lexer::Token;


#[derive(Debug, Clone)]
pub enum Expr {
    Int(isize),

    Float(f64),

    String(String),

    Symbol(String),

    Bool(bool),

    Binary {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },

    Prefix {
        op: Token,
        right: Box<Expr>,
    },

    Array(Vec<Expr>),

    Call {
        name: String,
        args: Vec<Expr>,
    },

    Assignment {
        assignee: Box<Expr>,
        right: Box<Expr>,
    },

    StructInitialize {
        name: String,

        fields: Vec<Stmt>,
    },

    Access {
        lhs: Box<Expr>,
        rhs: Box<Expr>
    },

    StmtHack(Stmt),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),

    Expression(Box<Expr>),

    VariableDeclaration {
        name: String,

        explicit_type: Option<Box<Type>>,

        value: Box<Expr>,
    },
    
    If {
        condition: Box<Expr>,
        body: Box<Stmt>,
        _else: Option<Box<Stmt>>,
    },

    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    },

    Link {
        library: String,
        _static: bool,
    },

    Field {
        name: String,
        _type: Box<Type>,
    },

    Return {
        value: Box<Expr>,
    },

    StructDeclaration {
        name: String,
        fields: Vec<Stmt>,
        functions: Vec<Stmt>,
    },

    StructInitField {
        name: String,
        value: Box<Expr>,
    },

    FunctionDeclaration {
        name: String,
        return_type: Box<Type>,

        arguments: Vec<Stmt>,
        body: Box<Stmt>,
    },

    Extern {
        name: String,
        symbol: String,

        return_type: Box<Type>,
        arguments: Vec<Stmt>,
    },

    Include {
        path: String,
    },
}

#[derive(Debug, Clone)]
pub enum Type {
    Symbol(String),

    Ref(Box<Self>),

    Array(Box<Self>),
}