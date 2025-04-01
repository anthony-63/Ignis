module lexer.lexer;

import std.stdio;
import std.regex;
import std.string;
import std.algorithm;

import lexer.tokens;

alias RegexHandler = void delegate(Lexer lexer, Regex!char regex) @system;

struct RegexPattern {
    Regex!char pattern;
    RegexHandler handler;
}

class Lexer {
    private RegexPattern[] patterns;
    private Token[] tokens = [];
    private string source;
    private size_t position = 0;

    this(string source) {
        this.source = source;
    
        patterns = reverse([
            RegexPattern(regex(r"\s+"), whitespace_handler()),
            RegexPattern(regex(`\/\/.*`), comment_handler()),
            RegexPattern(regex(`"[^"]*`), string_handler()),
            RegexPattern(regex(r"[0-9]+"), integer_handler()),
            RegexPattern(regex(r"[0-9]+(\.[0-9]+)"), float_handler()),
            RegexPattern(regex(r"[a-zA-Z_][a-zA-Z0-9_]*"), symbol_handler()),
            RegexPattern(regex(r"&"), default_handler(TokenKind.REF, "&")),
            RegexPattern(regex(r"\["), default_handler(TokenKind.OPEN_BRACKET, "[")),
			RegexPattern(regex(r"\]"), default_handler(TokenKind.CLOSE_BRACKET, "]")),
			RegexPattern(regex(r"\{"), default_handler(TokenKind.OPEN_CURLY, "{")),
			RegexPattern(regex(r"\}"), default_handler(TokenKind.CLOSE_CURLY, "}")),
			RegexPattern(regex(r"\("), default_handler(TokenKind.OPEN_PAREN, "(")),
			RegexPattern(regex(r"\)"), default_handler(TokenKind.CLOSE_PAREN, ")")),
			RegexPattern(regex(r"=="), default_handler(TokenKind.EQUALS, "==")),
			RegexPattern(regex(r"!="), default_handler(TokenKind.NOT_EQUALS, "!=")),
			RegexPattern(regex(r"="), default_handler(TokenKind.ASSIGNMENT, "=")),
			RegexPattern(regex(r"!"), default_handler(TokenKind.NOT, "!")),
			RegexPattern(regex(r"<="), default_handler(TokenKind.LESS_EQUALS, "<=")),
			RegexPattern(regex(r"<"), default_handler(TokenKind.LESS, "<")),
			RegexPattern(regex(r">="), default_handler(TokenKind.GREATER_EQUALS, ">=")),
			RegexPattern(regex(r">"), default_handler(TokenKind.GREATER, ">")),
			RegexPattern(regex(r"\|"), default_handler(TokenKind.OR, "||")),
			RegexPattern(regex(r"&&"), default_handler(TokenKind.AND, "&&")),
			RegexPattern(regex(r"\.\."), default_handler(TokenKind.RANGE, "..")),
			RegexPattern(regex(r"\."), default_handler(TokenKind.DOT, ".")),
			RegexPattern(regex(r"\;"), default_handler(TokenKind.SEMICOLON, ";")),
			RegexPattern(regex(r":"), default_handler(TokenKind.COLON, ":")),
			RegexPattern(regex(r"\?"), default_handler(TokenKind.QUESTION, "?")),
			RegexPattern(regex(r","), default_handler(TokenKind.COMMA, ",")),
			RegexPattern(regex(r"\+"), default_handler(TokenKind.PLUS_PLUS, "++")),
			RegexPattern(regex(r"--"), default_handler(TokenKind.MINUS_MINUS, "--")),
			RegexPattern(regex(r"\+="), default_handler(TokenKind.PLUS_EQUALS, "+=")),
			RegexPattern(regex(r"-="), default_handler(TokenKind.MINUS_EQUALS, "-=")),
			RegexPattern(regex(r"\+"), default_handler(TokenKind.PLUS, "+")),
			RegexPattern(regex(r"-"), default_handler(TokenKind.DASH, "-")),
			RegexPattern(regex(r"/"), default_handler(TokenKind.SLASH, "/")),
			RegexPattern(regex(r"\*"), default_handler(TokenKind.STAR, "*")),
			RegexPattern(regex(r"%"), default_handler(TokenKind.PERCENT, "%")),
            RegexPattern(regex(r"->"), default_handler(TokenKind.ARROW, "->")),
        ]);
    }

    void advance(size_t n) {
        position += n;
    }

    void push(Token token) {
        tokens ~= token;
    }

    char at() {
        return source[position];
    }

    string remainder() {
        return source[position..$];
    }

    bool eof() {
        return position >= source.length;
    }

    Token[] tokenize() {
        while(!eof()) {
            auto matched = false;
            foreach(pattern; patterns) {
                auto r = matchAll(remainder(), pattern.pattern);
                if(r && indexOf(remainder(), r.front[0]) == 0) {
                    pattern.handler(this, pattern.pattern);
                    matched = true;
                    break;
                }
            }
            if(!matched) {
                throw new Error(format("Invalid token '%s'", remainder()[0]));
            }
        }
        tokens ~= Token(TokenKind.EOF, "");

        return tokens;
    }

    private static RegexHandler default_handler(TokenKind kind, string value) {
        return (Lexer lexer, Regex!char regex) {
            lexer.advance(value.length);
            lexer.push(Token(kind, value));
        };
    }

    private static RegexHandler whitespace_handler() {
        return (Lexer lexer, Regex!char regex) {
            auto match = matchAll(lexer.remainder(), regex);
            lexer.advance(match.hit.length);
        };
    }

    private static RegexHandler comment_handler() {
        return (Lexer lexer, Regex!char regex) {
            auto match = matchAll(lexer.remainder(), regex);
            lexer.advance(match.hit.length);
            // TODO FIX
        };
    }

    private static RegexHandler integer_handler() {
        return (Lexer lexer, Regex!char regex) {
            auto match = matchFirst(lexer.remainder(), regex);
            lexer.push(Token(TokenKind.INT, match.hit));
            lexer.advance(match.hit.length);
        };
    }

    private static RegexHandler string_handler() {
        return (Lexer lexer, Regex!char regex) {
            auto match = matchFirst(lexer.remainder(), regex);
            auto strlit = lexer.remainder()[match.pre.length+1..match.hit.length];

            lexer.push(Token(TokenKind.STRING, strlit));
            lexer.advance(strlit.length+2);
        };
    }
    
    private static RegexHandler symbol_handler() {
        return (Lexer lexer, Regex!char regex) {
            auto match = matchFirst(lexer.remainder(), regex);
            auto strlit = lexer.remainder()[match.pre.length..match.hit.length];

            if(strlit in reserved_keywords) lexer.push(Token(reserved_keywords[strlit], strlit));
            else lexer.push(Token(TokenKind.IDENT, strlit));
            
            lexer.advance(strlit.length);
        };
    }

    private static RegexHandler float_handler() {
        return (Lexer lexer, Regex!char regex) {
            auto match = matchFirst(lexer.remainder(), regex);
            lexer.push(Token(TokenKind.DECIMEL, match.hit));
            lexer.advance(match.hit.length);
        };
    }
}