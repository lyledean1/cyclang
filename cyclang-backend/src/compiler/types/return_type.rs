extern crate llvm_sys;
use crate::compiler::types::{Base, BaseTypes, Func, TypeBase};
use llvm_sys::prelude::*;

#[derive(Debug, Clone)]
pub struct ReturnType {}

impl Base for ReturnType {
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Return
    }
}

impl Func for ReturnType {}

impl TypeBase for ReturnType {
    fn get_value(&self) -> LLVMValueRef {
        unimplemented!("No value ref for return type")
    }
}
