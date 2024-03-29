#![allow(dead_code)]
use crate::compiler::codegen::*;
use crate::compiler::types::num::NumberType;
use crate::compiler::types::{TypeBase};
use anyhow::Result;
use crate::compiler::context::ASTContext;
use crate::compiler::codegen::target::Target;
use crate::parser::Expression;

extern crate llvm_sys;
use llvm_sys::prelude::*;

pub mod codegen;
pub mod types;
pub mod context;

#[derive(Debug, Clone, Copy)]
pub struct CompileOptions {
    pub is_execution_engine: bool,
    pub target: Option<Target>,
}
struct ExprContext {
    alloca: Option<LLVMValueRef>,
}

pub fn compile(exprs: Vec<Expression>, compile_options: Option<CompileOptions>) -> Result<String> {
    // output LLVM IR
    let mut ast_ctx = ASTContext::init(compile_options)?;
    for expr in exprs {
        ast_ctx.match_ast(expr)?;
    }
    ast_ctx.dispose_and_get_module_str()
}
