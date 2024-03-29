use crate::compiler::codegen::{
    cstr_from_string, int1_type, int32_type, int64_type, int8_ptr_type,
};
use crate::compiler::types::bool::BoolType;
use crate::compiler::types::num::NumberType;
use crate::compiler::types::TypeBase;
use std::collections::HashMap;

extern crate llvm_sys;
use crate::compiler::context::ASTContext;
use crate::compiler::types::func::FuncType;
use crate::compiler::types::num64::NumberType64;
use cyclang_parser::{Expression, Type};
use anyhow::Result;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMType;

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
    pub fn new(
        context: &mut ASTContext,
        name: String,
        //TODO: check these arguments? Check the type?
        args: Vec<Expression>,
        return_type: Type,
        body: Expression,
        block: LLVMBasicBlockRef,
    ) -> Result<Self> {
        unsafe {
            let param_types: &mut Vec<*mut LLVMType> = &mut LLVMFunction::get_arg_types(args.clone());

            let mut function_type = LLVMFunctionType(
                LLVMVoidType(),
                param_types.as_mut_ptr(),
                args.len() as u32,
                0,
            );

            match return_type {
                Type::i32 => {
                    function_type =
                        LLVMFunctionType(int32_type(), param_types.as_mut_ptr(), args.len() as u32, 0);
                }
                Type::i64 => {
                    function_type =
                        LLVMFunctionType(int64_type(), param_types.as_mut_ptr(), args.len() as u32, 0);
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
            let function = LLVMAddFunction(
                context.codegen.module,
                cstr_from_string(&name).as_ptr(),
                function_type,
            );

            let func = FuncType {
                llvm_type: function_type,
                llvm_func: function,
                return_type: return_type.clone(),
            };
            context.func_cache.set(&name, Box::new(func), context.depth);

            let function_entry_block = context.codegen.append_basic_block(function, "entry");

            let previous_func = context.codegen.current_function.clone();
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
                        Type::i32 => {
                            let val = LLVMGetParam(function, i as u32);
                            let num = NumberType {
                                llvm_value: val,
                                llvm_value_pointer: None,
                                name: "param".into(),
                            };
                            new_function.set_func_var(v, Box::new(num));
                        }
                        Type::i64 => {
                            let val = LLVMGetParam(function, i as u32);
                            let num = NumberType64 {
                                llvm_value: val,
                                llvm_value_pointer: None,
                                name: "param".into(),
                            };
                            new_function.set_func_var(v, Box::new(num));
                        }
                        Type::String => {}
                        Type::Bool => {
                            let val = LLVMGetParam(function, i as u32);
                            let bool_type = BoolType {
                                builder: context.codegen.builder,
                                llvm_value: val,
                                llvm_value_pointer: val,
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

            context.codegen.current_function = new_function.clone();

            context
                .codegen
                .position_builder_at_end(function_entry_block);

            // Set func args here
            context.match_ast(body.clone())?;

            // Delete func args here
            // // Check to see if there is a Return type
            if return_type == Type::None {
                context.codegen.build_ret_void();
            }

            context.codegen.set_current_block(block);
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
            context.codegen.current_function = previous_func;
            Ok(new_function)
        }
    }

    fn get_arg_types(args: Vec<Expression>) -> Vec<*mut LLVMType> {
        let mut args_vec = vec![];
        for arg in args.into_iter() {
            match arg {
                Expression::FuncArg(_, t) => match t {
                    Type::Bool => args_vec.push(int1_type()),
                    Type::i32 => args_vec.push(int32_type()),
                    Type::i64 => args_vec.push(int64_type()),
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
