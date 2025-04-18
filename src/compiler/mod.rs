#![allow(unsafe_op_in_unsafe_fn)]

pub mod scope;
pub mod value;
pub mod namegen;

use std::{alloc::{self, Layout}, any::Any, collections::HashMap, ffi::{CStr, CString}, path::Path, process::{self, Command}};

use llvm_sys_180::{core::{LLVMAddFunction, LLVMAddGlobal, LLVMAppendBasicBlock, LLVMAppendBasicBlockInContext, LLVMBuildAdd, LLVMBuildAlloca, LLVMBuildAnd, LLVMBuildBr, LLVMBuildCall2, LLVMBuildCondBr, LLVMBuildFAdd, LLVMBuildFCmp, LLVMBuildFDiv, LLVMBuildFMul, LLVMBuildFSub, LLVMBuildGEP2, LLVMBuildGlobalString, LLVMBuildICmp, LLVMBuildInsertValue, LLVMBuildLoad2, LLVMBuildMul, LLVMBuildNot, LLVMBuildOr, LLVMBuildPointerCast, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildSDiv, LLVMBuildStore, LLVMBuildStructGEP2, LLVMBuildSub, LLVMConstInt, LLVMConstReal, LLVMContextCreate, LLVMCreateBuilderInContext, LLVMDoubleTypeInContext, LLVMFloatTypeInContext, LLVMFunctionType, LLVMGetAggregateElement, LLVMGetBasicBlockParent, LLVMGetInsertBlock, LLVMGetParam, LLVMGetReturnType, LLVMGetStructElementTypes, LLVMGetStructName, LLVMGetTypeKind, LLVMGetValueName, LLVMGetValueName2, LLVMHalfTypeInContext, LLVMIntTypeInContext, LLVMModuleCreateWithNameInContext, LLVMPointerType, LLVMPositionBuilderAtEnd, LLVMPrintModuleToFile, LLVMSetInitializer, LLVMStructType, LLVMStructTypeInContext, LLVMTypeOf, LLVMVoidTypeInContext}, prelude::{LLVMBasicBlockRef, LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef}, LLVMContext, LLVMTypeKind, LLVMValue};
use logos::Logos;
use namegen::{gen_id, gen_id_pre, gen_id_prepost};
use scope::IGScope;
use value::IGValue;

use llvm_sys_180::LLVMRealPredicate as FPredicate;
use llvm_sys_180::LLVMIntPredicate as IPredicate;

use crate::{lexer::{self, Token}, parser::{ast::{Expr, Stmt, Type}, is_kind, Parser}};

type TypeMap = HashMap<String, LLVMTypeRef>;

struct IGLib {
    lib: String,
    _static: bool,
}

pub struct Compiler{
    type_map: TypeMap,
    libs: Vec<IGLib>,

    include_paths: Vec<String>,

    context: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,

    current_scope: IGScope,

    output: String,
    outputs: Vec<String>,
    cwd: String,
}

impl Compiler {
    unsafe fn get_type_map(context: *mut LLVMContext) -> TypeMap {
        let mut type_map = TypeMap::new();

        type_map.insert("bool".into(), LLVMIntTypeInContext(context,1));
        type_map.insert("i8".into(), LLVMIntTypeInContext(context,8));
        type_map.insert("i16".into(), LLVMIntTypeInContext(context, 16));
        type_map.insert("i32".into(), LLVMIntTypeInContext(context, 32));
        type_map.insert("i64".into(), LLVMIntTypeInContext(context, 64));

        type_map.insert("f16".into(), LLVMHalfTypeInContext(context));
        type_map.insert("f32".into(), LLVMFloatTypeInContext(context));
        type_map.insert("f64".into(), LLVMDoubleTypeInContext(context));
        
        type_map.insert("bool".into(), LLVMIntTypeInContext(context, 1));
        type_map.insert("string".into(), LLVMPointerType(LLVMIntTypeInContext(context, 8), 0));
        type_map.insert("void".into(), LLVMVoidTypeInContext(context));

        type_map
    }

