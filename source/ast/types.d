module ast.types;

import ast.ast;

class SymbolType : Type {
    string name;

    bool refed;

    this(string _name) { name = _name; refed = false; }
    this(string _name, bool _ref) { name = _name; refed = _ref; }
}

class ArrayType : Type {
    Type inner;

    this(Type _inner) { inner = _inner; }
}