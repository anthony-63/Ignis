#![allow(unsafe_op_in_unsafe_fn)]

extern crate llvm_sys_180 as llvm_sys;

pub mod scope;

use std::{alloc::{self, Layout}, collections::HashMap, ffi::CString, path::Path};

use llvm_sys::{core::{LLVMContextCreate, LLVMCreateBuilder, LLVMCreateBuilderInContext, LLVMModuleCreateWithNameInContext}, prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef}};
use llvm_sys_180::core::LLVMPrintModuleToFile;
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
    fn get_type_map() -> TypeMap {
        let type_map = TypeMap::new();
        type_map
    }

    unsafe fn new() -> Self {
        let context: *mut llvm_sys::LLVMContext = LLVMContextCreate();

        Self {
            type_map: Self::get_type_map(),
            
            libs: vec![],

            included: vec![],
            include_paths: vec![],

            current_scope: IGScope::new(),

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