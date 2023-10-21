# Cyclang

A programming language I built in Rust - mainly for fun and my own learning! Uses PEG Parser in Rust for parsing and LLVM (llvm-sys) as the backend to compile to machine code binary.

Try the Fibonacci example in `/examples/fib.cyc`

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
cyclang --file ./examples/fib.cyc
```

This should output `6765`! 

##  Installing and Running (MacOS)

*Note*: I've only tested this on MacOS.

You will need LLVM 17 installed before you install cyclang, runn the following command
```
brew install llvm@17
```

Then the easiest way to install the binary currently is through the Rust package manager Cargo - see [Install Rust](https://www.rust-lang.org/tools/install). Once the step above is done, then run 
```
cargo install cyclang
```

## Run

Run the .cyc file

```
cyclang --file /path/to/file.cyc
```

## Test

Ensure you have the /bin folder set up (this will dump LLVM IR)

## Features

- [x] cli
- [x] repl
    - quite basic at the moment
- [x] JIT 

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

## Debugging Release Mode Errors

If getting errors with the release mode then use [Rust Sanitizer](https://github.com/japaric/rust-san) flags to debug.

Run the following command to identify memory issues
```
RUSTFLAGS="-Z sanitizer=address" cargo run --target={TARGET_ARCH} --release -- --file ./examples/simple.cyclo --output-llvm-ir
```

Where target architecture is your architecture i.e aarch64-apple-darwin

Also set proc_macro2 -> 1.66 if using Rust nightly compiler in the Cargo.toml
```
proc-macro2 = { version = "1.0.66", features=["default", "proc-macro"] }
```
