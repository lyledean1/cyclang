# A#

A simple programming language I built in Rust mainly for my own learning. Uses PEG Parser in Rust for parsing and LLVM (llvm-sys) to compile to a machine code binary.

## Install 

Download the repo and run 
```
cargo install --path=./
```

## Features

- [ ] REPL
- [ ] CLI Tool

## Grammar

- [x] Strings 
    - [x] Addition
- [x] Numbers 
    - [x] Addition
    - [x] Subtraction
    - [x] Multiplication
    - [x] Division
- [ ] Grouping
- [ ] Lists
- [ ] Dict
- [x] Boolean
- [ ] Print Statements
- [ ] Null Values
- [x] Variables 
    - [x] Reassignment
- [x] Let Statements `let x = a;`
- [ ] If Statements 
- [ ] For Loops
- [ ] Functions
    - [ ] Closures
- [ ] Classes

## Run

Install LLVM 13
```
brew install llvm@13
```

Set LLVM_SYS_130_PREFIX variable before you run `cargo run`
```
export LLVM_SYS_130_PREFIX=/PATH/TO/LLVM13/VERSION
```

Some examples are in the example folder, just run 
```
cargo run example/fibonacci.asharp
```
