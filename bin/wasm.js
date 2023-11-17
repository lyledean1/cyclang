const fs = require('fs');
const path = require('path');

async function loadWasmFile(filePath) {
    const wasmBuffer = fs.readFileSync(filePath);
    const wasmModule = await WebAssembly.compile(wasmBuffer);
    const wasmInstance = await WebAssembly.instantiate(wasmModule);

    return wasmInstance;
}

async function runWasm() {
    const wasmFilePath = path.join(__dirname, 'output.wasm');
    const wasmInstance = await loadWasmFile(wasmFilePath);
    const result = wasmInstance.exports.fib(30);
    console.log('fib(20) =', result);
}

runWasm();