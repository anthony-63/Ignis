import std.stdio;
import std.file;
import std.path;

import lexer.lexer;
import parser.parser;
import compiler.compiler;
import llvm;

void usage(string filename) {
	writefln("Usage: %s <input> <output>", filename);
}

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
	auto ast = Parser.parse(tokens);
	
	auto includes = [dirName(args[0]) ~ "/std"];
	
	auto compiler = new Compiler(ast, includes);
	compiler.compile(args[2], dirName(args[2]));
}
