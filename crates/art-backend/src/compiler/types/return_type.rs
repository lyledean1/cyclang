extern crate llvm_sys;
use crate::compiler::types::{BaseTypes, TypeBase};

#[derive(Debug, Clone)]
pub struct ReturnType {}

impl TypeBase for ReturnType {
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Return
    }
}
