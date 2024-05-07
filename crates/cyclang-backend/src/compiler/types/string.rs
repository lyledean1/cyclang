use crate::compiler::types::{BaseTypes, TypeBase};

extern crate llvm_sys;
use anyhow::Result;

use crate::compiler::codegen::builder::LLVMCodegenBuilder;
use llvm_sys::prelude::*;

#[derive(Debug, Clone)]
pub struct StringType {
    pub name: String,
    pub llvm_value: LLVMValueRef,
    pub length: *mut usize,
    pub llvm_value_pointer: Option<LLVMValueRef>,
    pub str_value: String,
}
impl TypeBase for StringType {
    fn get_value(&self) -> LLVMValueRef {
        self.llvm_value
    }
    fn get_ptr(&self) -> Option<LLVMValueRef> {
        match self.llvm_value_pointer {
            Some(v) => Some(v),
            None => {
                unreachable!("No pointer for this value")
            }
        }
    }
    fn get_str(&self) -> String {
        self.str_value.clone()
    }
    fn print(&self, codegen: &mut LLVMCodegenBuilder) -> Result<()> {
        let string_print_func = codegen.llvm_func_cache.get("stringPrint").unwrap();
        let lhs_value = self.get_value();
        let args = vec![lhs_value];
        codegen.build_call(string_print_func, args, 1, "");
        Ok(())
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::String
    }
}