    unsafe fn get_type(&self, _type: Type) -> LLVMTypeRef {
        if let Type::Symbol(t) = _type {
            if let Some(t) = self.type_map.get(&t) {
                t.clone()
            } else if let Some(t) = self.current_scope.resolve(t.clone()) {
                t._type
            } else {
                panic!("Invalid type {:?}", t);
            }
        } else if let Type::Ref(t) = _type {
            self.get_type(*t)
        } else {
            panic!("No support for type {:?}", _type);
        }
    }

    unsafe fn get_type_by_name(&self, name: &str) -> LLVMTypeRef {
        self.get_type(Type::Symbol(name.into()))
    }

    unsafe fn new(output: String, include_paths: Vec<String>, cwd: Option<String>) -> Self {
        let context: *mut LLVMContext = LLVMContextCreate();
        let _cwd = if let Some(c) = cwd {
            c
        } else {
            std::env::current_dir().unwrap().to_string_lossy().to_string()
        };

        Self {
            type_map: Self::get_type_map(context),
            
            libs: vec![],

            include_paths,
            outputs: vec![Path::new(&output.clone()).with_extension("ll").to_string_lossy().to_string()],

            current_scope: IGScope::new(None, None, None, None),

            module: LLVMModuleCreateWithNameInContext(get_cstring("ignis".into()), context),
            builder: LLVMCreateBuilderInContext(context),
            context,
            output,
            cwd: _cwd,
        }
    }  

    fn execute_command(cmd: &str, args: Vec<&str>) {
        println!("executing: {} {:?}", cmd, args);

        let out = Command::new(cmd)
            .args(args)
            .output()
            .expect(&format!("Failed to execute ''{}'", cmd));
        if !out.status.success() {
            println!("'{}' executed with code {:?}", cmd, out.status.code().unwrap());
        }
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        if !stdout.is_empty() {
            println!("{}", stdout);
        } 
        if !stderr.is_empty() {
            println!("{}", stderr);
        }
    }

    pub fn compile(output: &Path, ast: Stmt, include_paths: Vec<String>, cwd: Option<String>, inside: bool) -> Self {
        unsafe {
            let mut compiler = Self::new(output.to_string_lossy().to_string(), include_paths, cwd);
            compiler.visit_block(ast);
            compiler.write_ir(&output.with_extension("ll"));

            if !inside {
                let mut obj_files = vec![];
                for out in compiler.outputs.iter() {
                    let path = Path::new(&out.clone()).with_extension("o");
                    obj_files.push(path.to_string_lossy().to_string());
                    Self::execute_command("llc", vec!["--filetype=obj", &out, "-o", path.to_str().unwrap()]);
                }
                
                let mut comp_arg = vec!["-o", output.to_str().unwrap(), "-no-pie"];

                for obj in &obj_files {
                    comp_arg.push(obj);
                }

                let mut libs = vec![];

                for lib in compiler.libs.iter() {
                    libs.push(if lib._static {
                        lib.lib.clone()
                    } else {
                        ["-l", &lib.lib].join("")
                    });
                }
                
                for lib in libs.iter() {
                    comp_arg.push(lib);
                }

                Self::execute_command("gcc", comp_arg);

                for obj in &obj_files {
                    std::fs::remove_file(obj).expect("Failed to remove obj files");
                }

                for out in compiler.outputs.clone() {
                    // std::fs::remove_file(out).expect("Failed to remove llvm ir files");
                }
            }

            compiler

        }
    }

    unsafe fn visit_block(&mut self, stmt: Stmt) {
        if let Stmt::Block(block) = stmt {
            for s in block {
                self.visit(s);
            }
        } else {
            panic!("Expected block");
        }
    }

    unsafe fn visit(&mut self, stmt: Stmt) {
        if let Stmt::Expression(expr) = stmt {
            self.visit_expression(*expr);
        } else if let Stmt::FunctionDeclaration { .. } = stmt {
            self.visit_function_declaration(stmt.clone());
        } else if let Stmt::Return { .. } = stmt {
            self.visit_return(stmt.clone());
        } else if let Stmt::VariableDeclaration { .. } = stmt {
            self.visit_variable_declaration(stmt.clone(), true);
        } else if let Stmt::Extern { .. } = stmt {
            self.visit_extern(stmt.clone());   
        } else if let Stmt::Include { .. } = stmt {
            self.visit_include(stmt.clone());
        } else if let Stmt::Link { library, _static } = stmt {
            self.libs.push(IGLib { lib: library, _static: _static })
        } else if let Stmt::While { .. } = stmt {
            self.visit_while(stmt.clone());  
        } else if let Stmt::StructDeclaration { .. } = stmt {
            self.visit_struct_declaration(stmt.clone());  
        } else {
            panic!("Unsupported statement: {:?}", stmt);
        }
    }

