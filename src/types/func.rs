use crate::parser::Expression;
extern crate llvm_sys;
use crate::types::{Arithmetic, Base, BaseTypes, Comparison, Debug, TypeBase};
use llvm_sys::prelude::*;

//TODO: create new functon
#[derive(Debug, Clone)]
pub struct FuncType {
    pub body: Expression,
    pub args: Vec<String>,
    pub llvm_type: LLVMTypeRef,
    pub llvm_func: LLVMValueRef,
}

impl Base for FuncType {
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Func
    }
}

impl Arithmetic for FuncType {}

impl Comparison for FuncType {}

impl Debug for FuncType {}

impl TypeBase for FuncType {
    fn get_value(&self) -> LLVMValueRef {
        self.llvm_func
    }
    fn get_llvm_type(&self) -> LLVMTypeRef {
        self.llvm_type
    }
    fn get_args(&self) -> Vec<String> {
        self.args.clone()
    }
}
