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
    EQUALS,
    NOT,
    NOT_EQUALS,

    LESS,
    LESS_EQUALS,
    GREATER,
    GREATER_EQUALS,

    OR,
    AND,

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

    INCLUDE,
}

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