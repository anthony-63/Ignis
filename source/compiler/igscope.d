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

    LLVMValueRef define(string name, LLVMValueRef value, LLVMTypeRef type) {
        symbols[name] = IGValue(value, type);
        return value;
    }

    IGValue lookup(string name) {
        return resolve(name);
    }

    private IGValue resolve(string name) {
        if(symbols.length < 1) return IGValue(null, null, false);
        if(name in symbols) {
            return symbols[name];
        }
        auto tryresolve = parent.resolve(name);
        if(tryresolve.resolved) {
            return tryresolve;
        } else {
            return IGValue(null, null, false);
        }
    }
}