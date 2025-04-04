module lexer.tokens;

import std.stdio;

enum TokenKind {
    EOF,

    INT,
    DECIMEL,
    STRING,

    IDENT,
    
    OPEN_BRACKET,
    CLOSE_BRACKET,

    OPEN_CURLY,
    CLOSE_CURLY,

    OPEN_PAREN,
    CLOSE_PAREN,

    ASSIGNMENT,


    CMPEXPR_START,

    EQUALS,
    NOT,
    NOT_EQUALS,

    LESS,
    LESS_EQUALS,
    GREATER,
    GREATER_EQUALS,

    OR,
    AND,

    CMPEXPR_END,


    DOT,
    RANGE,

    SEMICOLON,
    COLON,
    QUESTION,
    COMMA,

    PLUS_PLUS,
    MINUS_MINUS,
    PLUS_EQUALS,
    MINUS_EQUALS,

    PLUS,
    DASH,
    SLASH,
    STAR,
    PERCENT,
    POW,
    ARROW,

    REF,

    INCLUDE,

    SUB,
    STRUCT,
    RETURN,

    IF,
    ELSE,
    FOR,
    WHILE,
    TYPEOF,
    SIZEOF,
    MUT,
    IMMUT,

    NEW,
}

const TokenKind[string] reserved_keywords = [
    "include": TokenKind.INCLUDE,
    "if": TokenKind.IF,
    "else": TokenKind.ELSE,
    "for": TokenKind.FOR,
    "while": TokenKind.WHILE,
    "sizeof": TokenKind.SIZEOF,
    "typeof": TokenKind.TYPEOF,
    "mut": TokenKind.MUT,
    "immut": TokenKind.IMMUT,
    "sub": TokenKind.SUB,
    "struct": TokenKind.STRUCT,
    "return": TokenKind.RETURN,
    "new": TokenKind.NEW,
];

struct Token {
    TokenKind kind;
    string value;

    bool is_any(TokenKind[] expected) {
        foreach(t; expected) {
            if(kind == t) return true;
        }
        return false;
    }

    void print() {
        if(is_any([TokenKind.IDENT, TokenKind.INT, TokenKind.DECIMEL, TokenKind.STRING])) {
            writeln(kind, ": '", value, "'");        
        } else {
            writeln(kind);
        }
    }
}