module compiler.compiler;

import std.file;
import std.conv;
import std.path;
import std.string;
import std.process;
import std.stdio : writeln;
import std.array;
import std.format;
import std.algorithm;
import std.typecons;
import std.variant;
import std.datetime;

import ast.ast;
import ast.types;
import ast.statements;
import ast.expressions;
import lexer.lexer;
import parser.parser;
import lexer.tokens;

import llvm;
import uid;

import compiler.igscope;
import compiler.value;

struct IGLib {
    string lib;
    bool _static = false;
}

string[] all_outputs;

class Compiler {
    Stmt[] ast;

    LLVMTypeRef[string] type_map;

    IGLib[] libs;

    Compiler[] included;

    LLVMContextRef ctx;

    LLVMModuleRef _module;
    LLVMBuilderRef builder;

    IGScope current_scope;

    string[] include_paths;


    this(BlockStmt root, string[] includes) {
        ast = root.body;
        include_paths = includes;

        ctx = LLVMContextCreate();

        _module = LLVMModuleCreateWithNameInContext("ignis", ctx);
        builder = LLVMCreateBuilderInContext(ctx);

        current_scope = new IGScope();

        type_map = [
            "i8": LLVMInt8TypeInContext(ctx),
            "i16": LLVMInt16TypeInContext(ctx),
            "i32": LLVMInt32TypeInContext(ctx),
            "i64": LLVMInt64TypeInContext(ctx),

            "f16": LLVMHalfTypeInContext(ctx),
            "f32": LLVMFloatTypeInContext(ctx),
            "f64": LLVMDoubleTypeInContext(ctx),

            "bool": LLVMIntTypeInContext(ctx, 1),
            "string": LLVMPointerType(LLVMInt8TypeInContext(ctx), 0),
            "void": LLVMVoidTypeInContext(ctx),
        ];
    }

    LLVMTypeRef get_type(Type type) {
        if(auto t = cast(SymbolType)type) {
            return type_map[t.name];
        } else if(auto t = cast(RefType)type) {
            return LLVMPointerType(get_type(t.inner), 0);  
        } else {
            assert(false, format("No support for %s types", type));
        }
    }

    string output;
    string cwd;

    void compile(string _output, string _cwd, bool inside=false) {
        output = _output;
        cwd = _cwd;

        visit_block(ast);
        
        auto ll_file = output ~ ".ll";
        auto obj_file = output ~ ".o";

        LLVMPrintModuleToFile(_module, ll_file.toStringz(), null);
        auto pid = spawnProcess(["llc", "--filetype=obj", ll_file, "-o", obj_file]);
        wait(pid);
        auto args = ["gcc", "-o", output, "-no-pie"];
        foreach(lib; libs) {
            auto ls = lib._static ? lib.lib : "-l" ~ lib.lib;
            args ~= ls;
        }

        all_outputs ~= _output ~ ".o";

        if(!inside) {
            args ~= all_outputs;
            pid = spawnProcess(args);
            wait(pid);

            foreach(o; all_outputs) {
                remove(o);
                remove(setExtension(o, ".ll"));
            }
        }
    }

    void visit(Stmt stmt) {
        if(auto expr = cast(ExprStmt)stmt) {
            visit_expr(expr.expression);
        } else if(auto decl = cast(VarDeclStmt)stmt) {
            visit_var_decl(decl);
        } else if(auto ret = cast(ReturnStmt)stmt) {
            visit_ret(ret);
        } else if(auto fdecl = cast(FuncDeclStmt)stmt) {
            visit_function_decl(fdecl);
        }  else if(auto ifstmt = cast(IfStmt)stmt) {
            visit_if(ifstmt);  
        } else if(auto lib = cast(LinkStmt)stmt) {
            add_lib(lib);
        } else if(auto ext = cast(ExternStmt)stmt) {
            visit_extern(ext);
        } else if(auto inc = cast(IncludeStmt)stmt) {
            visit_include(inc);
        } else {
            assert(false, format("Unsupported statement %s", stmt));
        }
    }

    LLVMTypeRef[] get_arg_types(FieldStmt[] args) {
        LLVMTypeRef[] types;
        foreach(arg; args) types ~= get_type(arg.type);
        return types;
    }

    LLVMValueRef[] get_arg_values(Expr[] args) {
        LLVMValueRef[] values;
        foreach(arg; args) values ~= resolve_value(arg).value;
        return values;
    }

