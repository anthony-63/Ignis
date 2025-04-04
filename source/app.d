import std.stdio;
import std.file;

import lexer.lexer;
import parser.parser;
import compiler.compiler;
import llvm;

void usage(string filename) {
	writefln("Usage: %s <input> <output>", filename);
}

pragma(lib, "/usr/lib/llvm-10/lib/libLLVM-10.so");

void main(string[] args) {
	auto filename = args[0];
	if(args.length < 3) {
		usage(filename);
		return;
	}

	LLVMInitializeNativeTarget();
    LLVMInitializeNativeAsmPrinter();
    LLVMInitializeNativeAsmParser();

	auto source = readText(args[1]);
	
	auto lexer = new Lexer(source);
	auto tokens = lexer.tokenize();
	writeln(tokens);
	auto ast = Parser.parse(tokens);
	auto compiler = new Compiler(ast);
	compiler.compile(args[2]);
}
