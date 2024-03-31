use crate::compiler::codegen::builder::LLVMCodegenBuilder;
use crate::compiler::context::{ASTContext};
use crate::compiler::types::TypeBase;
use anyhow::Result;
use cyclang_parser::Expression;

pub trait Visitor<T> {
    fn visit_number(&mut self, expression: &Expression, codegen: &LLVMCodegenBuilder) -> Result<T>;

    fn visit_string(&mut self, expression: &Expression, codegen: &LLVMCodegenBuilder) -> Result<T>;

    fn visit_bool(&mut self, expression: &Expression, codegen: &LLVMCodegenBuilder) -> Result<T>;

    fn visit_variable_expr(
        &mut self,
        expression: &Expression,
        codegen: &LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<T>;

    fn visit_list_expr(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;

    fn visit_list_index_expr(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;

    fn visit_list_assign_expr(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;

    fn visit_nil(&mut self) -> Result<Box<dyn TypeBase>>;

    fn visit_binary_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;

    fn visit_grouping_stmt(
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

    fn visit_block_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;

    fn visit_call_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;

    fn visit_func_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;

    fn visit_if_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;

    fn visit_while_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;

    fn visit_for_loop_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;

    fn visit_print_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;

    fn visit_return_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>>;
}
