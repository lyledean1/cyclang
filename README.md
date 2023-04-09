# A#

A programming language I built in Rust - mainly for fun and my own learning! Uses PEG Parser in Rust for parsing and LLVM (llvm-sys) as the backend to compile to machine code binary.

## Install 

Download the repo and run 
```
cargo install --path=./
```

## Run

Run the .asharp file 

```
asharp --file /path/to/file.asharp
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
- [x] Let Statements
- [ ] If Statements 
- [ ] For Loops
- [ ] Functions
    - [ ] Closures
- [ ] Classes

## Run

Install LLVM 16
```
brew install llvm@16
```

Set LLVM_SYS_160_PREFIX variable before you run `cargo run`
```
export LLVM_SYS_160_PREFIX=/PATH/TO/LLVM16/VERSION
```

Some examples are in the example folder, just run 
```
cargo run example/fibonacci.asharp
```
