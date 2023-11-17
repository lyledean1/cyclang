const fs = require('fs');
const path = require('path');
const { performance } = require('perf_hooks');

function fibJS(n) {
    if (n < 2) {
        return n;
    }
    return fibJS(n - 1) + fibJS(n - 2);
}

async function loadWasmFile(filePath) {
    const wasmBuffer = fs.readFileSync(filePath);
    const wasmModule = await WebAssembly.compile(wasmBuffer);
    const wasmInstance = await WebAssembly.instantiate(wasmModule);

    return wasmInstance;
}

async function runWasm() {
    const wasmFilePath = path.join(__dirname, './fib.wasm');
    const wasmInstance = await loadWasmFile(wasmFilePath);

    const wasmOptFilePath = path.join(__dirname, './fib_opt.wasm');
    const wasmOptInstance = await loadWasmFile(wasmOptFilePath);


    const n = 30;

    // Measure WebAssembly execution time
    let start = performance.now();
    const resultWasm = wasmInstance.exports.fib(n);
    let end = performance.now();
    console.log(`Cyclang WASM fib(${n}) =`, resultWasm, `| Time: ${(end - start).toFixed(2)} ms`);

    start = performance.now();
    const resultOptWasm = wasmOptInstance.exports.fib(n);
    end = performance.now();
    console.log(`Cyclang WASM (Optimized) fib(${n}) =`, resultOptWasm, `| Time: ${(end - start).toFixed(2)} ms`);


    // Measure JavaScript execution time
    start = performance.now();
    const resultJS = fibJS(n);
    end = performance.now();
    console.log(`JavaScript fib(${n}) =`, resultJS, `| Time: ${(end - start).toFixed(2)} ms`);
}

runWasm();