    unsafe fn visit_expression(&mut self, expr: Expr) {
        if let Expr::Binary { .. } = expr {
            self.visit_binexpr(expr);
        } else if let Expr::Assignment { .. } = expr {
            self.visit_assignment_expr(expr);
        } else if let Expr::Call { .. } = expr {
            self.visit_call_expr(expr);
        } else {
            panic!("Unsupported expression: {:?}", expr);
        }
    }

    unsafe fn visit_struct_declaration(&mut self, stmt: Stmt) {
        let Stmt::StructDeclaration { name, fields, functions } = stmt else {
            panic!("Expected struct declaration");
        };

        let field_types = self.get_arg_types(fields.clone());
        let _type = LLVMStructTypeInContext(self.context,field_types.clone().as_mut_ptr(), field_types.len() as u32, 0);

        self.current_scope.define_type(name.clone(), _type, false, true);

        for (i, field_stmt) in fields.iter().enumerate() {
            let Stmt::Field { name: _name, _type } = field_stmt else {
                panic!("Expected field");
            };
            let mut newname= name.clone();
            newname.push_str(".");
            newname.push_str(&_name);


            self.current_scope.define_field(newname, i);
        }
    }

    unsafe fn visit_extern(&mut self, stmt: Stmt) {
        let Stmt::Extern { name, symbol, return_type, arguments } = stmt else {
            panic!("Expected extern");
        };

        let types = self.get_arg_types(arguments.clone());
        let ret_type = self.get_type(*return_type);
        let func_type = LLVMFunctionType(ret_type, types.clone().as_mut_ptr(), types.len() as u32, 0);
        let func = LLVMAddFunction(self.module, get_cstring(symbol), func_type);

        self.current_scope.define(name, func, func_type, false, true);
    }

    unsafe fn visit_while(&mut self, stmt: Stmt) {
        let Stmt::While { condition, body } = stmt else {
            panic!("Expected while statement");
        };

        let thenbb = self.create_basic_block("while_inside".into());
        let outisdebb = self.create_basic_block("while_outside".into());
    
        LLVMBuildBr(self.builder, thenbb);

        LLVMPositionBuilderAtEnd(self.builder, thenbb);
        self.visit_block(*body);
        let cond = self.visit_conditional(*condition);
        LLVMBuildCondBr(self.builder, cond.value, thenbb, outisdebb);
        LLVMPositionBuilderAtEnd(self.builder, outisdebb);
    }

    unsafe fn create_basic_block(&mut self, name: String) -> LLVMBasicBlockRef {
        let realname = gen_id_pre(name);
        LLVMAppendBasicBlockInContext(self.context, self.get_current_function(), realname)
    }

    unsafe fn get_current_function(&mut self) -> LLVMValueRef {
        let bl = LLVMGetInsertBlock(self.builder);
        LLVMGetBasicBlockParent(bl)
    }

    unsafe fn visit_conditional(&mut self, expr: Expr) -> IGValue {
        if let Expr::Binary { .. } = expr {
            self.visit_binexpr(expr)
        } else if let Expr::Bool(_) = expr {
            self.resolve_value(expr)
        } else if let Expr::Prefix { .. } = expr {
            self.visit_prefix(expr)
        } else {
            panic!("Expected conditional statement, got {:?}", expr);
        }
    }

    fn get_include_path(&self, inc: String) -> String {
        if std::fs::exists(inc.clone()).unwrap() {
            return inc;
        }

        let cwd_path = Path::new(&self.cwd).join(inc.clone());
        if std::fs::exists(cwd_path.clone()).unwrap() {
            return cwd_path.to_string_lossy().to_string();
        }

        for path in &self.include_paths {
            let inc_path = Path::new(path).join(inc.clone());
            if std::fs::exists(inc_path.clone()).unwrap() {
                return inc_path.to_string_lossy().to_string();
            }
        }

        panic!("Failed to include source file {:?}", inc);
    }

