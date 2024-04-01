extern crate llvm_sys;

use crate::compiler::types::{BaseTypes, TypeBase};
use llvm_sys::prelude::*;

#[derive(Debug, Clone)]
pub struct ListType {
    pub llvm_value: LLVMValueRef,
    pub llvm_value_ptr: LLVMValueRef,
    pub llvm_type: LLVMTypeRef,
}

impl TypeBase for ListType {
    fn get_value(&self) -> LLVMValueRef {
        self.llvm_value
    }

    fn get_ptr(&self) -> Option<LLVMValueRef> {
        Some(self.llvm_value_ptr)
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::List(Box::new(BaseTypes::Number))
    }
    fn get_llvm_type(&self) -> LLVMTypeRef {
        self.llvm_type
    }
}
