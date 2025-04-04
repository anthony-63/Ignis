module compiler.igscope;

import std.format;
import std.stdio;

import compiler.value;

import llvm;

alias SymbolTable = IGValue[string];

class IGScope {
    SymbolTable symbols;
    IGScope parent;
    string name;

    this(SymbolTable _symbols = null, string _name = "global", IGScope _parent = null) {
        symbols = _symbols;
        parent = _parent;
        name = _name;
    }

    IGScope copy() {
        return new IGScope(symbols, name, parent);
    }

    LLVMValueRef define(string name, LLVMValueRef value, LLVMTypeRef type, bool mutable=true) {
        symbols[name] = IGValue(value, type);
        symbols[name].mutable = mutable;
        return value;
    }

    IGValue lookup(string name) {
        return resolve(name);
    }

    private IGValue resolve(string name) {
        if(name in symbols) {
            return symbols[name];
        }
        if(parent !is null) {
            auto tryresolve = parent.resolve(name);
            if(tryresolve.resolved) return tryresolve;
        }
        return IGValue(null, null, false);
    }
}