# Cyclo-lang

A programming language I built in Rust - mainly for fun and my own learning! Uses PEG Parser in Rust for parsing and LLVM (llvm-sys) as the backend to compile to machine code binary.

Try the Fibonacci example in `/examples/fib.cyclo`

*Note*: *this isn't the most efficient Fibonacci algorithm but its showing off the language capabilities*

```rust
fn fib(int n) -> int {
    if (n < 2) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}
print(fib(20));
```

This should output `6765`! 

## Install 

Download the repo and run 
```
cargo install --path=./
```

## Run

Run the .cyclo file

```
cyclo --file /path/to/file.cyclo
```

## Features

- [x] cli
- [ ] repl
- [ ] JIT

## Grammar

- [x] Strings 
    - [x] Addition
- [x] Numbers 
    - [x] Addition
    - [x] Subtraction
    - [x] Multiplication
    - [x] Division
- [x] Boolean
- [x] Grouping
- [ ] Lists
- [ ] Map
- [x] Boolean
- [x] Print Statements
- [ ] Null Values
- [x] Variables 
    - [x] Reassignment
- [x] Let Statements
- [x] If Statements 
- [x] While Statements
- [x] For Loops
    - [x] Loop over range
    - [ ] Loop over values in list 
    - [ ] Loop over valuei in map
- [ ] Functions
    - [ ] Call function ()
    - [ ] Lambda Functions
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
cargo run example/fibonacci.cyclo
```
