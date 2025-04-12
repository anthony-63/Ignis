use llvm_sys_180::{core::LLVMGetTypeKind, prelude::{LLVMTypeRef, LLVMValueRef}};


#[derive(Clone, Debug)]
pub struct IGValue {
    pub value: LLVMValueRef,
    pub _type: LLVMTypeRef,
    pub mutable: bool,
    pub public: bool,
    pub parent: Option<String>,
}

impl IGValue {
    pub fn new(value: LLVMValueRef, _type: LLVMTypeRef) -> Self {
        Self {
            value,
            _type,
            mutable: true,
            public: true,
            parent: None,
        }
    }

    pub fn new_struct(value: LLVMValueRef, _type: LLVMTypeRef, parent: String) -> Self {
        Self {
            value,
            _type,
            mutable: true,
            public: true,
            parent: Some(parent),
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