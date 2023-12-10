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

fn get_value_for_print_argument(
    context: &mut ASTContext,
    name: &str,
    value: BoolType,
) -> LLVMValueRef {
    match value.get_ptr() {
        Some(v) => context.build_load(v, int1_type(), cstr_from_string(name).as_ptr()),
        None => value.get_value(),
    }
}

impl Debug for BoolType {
    fn print(&self, astcontext: &mut ASTContext) {
        let value = get_value_for_print_argument(astcontext, "", self.clone());

        let bool_func_args: Vec<LLVMValueRef> = vec![value];

        match astcontext.llvm_func_cache.get("bool_to_str") {
            Some(bool_to_string) => {
                let str_value = astcontext.build_call(bool_to_string, bool_func_args, 1, "");

                let print_args: Vec<LLVMValueRef> = vec![str_value];
                match astcontext.llvm_func_cache.get("printf") {
                    Some(print_func) => {
                        astcontext.build_call(print_func, print_args, 1, "");
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

impl TypeBase for BoolType {
    fn new(_value: Box<dyn Any>, _name: String, context: &mut ASTContext) -> Box<dyn TypeBase>
    where
        Self: Sized,
    {
        let value_as_bool = match _value.downcast_ref::<bool>() {
            Some(val) => *val,
            None => panic!("The input value must be a bool"),
        };
        let mut num = 0;
        if let true = value_as_bool {
            num = 1
        }
        let bool_value = context.const_int(int1_type(), num, 0);
        let c_string = CString::new(_name.clone()).unwrap();
        let c_pointer: *const i8 = c_string.as_ptr();
        let alloca = context.build_alloca_store(bool_value, int1_type(), c_pointer);
        Box::new(BoolType {
            name: _name,
            builder: context.builder,
            llmv_value: bool_value,
            llmv_value_pointer: alloca,
        })
    }
    fn assign(&mut self, _astcontext: &mut ASTContext, _rhs: Box<dyn TypeBase>) {
        match _rhs.get_type() {
            BaseTypes::Bool => {
                _astcontext.build_load_store(
                    _rhs.get_ptr().unwrap(),
                    self.get_ptr().unwrap(),
                    int1_type(),
                    cstr_from_string("load_bool").as_ptr(),
                );
            }
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
