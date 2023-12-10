#![allow(dead_code)]

use crate::compiler::llvm::{cstr_from_string, int32_type, int64_type, int8_ptr_type};
use crate::compiler::types::bool::BoolType;
use crate::compiler::types::num::NumberType;
use crate::compiler::types::TypeBase;
use std::collections::HashMap;
use std::ffi::c_char;
extern crate llvm_sys;
use crate::compiler::types::func::FuncType;
use crate::compiler::types::num64::NumberType64;
use crate::cyclo_error::CycloError;
use crate::parser::{Expression, Type};
use libc::c_uint;
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
    pub printf_str_num64_value: LLVMValueRef,
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

    /// build_load
    ///
    /// This reads a value from one memory location via the LLVMBuildLoad instruction
    ///
    /// # Arguments
    ///
    /// * `ptr` - The LLVM Value you are loading from memory
    /// * `ptr_type` - The LLVM Type you will be storing in memory
    /// * `name` - The LLVM name of the alloca
    ///
    pub fn build_load(
        &self,
        ptr: LLVMValueRef,
        ptr_type: LLVMTypeRef,
        name: *const c_char,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildLoad2(self.builder, ptr_type, ptr, name) }
    }

    /// build_store
    ///
    /// This stores a value into memory on the stack via the LLVMBuildStore instruction
    ///
    /// # Arguments
    ///
    /// * `val` - The LLVM Value you are storing into memory
    /// * `ptr` - The LLVM pointer you will be storing the value in memory
    ///
    pub fn build_store(&self, val: LLVMValueRef, ptr: LLVMValueRef) {
        unsafe {
            LLVMBuildStore(self.builder, val, ptr);
        }
    }

    /// build_alloca
    ///
    /// This builds memory on the stack via the LLVMBuildAlloca instruction
    ///
    /// # Arguments
    ///
    /// * `ptr_type` - The LLVM Type you will be storing in memory
    /// * `name` - The LLVM name of the alloca
    ///
    pub fn build_alloca(&self, ptr_type: LLVMTypeRef, name: *const c_char) -> LLVMValueRef {
        unsafe { LLVMBuildAlloca(self.builder, ptr_type, name) }
    }

    /// build_alloca_store
    ///
    /// This calls LLVM to allocate memory on the stack via the LLVMBuildAlloca function and then
    /// stores the provided value into that new allocated stack memory. It then returns a pointer to that value.
    ///
    /// # Arguments
    ///
    /// * `val` - The LLVM Value you will be storing in memory
    /// * `ptr_type` - The LLVM Type you will be storing in memory
    /// * `name` - The LLVM name of the alloca
    ///
    pub fn build_alloca_store(
        &self,
        val: LLVMValueRef,
        ptr_type: LLVMTypeRef,
        name: *const c_char,
    ) -> LLVMValueRef {
        let ptr = self.build_alloca(ptr_type, name);
        self.build_store(val, ptr);
        ptr
    }

    /// build_load_store
    ///
    /// This reads a value from one memory location via the LLVMBuildLoad instruction
    /// and writes it to another via the LLVMBuildStore location.
    ///
    /// # Arguments
    ///
    /// * `load_ptr` - The LLVM Value you are loading from memory
    /// * `store_ptr` - The LLVM Type you will be storing in memory
    /// * `ptr_type` - The LLVM Type you will be storing in memory
    /// * `name` - The LLVM name of the alloca
    ///
    pub fn build_load_store(
        &self,
        load_ptr: LLVMValueRef,
        store_ptr: LLVMValueRef,
        ptr_type: LLVMTypeRef,
        name: *const c_char,
    ) {
        let rhs_val = self.build_load(load_ptr, ptr_type, name);
        self.build_store(rhs_val, store_ptr);
    }

    pub fn append_basic_block(&self, function: LLVMValueRef, name: &str) -> LLVMBasicBlockRef {
        unsafe { LLVMAppendBasicBlock(function, cstr_from_string(name).as_ptr()) }
    }

    pub fn build_call(
        &self,
        func: LLVMFunction,
        args: Vec<LLVMValueRef>,
        num_args: c_uint,
        name: &str,
    ) -> LLVMValueRef {
        unsafe {
            LLVMBuildCall2(
                self.builder,
                func.func_type,
                func.function,
                args.clone().as_mut_ptr(),
                num_args,
                cstr_from_string(name).as_ptr(),
            )
        }
    }

    pub fn cast_i32_to_i64(
        &self,
        mut lhs_value: LLVMValueRef,
        rhs_value: LLVMValueRef,
    ) -> LLVMValueRef {
        unsafe {
            let lhs_value_type = LLVMTypeOf(lhs_value);
            let lhs_value_width = LLVMGetIntTypeWidth(lhs_value_type);
            let rhs_value_type = LLVMTypeOf(rhs_value);
            let rhs_value_width = LLVMGetIntTypeWidth(rhs_value_type);

            if let (32, 64) = (lhs_value_width, rhs_value_width) {
                lhs_value = LLVMBuildSExt(
                    self.builder,
                    lhs_value,
                    int64_type(),
                    cstr_from_string("cast_to_i64").as_ptr(),
                );
            }
            lhs_value
        }
    }

    pub fn get_value_name(&self, value: LLVMValueRef) -> *const i8 {
        unsafe { LLVMGetValueName(value) }
    }

    pub fn build_br(&self, block: LLVMBasicBlockRef) -> LLVMValueRef {
        unsafe { LLVMBuildBr(self.builder, block) }
    }

    pub fn build_cond_br(
        &self,
        cond: LLVMValueRef,
        then_block: LLVMBasicBlockRef,
        else_block: LLVMBasicBlockRef,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildCondBr(self.builder, cond, then_block, else_block) }
    }

    pub fn position_builder_at_end(&self, block: LLVMBasicBlockRef) {
        unsafe {
            LLVMPositionBuilderAtEnd(self.builder, block);
        }
    }

    pub fn build_ret_void(&self) {
        unsafe {
            LLVMBuildRetVoid(self.builder);
        }
    }

    pub fn build_ret(&self, value: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildRet(self.builder, value) }
    }

    pub fn const_int(
        &self,
        int_type: LLVMTypeRef,
        val: ::libc::c_ulonglong,
        sign_extend: LLVMBool,
    ) -> LLVMValueRef {
        unsafe { LLVMConstInt(int_type, val, sign_extend) }
    }

    pub fn const_array(
        &self,
        element_type: LLVMTypeRef,
        const_values: *mut LLVMValueRef,
        length: u64,
    ) -> LLVMValueRef {
        unsafe { LLVMConstArray2(element_type, const_values, length) }
    }

    pub fn array_type(&self, element_type: LLVMTypeRef, element_count: u64) -> LLVMTypeRef {
        unsafe { LLVMArrayType2(element_type, element_count) }
    }

    pub fn build_gep(
        &self,
        llvm_type: LLVMTypeRef,
        ptr: LLVMValueRef,
        indices: *mut LLVMValueRef,
        num_indices: ::libc::c_uint,
        name: *const ::libc::c_char,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildGEP2(self.builder, llvm_type, ptr, indices, num_indices, name) }
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
    ) -> Result<Self, CycloError> {
        let param_types: &mut Vec<*mut llvm_sys::LLVMType> =
            &mut LLVMFunction::get_arg_types(args.clone());

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
            context.module,
            cstr_from_string(&name).as_ptr(),
            function_type,
        );

        let func = FuncType {
            llvm_type: function_type,
            llvm_func: function,
            return_type: return_type.clone(),
        };
        context.func_cache.set(&name, Box::new(func), context.depth);

        let function_entry_block = context.append_basic_block(function, "entry");

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
                    Type::i32 => {
                        let val = LLVMGetParam(function, i as u32);
                        let num = NumberType {
                            llmv_value: val,
                            llmv_value_pointer: None,
                            name: "param".into(),
                            cname: cstr_from_string("param").as_ptr(),
                        };
                        new_function.set_func_var(v, Box::new(num));
                    }
                    Type::i64 => {
                        let val = LLVMGetParam(function, i as u32);
                        let num = NumberType64 {
                            llmv_value: val,
                            llmv_value_pointer: None,
                            name: "param".into(),
                            cname: cstr_from_string("param").as_ptr(),
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

        context.position_builder_at_end(function_entry_block);

        // Set func args here
        context.match_ast(body.clone())?;

        // Delete func args here
        // // Check to see if there is a Return type
        if return_type == Type::None {
            context.build_ret_void();
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
        Ok(new_function)
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