    unsafe fn visit_prefix(&mut self, expr: Expr) -> IGValue {
        let Expr::Prefix { op, right } = expr else {
            panic!("Expected prefix expression");
        };

        let rhs = self.resolve_value(*right);

        let val = match op {
            Token::Not => LLVMBuildNot(self.builder, rhs.value, gen_id()),
            _ => panic!("Unsupported prefix expression: {:?}", op)
        };

        IGValue::new(val, self.get_type_by_name("bool"))
    }

    unsafe fn visit_include(&mut self, stmt: Stmt) {
        let Stmt::Include { path } = stmt else {
            panic!("Expected include");
        };

        let inc = self.get_include_path(path);
        let inc_path = Path::new(&inc);

        let outpath = Path::new(&self.output).parent().unwrap();
        let source = std::fs::read_to_string(inc_path).expect("Failed to include file<NOT FOUND>");
        let lexer = lexer::Token::lexer(&source);
        let mut tokens = vec![];

        for t in lexer {
            match t {
                Ok(tok) => tokens.push(tok),
                Err(e) => {
                    if !e.is_empty() {
                        println!("Invalid token: {}", e);
                        return;
                    }
                },
            }
        }

        tokens.push(Token::EOF);

        let ast = Parser::parse(tokens);
        let partial = outpath.join(inc_path.to_string_lossy().to_string().replace("\\", "_").replace("/", "_"));
        let mut compiler = Self::compile(Path::new(&partial), ast, self.include_paths.clone(), Some(inc_path.parent().unwrap().to_string_lossy().to_string()), true);
        for (name, value) in compiler.current_scope.symbols {
            let tk = LLVMGetTypeKind(LLVMTypeOf(value.clone().value));
            if tk == LLVMTypeKind::LLVMPointerTypeKind && value.public {
                let func = LLVMAddFunction(self.module,  LLVMGetValueName(value.value), value._type);
                self.current_scope.define(name, func, value._type, false, false);
            } else {
                self.current_scope.define(name, value.value, value._type, false, false);
            }
        }
        for (name, i) in compiler.current_scope.fields {
            self.current_scope.fields.insert(name, i);
        }

        self.libs.append(&mut compiler.libs);
        self.outputs.append(&mut compiler.outputs);
    }

    unsafe fn visit_assignment_expr(&mut self, expr: Expr) {
        let Expr::Assignment { assignee, right } = &expr else {
            panic!("Expected assignment expression");
        };

        let ptr = if let Expr::Symbol(name) = *assignee.clone() {
            self.current_scope.resolve(name.clone()).unwrap()
        } else if let Expr::Access { .. } = *assignee.clone() {
            &self.resolve_value(*assignee.clone())
        } else {
            panic!("Invalid expression on LHS of assignment, got {:?}", expr);
        };

        let new_ptr = ptr.clone();

        let val = self.resolve_value(*right.clone());
        
        if !new_ptr.mutable {
            panic!("Attempted to assign to an immutable variable {:?}", assignee);
        }
        LLVMBuildStore(self.builder, val.value, new_ptr.value);
    }

    unsafe fn visit_call_expr(&mut self, expr: Expr) -> Option<IGValue> {
        let Expr::Call { name, args } = expr else {
            panic!("Expected call expression");
        };

        let Some(f) = self.current_scope.resolve(name.clone()) else {
            panic!("Failed to resolve function {:?}", name);
        };

        let f_type = f.clone()._type;
        let f_value = f.clone().value;

        let args = self.get_arg_values(args);

        if LLVMGetTypeKind(self.get_type_by_name("void")) == LLVMGetTypeKind(LLVMGetReturnType(f_type)) {
            LLVMBuildCall2(self.builder, f_type, f_value, args.clone().as_mut_ptr(), args.len() as u32, get_cstring("".into()));   
            return None;
        }

        Some(IGValue::new(LLVMBuildCall2(self.builder, f_type, f_value, args.clone().as_mut_ptr(), args.len() as u32, gen_id()), f_type))
    }

