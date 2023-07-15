#![allow(dead_code)]

use crate::types::llvm::int8_type;
use crate::types::TypeBase;
use std::collections::HashMap;
extern crate llvm_sys;
use crate::parser::{Expression, Type};
use crate::types::func::FuncType;
use crate::types::llvm::c_str;
use llvm_sys::core::*;
use llvm_sys::LLVMType;
use llvm_sys::prelude::*;

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
    pub printf_str_value: LLVMValueRef,
    pub printf_str_num_value: LLVMValueRef,
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

#[derive(Clone, Debug)]
pub struct LLVMFunction {
    pub function: LLVMValueRef,
    pub func_type: LLVMTypeRef,
    pub entry_block: LLVMBasicBlockRef,
    pub block: LLVMBasicBlockRef,
    pub symbol_table: HashMap<String, LLVMValueRef>,
    pub args: Vec<LLVMTypeRef>,
}

impl LLVMFunction {
    pub unsafe fn new(
        context: &mut ASTContext,
        name: String,
        //TODO: check these arguments? Check the type?
        args: Vec<Expression>,
        body: Expression,
        block: LLVMBasicBlockRef,
    ) -> Self {
        let function_name = c_str(&name);

        // let param_types: &mut Vec<*mut llvm_sys::LLVMType> = &mut LLVMFunction::get_arg_types(args.clone());
        let param_types: &mut Vec<*mut llvm_sys::LLVMType> = &mut vec![int8_type()];
        // Then get Return Type

        println!("{:?}", param_types);

        // Look Ahead to get Function Type

        let function_type = LLVMFunctionType(
            LLVMVoidType(),
            param_types.as_mut_ptr(),
            args.len() as u32,
            0,
        );
        let function = LLVMAddFunction(context.module, function_name, function_type);
        let function_entry_block: *mut llvm_sys::LLVMBasicBlock =
            LLVMAppendBasicBlock(function, c_str!("entry"));

        let previous_func = context.current_function.clone();
        let mut new_function = LLVMFunction {
            function,
            func_type: function_type,
            entry_block: function_entry_block,
            block: function_entry_block,
            symbol_table: HashMap::new(),
            args: param_types.to_vec(),
        };

        new_function.set_func_var("val", LLVMGetParam(function, 0));

        context.current_function = new_function.clone();

        LLVMPositionBuilderAtEnd(context.builder, function_entry_block);

        // Set func args here
        context.match_ast(body.clone());

        // Delete func args here
        LLVMBuildRetVoid(context.builder);

        context.set_current_block(block);
        context.var_cache.set(
            name.as_str(),
            Box::new(FuncType {
                body,
                args: vec![],
                llvm_type: function_type,
                llvm_func: function,
                llvm_func_ref: new_function.clone(),
            }),
            context.depth,
        );
        //reset previous function
        context.current_function = previous_func;
        new_function
    }

    fn get_arg_types(args: Vec<Expression>) -> Vec<*mut LLVMType> {
        let mut args_vec = vec![];
        for arg in args.into_iter() {
            match arg {
                Expression::FuncArg(_, t) => {
                    match t {
                        Type::Int => {
                            args_vec.push(int8_type())
                        }
                        Type::String => {
                            args_vec.push(int8_type())
                        }
                        _=> {
                            unreachable!("unknown type {:?}", t)
                        }
                    }
                }
                _ => {
                    unreachable!("this should only be FuncArg, got {:?}", arg)
                }
            }

        }
        args_vec
    }

    fn set_func_var(&mut self, key: &str, value: LLVMValueRef) {
        self.symbol_table.insert(key.to_string(), value);
    }

    fn get_func_var(&self, key: &str) -> Option<LLVMValueRef> {
        self.symbol_table.get(key).cloned()
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
        self.map.get(key).cloned()
    }
}
