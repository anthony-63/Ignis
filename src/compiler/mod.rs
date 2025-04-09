#![allow(unsafe_op_in_unsafe_fn)]

pub mod scope;
pub mod value;
pub mod namegen;

use std::{alloc::{self, Layout}, collections::HashMap, ffi::CString, path::Path};

use llvm_sys_180::{core::{LLVMAddFunction, LLVMAppendBasicBlock, LLVMAppendBasicBlockInContext, LLVMBuildAlloca, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildStore, LLVMConstInt, LLVMContextCreate, LLVMCreateBuilderInContext, LLVMDoubleTypeInContext, LLVMFloatTypeInContext, LLVMFunctionType, LLVMGetParam, LLVMHalfTypeInContext, LLVMIntTypeInContext, LLVMModuleCreateWithNameInContext, LLVMPointerType, LLVMPositionBuilderAtEnd, LLVMPrintModuleToFile, LLVMVoidTypeInContext}, prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef}, LLVMContext};
use namegen::{gen_id_pre, gen_id_prepost};
use scope::IGScope;
use value::IGValue;

use crate::parser::ast::{Expr, Stmt, Type};

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
        } if let Stmt::Return { .. } = stmt {
            self.visit_return(stmt.clone());
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

    unsafe fn resolve_value(&mut self, value: Expr) -> IGValue {
        if let Expr::Int(i) = value {
            let _type = self.get_type_by_name("i32");
            return IGValue::new(LLVMConstInt(_type, i as u64, 0), _type);
        }

        panic!("Unsupported value: {:?}", value);
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