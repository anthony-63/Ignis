import std.stdio;
import std.file;

import lexer.lexer;
import parser.parser;
import compiler.compiler;

void usage(string filename) {
	writefln("Usage: %s <input> <output>", filename);
}

void main(string[] args) {
	auto filename = args[0];
	if(args.length < 3) {
		usage(filename);
		return;
	}

	auto source = readText(args[1]);
	
	auto lexer = new Lexer(source);
	auto tokens = lexer.tokenize();
	auto ast = Parser.parse(tokens);
	auto compiler = new Compiler(ast);
	compiler.compile(args[2]);
}
