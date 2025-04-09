use llvm_sys_180::prelude::{LLVMTypeRef, LLVMValueRef};


#[derive(Clone)]
pub struct IGValue {
    pub value: LLVMValueRef,
    pub _type: LLVMTypeRef,
    pub mutable: bool,
    pub public: bool,
}

impl IGValue {
    pub fn new(value: LLVMValueRef, _type: LLVMTypeRef) -> Self {
        Self {
            value,
            _type,
            mutable: true,
            public: true,
        }
    }
}