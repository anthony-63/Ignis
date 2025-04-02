module compiler.compiler;

import std.file;
import std.conv;
import std.process;
import std.stdio : writeln;
import std.array;
import std.format;
import std.algorithm;

import ast.ast;
import ast.types;
import ast.statements;
import ast.expressions;

const string[string] type_map = [
    "i8": "b",
    "i16": "h",
    "i32": "w",
    "f32": "s",
    "i64": "l",
    "f64": "d",
    "void": "",
];

class Compiler {
    Stmt[] ast;

    string ir = "";
    size_t uid = 0;

    this(BlockStmt root) {
        ast = root.body;
    }

    void compile(string output) {
        FuncDeclStmt main = get_main();
        transform_main(main);
        
        foreach(stmt; ast) {
            if(auto func = cast(FuncDeclStmt)stmt) {
                emit_function(func);
            } else if(auto struc = cast(StructDeclStmt)stmt) {
                
            } else {
                assert(false, "Expecting only a function or struct in the top level");
            }
        }

        write(output ~ ".ssa", ir);
        auto qbe = spawnProcess(["./qbe/linux/qbe", output ~ ".ssa", "-o", output ~ ".s"]);
        wait(qbe);
        auto gcc = spawnProcess(["gcc", output ~ ".s", "-o", output]);
        wait(gcc);
        // remove(output ~ ".ssa");
        remove(output ~ ".s");
    }

    void transform_main(FuncDeclStmt main) {
        if(auto ret = cast(SymbolType)main.return_type) {
            if(ret.name == "void") main.return_type = new SymbolType("i32");
        }
    }

    string get_raw_type(Type type) {
        if(auto stype = cast(SymbolType)type) {
            auto type_name = (cast(SymbolType)stype).name;
            if(type_name !in type_map) {
                assert(false, format("Invalid type %s", type_name));
            }
            return type_map[type_name]; 
        } else {
            assert(false, format("No support for ignis type %s", type));
        }

    }

    void emit_expr(Expr expr) {
        if(auto num = cast(IntExpr)expr) {
            ir ~= to!string(num.val);
        }
    }

    void emit_function(FuncDeclStmt func) {
        auto export_str = func.ident == "main" ? "export " : "";
        bool returned = false;
        string raw_type = get_raw_type(func.return_type);

        string[] args;
        foreach(arg; func.args) {
            args ~= get_raw_type(arg.type) ~ " %" ~ arg.ident;            
        }

        ir ~= format("
%sfunction %s $%s(%s) {
@start
",
        export_str, raw_type, func.ident, args.join(", "));

        foreach(stmt; func.body) {
            if(auto ret = cast(ReturnStmt)stmt) {
                ir ~= "ret ";
                emit_expr(ret.ret);
                ir ~= "\n";
                returned = true;
            } else if(auto expr = cast(ExprStmt)stmt) {
                emit_expr(expr.expression);
            }
        }

        if(!returned && func.ident == "main") {
            ir ~= "ret 0\n";
        } else {
            if(raw_type == "" && !returned) {
                ir ~= "ret\n";
            } else if(!returned) {
                assert(false, format("Expected return for function '%s'", func.ident));
            }
        }

        ir ~= "}\n";
    }

    FuncDeclStmt get_main() {
        FuncDeclStmt main = null;
        auto found = 0;

        foreach(stmt; ast) {
            if(auto f = cast(FuncDeclStmt)stmt) {
                if(f.ident == "main") {
                    main = f;
                    found++;
                }
            }
        }
        if(main is null) {
            assert(false, "Failed to find main function on the top level");
        } else if(found > 1) {
            assert(false, "ONLY DEFINE ONE MAIN FUNCTION");
        }
        return main;
    }
}