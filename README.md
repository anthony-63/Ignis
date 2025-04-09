# Ignis
Fast, compiled, statically typed system programming lanuage.

## WARNING!!!
IGNIS DOES NOT WORK RIGHT NOW, THE COMPILER IS ALMOST COMPLETELY UNFINISHED

## Prequesites
1. [Rust](https://rustup.rs/)
2. [LLVM-18](https://github.com/llvm/llvm-project/releases/tag/llvmorg-18.1.8)
2. [LibXML2](https://github.com/KWARC/rust-libxml?tab=readme-ov-file#installation-prerequisites)

### How to build
1. Clone the repo ``git clone https://github.com/anthony-63/Ignis``
3. Call ``cargo build --release`` and there you go! You should have ``ignis`` in ``./target/release``.

### How to use
```
ignis examples/01.ignis ./test
```
you should now have ``./test`` :)