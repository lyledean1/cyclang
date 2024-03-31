use crate::compiler::codegen::builder::LLVMCodegenBuilder;
use crate::compiler::context::{ASTContext, VariableCache};
use crate::compiler::types::TypeBase;
use anyhow::Result;
use cyclang_parser::Expression;

pub trait Visitor<T> {
    fn visit_number(&mut self, expression: &Expression, codegen: &LLVMCodegenBuilder) -> Result<T>;

    fn visit_string(&mut self, expression: &Expression, codegen: &LLVMCodegenBuilder) -> Result<T>;

    fn visit_bool(&mut self, expression: &Expression, codegen: &LLVMCodegenBuilder) -> Result<T>;

    fn visit_variable(
        &mut self,
        expression: &Expression,
        codegen: &LLVMCodegenBuilder,
        var_cache: &VariableCache,
    ) -> Result<T>;

    fn visit_list(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;

    fn visit_list_index(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;

    fn visit_list_assign(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;

    fn visit_nil(&mut self) -> Result<Box<dyn TypeBase>>;

    fn visit_binary(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;

    fn visit_grouping(
        &mut self,
        left: Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;

    fn visit_let_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;
}
