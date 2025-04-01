module ast.types;

import ast.ast;

class SymbolType : Type {
    string name;

    this(string _name) { name = _name; }
}

class ArrayType : Type {
    Type inner;

    this(Type _inner) { inner = _inner; }
}