    unsafe fn visit_variable_declaration(&mut self, stmt: Stmt, define: bool) {
        let Stmt::VariableDeclaration { name, explicit_type, value } = stmt else {
            panic!("Expected variable declaration");
        };

        if !self.current_scope.resolve(name.clone()).is_some() {
            let val = self.resolve_value(*value);
            let alloca = LLVMBuildAlloca(self.builder, val._type, gen_id_pre(name.clone()));
            LLVMBuildStore(self.builder, val.value, alloca);
            if define {
                if val.parent.is_some() {
                    self.current_scope.define_struct(name, val.value, val._type, true, true, val.parent.unwrap());
                } else {
                    self.current_scope.define(name, alloca, val._type, true, true);
                }
            }
        } else {
            panic!("Cannot redefine variable {:?}", name);
        }
    }

    unsafe fn get_arg_types(&self, args: Vec<Stmt>) -> Vec<LLVMTypeRef> {
        let mut types = vec![];
        for arg in args {
            let Stmt::Field { name, _type } = arg else {
                panic!("Expected field in args");
            };

            types.push(self.get_type(*_type));
        }
        types
    }

    
    unsafe fn get_arg_values(&mut self, args: Vec<Expr>) -> Vec<LLVMValueRef> {
        let mut values = vec![];
        for arg in args {
            values.push(self.resolve_value(arg).value);
        }
        values
    }

    unsafe fn visit_function_declaration(&mut self, stmt: Stmt) {
        let Stmt::FunctionDeclaration { name, return_type, arguments, body } = stmt else {
            panic!("Expected function declaration");
        };

        let arg_types = self.get_arg_types(arguments.clone());
        let mut ret_type = self.get_type(*return_type.clone());

        if self.get_type_by_name("void") == ret_type && name == "main" {
            ret_type = self.get_type(Type::Symbol("i32".into()));
        }
    
        let func_type = LLVMFunctionType(ret_type, arg_types.clone().as_mut_ptr(), arg_types.len() as u32, 0);
        let func = LLVMAddFunction(self.module, get_cstring(name.clone()), func_type);
        let block = LLVMAppendBasicBlockInContext(self.context, func, gen_id_prepost(name.clone(), "ignis_entry".into()));

        let outer_scope = self.current_scope.clone();

        self.current_scope = IGScope::new(None, None, Some(name.clone()), Some(Box::new(outer_scope.clone())));
        LLVMPositionBuilderAtEnd(self.builder, block);
        
        for (i, s) in arguments.iter().enumerate() {
            let Stmt::Field { name, _type } = s else {
                panic!("Expected field in args");
            };

            let t = arg_types[i];
            let alloca = LLVMBuildAlloca(self.builder, t, gen_id_pre(name.clone()));
            LLVMBuildStore(self.builder, LLVMGetParam(func, i as u32), alloca);
            self.current_scope.define(name.clone(), alloca, t, false, true);
        }

        self.current_scope.define(name.clone(), func, func_type, false, true);
        self.visit_block(*body);

        if self.get_type_by_name("void") == self.get_type(*return_type) && name == "main" {
            LLVMBuildRet(self.builder, self.resolve_value(Expr::Int(0)).value);
        } else if self.get_type_by_name("void") == ret_type {
            LLVMBuildRetVoid(self.builder);

        }

        self.current_scope = outer_scope;
        self.current_scope.define(name.clone(), func, func_type, false, true);
    }

