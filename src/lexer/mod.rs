use logos::Logos;


#[derive(Debug, Logos, PartialEq, Clone)]
#[logos(skip r"[ \t\n]+")]
#[logos(skip r"\/\/.*")]
#[logos(error = String)]
pub enum Token {    
    #[regex("[0-9]+", |lex| lex.slice().parse::<isize>().unwrap(), priority=1)]
    Integer(isize),

    #[regex(r"[0-9]+(\.[0-9]+)", |lex| lex.slice().parse::<f64>().unwrap(), priority=2)]
    Decimel(f64),

    #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| lex.slice().to_owned())]
    String(String),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    Identifier(String),

    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,

    #[token("{")]
    OpenCurly,
    #[token("}")]
    CloseCurly,

    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    
    #[token("=")]
    Assignment,

    #[token("==")]
    Equals,
    #[token("!=")]
    NotEquals,


    #[token("<")]
    Less,
    #[token("<=")]
    LessOrEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterOrEqual,

    #[token("!")]
    Not,
    #[token("||")]
    Or,
    #[token("&&")]
    And,

    #[token(".")]
    Dot,
    #[token("..")]
    Range,

    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token("?")]
    Question,
    
    #[token(",")]
    Comma,

    #[token("++")]
    PlusPlus,
    #[token("--")]
    MinusMinus,
    #[token("+=")]
    PlusEquals,
    #[token("-=")]
    MinusEquals,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,

    #[token("%")]
    Mod,
    #[token("^^")]
    Power,
    
    #[token("->")]
    Arrow,
    #[token("ref")]
    Reference,

    #[token("include")]
    Include,

    #[token("sub")]
    Subroutine,
    #[token("return")]
    Return,
    
    #[token("struct")]
    Struct,
    #[token("new")]
    New,

    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("for")]
    For,
    #[token("while")]
    While,

    #[token("sizeof")]
    Sizeof,

    #[token("mut")]
    Mut,
    #[token("immut")]
    Immut,

    #[token("linkstatic")]
    LinkStatic,
    #[token("linklib")]
    LinkLib,

    #[token("extern")]
    Extern,
}