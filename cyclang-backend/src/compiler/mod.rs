use crate::compiler::codegen::builder::LLVMCodegenBuilder;
use crate::compiler::codegen::target::Target;
use crate::compiler::context::{ASTContext, LLVMCodegenVisitor};
use crate::compiler::types::TypeBase;
use crate::compiler::visitor::Visitor;
use anyhow::Result;
use cyclang_parser::Expression;

extern crate llvm_sys;

pub mod codegen;
pub mod context;
pub mod types;
pub mod visitor;

#[derive(Debug, Clone, Copy)]
pub struct CompileOptions {
    pub is_execution_engine: bool,
    pub target: Option<Target>,
}

pub fn compile(exprs: Vec<Expression>, compile_options: Option<CompileOptions>) -> Result<String> {
    // output LLVM IR
    let mut ast_ctx = ASTContext::init()?;
    let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor {});
    let mut codegen = LLVMCodegenBuilder::init(compile_options)?;
    for expr in exprs {
        ast_ctx.match_ast(expr, &mut visitor, &mut codegen)?;
    }
    codegen.dispose_and_get_module_str()
}