    unsafe fn visit_return(&mut self, stmt: Stmt) {
        let Stmt::Return { value } = stmt else {
            panic!("Expected return");
        };

        LLVMBuildRet(self.builder, self.resolve_value(*value).value);
    } 

    
    unsafe fn visit_op(&mut self, left: IGValue, right: IGValue, op: Token, floating: bool) -> LLVMValueRef {
        let name = gen_id_pre("op".into());
        let lhs = left.value;
        let rhs = right.value;

        match op {
            Token::Plus => if floating { LLVMBuildFAdd(self.builder, lhs, rhs, name) } else { LLVMBuildAdd(self.builder, lhs, rhs, name) }
            Token::Minus => if floating { LLVMBuildFSub(self.builder, lhs, rhs, name) } else { LLVMBuildSub(self.builder, lhs, rhs, name) }
            Token::Multiply => if floating { LLVMBuildFMul(self.builder, lhs, rhs, name) } else { LLVMBuildMul(self.builder, lhs, rhs, name) }
            Token::Divide => if floating { LLVMBuildFDiv(self.builder, lhs, rhs, name) } else { LLVMBuildSDiv(self.builder, lhs, rhs, name) }
            Token::Greater => if floating { LLVMBuildFCmp(self.builder, FPredicate::LLVMRealUGT, lhs, rhs, name) } else { LLVMBuildICmp(self.builder, IPredicate::LLVMIntUGT, lhs, rhs, name) }
            Token::GreaterOrEqual => if floating { LLVMBuildFCmp(self.builder, FPredicate::LLVMRealUGE, lhs, rhs, name) } else { LLVMBuildICmp(self.builder, IPredicate::LLVMIntUGE, lhs, rhs, name) }
            Token::Less => if floating { LLVMBuildFCmp(self.builder, FPredicate::LLVMRealULT, lhs, rhs, name) } else { LLVMBuildICmp(self.builder, IPredicate::LLVMIntULT, lhs, rhs, name) }
            Token::LessOrEqual => if floating { LLVMBuildFCmp(self.builder, FPredicate::LLVMRealULE, lhs, rhs, name) } else { LLVMBuildICmp(self.builder, IPredicate::LLVMIntULE, lhs, rhs, name) }
            Token::Equals => if floating { LLVMBuildFCmp(self.builder, FPredicate::LLVMRealUEQ, lhs, rhs, name) } else { LLVMBuildICmp(self.builder, IPredicate::LLVMIntEQ, lhs, rhs, name) }
            Token::NotEquals => if floating { LLVMBuildFCmp(self.builder, FPredicate::LLVMRealUNE, lhs, rhs, name) } else { LLVMBuildICmp(self.builder, IPredicate::LLVMIntNE, lhs, rhs, name) }
            Token::Or => LLVMBuildOr(self.builder, lhs, rhs, name),
            Token::And => LLVMBuildAnd(self.builder, lhs, rhs, name),
            _ => panic!("Invalid operation {:?}", op),
        }
    }

    unsafe fn visit_binexpr(&mut self, binexpr: Expr) -> IGValue {
        let Expr::Binary { left, op, right } = binexpr else {
            panic!("Expected binary expression");
        };

        let lvalue = self.resolve_value(*left.clone());
        let rvalue = self.resolve_value(*right.clone());

        let _type: LLVMTypeRef;
        
        fn is_bool(op: &Token) -> bool {
            is_kind(op, &Token::Equals) |
            is_kind(op, &Token::NotEquals) |
            is_kind(op, &Token::Less) |
            is_kind(op, &Token::LessOrEqual) |
            is_kind(op, &Token::Greater) |
            is_kind(op, &Token::GreaterOrEqual) |
            is_kind(op, &Token::Not) |
            is_kind(op, &Token::Or) |
            is_kind(op, &Token::And)
        }

        if is_bool(&op) {
            _type = self.get_type_by_name("bool");
        } else {
            _type = lvalue._type;
        }

        if lvalue.are_both(rvalue.clone(), self.get_type_by_name("i32")) {
            IGValue::new(self.visit_op(lvalue, rvalue, op, false), _type)
        } else if lvalue.are_both(rvalue.clone(), self.get_type_by_name("f32")) {
            IGValue::new(self.visit_op(lvalue, rvalue, op, true), _type)
        } else {
            panic!("Unsupported operation '{:?}' between {:?} and {:?}", op, left, right);
        }
    }

    unsafe fn resolve_mutable_symbol(&mut self, expr: Expr) -> IGValue {
        let Expr::Symbol(symbol) = expr else {
            panic!("Expectd symbol but got {:?}", expr);
        };

        let Some(val) = self.current_scope.resolve(symbol.clone()) else {
            panic!("Failed to resolve symbol: {:?}", symbol);
        };
        IGValue::new(
            LLVMBuildLoad2(self.builder, LLVMPointerType(val._type, 0), val.value, gen_id()),
            val._type
        )
    }

