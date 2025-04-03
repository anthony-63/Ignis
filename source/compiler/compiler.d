module compiler.compiler;

import std.file;
import std.conv;
import std.string;
import std.process;
import std.stdio : writeln;
import std.array;
import std.format;
import std.algorithm;
import std.typecons;
import std.variant;

import lexer.tokens;

import ast.ast;
import ast.types;
import ast.statements;
import ast.expressions;

import llvm;

import compiler.igscope;
import compiler.value;

class Compiler {
    Stmt[] ast;

    LLVMTypeRef[string] type_map;

    LLVMContextRef ctx;

    LLVMModuleRef _module;
    LLVMBuilderRef builder;

    IGScope current_scope;

    this(BlockStmt root) {
        ast = root.body;

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
        ];
    }

    LLVMTypeRef get_type(Type type) {
        if(auto t = cast(SymbolType)type) {
            return type_map[t.name];
        } else {
            assert(false, format("No support for %s types", type));
        }
    }

    void compile(string output) {
        foreach(stmt; ast) visit(stmt);
        
        auto ll_file = output ~ ".ll";
        auto asm_file = output ~ ".s";

        LLVMPrintModuleToFile(_module, ll_file.toStringz(), null);
        auto pid = spawnProcess(["llc", ll_file, "-o", asm_file]);
        wait(pid);
        pid = spawnProcess(["gcc", asm_file, "-o", output]);
        wait(pid);
        // remove(ll_file);
        remove(asm_file);
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
        } 
        // else if(auto ifstmt = cast(IfStmt)stmt) {
        //     visit_if(ifstmt);  
        // } 
        else {
            assert(false, format("Unsupported statement %s", stmt));
        }
    }

    void visit_if(IfStmt stmt) {
        // TODO
    }

    void visit_expr(Expr expr) {
        if(auto binexpr = cast(BinExpr)expr) {
            visit_binexpr(binexpr);
        } else if(auto assign = cast(AssignmentExpr)expr) {
            visit_assign(assign);
        }
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

    IGValue visit_binexpr(BinExpr binexpr) {
        auto op = binexpr.op.kind;
        auto lvalue = resolve_value(binexpr.left);
        auto rvalue = resolve_value(binexpr.right);
        auto type = lvalue.type;

        if(lvalue.are_both(rvalue, get_type(new SymbolType("i32")))) {
            return IGValue(visit_op(lvalue, rvalue, op, false), type);
        } else if(lvalue.are_both(rvalue, get_type(new SymbolType("i32")))) {
            return IGValue(visit_op(lvalue, rvalue, op, true), type);
        } else {
            assert(false, format("Unsupported operation between %s and %s", binexpr.left, binexpr.right));
        }
    }

    void visit_var_decl(VarDeclStmt decl) {
        auto name = decl.ident;
        auto val = resolve_value(decl.value);
        
        if(!current_scope.lookup(name).resolved) {
            auto alloca = LLVMBuildAlloca(builder, val.type, name.toStringz());
            LLVMBuildStore(builder, val.value, alloca);
            current_scope.define(name, alloca, val.type, decl.mutable);
        } else {
            assert(false, format("Cannot redefine variable %s", name));
        }
    }

    void visit_function_decl(FuncDeclStmt fdecl) {
        auto arg_names = fdecl.args.map!("a.ident");
        auto arg_types_ig = fdecl.args.map!("a.type");
        
        LLVMTypeRef[] arg_types = [];
        foreach(type; arg_types_ig) {
            arg_types ~= get_type(type);
        }

        auto ret_type = get_type(fdecl.return_type);
        auto func_type = LLVMFunctionType(get_type(fdecl.return_type), arg_types.ptr, cast(uint)arg_types.length, 0);
        auto func = LLVMAddFunction(_module, fdecl.ident.toStringz(), func_type);
        auto block = LLVMAppendBasicBlockInContext(ctx, func, (fdecl.ident ~ "_ignis_entry").toStringz());
        
        auto outer_scope = current_scope.copy();
        
        LLVMPositionBuilderAtEnd(builder, block);
        current_scope = new IGScope();
        current_scope.name = fdecl.ident;
        current_scope.parent = outer_scope;

        current_scope.define(fdecl.ident, func, ret_type);
        foreach(stmt; fdecl.body) {
            visit(stmt);
        }

        current_scope = outer_scope;
        current_scope.define(fdecl.ident, func, ret_type);
    }

    void visit_ret(ReturnStmt ret) {
        LLVMBuildRet(builder, resolve_value(ret.ret).value);
    }

    LLVMValueRef visit_op(IGValue left, IGValue right, TokenKind op, bool floating) {
        LLVMValueRef function(LLVMBuilderRef builder, LLVMValueRef lhs, LLVMValueRef rhs, immutable(char*) name)[TokenKind] build_lu;
        immutable(char*)[TokenKind] name_lu = [
            TokenKind.PLUS: "add_tmp".toStringz(),
            TokenKind.DASH: "sub_tmp".toStringz(),
            TokenKind.STAR: "mul_tmp".toStringz(),
            TokenKind.SLASH: "div_tmp".toStringz(),
        ];

        if(floating) {
            build_lu = [
                TokenKind.PLUS: (builder, lhs, rhs, name) { return LLVMBuildFAdd(builder, lhs, rhs, name); },
                TokenKind.DASH: (builder, lhs, rhs, name) { return LLVMBuildFSub(builder, lhs, rhs, name); },
                TokenKind.STAR: (builder, lhs, rhs, name) { return LLVMBuildFMul(builder, lhs, rhs, name); },
                TokenKind.SLASH: (builder, lhs, rhs, name) { return LLVMBuildFDiv(builder, lhs, rhs, name); },
            ];
        } else {
            build_lu = [
                TokenKind.PLUS: (builder, lhs, rhs, name) { return LLVMBuildAdd(builder, lhs, rhs, name); },
                TokenKind.DASH: (builder, lhs, rhs, name) { return LLVMBuildSub(builder, lhs, rhs, name); },
                TokenKind.STAR: (builder, lhs, rhs, name) { return LLVMBuildMul(builder, lhs, rhs, name); },
                TokenKind.SLASH: (builder, lhs, rhs, name) { return LLVMBuildSDiv(builder, lhs, rhs, name); },
            ];
        }
        return build_lu[op](builder, left.value, right.value, name_lu[op]);
    }
    
    IGValue resolve_value(Expr value) {
        if(auto iexpr = cast(IntExpr)value) {
            auto ival = iexpr.val;
            auto itype = get_type(new SymbolType("i32"));
            return IGValue(LLVMConstInt(itype, ival, false), itype);
        } else if(auto fexpr = cast(FloatExpr)value) {
            auto fval = fexpr.val;
            auto ftype = get_type(new SymbolType("i32"));
            return IGValue(LLVMConstReal(ftype, cast(double)fval), ftype);
        } else if(auto ident = cast(SymbolExpr)value) {
            auto val = current_scope.lookup(ident.value);
            if(val.resolved) {
                return IGValue(LLVMBuildLoad2(builder, val.type, val.value, ident.value.toStringz()), val.type);
            } else {
                assert(false, format("Failed to resolve %s", ident.value));
            }
        } else if(auto binexpr = cast(BinExpr)value) {
            auto val = visit_binexpr(binexpr);
            return IGValue(val.value, val.type);
        } else if(auto str = cast(StringExpr)value) {
            auto type = LLVMPointerType(type_map["i8"], 0);
            auto val = LLVMBuildPointerCast(builder, LLVMBuildGlobalString(builder, str.value.toStringz(), "".toStringz()), type, "0".toStringz());
            return IGValue(val, type);
        }

        assert(false, format("Unsupported value: %s", value));
    }
}