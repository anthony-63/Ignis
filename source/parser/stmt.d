module parser.stmt;

import std.stdio;
import std.format;

import ast.ast;
import ast.types;
import ast.statements;
import ast.expressions;
import parser.expr;
import lexer.tokens;
import parser.parser;
import parser.lookups;
import parser.types;

Stmt parse_stmt(Parser parser) {
    if(parser.current.kind in stmt_lu) {
        return stmt_lu[parser.current.kind](parser);
    }

    auto expr = parse_expr(parser, BindingPower.Default);

    if(auto hexpr = cast(HackedExpr)expr) {
        return hexpr.get_stmt();
    }

    parser.expect(TokenKind.SEMICOLON);

    return new ExprStmt(expr);
}

Stmt parse_ret_stmt(Parser parser) {
    parser.advance();

    auto val = parse_expr(parser, BindingPower.Default);
    parser.expect(TokenKind.SEMICOLON);

    return new ReturnStmt(val);
}

Stmt parse_var_decl_stmt(Parser parser) {
    auto mutable = parser.advance().kind == TokenKind.MUT;

    auto name = parser.expect_error(TokenKind.IDENT, "Expected identifier in variable declaration").value;

    Type explicit_type = null;

    if(parser.current.kind == TokenKind.COLON) {
        parser.advance();
        explicit_type = parse_type(parser, BindingPower.Default);
    }

    parser.expect(TokenKind.ASSIGNMENT);

    auto val = parse_expr(parser, BindingPower.Assignment);
    parser.expect(TokenKind.SEMICOLON);

    return new VarDeclStmt(name, mutable, val, explicit_type);
}

Stmt parse_function_decl(Parser parser, SymbolExpr name) {
    parser.expect(TokenKind.OPEN_PAREN);

    // writeln("-------------------------");
    // writeln("|     FUNCTION DECL     |");
    // writeln("-------------------------");
    // writeln("subroutine name: ", name.value);
    FieldStmt[] args;
    while(parser.has_tokens() && parser.current.kind != TokenKind.CLOSE_PAREN) {
        auto field_name = parser.advance();
        if(field_name.value == "this" || parser.current.value == "this") {
            if(field_name.kind == TokenKind.REF) args ~= new FieldStmt("this", new RefType(new SymbolType("this")));
            else if(field_name.kind == TokenKind.IDENT) args ~= new FieldStmt("this", new SymbolType("this"));
            else assert(false, format("Invalid 'this' argument in function '%s'", name.value));
            if(parser.current.value == "this") parser.advance();
            // writeln("'this' arg passed in by type: ", args[$-1].type);
        } else {
            auto type = parse_type(parser, BindingPower.Default);
            // writeln("arg type: ", type);
            args ~= new FieldStmt(field_name.value, type);
            // writeln("arg name: ", field_name);

        }

        if(parser.current.kind != TokenKind.CLOSE_PAREN) {
            parser.expect(TokenKind.COMMA);
        }

    }
    parser.expect(TokenKind.CLOSE_PAREN);

    Type return_type = new SymbolType("void");
    // writeln("return type: ", return_type);

    if(parser.current.kind == TokenKind.IDENT) {
        return_type = parse_type(parser, BindingPower.Member);
    }

    Stmt[] body;

    parser.expect(TokenKind.OPEN_CURLY);
    while(parser.has_tokens() && parser.current.kind != TokenKind.CLOSE_CURLY) {
        body ~= parse_stmt(parser);
    }

    // writeln(body);

    parser.expect(TokenKind.CLOSE_CURLY);

    return new FuncDeclStmt(name.value, args, body, return_type);
}

Stmt parse_struct_decl(Parser parser, SymbolExpr name) {
    parser.expect(TokenKind.OPEN_CURLY);
    // writeln();

    // writeln("-------------------------");
    // writeln("|      STRUCT DECL      |");
    // writeln("-------------------------");
    // writeln("struct name: ", name.value);

    FieldStmt[] fields;
    FuncDeclStmt[] funcs;

    while(parser.has_tokens() && parser.current.kind != TokenKind.CLOSE_CURLY) {
        auto field_name = parser.expect(TokenKind.IDENT);
        // writeln("field name: ", field_name);

        if(parser.current.kind == TokenKind.ARROW) {
            parser.advance();
            parser.expect(TokenKind.SUB);
            // writeln("-------------------------");
            // writeln("|      MEMBER FUNC      |");
            funcs ~= cast(FuncDeclStmt)parse_function_decl(parser, new SymbolExpr(field_name.value));
            continue;
        }

        auto type = parse_type(parser, BindingPower.Default);
        // writeln("field type: ", type);

        fields ~= new FieldStmt(field_name.value, type);
        if(parser.current.kind != TokenKind.CLOSE_CURLY) {
            parser.expect(TokenKind.COMMA);
        }
    }

    parser.advance();

    return new StructDeclStmt(name.value, fields, funcs);
}