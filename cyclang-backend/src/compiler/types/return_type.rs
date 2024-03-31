extern crate llvm_sys;
use crate::compiler::types::{BaseTypes, Func, TypeBase};
use llvm_sys::prelude::*;

#[derive(Debug, Clone)]
pub struct ReturnType {}

impl Func for ReturnType {}

impl TypeBase for ReturnType {
    fn get_value(&self) -> LLVMValueRef {
        unimplemented!("No value ref for return type")
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Return
    }
}
