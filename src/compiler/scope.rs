use std::collections::HashMap;

use llvm_sys_180::{core::{LLVMConstInt, LLVMIntType}, prelude::{LLVMTypeRef, LLVMValueRef}};

use super::value::IGValue;

type SymbolTable = HashMap<String, IGValue>;
type FieldTable = HashMap<String, usize>;

#[derive(Clone)]
pub struct IGScope {
    pub symbols: SymbolTable,
    pub fields: FieldTable,
    parent: Option<Box<Self>>,
    name: Option<String>,
}

impl IGScope {
    pub fn new(symbols: Option<SymbolTable>, fields: Option<FieldTable>, name: Option<String>, parent: Option<Box<Self>>) -> Self {
        let _symbols = symbols.unwrap_or_default();
        let _fields = fields.unwrap_or_default();
        Self {
            symbols: _symbols,
            fields: _fields,
            parent,
            name,
        }
    }

    pub fn define(&mut self, name: String, value: LLVMValueRef, _type: LLVMTypeRef, mutable: bool, public: bool) -> LLVMValueRef {
        self.symbols.insert(name, IGValue {
            _type,
            value,
            mutable,
            public,
            parent: None,
        });

        value
    }

    pub fn define_struct(&mut self, name: String, value: LLVMValueRef, _type: LLVMTypeRef, mutable: bool, public: bool, parent: String) -> LLVMValueRef {
        self.symbols.insert(name, IGValue {
            _type,
            value,
            mutable,
            public,
            parent: Some(parent),
        });

        value
    }

    pub fn define_field(&mut self, name: String, index: usize) {
        self.fields.insert(name, index);
    }

    pub fn define_type(&mut self, name: String, _type: LLVMTypeRef, mutable: bool, public: bool) {
        self.symbols.insert(name, IGValue {
            _type,
            value: unsafe { LLVMConstInt(LLVMIntType(1), 0, 0) },
            mutable,
            public,
            parent: None,
        });
    }
    
    pub fn resolve(&self, name: String) -> Option<&IGValue> {
        if self.symbols.contains_key(&name) {
            return Some(self.symbols.get(&name).unwrap());   
        }

        if let Some(parent) = &self.parent {
            return parent.resolve(name);
        }

        None
    }

    pub fn resolve_field(&self, name: String) -> Option<&usize> {
        if self.fields.contains_key(&name) {
            return Some(self.fields.get(&name).unwrap());   
        }

        if let Some(parent) = &self.parent {
            return parent.resolve_field(name);
        }

        None
    }
}