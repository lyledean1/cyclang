use crate::compiler::codegen::*;

extern crate llvm_sys;
use crate::compiler::codegen::builder::LLVMCodegenBuilder;
use crate::compiler::types::{BaseTypes, Func, TypeBase};
use anyhow::anyhow;
use anyhow::Result;
use llvm_sys::prelude::*;

#[derive(Clone)]
pub struct BoolType {
    pub builder: LLVMBuilderRef,
    pub llvm_value: LLVMValueRef,
    pub llvm_value_pointer: LLVMValueRef,
    pub name: String,
}

fn get_value_for_print_argument(
    codegen: &mut LLVMCodegenBuilder,
    name: &str,
    value: BoolType,
) -> Vec<LLVMValueRef> {
    match value.get_ptr() {
        Some(v) => vec![codegen.build_load(v, int1_type(), name)],
        None => vec![value.get_value()],
    }
}

impl TypeBase for BoolType {
    fn get_value(&self) -> LLVMValueRef {
        self.llvm_value
    }
    fn get_ptr(&self) -> Option<LLVMValueRef> {
        Some(self.llvm_value_pointer)
    }
    fn print(&self, codegen: &mut LLVMCodegenBuilder) -> Result<()> {
        let bool_func_args = get_value_for_print_argument(codegen, "", self.clone());

        let bool_to_string_func = codegen
            .llvm_func_cache
            .get("bool_to_str")
            .ok_or(anyhow!("unable to find bool_to_str function"))?;
        let str_value = codegen.build_call(bool_to_string_func, bool_func_args, 1, "");
        let print_args: Vec<LLVMValueRef> = vec![str_value];
        let print_func = codegen
            .llvm_func_cache
            .get("printf")
            .ok_or(anyhow!("unable to find printf function"))?;
        codegen.build_call(print_func, print_args, 1, "");
        Ok(())
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Bool
    }
}
impl Func for BoolType {}
