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

## Installing and Running

### Quick Install (No LLVM Required)

Download and install the latest prebuilt binary:

```bash
curl -fsSL https://raw.githubusercontent.com/lyledean1/cyclang/main/install.sh | bash
```

### From Source (LLVM Required)

You will need LLVM 21 installed before you install Cyclang, 

For MacOS run the following command

```
brew install llvm@21
```

For Ubuntu install the following packages

```
  llvm-21 
  llvm-21-tools 
  llvm-21-dev 
  clang-21 
  libpolly-21-dev
```

And run `make set-llvm-sys-ffi-workaround`

Then the easiest way to install the binary currently is through the Rust package manager Cargo - see [Install Rust](https://www.rust-lang.org/tools/install). Once the step above is done, then run 
```
cargo install cyclang
```

See the [book](https://lyledean1.github.io/cyclang/setup.html) for a more detailed guide on setup.
