extern crate llvm_sys;
use crate::parser::Expression;
use crate::types::{Arithmetic, Base, BaseTypes, Comparison, Debug, Func, TypeBase};
use llvm_sys::prelude::*;

// TODO: implement base type
// return type i.e if int,
// figure out how to allocate values
#[derive(Debug, Clone)]
pub struct ParamType {
    pub llvm_value_ref: LLVMValueRef,
    // pub base_type: LLVMTypeRef,
}

impl Base for ParamType {
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Param
    }
}

impl Arithmetic for ParamType {}

impl Comparison for ParamType {}

impl Debug for ParamType {}

impl Func for ParamType {}

impl TypeBase for ParamType {
    fn get_value(&self) -> LLVMValueRef {
        unimplemented!("No value ref for block type")
    }
}
