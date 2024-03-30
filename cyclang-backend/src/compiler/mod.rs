use crate::compiler::codegen::target::Target;
use crate::compiler::context::ASTContext;
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
    let mut ast_ctx = ASTContext::init(compile_options)?;
    for expr in exprs {
        ast_ctx.match_ast(expr)?;
    }
    ast_ctx.dispose_and_get_module_str()
}
