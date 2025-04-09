#![allow(unsafe_op_in_unsafe_fn)]

pub mod scope;
pub mod value;

use std::{alloc::{self, Layout}, collections::HashMap, ffi::CString, path::Path};

use llvm_sys_180::{core::{LLVMContextCreate, LLVMCreateBuilderInContext, LLVMDoubleTypeInContext, LLVMFloatTypeInContext, LLVMHalfTypeInContext, LLVMInt8TypeInContext, LLVMIntType, LLVMIntTypeInContext, LLVMModuleCreateWithNameInContext, LLVMPointerType, LLVMPointerTypeInContext, LLVMPrintModuleToFile, LLVMVoidTypeInContext}, orc2::LLVMOrcCAPIDefinitionGeneratorTryToGenerateFunction, prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef}, LLVMContext};
use scope::IGScope;

use crate::parser::ast::Stmt;

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
            let compiler = Self::new();
            compiler.write_ir(&output.with_extension("ll"));
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