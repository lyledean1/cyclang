# Cyclang

A programming language I built in Rust - mainly for fun and my own learning! Uses PEG Parser in Rust for parsing and LLVM (llvm-sys) as the backend to compile to machine code binary. Check the [user guide](https://lyledean1.github.io/cyclang/overview.html) for a detailed overview of the language.

Try the Fibonacci example in `/examples/fib.cyc`

```rust
fn fib(i32 n) -> i32 {
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

## WebAssembly (WASM)  

To run the WASM example that compares Cyclang output (and an optimised version of the IR) against JS use the following command:
```
make fib-wasm
```

Below is a comparison of times in Node, Cyclang (unoptimized) WASM and Cyclang optimized WASM for a simple Fibonacci example. 

<img width="668" alt="Screenshot 2023-11-17 at 18 08 23" src="https://github.com/lyledean1/cyclang/assets/20296911/646f04d6-cc16-4045-b9b7-6e9438e810f6">


Ensure you have `wasm-ld` installed to convert LLVM object IR to a `.wasm` file. This should come with the LLVM 17 installation - instructions below.

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

See the [book](https://lyledean1.github.io/cyclang/setup.html) for a more detailed guide on setup.

