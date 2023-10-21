#![allow(dead_code)]

use crate::compiler::llvm::{cstr_from_string, int32_type, int8_ptr_type};
use crate::compiler::types::bool::BoolType;
use crate::compiler::types::num::NumberType;
use crate::compiler::types::TypeBase;
use std::collections::HashMap;
extern crate llvm_sys;
use crate::compiler::llvm::c_str;
use crate::compiler::types::func::FuncType;
use crate::parser::{Expression, Type};
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMType;

use super::int1_type;

pub struct ASTContext {
    pub builder: LLVMBuilderRef,
    pub module: LLVMModuleRef,
    pub context: LLVMContextRef,
    pub var_cache: VariableCache,
    pub func_cache: VariableCache,
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

#[derive(Clone)]
pub struct LLVMFunction {
    pub function: LLVMValueRef,
    pub func_type: LLVMTypeRef,
    pub entry_block: LLVMBasicBlockRef,
    pub block: LLVMBasicBlockRef,
    pub symbol_table: HashMap<String, Box<dyn TypeBase>>,
    pub args: Vec<LLVMTypeRef>,
    pub return_type: Type,
}

impl LLVMFunction {
    pub unsafe fn new(
        context: &mut ASTContext,
        name: String,
        //TODO: check these arguments? Check the type?
        args: Vec<Expression>,
        return_type: Type,
        body: Expression,
        block: LLVMBasicBlockRef,
    ) -> Self {
        let function_name = c_str(&name);
        let param_types: &mut Vec<*mut llvm_sys::LLVMType> =
            &mut LLVMFunction::get_arg_types(args.clone());

        let mut function_type = LLVMFunctionType(
            LLVMVoidType(),
            param_types.as_mut_ptr(),
            args.len() as u32,
            0,
        );

        match return_type {
            Type::Int => {
                function_type =
                    LLVMFunctionType(int32_type(), param_types.as_mut_ptr(), args.len() as u32, 0);
            }
            Type::Bool => {
                function_type =
                    LLVMFunctionType(int1_type(), param_types.as_mut_ptr(), args.len() as u32, 0);
            }
            Type::None => {
                // skip
            }
            _ => {
                unimplemented!("not implemented")
            }
        }

        // get correct function return type
        let function = LLVMAddFunction(context.module, function_name, function_type);

        let func = FuncType {
            llvm_type: function_type,
            llvm_func: function,
            return_type: return_type.clone(),
        };
        context.func_cache.set(&name, Box::new(func), context.depth);

        let function_entry_block: *mut llvm_sys::LLVMBasicBlock =
            LLVMAppendBasicBlock(function, cstr_from_string("entry"));

        let previous_func = context.current_function.clone();
        let mut new_function = LLVMFunction {
            function,
            func_type: function_type,
            entry_block: function_entry_block,
            block: function_entry_block,
            symbol_table: HashMap::new(),
            args: param_types.to_vec(),
            return_type: return_type.clone(),
        };

        for (i, val) in args.iter().enumerate() {
            match val {
                Expression::FuncArg(v, t) => match t {
                    Type::Int => {
                        let val = LLVMGetParam(function, i as u32);
                        let num = NumberType {
                            llmv_value: val,
                            llmv_value_pointer: None,
                            name: "param".into(),
                            cname: cstr_from_string("param"),
                        };
                        new_function.set_func_var(v, Box::new(num));
                    }
                    Type::String => {}
                    Type::Bool => {
                        let val = LLVMGetParam(function, i as u32);
                        let bool_type = BoolType {
                            builder: context.builder,
                            llmv_value: val,
                            llmv_value_pointer: val,
                            name: "bool_param".into(),
                        };
                        new_function.set_func_var(v, Box::new(bool_type));
                    }
                    _ => {
                        unreachable!("type {:?} not found", t)
                    }
                },
                _ => {
                    unreachable!("this should only be FuncArg, got {:?}", val)
                }
            }
        }

        context.current_function = new_function.clone();

        LLVMPositionBuilderAtEnd(context.builder, function_entry_block);

        // Set func args here
        context.match_ast(body.clone());

        // Delete func args here
        // // Check to see if there is a Return type
        if return_type == Type::None {
            LLVMBuildRetVoid(context.builder);
        }

        context.set_current_block(block);
        context.var_cache.set(
            name.as_str(),
            Box::new(FuncType {
                llvm_type: function_type,
                llvm_func: function,
                return_type,
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
                Expression::FuncArg(_, t) => match t {
                    Type::Bool => args_vec.push(int1_type()),
                    Type::Int => args_vec.push(int32_type()),
                    Type::String => args_vec.push(int8_ptr_type()),
                    _ => {
                        unreachable!("unknown type {:?}", t)
                    }
                },
                _ => {
                    unreachable!("this should only be FuncArg, got {:?}", arg)
                }
            }
        }
        args_vec
    }

    fn set_func_var(&mut self, key: &str, value: Box<dyn TypeBase>) {
        self.symbol_table.insert(key.to_string(), value);
    }

    fn get_func_var(&self, key: &str) -> Option<Box<dyn TypeBase>> {
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
