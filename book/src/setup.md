
##  Installing and Running (MacOS)

*Note*: I've only tested this on MacOS.

You will need LLVM 20 installed before you install Cyclang, runn the following command
```
brew install llvm@20
```

Then the easiest way to install the binary currently is through the Rust package manager Cargo - see [Install Rust](https://www.rust-lang.org/tools/install). Once the step above is done, then run 
```
cargo install cyclang
```

## Test

Ensure you have the /bin folder set up (this will dump LLVM IR). Run tests through `make test`.

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
