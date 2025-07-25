extern crate llvm_sys;
use crate::compiler::types::{BaseTypes, TypeBase};
use art_parser::Type;
use llvm_sys::prelude::*;

// FuncType -> Exposes the Call Func (i.e after function has been executed)
// So can provide the return type to be used after execution
#[derive(Clone)]
pub struct FuncType {
    pub return_type: Type,
    pub llvm_type: LLVMTypeRef,
    pub llvm_func: LLVMValueRef,
}

impl TypeBase for FuncType {
    fn get_value(&self) -> LLVMValueRef {
        self.llvm_func
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Func
    }

    fn get_llvm_type(&self) -> LLVMTypeRef {
        self.llvm_type
    }

    fn get_return_type(&self) -> Type {
        self.return_type.clone()
    }
}
