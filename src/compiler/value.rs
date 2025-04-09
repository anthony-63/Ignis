use llvm_sys_180::{core::LLVMGetTypeKind, prelude::{LLVMTypeRef, LLVMValueRef}};


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

    pub unsafe fn is_type(&self, _type: LLVMTypeRef) -> bool {
        LLVMGetTypeKind(_type) == LLVMGetTypeKind(self._type)
    }

    pub unsafe fn same_type(&self, rhs: Self) -> bool {
        self.is_type(rhs._type)
    }

    pub unsafe fn are_both(&self, rhs: Self, _type: LLVMTypeRef) -> bool {
        self.same_type(rhs) && self.is_type(_type)
    }
}