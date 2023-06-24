extern crate llvm_sys;
use crate::parser::Expression;
use crate::types::{Arithmetic, Base, BaseTypes, Comparison, Debug, TypeBase, Func};
use llvm_sys::prelude::*;

//TODO: create new functon
#[derive(Debug, Clone)]
pub struct BlockType {
    pub values: Vec<Expression>,
}

impl Base for BlockType {
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Block
    }
}

impl Arithmetic for BlockType {}

impl Comparison for BlockType {}

impl Debug for BlockType {}

impl Func for BlockType {}

impl TypeBase for BlockType {
    fn get_value(&self) -> LLVMValueRef {
        unimplemented!("No value ref for block type")
    }
}
