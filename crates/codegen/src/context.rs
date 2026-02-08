use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMTypeRef, LLVMValueRef};
use std::collections::HashMap;

#[derive(Clone)]
pub struct LLVMCallFn {
    pub function: LLVMValueRef,
    pub func_type: LLVMTypeRef,
}

#[derive(Clone)]
pub struct LLVMFunction {
    pub function: LLVMValueRef,
    pub func_type: LLVMTypeRef,
    pub entry_block: LLVMBasicBlockRef,
    pub block: LLVMBasicBlockRef,
}

pub struct LLVMFunctionCache {
    map: HashMap<String, LLVMCallFn>,
}

impl Default for LLVMFunctionCache {
    fn default() -> Self {
        Self::new()
    }
}

impl LLVMFunctionCache {
    pub fn new() -> Self {
        LLVMFunctionCache {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: LLVMCallFn) {
        self.map.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<LLVMCallFn> {
        // Clone to keep API simple for builder/codegen call sites.
        self.map.get(key).cloned()
    }
}
