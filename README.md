# Cyclo-lang

A programming language I built in Rust - mainly for fun and my own learning! Uses PEG Parser in Rust for parsing and LLVM (llvm-sys) as the backend to compile to machine code binary.

Try the Fibonacci example in `/examples/fib.cyclo`

*Note*: *this isn't the most efficient Fibonacci algorithm but it's just an example of using recursion in the language, its also only been tested on an M1 Mac, so there might be bugs on other architectures*

```rust
fn fib(int n) -> int {
    if (n < 2) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}
print(fib(20));
```

You will need [Rust](https://www.rust-lang.org/tools/install) installed to run the below command.

```
cargo run -- --file ./examples/fib.cyclo
```

This should output `6765`! 

## LLVM Set Up 

Install LLVM 17
```
brew install llvm@17
```

The build.rs file should set this up, but incase you need to set this manually

Set LLVM_SYS_170_PREFIX variable before you run `cargo run`
```
export LLVM_SYS_170_PREFIX=/PATH/TO/LLVM17/VERSION
```

## Run

Run the .cyclo file

```
cyclo --file /path/to/file.cyclo
```

## Test

Ensure you have the /bin folder set up (this will dump LLVM IR)

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
- [x] Functions
    - [x] Call function ()
    - [ ] Lambda Functions
    - [ ] Closures
- [ ] Classes
