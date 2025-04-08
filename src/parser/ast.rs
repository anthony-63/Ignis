use crate::lexer::Token;



pub enum Expr {
    Int(isize),

    Float(f64),

    String(String),

    Symbol(String),

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
        assignee: String,
        right: Box<Expr>,
    },

    StructInitialize {
        name: String,

        fields: Vec<Stmt>,
    },
}

pub enum Stmt {
    Block(Vec<Stmt>),

    Expression(Box<Expr>),

    VariableDeclaration {
        name: String,
        mutable: bool,

        explicit_type: Option<Box<Type>>,

        value: Box<Expr>,
    },
    
    If {
        condition: Box<Expr>,
        body: Box<Stmt>,
        _else: Box<Stmt>,
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
        fields: Box<Stmt>,
        functions: Box<Stmt>,
    },

    StructInitField {
        name: String,
        value: Box<Expr>,
    },

    FunctionDeclaration {
        name: String,
        return_type: Option<Box<Type>>,

        arguments: Vec<Stmt>,
        body: Box<Stmt>,
    },

    Extern {
        name: String,
        symbol: String,

        return_type: Option<Box<Type>>,
        arguments: Vec<Stmt>,
    },

    Include {
        path: String,
    },
}

pub enum Type {
    Symbol(String),

    Ref(Box<Self>),

    Array(Box<Self>),
}