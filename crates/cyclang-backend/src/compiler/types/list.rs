extern crate llvm_sys;

use crate::compiler::codegen::builder::LLVMCodegenBuilder;
use crate::compiler::types::{BaseTypes, TypeBase};
use anyhow::anyhow;
use anyhow::Result;
use llvm_sys::prelude::*;
use crate::compiler::codegen::int32_ptr_type;
use crate::compiler::types::num::NumberType;

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
    fn print(&self, codegen: &mut LLVMCodegenBuilder) -> Result<()> {
        if let BaseTypes::List(inner_type) = self.get_type() {
            let inner_type_func = get_c_print_fn_name(*inner_type);
            let print_func = codegen.llvm_func_cache.get(inner_type_func).ok_or(anyhow!("unable to get func {}", inner_type_func))?;
            codegen.build_call(print_func, vec![self.get_value()], 1, "");
            return Ok(())
        }
        Err(anyhow!("unable to print list type {:?}", self.get_type()))
    }

    fn len(&self, codegen: &mut LLVMCodegenBuilder) -> Result<Box<dyn TypeBase>> {
        if let BaseTypes::List(inner_type) = self.get_type() {
            let inner_type_func = get_c_len_fn_name(*inner_type);
            let len_func = codegen.llvm_func_cache.get(inner_type_func).ok_or(anyhow!("unable to get func {}", inner_type_func))?;
            let value = codegen.build_call(len_func, vec![self.get_value()], 1, "");
            let ptr = codegen.build_alloca_store(value, int32_ptr_type(), "length");
            return Ok(Box::new(NumberType{
                llvm_value: value,
                llvm_value_pointer: Some(ptr),
                name: "".to_string(),
            }))
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

fn get_c_print_fn_name(base_type: BaseTypes) -> &'static str {
    match base_type {
        BaseTypes::String => "printStringList",
        BaseTypes::Number => "printInt32List",
        _ => {
            unreachable!("No print function set up for type {:?}", base_type)
        }
    }
}

fn get_c_len_fn_name(base_type: BaseTypes) -> &'static str {
    match base_type {
        BaseTypes::String => "lenStringList",
        BaseTypes::Number => "lenInt32List",
        _ => {
            unreachable!("No print function set up for type {:?}", base_type)
        }
    }
}
