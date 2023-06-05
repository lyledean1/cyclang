#![allow(dead_code)]

use crate::types::TypeBase;
use std::collections::HashMap;
extern crate llvm_sys;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use crate::types::llvm::c_str;
use crate::parser::Expression;
use crate::types::func::FuncType;

use std::ptr;

macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    };
}

pub struct ASTContext {
    pub builder: LLVMBuilderRef,
    pub module: LLVMModuleRef,
    pub context: LLVMContextRef,
    pub var_cache: VariableCache,
    pub llvm_func_cache: LLVMFunctionCache,
    pub current_function: LLVMFunction,
    pub depth: i32,
}

impl ASTContext {
    pub fn get_depth(&self) -> i32 {
        self.depth
    }
    pub fn incr(&mut self) {
        self.depth += 1;
    }
    pub fn decr(&mut self) {
        self.depth -= 1;
    }
}

#[derive(Clone)]
struct Container {
    pub locals: HashMap<i32, bool>,
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

    pub fn set(&mut self, key: &str, trait_object: Box<dyn TypeBase>, depth: i32) {
        let mut locals: HashMap<i32, bool> = HashMap::new();
        locals.insert(depth, true);
        self.map.insert(
            key.to_string(),
            Container {
                locals,
                trait_object,
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
        if let Some(v) = self.local.get(&depth) {
            for local in v.iter() {
                self.map.remove(&local.to_string());
            }
            self.local.remove(&depth);
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
    pub entry_block: LLVMBasicBlockRef,
    pub block: LLVMBasicBlockRef,
}

impl LLVMFunction {
    pub unsafe fn new(context: &mut ASTContext, name: String, body: Expression, block: LLVMBasicBlockRef) -> Self {
        let function_name = c_str(&name);

        let function_type = LLVMFunctionType(LLVMVoidType(), ptr::null_mut(), 0, 0);
        let function = LLVMAddFunction(context.module, function_name, function_type);
        let function_entry_block: *mut llvm_sys::LLVMBasicBlock =
            LLVMAppendBasicBlock(function, c_str!("entry"));
        
        LLVMPositionBuilderAtEnd(context.builder, function_entry_block);

        context.match_ast(body.clone());
        LLVMBuildRetVoid(context.builder);

        context.set_current_block(block);
        context.var_cache.set(
            name.as_str(),
            Box::new(FuncType {
                body: body,
                args: vec![],
                llvm_type: function_type,
                llvm_func: function,
            }),
            context.depth,
        );
        LLVMFunction { function: function, func_type: function_type, entry_block: function_entry_block, block: block }
    }
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