    void visit_extern(ExternStmt ext) {
        auto types = get_arg_types(ext.args);
        auto ret_type = get_type(ext.return_type);
        auto func_type = LLVMFunctionType(ret_type, types.ptr, cast(uint)types.length, 0);
        auto func = LLVMAddFunction(_module, ext.symbol.toStringz(), func_type);
        
        current_scope.define(ext.name, func, func_type, false);
    }

    string get_include_path(string path) {
        if(exists(path)) return path;

        auto cwdpath = cwd ~ "/" ~ path;
        if(exists(cwdpath)) return cwdpath;

        foreach(include; include_paths) {
            auto incpath = include ~ "/" ~ path;
            if(exists(incpath)) return incpath;
        }
        assert(false, format("Failed to include source file %s", path));
    } 

    void visit_include(IncludeStmt inc) {
        auto inc_path = get_include_path(inc.path);
        auto path = dirName(output);
        auto lexer = new Lexer(readText(inc_path));
        auto tokens = lexer.tokenize();
        auto ast = Parser.parse(tokens);
        auto compiler = new Compiler(ast, include_paths);
        compiler.compile(path ~ "/" ~ stripExtension(baseName(inc_path)), dirName(inc_path), true);
        foreach(v, symbol; compiler.current_scope.symbols) {
            if(LLVMGetTypeKind(LLVMTypeOf(symbol.value)) == LLVMPointerTypeKind && symbol._public) {
                auto func = LLVMAddFunction(_module, v.toStringz(), symbol.type);
                current_scope.define(v, func, symbol.type, false, false);
            }
        }
    }

    void add_lib(LinkStmt stmt) {
        libs ~= IGLib(stmt.lib, stmt._static);
    }

    void visit_block(Stmt[] block) {
        foreach(stmt; block) visit(stmt);
    }

    LLVMBasicBlockRef visit_if(IfStmt stmt) {
        auto outer_scope = current_scope.copy();

        current_scope = new IGScope();
        current_scope.name = "_" ~ newid() ~ "_autogen_if";
        current_scope.parent = outer_scope;

        auto cond = visit_cond(stmt.cond).value;

        auto thenbb = create_basic_block("ignis_then");
        auto elsebb = create_basic_block("ignis_else");
        auto mergebb = create_basic_block("ignis_merge");

        LLVMBuildCondBr(builder, cond, thenbb, elsebb);

        LLVMPositionBuilderAtEnd(builder, thenbb);
        visit_block(stmt.body);
        LLVMBuildBr(builder, mergebb);
        
        LLVMPositionBuilderAtEnd(builder, elsebb);

        if(stmt._else.length < 1) {
            LLVMBuildBr(builder, mergebb);
            return mergebb;
        } else if(auto ifelse = cast(IfStmt)stmt._else[0]) {
            LLVMPositionBuilderAtEnd(builder, elsebb);

            elsebb = LLVMGetInsertBlock(builder);

            auto br = visit_if(ifelse);
            LLVMPositionBuilderAtEnd(builder, br);
            LLVMBuildBr(builder, mergebb);
            LLVMMoveBasicBlockAfter(mergebb, br);
            LLVMPositionBuilderAtEnd(builder, mergebb);

            return mergebb;
        } else {
            visit_block(stmt._else);
            LLVMPositionBuilderAtEnd(builder, mergebb);
            return mergebb;
        }
        current_scope = outer_scope;
        return null;
    }

    IGValue visit_cond(Expr cond) {
        if(auto c = cast(BinExpr)cond)
            return visit_binexpr(c, true);
        else assert(false, "Expected conditional statement");
    }

    void visit_expr(Expr expr) {
        if(auto binexpr = cast(BinExpr)expr) {
            visit_binexpr(binexpr);
        } else if(auto assign = cast(AssignmentExpr)expr) {
            visit_assign(assign);
        } else if(auto call = cast(CallExpr)expr) {
            build_call_expr(call);
        }
    }

    void build_noop() {
        visit_var_decl(new VarDeclStmt("__noop__" ~ newid(), false,
            new BinExpr(new IntExpr(0), Token(TokenKind.PLUS, "+"), new IntExpr(0))
        , new SymbolType("i32")), true);
    }
    
    string newid() {
        return genStringUID() ~ "z";
    }

