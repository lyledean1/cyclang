use crate::compiler::codegen::builder::LLVMCodegenBuilder;
use crate::compiler::codegen::target::Target;
use crate::compiler::context::{ASTContext, LLVMCodegenVisitor};
use crate::compiler::types::TypeBase;
use crate::compiler::visitor::Visitor;
use anyhow::Result;
use cyclang_parser::Expression;
use crate::compiler::types::func::FuncType;

extern crate llvm_sys;

pub mod cache;
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

    let bool_to_str_func = codegen.llvm_func_cache.get("boolToStrZig").unwrap();

    ast_ctx.func_cache.set("boolToStrZig", Box::new(FuncType{
        return_type: bool_to_str_func.return_type,
        llvm_type: bool_to_str_func.func_type,
        llvm_func: bool_to_str_func.function,
    }), 0);

    for expr in exprs {
        ast_ctx.match_ast(expr, &mut visitor, &mut codegen)?;
    }
    codegen.dispose_and_get_module_str()
}
