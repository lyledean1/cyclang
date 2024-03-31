extern crate llvm_sys;

use crate::compiler::context::ASTContext;
use crate::compiler::types::{BaseTypes, Func, TypeBase};
use llvm_sys::core::LLVMConstArray2;
use llvm_sys::prelude::*;
use std::any::Any;

#[derive(Debug, Clone)]
pub struct ListType {
    pub llvm_value: LLVMValueRef,
    pub llvm_value_ptr: LLVMValueRef,
    pub llvm_type: LLVMTypeRef,
}

impl TypeBase for ListType {
    fn new(_value: Box<dyn Any>, _name: String, _context: &mut ASTContext) -> Box<dyn TypeBase> {
        let value_as_expr_list = match _value.downcast_ref::<Box<Vec<Box<dyn TypeBase>>>>() {
            Some(val) => val,
            None => panic!("The input value is incorrect"),
        };
        let first_element = value_as_expr_list.first().unwrap();
        let mut elements = vec![];
        for x in value_as_expr_list.iter() {
            elements.push(x.get_value());
        }

        unsafe {
            let _llvm_array_value = LLVMConstArray2(
                first_element.get_llvm_type(),
                elements.as_mut_ptr(),
                value_as_expr_list.len() as u64,
            );
            unimplemented!()
        }
    }
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

impl Func for ListType {}
