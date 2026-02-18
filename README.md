# Cyclang

Cyclang is a small programming language implemented in Rust. It parses with a PEG parser and compiles to native machine code via LLVM. See the [user guide](https://lyledean1.github.io/cyclang/overview.html) for the language overview.

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

## Install

### Quick Install (no LLVM required)

Download and install the latest prebuilt binary:

```bash
curl -fsSL https://raw.githubusercontent.com/lyledean1/cyclang/main/install.sh | bash
```

### From source (LLVM required)

Install LLVM 21 first.

MacOS:

```
brew install llvm@21
```

Ubuntu packages:

```
  llvm-21 
  llvm-21-tools 
  llvm-21-dev 
  clang-21 
  libpolly-21-dev
```

Then run `make set-llvm-sys-ffi-workaround`.

Install with Cargo (requires Rust): [Install Rust](https://www.rust-lang.org/tools/install).
Once LLVM is set up, run:
```
cargo install cyclang
```

See the [book](https://lyledean1.github.io/cyclang/setup.html) for a more detailed guide on setup.
