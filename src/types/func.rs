use crate::context::ASTContext;
use crate::parser::Expression;
extern crate llvm_sys;
use crate::types::{BaseTypes, TypeBase};
use llvm_sys::prelude::*;

//TODO: create new functon
#[derive(Debug, Clone)]
pub struct FuncType {
    pub body: Expression,
    pub args: Option<Vec<String>>,
    pub llvm_type: LLVMTypeRef,
    pub llvm_func: LLVMValueRef,
}

impl TypeBase for FuncType {
    fn get_value(&self) -> LLVMValueRef {
        self.llvm_func
    }
    fn get_llvm_type(&self) -> LLVMTypeRef {
        self.llvm_type
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Func
    }
    fn print(&self, _ast_context: &mut ASTContext) {
        unreachable!("Shouldn't be able to print func type")
    }
}
