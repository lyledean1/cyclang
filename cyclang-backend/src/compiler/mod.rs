use crate::compiler::codegen::target::Target;
use crate::compiler::context::{ASTContext, LLVMCodegenVisitor};
use anyhow::Result;
use cyclang_parser::Expression;
use crate::compiler::types::TypeBase;
use crate::compiler::visitor::Visitor;

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
    let mut ast_ctx = ASTContext::init(compile_options)?;
    let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor{});
    for expr in exprs {
        ast_ctx.match_ast(expr, &mut visitor)?;
    }
    ast_ctx.dispose_and_get_module_str()
}

