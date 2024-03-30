use crate::compiler::codegen::builder::LLVMCodegenBuilder;
use anyhow::Result;
use cyclang_parser::Expression;
use crate::compiler::context::{VariableCache};
use crate::compiler::types::TypeBase;

pub trait Visitor<T> {
    fn visit_number(&mut self, expression: &Expression, codegen: &LLVMCodegenBuilder) -> Result<T>;

    fn visit_string(&mut self, expression: &Expression, codegen: &LLVMCodegenBuilder) -> Result<T>;

    fn visit_bool(&mut self, expression: &Expression, codegen: &LLVMCodegenBuilder) -> Result<T>;

    fn visit_variable(&mut self, expression: &Expression, codegen: &LLVMCodegenBuilder, var_cache: &VariableCache) -> Result<T>;

    fn visit_binary(
        &mut self,
        left: &Expression,
        codegen: &LLVMCodegenBuilder,
    ) -> Result<Box<dyn TypeBase>> ;
    fn visit_list(&mut self, left: &Expression, codegen: &LLVMCodegenBuilder) -> Result<Box<dyn TypeBase>>;
}
