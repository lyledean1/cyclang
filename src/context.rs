#![allow(dead_code)]

use crate::types::TypeBase;
use std::collections::HashMap;
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
    pub depth: i32,
}

impl ASTContext {
    pub fn get_depth(&self) -> i32 {
        self.depth
    }
    pub fn incr(&mut self) {
        self.depth = self.depth + 1;
    }
    pub fn decr(&mut self) {
        self.depth = self.depth - 1;
    }
}

#[derive(Clone)]
struct Container {
    pub trait_object: Box<dyn TypeBase>,
}
pub struct VariableCache {
    map: HashMap<String, Container>,
    local: HashMap<i32, Vec<String>>,
}

impl Default for VariableCache {
    fn default() -> Self {
        Self::new()
    }
}

impl VariableCache {
    pub fn new() -> Self {
        VariableCache {
            map: HashMap::new(),
            local: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: Box<dyn TypeBase>, depth: i32) {
        self.map.insert(
            key.to_string(),
            Container {
                trait_object: value,
            },
        );
        match self.local.get(&depth) {
            Some(val) => {
                let mut val_clone = val.clone();
                val_clone.push(key.to_string());
                self.local.insert(depth, val_clone);
            }
            None => {
                self.local.insert(depth, vec![key.to_string()]);
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<Box<dyn TypeBase>> {
        match self.map.get(key) {
            Some(v) => Some(dyn_clone::clone_box(&*v.trait_object)),
            None => None,
        }
    }

    fn del(&mut self, key: &str) {
        self.map.remove(key);
    }

    pub fn del_locals(&mut self, depth: i32) {
        match self.local.get(&depth) {
            Some(v) => {
                for local in v.iter() {
                    self.map.remove(&local.to_string());
                }
                self.local.remove(&depth);
            },
            None => {

            },
        }
    }
}

pub struct LLVMFunctionCache {
    map: HashMap<String, LLVMFunction>,
}

impl Default for LLVMFunctionCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
pub struct LLVMFunction {
    pub function: LLVMValueRef,
    pub func_type: LLVMTypeRef,
    pub block: LLVMBasicBlockRef,
    // var cache?
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
