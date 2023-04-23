#![allow(dead_code)]

use std::collections::HashMap;
use crate::types::{TypeBase};
extern crate llvm_sys;
use llvm_sys::prelude::*;

pub struct ASTContext {
    pub builder: LLVMBuilderRef,
    pub module: LLVMModuleRef,
    pub context: LLVMContextRef,
    pub var_cache: VariableCache,
    pub llvm_func_cache: LLVMFunctionCache,
    pub current_function: LLVMFunction,
    pub current_block: LLVMBasicBlockRef,
}

#[derive(Clone)]
struct Container {
    pub trait_object: Box<dyn TypeBase>,
}
pub struct VariableCache {
    map: HashMap<String, Container>,
}

impl VariableCache {
    pub fn new() -> Self {
        VariableCache {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: Box<dyn TypeBase>) {
        self.map.insert(
            key.to_string(),
            Container {
                trait_object: value,
            },
        );
    }

    pub fn get(&self, key: &str) -> Option<Box<dyn TypeBase>> {
        match self.map.get(key) {
            Some(v) => Some(dyn_clone::clone_box(&*v.trait_object)),
            None => None,
        }
    }
}

pub struct LLVMFunctionCache {
    map: HashMap<String, LLVMFunction>,
}

#[derive(Clone, Copy)]
pub struct LLVMFunction {
    pub function: LLVMValueRef,
    pub func_type: LLVMTypeRef,
}

impl LLVMFunctionCache {
    pub fn new() -> Self {
        LLVMFunctionCache {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: LLVMFunction) {
        self.map.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<LLVMFunction> {
        //HACK, copy each time, probably want one reference to this
        self.map.get(key).copied()
    }
}
