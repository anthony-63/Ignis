#![allow(unsafe_op_in_unsafe_fn)]

pub mod scope;
pub mod value;
pub mod namegen;

use std::{alloc::{self, Layout}, collections::HashMap, ffi::CString, path::Path};

use llvm_sys_180::{core::{LLVMAddFunction, LLVMAppendBasicBlock, LLVMAppendBasicBlockInContext, LLVMBuildAlloca, LLVMBuildAnd, LLVMBuildFAdd, LLVMBuildFCmp, LLVMBuildFDiv, LLVMBuildFMul, LLVMBuildFSub, LLVMBuildGlobalString, LLVMBuildICmp, LLVMBuildLoad2, LLVMBuildOr, LLVMBuildPointerCast, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildSDiv, LLVMBuildStore, LLVMConstInt, LLVMConstReal, LLVMContextCreate, LLVMCreateBuilderInContext, LLVMDoubleTypeInContext, LLVMFloatTypeInContext, LLVMFunctionType, LLVMGetParam, LLVMHalfTypeInContext, LLVMIntTypeInContext, LLVMModuleCreateWithNameInContext, LLVMPointerType, LLVMPositionBuilderAtEnd, LLVMPrintModuleToFile, LLVMVoidTypeInContext}, prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef}, LLVMContext};
use namegen::{gen_id, gen_id_pre, gen_id_prepost};
use scope::IGScope;
use value::IGValue;

use llvm_sys_180::LLVMRealPredicate as FPredicate;
use llvm_sys_180::LLVMIntPredicate as IPredicate;

use crate::{lexer::Token, parser::{ast::{Expr, Stmt, Type}, is_kind}};

type TypeMap = HashMap<String, LLVMTypeRef>;

struct IGLib {
    lib: String,
    _static: bool,
}

pub struct Compiler{
    type_map: TypeMap,
    libs: Vec<IGLib>,

    included: Vec<Compiler>,
    include_paths: Vec<String>,

    context: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,

    current_scope: IGScope,
}

impl Compiler {
    unsafe fn get_type_map(context: *mut LLVMContext) -> TypeMap {
        let mut type_map = TypeMap::new();

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
            *self.type_map.get(&t).unwrap_or_else(|| panic!("Invalid type {:?}", t))
        } else if let Type::Ref(t) = _type {
            self.get_type(*t)
        } else {
            panic!("No support for type {:?}", _type);
        }
    }

    unsafe fn get_type_by_name(&self, name: &str) -> LLVMTypeRef {
        self.get_type(Type::Symbol(name.into()))
    }

    unsafe fn new() -> Self {
        let context: *mut LLVMContext = LLVMContextCreate();

        Self {
            type_map: Self::get_type_map(context),
            
            libs: vec![],

            included: vec![],
            include_paths: vec![],

            current_scope: IGScope::new(None, None, None),

            module: LLVMModuleCreateWithNameInContext(get_cstring("ignis".into()), context),
            builder: LLVMCreateBuilderInContext(context),
            context,
        }
    }

    pub fn compile(output: &Path, ast: Stmt) {
        unsafe {
            let mut compiler = Self::new();
            compiler.visit_block(ast);
            compiler.write_ir(&output.with_extension("ll"));
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
        if let Stmt::FunctionDeclaration { .. } = stmt {
            self.visit_function_declaration(stmt.clone());
        } else if let Stmt::Return { .. } = stmt {
            self.visit_return(stmt.clone());
        } else if let Stmt::VariableDeclaration { .. } = stmt {
            self.visit_variable_declaration(stmt.clone(), true);
        } else {
            panic!("Unsupported statement: {:?}", stmt);
        }
    }

    unsafe fn visit_variable_declaration(&mut self, stmt: Stmt, define: bool) {
        let Stmt::VariableDeclaration { name, mutable, explicit_type, value } = stmt else {
            panic!("Expected variable declaration");
        };

        if !self.current_scope.resolve(name.clone()).is_some() {
            let val = self.resolve_value(*value);
            let alloca = LLVMBuildAlloca(self.builder, val._type, gen_id_pre(name.clone()));
            LLVMBuildStore(self.builder, val.value, alloca);
            if define {
                self.current_scope.define(name, val.value, val._type, mutable, true);
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

        self.current_scope = IGScope::new(None, Some(name.clone()), Some(Box::new(outer_scope.clone())));
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
            Token::Plus => if floating { LLVMBuildFAdd(self.builder, lhs, rhs, name) } else { LLVMBuildFAdd(self.builder, lhs, rhs, name) }
            Token::Minus => if floating { LLVMBuildFSub(self.builder, lhs, rhs, name) } else { LLVMBuildFSub(self.builder, lhs, rhs, name) }
            Token::Multiply => if floating { LLVMBuildFMul(self.builder, lhs, rhs, name) } else { LLVMBuildFMul(self.builder, lhs, rhs, name) }
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
        } else if let Expr::Binary { .. } = value {
            self.visit_binexpr(value)
        } else if let Expr::String(s) = value.clone() {
            let _type = LLVMPointerType(self.get_type_by_name("i8"), 0);
            let val = LLVMBuildPointerCast(self.builder, LLVMBuildGlobalString(self.builder, get_cstring(s), gen_id()), _type, gen_id());
            IGValue::new(val, _type)
        } else if let Expr::Call { name, args } = value.clone() {
            panic!("call expressions not supported yet");
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