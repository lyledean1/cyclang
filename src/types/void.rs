extern crate llvm_sys;
use crate::types::{Arithmetic, Base, BaseTypes, Comparison, Debug, Func, TypeBase};
use llvm_sys::prelude::*;

// TODO: implement base type
// return type i.e if int,
// figure out how to allocate values
#[derive(Debug, Clone)]
pub struct VoidType {}

impl Base for VoidType {
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Void
    }
}

impl Arithmetic for VoidType {}

impl Comparison for VoidType {}

impl Debug for VoidType {}

impl Func for VoidType {}

impl TypeBase for VoidType {
    fn get_value(&self) -> LLVMValueRef {
        unimplemented!("No value ref for void type")
    }
}
