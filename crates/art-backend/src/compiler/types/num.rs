use crate::compiler::types::{BaseTypes, TypeBase};

extern crate llvm_sys;
use llvm_sys::prelude::*;

#[derive(Debug, Clone)]
pub struct NumberType {
    //TODO: remove pub use of these
    pub llvm_value: LLVMValueRef,
    pub llvm_value_pointer: Option<LLVMValueRef>,
    pub name: String,
}

impl TypeBase for NumberType {
    fn get_value(&self) -> LLVMValueRef {
        self.llvm_value
    }
    fn get_ptr(&self) -> Option<LLVMValueRef> {
        self.llvm_value_pointer
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Number
    }
}
