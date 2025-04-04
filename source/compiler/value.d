module compiler.value;

import llvm;

struct IGValue {
    LLVMValueRef value;
    LLVMTypeRef type;

    bool resolved = true;

    bool mutable = true;

    bool _public = true;

    bool is_type(LLVMTypeRef _type) {
        return LLVMGetTypeKind(type) == LLVMGetTypeKind(_type);
    }

    bool same_type(IGValue rhs) {
        return is_type(rhs.type);
    }

    bool are_both(IGValue rhs, LLVMTypeRef _type) {
        return same_type(rhs) && is_type(_type);
    }
}