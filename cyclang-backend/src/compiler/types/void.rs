extern crate llvm_sys;
use crate::compiler::types::{Base, BaseTypes, Func, TypeBase};
use llvm_sys::prelude::*;

#[derive(Debug, Clone)]
pub struct VoidType {}

impl Base for VoidType {
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Void
    }
}

impl Func for VoidType {}

impl TypeBase for VoidType {
    fn get_value(&self) -> LLVMValueRef {
        unimplemented!("No value ref for void type")
    }
}
