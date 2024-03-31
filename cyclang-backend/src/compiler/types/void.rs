extern crate llvm_sys;
use crate::compiler::types::{BaseTypes, Func, TypeBase};
use llvm_sys::prelude::*;

#[derive(Debug, Clone)]
pub struct VoidType {}

impl Func for VoidType {}

impl TypeBase for VoidType {
    fn get_value(&self) -> LLVMValueRef {
        unimplemented!("No value ref for void type")
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Void
    }
}
