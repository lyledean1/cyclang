extern crate llvm_sys;
use crate::compiler::types::{BaseTypes, TypeBase};

#[derive(Debug, Clone)]
pub struct VoidType {}

impl TypeBase for VoidType {
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Void
    }
}
