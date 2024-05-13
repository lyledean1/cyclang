extern crate llvm_sys;

use crate::compiler::codegen::builder::LLVMCodegenBuilder;
use crate::compiler::types::{BaseTypes, TypeBase};
use anyhow::anyhow;
use llvm_sys::prelude::*;

#[derive(Debug, Clone)]
pub struct ListType {
    pub llvm_value: LLVMValueRef,
    pub llvm_value_ptr: LLVMValueRef,
    pub llvm_type: LLVMTypeRef,
    pub inner_type: BaseTypes,
}

impl TypeBase for ListType {
    fn get_value(&self) -> LLVMValueRef {
        self.llvm_value
    }

    fn get_ptr(&self) -> Option<LLVMValueRef> {
        Some(self.llvm_value_ptr)
    }
    fn print(&self, codegen: &mut LLVMCodegenBuilder) -> anyhow::Result<()> {
        if let BaseTypes::List(inner_type) = self.get_type() {
            match *inner_type {
                BaseTypes::String => {
                    let print_list_func = codegen.llvm_func_cache.get("printStringList").unwrap();
                    codegen.build_call(print_list_func, vec![self.get_value()], 1, "");
                    return Ok(());
                }
                BaseTypes::Number => {
                    let print_list_func = codegen.llvm_func_cache.get("printInt32List").unwrap();
                    codegen.build_call(print_list_func, vec![self.get_value()], 1, "");
                    return Ok(());
                }
                _ => {
                    unimplemented!("type {:?} not implemented", self.get_type())
                }
            }
        }
        Err(anyhow!("unable to print list type {:?}", self.get_type()))
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::List(Box::new(self.inner_type.clone()))
    }

    fn get_llvm_type(&self) -> LLVMTypeRef {
        self.llvm_type
    }
}