    unsafe fn resolve_value(&mut self, value: Expr) -> IGValue {
        if let Expr::Int(i) = value {
            let _type = self.get_type_by_name("i32");
            IGValue::new(LLVMConstInt(_type, i as u64, 0), _type)
        } else if let Expr::Float(f) = value {
            let _type = self.get_type_by_name("f32");
            IGValue::new(LLVMConstReal(_type, f), _type)
        } else if let Expr::Symbol(symbol) = value {
            let Some(val) = self.current_scope.resolve(symbol.clone()) else {
                panic!("Failed to resolve symbol: {:?}", symbol);
            };
            IGValue::new(
                LLVMBuildLoad2(self.builder, val._type, val.value, gen_id()),
                val._type
            )
        } else if let Expr::Bool(b) = value {
            let _type = self.get_type_by_name("bool");
            IGValue::new(LLVMConstInt(_type, b as u64, 0), _type)
        } else if let Expr::Binary { .. } = value {
            self.visit_binexpr(value)
        } else if let Expr::String(s) = value.clone() {
            let _type = LLVMPointerType(self.get_type_by_name("i8"), 0);
            let val = LLVMBuildPointerCast(self.builder, LLVMBuildGlobalString(self.builder, get_cstring(s), gen_id()), _type, gen_id());
            IGValue::new(val, _type)
        } else if let Expr::Call { .. } = value.clone() {
            self.visit_call_expr(value).expect("Expected value from call expression")
        } else if let Expr::StructInitialize { name, fields } = value.clone() {
            let _type = self.current_scope.resolve(name.clone()).expect(&format!("Failed to find type '{}' in context", name)).clone();
            let alloca = LLVMBuildAlloca(self.builder, _type._type, gen_id());

            for (i, field) in fields.iter().enumerate() {
                let Stmt::StructInitField { name, value } = field else {
                    panic!("Expected struct init field");
                };

                let val = self.resolve_value(*value.clone());

                let ptr = LLVMBuildStructGEP2(self.builder, _type._type, alloca, i as u32, gen_id());
                LLVMBuildStore(self.builder, val.value, ptr);
            }

            IGValue::new_struct(alloca, _type._type, name)
        } else if let Expr::Access { lhs, rhs } = value.clone() {
            let Expr::Symbol(sname) = *lhs else {
                panic!("Only single depth access expressions allowed right now");
            };
            let Expr::Symbol(fname) = *rhs else {
                panic!("Expected symbol on RHS of member access expression");
            };

            
            let val = self.current_scope.resolve(sname.clone());
            let Some(value) = val else {
                panic!("Failed to resolve {:?}", sname);
            };

            println!("PARENT {} {:?}", sname, value);
            let Some(parent) = value.parent.clone() else {
                panic!("Failed to get member type")
            };

            let mut fullname = parent.clone();
            fullname.push('.');
            fullname.push_str(&fname);

            let Some(index) = self.current_scope.resolve_field(fullname.clone()) else {
                panic!("Failed to resolve member {}", fullname);
            };

            let ptr = LLVMBuildStructGEP2(self.builder, value._type, value.value, *index as u32, gen_id());
            IGValue::new(ptr, value._type)
        } else {
            panic!("Unsupported value: {:?}", value);
        }

    }

    unsafe fn write_ir(&self, output: &Path) {
        let mut err: *mut i8 = alloc::alloc(Layout::array::<i8>(256).unwrap()) as *mut i8;
        LLVMPrintModuleToFile(self.module, get_cstring(output.to_string_lossy().to_string()), &mut err);
    }
}

fn get_cstring(s: String) -> *mut i8 {
    let cs = CString::new(s).unwrap();
    let cv: Vec<u8> = cs.into_bytes_with_nul();
    let mut tmp: Vec<i8> = cv.into_iter().map(|c| c as i8).collect::<_>();
    tmp.as_mut_ptr()
}

fn is_program_in_path(program: &str) -> bool {
    if let Ok(path) = std::env::var("PATH") {
        for p in path.split(":") {
            let p_str = format!("{}/{}", p, program);
            if std::fs::metadata(p_str).is_ok() {
                return true;
            }
        }
    }
    false
}  