use crate::context::ASTContext;
extern crate llvm_sys;
use crate::parser::Expression;
use crate::types::{BaseTypes, TypeBase};
use llvm_sys::prelude::*;

//TODO: create new functon
#[derive(Debug, Clone)]
pub struct BlockType {
    pub values: Vec<Expression>,
}

impl TypeBase for BlockType {
    fn get_value(&self) -> LLVMValueRef {
        unimplemented!("No value ref for block type")
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Block
    }
    fn print(&self, _ast_context: &mut ASTContext) {
        unreachable!("Shouldn't be able to print block type")
    }
}
