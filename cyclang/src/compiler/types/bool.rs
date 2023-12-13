use crate::compiler::llvm::context::ASTContext;
use crate::compiler::llvm::*;

use cyclang_macros::{BaseMacro, ComparisonMacro};
use std::any::Any;

extern crate llvm_sys;
use super::Arithmetic;
use crate::compiler::types::{Base, BaseTypes, Comparison, Debug, Func, TypeBase};
use llvm_sys::core::*;
use llvm_sys::prelude::*;

#[derive(Debug, Clone, BaseMacro, ComparisonMacro)]
#[base_type("BaseTypes::Bool")]
pub struct BoolType {
    pub builder: LLVMBuilderRef,
    pub llvm_value: LLVMValueRef,
    pub llvm_value_pointer: LLVMValueRef,
    pub name: String,
}

impl Arithmetic for BoolType {}

fn get_value_for_print_argument(
    context: &mut ASTContext,
    name: &str,
    value: BoolType,
) -> Vec<LLVMValueRef> {
    match value.get_ptr() {
        Some(v) => vec![context.build_load(v, int1_type(), name)],
        None => vec![value.get_value()],
    }
}

impl Debug for BoolType {
    fn print(&self, astcontext: &mut ASTContext) {
        let bool_func_args = get_value_for_print_argument(astcontext, "", self.clone());

        let bool_to_string_func = astcontext.llvm_func_cache.get("bool_to_str").unwrap();
        let str_value = astcontext.build_call(bool_to_string_func, bool_func_args, 1, "");
        let print_args: Vec<LLVMValueRef> = vec![str_value];
        let print_func = astcontext.llvm_func_cache.get("printf").unwrap();
        astcontext.build_call(print_func, print_args, 1, "");
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
        let bool_value = context.const_int(int1_type(), value_as_bool.into(), 0);
        let alloca = context.build_alloca_store(bool_value, int1_type(), &_name);
        Box::new(BoolType {
            name: _name,
            builder: context.builder,
            llvm_value: bool_value,
            llvm_value_pointer: alloca,
        })
    }
    fn assign(&mut self, _astcontext: &mut ASTContext, _rhs: Box<dyn TypeBase>) {
        match _rhs.get_type() {
            BaseTypes::Bool => {
                _astcontext.build_load_store(
                    _rhs.get_ptr().unwrap(),
                    self.get_ptr().unwrap(),
                    int1_type(),
                    "load_bool",
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
        self.llvm_value
    }
    fn get_ptr(&self) -> Option<LLVMValueRef> {
        Some(self.llvm_value_pointer)
    }
}

impl Func for BoolType {}
