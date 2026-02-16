mod semantic_analyzer;
mod type_resolver;
mod validation_rules;
mod desugar;

use crate::compiler::semantic_analyzer::SemanticAnalyzer;
use crate::compiler::type_resolver::TypeResolver;
use anyhow::Result;
use codegen::builder::LLVMCodegenBuilder;
use codegen::code_generator::CodeGenerator;
use parser::Expression;

pub use codegen::CompileOptions;
pub use desugar::desugar_program;

pub fn compile(exprs: Vec<Expression>, options: Option<CompileOptions>) -> Result<String> {
    let exprs = desugar::desugar_program(exprs);
    let mut type_resolver = TypeResolver::new();
    let mut typed_exprs = Vec::new();
    for expr in exprs {
        let (typed_expr, ty) = type_resolver.resolve_expression(&expr)?;
        typed_exprs.push((typed_expr, ty));
    }

    let mut analyzer = SemanticAnalyzer::new();

    analyzer.validate_program(&typed_exprs)?;
    for (typed_expr, _) in &typed_exprs {
        analyzer.analyze(typed_expr)?;
    }

    let mut codegen_builder = LLVMCodegenBuilder::init(options)?;
    let mut generator = CodeGenerator::new(&mut codegen_builder);

    for (typed_expr, _ty) in typed_exprs {
        generator.generate_expression(&typed_expr)?;
    }

    codegen_builder.dispose_and_get_module_str()
}