    void visit_assign(AssignmentExpr assign) {
        auto val = resolve_value(assign.right);
        auto left = current_scope.lookup(assign.left);

        if(!left.resolved) {
            assert(false, format("Assignemnt to variable '%s' before declaration", assign.left));
        }

        if(!left.mutable) {
            assert(false, format("Attempted to assign to immutable variable '%s'", assign.left));
        }

        LLVMBuildStore(builder, val.value, left.value);
    }

    IGValue visit_binexpr(BinExpr binexpr, bool cond=false) {
        auto op = binexpr.op.kind;
        auto lvalue = resolve_value(binexpr.left);
        auto rvalue = resolve_value(binexpr.right);
        LLVMTypeRef type;

        if(binexpr.op.kind > TokenKind.CMPEXPR_START && binexpr.op.kind < TokenKind.CMPEXPR_END) {
            type = get_type(new SymbolType("bool"));
        } else {
            assert(!cond, "Expected conditional expression");
            type = lvalue.type;
        }

        if(lvalue.are_both(rvalue, get_type(new SymbolType("i32")))) {
            return IGValue(visit_op(lvalue, rvalue, op, false), type);
        } else if(lvalue.are_both(rvalue, get_type(new SymbolType("f32")))) {
            return IGValue(visit_op(lvalue, rvalue, op, true), type);
        } else {
            assert(false, format("Unsupported operation between %s and %s", binexpr.left, binexpr.right));
        }
    }

    LLVMValueRef get_current_function() {
        auto bl = LLVMGetInsertBlock(builder);
        return LLVMGetBasicBlockParent(bl);
    }

    IGValue build_call_expr(CallExpr call) {
        auto f = current_scope.lookup(call.name);
        if(!f.resolved) {
           assert(false, format("Failed to resolve function %s", call.name));
        }
        auto args = get_arg_values(call.args);

        if(LLVMGetTypeKind(type_map["void"]) == LLVMGetTypeKind(LLVMGetReturnType(f.type))) {
            LLVMBuildCall(builder, f.value, args.ptr, cast(uint)args.length, "".toStringz());
            return IGValue(null, null, false);
        }
        return IGValue(LLVMBuildCall(builder, f.value, args.ptr, cast(uint)args.length, newid().toStringz()), f.type);
    }

    void visit_var_decl(VarDeclStmt decl, bool dont_define=false) {
        auto name = decl.ident;
        auto val = resolve_value(decl.value);
        
        if(!current_scope.lookup(name).resolved) {
            auto alloca = LLVMBuildAlloca(builder, val.type, (name ~ newid()).toStringz());
            LLVMBuildStore(builder, val.value, alloca);
            if(!dont_define)
                current_scope.define(name, alloca, val.type, decl.mutable);
        } else {
            assert(false, format("Cannot redefine variable %s", name));
        }
    }

    LLVMBasicBlockRef create_basic_block(string name) {
        auto realname = name ~ newid();
        return LLVMAppendBasicBlockInContext(ctx, get_current_function(), realname.toStringz());
    }

    void visit_function_decl(FuncDeclStmt fdecl) {
        auto arg_types = get_arg_types(fdecl.args);
        
        auto ret_type = get_type(fdecl.return_type);

        if(type_map["void"] == get_type(fdecl.return_type) && fdecl.ident == "main") {
            ret_type = get_type(new SymbolType("i32"));
        }

        auto func_type = LLVMFunctionType(ret_type, arg_types.ptr, cast(uint)arg_types.length, 0);
        auto func = LLVMAddFunction(_module, fdecl.ident.toStringz(), func_type);
        auto block = LLVMAppendBasicBlockInContext(ctx, func, (fdecl.ident ~ newid() ~ "_ignis_entry").toStringz());
        
        auto outer_scope = current_scope.copy();
        
        LLVMPositionBuilderAtEnd(builder, block);
        current_scope = new IGScope();
        current_scope.name = fdecl.ident;
        current_scope.parent = outer_scope;

        foreach(i, arg; fdecl.args) {
            auto type = get_type(arg.type);
            auto alloca = LLVMBuildAlloca(builder, type, (arg.ident ~ newid()).toStringz());
            LLVMBuildStore(builder, LLVMGetParam(func, cast(uint)i), alloca);
            current_scope.define(arg.ident, alloca, type, false);
        }

        current_scope.define(fdecl.ident, func, func_type);
        visit_block(fdecl.body);

        if(type_map["void"] == get_type(fdecl.return_type) && fdecl.ident == "main") {
            LLVMBuildRet(builder, resolve_value(new IntExpr(0)).value);
        } else if(type_map["void"] == get_type(fdecl.return_type)) {
            LLVMBuildRetVoid(builder);
        }

        current_scope = outer_scope;
        current_scope.define(fdecl.ident, func, func_type);
    }

