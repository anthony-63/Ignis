module parser.types;

import std.stdio;
import std.format;

import ast.ast;
import ast.types;
import lexer.tokens;
import parser.parser;
import parser.lookups;
import ast.expressions;

alias TypeNUDHandler = Type function(Parser parser);
alias TypeLEDHandler = Type function(Parser parser, Type left, BindingPower bp);

alias TypeNUDLookup = TypeNUDHandler[TokenKind];
alias TypeLEDLookup = TypeLEDHandler[TokenKind];
alias TypeBPLookup = BindingPower[TokenKind];

TypeBPLookup type_bp_lu = null;
TypeNUDLookup type_nud_lu = null;
TypeLEDLookup type_led_lu = null;

void type_led(TokenKind kind, BindingPower bp, TypeLEDHandler handler) {
    type_bp_lu[kind] = bp;
    type_led_lu[kind] = handler;
}

void type_nud(TokenKind kind, TypeNUDHandler handler) {
    type_nud_lu[kind] = handler;
}

Type parse_symbol_type(Parser parser) {
    return new SymbolType(parser.expect(TokenKind.IDENT).value);
}

Type parse_array_type(Parser parser) {
    parser.advance();
    parser.expect(TokenKind.CLOSE_BRACKET);
    // TODO PARSE ARRAY SIZE

    auto type = parse_type(parser, BindingPower.Default);
    return new ArrayType(type);
}

Type parse_type(Parser parser, BindingPower bp) {
    auto kind = parser.current.kind;
    assert(kind in type_nud_lu, format("TYPE NUD function not existant for (%s)", kind));

    auto nud_fn = type_nud_lu[kind];
    auto left = nud_fn(parser);
    
    while(parser.current.kind in type_bp_lu && type_bp_lu[parser.current.kind] > bp) {
        kind = parser.current.kind;
        assert(kind in type_led_lu, format("TYPE LED function not existant for (%s)", kind));
        
        auto led_fn = type_led_lu[kind];
        left = led_fn(parser, left, bp);
    }

    return left;
}

Type parse_ref_type(Parser parser) {
    parser.advance();

    return new RefType(parse_type(parser, BindingPower.Default));
}