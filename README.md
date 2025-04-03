# Ignis
Fast, compiled, statically typed system programming lanuage.

## WARNING!!!
IGNIS DOES NOT WORK RIGHT NOW, THE COMPILER IS ALMOST COMPLETELY UNFINISHED

## Dependencies
Make sure you have LLVM 10.0.0 installed, alongside with GCC, and verify both can be found in your PATH

### How to build
1. Download and install [dmd](https://dlang.org/download.html#dmd)
2. Clone the repo ``git clone https://github.com/anthony-63/Ignis``
3. Get llvm-d ``git submodule update --init --recursive``
2. Call ``dub build`` and there you go! You should have ``ignis`` in your root folder.

### How to use
```
./ignis examples/01.ignis ./test
```
you should now have ``./test`` :)