    void visit_ret(ReturnStmt ret) {
        LLVMBuildRet(builder, resolve_value(ret.ret).value);
    }

    LLVMValueRef visit_op(IGValue left, IGValue right, TokenKind op, bool floating) {
        immutable(char*) name = ("op_" ~ newid()).toStringz();

        auto lhs = left.value;
        auto rhs = right.value;

        switch(op) {
            case TokenKind.PLUS: return (floating ? LLVMBuildFAdd(builder, lhs, rhs, name) : LLVMBuildAdd(builder, lhs, rhs, name));
            case TokenKind.DASH: return (floating ? LLVMBuildFSub(builder, lhs, rhs, name) : LLVMBuildSub(builder, lhs, rhs, name));
            case TokenKind.STAR: return (floating ? LLVMBuildFMul(builder, lhs, rhs, name) : LLVMBuildMul(builder, lhs, rhs, name));
            case TokenKind.SLASH: return (floating ? LLVMBuildFDiv(builder, lhs, rhs, name) : LLVMBuildSDiv(builder, lhs, rhs, name));
            case TokenKind.GREATER: return (floating ? LLVMBuildFCmp(builder, LLVMRealUGT, lhs, rhs, name) : LLVMBuildICmp(builder, LLVMIntUGT, lhs, rhs, name));
            case TokenKind.GREATER_EQUALS: return (floating ? LLVMBuildFCmp(builder, LLVMRealUGE, lhs, rhs, name) : LLVMBuildICmp(builder, LLVMIntUGE, lhs, rhs, name));
            case TokenKind.LESS: return (floating ? LLVMBuildFCmp(builder, LLVMRealULT, lhs, rhs, name) : LLVMBuildICmp(builder, LLVMIntULT, lhs, rhs, name));
            case TokenKind.LESS_EQUALS: return (floating ? LLVMBuildFCmp(builder, LLVMRealULE, lhs, rhs, name) : LLVMBuildICmp(builder, LLVMIntULE, lhs, rhs, name));
            case TokenKind.EQUALS: return (floating ? LLVMBuildFCmp(builder, LLVMRealUEQ, lhs, rhs, name) : LLVMBuildICmp(builder, LLVMIntEQ, lhs, rhs, name));
            case TokenKind.NOT_EQUALS: return (floating ? LLVMBuildFCmp(builder, LLVMRealUNE, lhs, rhs, name) : LLVMBuildICmp(builder, LLVMIntNE, lhs, rhs, name));
            case TokenKind.OR: return LLVMBuildOr(builder, lhs, rhs, name);
            case TokenKind.AND: return LLVMBuildAnd(builder, lhs, rhs, name);
            default: assert(false, format("Invalid operation %s", op));
        }
    }
    
    IGValue resolve_value(Expr value) {
        if(auto iexpr = cast(IntExpr)value) {
            auto ival = iexpr.val;
            auto itype = get_type(new SymbolType("i32"));
            return IGValue(LLVMConstInt(itype, ival, false), itype);
        } else if(auto fexpr = cast(FloatExpr)value) {
            auto fval = fexpr.val;
            auto ftype = get_type(new SymbolType("f32"));
            return IGValue(LLVMConstReal(ftype, cast(double)fval), ftype);
        } else if(auto ident = cast(SymbolExpr)value) {
            auto val = current_scope.lookup(ident.value);
            if(val.resolved) {
                return IGValue(LLVMBuildLoad2(builder, val.type, val.value, newid().toStringz()), val.type);
            } else {
                assert(false, format("Failed to resolve %s", ident.value));
            }
        } else if(auto binexpr = cast(BinExpr)value) {
            auto val = visit_binexpr(binexpr);
            return IGValue(val.value, val.type);
        } else if(auto str = cast(StringExpr)value) {
            auto type = LLVMPointerType(type_map["i8"], 0);
            auto val = LLVMBuildPointerCast(builder, LLVMBuildGlobalString(builder, str.value.toStringz(), newid().toStringz()), type, "0".toStringz());
            return IGValue(val, type);
        } else if(auto call = cast(CallExpr)value) {
            auto r = build_call_expr(call);
            if(r.resolved) return r;
            else assert(false, "Attempted to get value from void call");
        }

        assert(false, format("Unsupported value: %s", value));
    }
}