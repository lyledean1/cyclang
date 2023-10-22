use crate::compiler::llvm::context::ASTContext;
use crate::compiler::llvm::*;

use cyclang_macros::{BaseMacro, ComparisonMacro};
use std::any::Any;
use std::ffi::CString;

extern crate llvm_sys;
use super::Arithmetic;
use crate::compiler::llvm::cstr_from_string;
use crate::compiler::types::{Base, BaseTypes, Comparison, Debug, Func, TypeBase};
use llvm_sys::core::*;
use llvm_sys::prelude::*;

#[derive(Debug, Clone, BaseMacro, ComparisonMacro)]
#[base_type("BaseTypes::Bool")]
pub struct BoolType {
    pub builder: LLVMBuilderRef,
    pub llmv_value: LLVMValueRef,
    pub llmv_value_pointer: LLVMValueRef,
    pub name: String,
}

impl Arithmetic for BoolType {}

unsafe fn get_value_for_print_argument(
    builder: LLVMBuilderRef,
    name: *const i8,
    value: BoolType,
) -> LLVMValueRef {
    match value.get_ptr() {
        Some(v) => LLVMBuildLoad2(builder, int1_type(), v, name),
        None => value.get_value(),
    }
}

impl Debug for BoolType {
    fn print(&self, ast_context: &mut ASTContext) {
        unsafe {
            let value =
                get_value_for_print_argument(ast_context.builder, self.get_name(), self.clone());

            let mut bool_func_args: Vec<LLVMValueRef> = vec![value];

            match ast_context.llvm_func_cache.get("bool_to_str") {
                Some(bool_to_string) => {
                    let str_value = LLVMBuildCall2(
                        ast_context.builder,
                        bool_to_string.func_type,
                        bool_to_string.function,
                        bool_func_args.as_mut_ptr(),
                        1,
                        cstr_from_string("").as_ptr(),
                    );

                    let mut print_args: Vec<LLVMValueRef> = vec![str_value];
                    match ast_context.llvm_func_cache.get("printf") {
                        Some(print_func) => {
                            LLVMBuildCall2(
                                ast_context.builder,
                                print_func.func_type,
                                print_func.function,
                                print_args.as_mut_ptr(),
                                1,
                                cstr_from_string("").as_ptr(),
                            );
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }
}

impl TypeBase for BoolType {
    fn new(_value: Box<dyn Any>, _name: String, _context: &mut ASTContext) -> Box<dyn TypeBase>
    where
        Self: Sized,
    {
        let value_as_bool = match _value.downcast_ref::<bool>() {
            Some(val) => *val,
            None => panic!("The input value must be a bool"),
        };
        unsafe {
            let mut num = 0;
            if let true = value_as_bool {
                num = 1
            }
            let bool_value = LLVMConstInt(int1_type(), num, 0);
            let c_string = CString::new(_name.clone()).unwrap();
            let c_pointer: *const i8 = c_string.as_ptr();
            let alloca = LLVMBuildAlloca(_context.builder, int1_type(), c_pointer);
            LLVMBuildStore(_context.builder, bool_value, alloca);
            Box::new(BoolType {
                name: _name,
                builder: _context.builder,
                llmv_value: bool_value,
                llmv_value_pointer: alloca,
            })
        }
    }
    fn assign(&mut self, _ast_context: &mut ASTContext, _rhs: Box<dyn TypeBase>) {
        match _rhs.get_type() {
            BaseTypes::Bool => unsafe {
                let rhs_val = LLVMBuildLoad2(
                    _ast_context.builder,
                    int1_type(),
                    _rhs.get_ptr().unwrap(),
                    cstr_from_string("load_bool").as_ptr(),
                );
                LLVMBuildStore(self.builder, rhs_val, self.get_ptr().unwrap());
            },
            _ => {
                unreachable!(
                    "Can't reassign variable {:?} that has type {:?} to type {:?}",
                    self.name,
                    self.get_type(),
                    _rhs.get_type()
                )
            }
        }
    }
    fn get_value(&self) -> LLVMValueRef {
        self.llmv_value
    }
    fn get_ptr(&self) -> Option<LLVMValueRef> {
        Some(self.llmv_value_pointer)
    }
}

impl Func for BoolType {}
