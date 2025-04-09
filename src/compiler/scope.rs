use std::collections::HashMap;

use llvm_sys_180::prelude::{LLVMTypeRef, LLVMValueRef};

use super::value::IGValue;

type SymbolTable = HashMap<String, IGValue>;

#[derive(Clone)]
pub struct IGScope {
    symbols: SymbolTable,
    parent: Option<Box<Self>>,
    name: Option<String>,
}

impl IGScope {
    pub fn new(symbols: Option<SymbolTable>, name: Option<String>, parent: Option<Box<Self>>) -> Self {
        let _symbols = symbols.unwrap_or(SymbolTable::new());
        Self {
            symbols: _symbols,
            parent,
            name,
        }
    }

    pub fn define(&mut self, name: String, value: LLVMValueRef, _type: LLVMTypeRef, mutable: bool, public: bool) -> LLVMValueRef {
        self.symbols.insert(name, IGValue {
            _type,
            value,
            mutable,
            public
        });

         value
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
}