# WebAssembly

Output the WASM File for use. 
```
llc -march=wasm32 -filetype=obj ./bin/main.ll -o ./bin/output.o
wasm-ld --no-entry --export-all -o ./bin/output.wasm ./bin/output.o
```

To convert this to a human readable format use
```
wasm2wat ./bin/output.wasm -o ./bin/output.wat  
``` 

There is an example of a Fibonacci sequence in `./examples/wasm/fib.js` which loads and executes the Fibonacci sequence. Just run `node ./examples/wasm/fib.js`.