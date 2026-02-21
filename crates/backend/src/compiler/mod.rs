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
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

pub use codegen::CompileOptions;
pub use desugar::desugar_program;

pub fn compile(exprs: Vec<Expression>, options: Option<CompileOptions>) -> Result<String> {
    let (extern_modules, exprs) = extract_extern_modules(exprs);
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
    let _extern_bc_files = link_extern_modules(&mut codegen_builder, &extern_modules)?;
    let mut generator = CodeGenerator::new(&mut codegen_builder);

    for (typed_expr, _ty) in typed_exprs {
        generator.generate_expression(&typed_expr)?;
    }

    codegen_builder.dispose_and_get_module_str()
}

fn extract_extern_modules(exprs: Vec<Expression>) -> (Vec<String>, Vec<Expression>) {
    let mut modules = Vec::new();
    let mut filtered = Vec::new();
    for expr in exprs {
        match expr {
            Expression::ExternModule(path) => modules.push(path),
            other => filtered.push(other),
        }
    }
    (modules, filtered)
}

fn link_extern_modules(
    builder: &mut LLVMCodegenBuilder,
    modules: &[String],
) -> Result<Vec<NamedTempFile>> {
    let mut temp_files = Vec::new();
    for module in modules {
        let bc_path = if module.ends_with(".bc") {
            module.clone()
        } else {
            let temp = compile_c_to_bc(module)?;
            let path = temp.path().to_str().unwrap().to_string();
            temp_files.push(temp);
            path
        };
        builder.link_bitcode_file(&bc_path)?;
    }
    Ok(temp_files)
}

fn compile_c_to_bc(path: &str) -> Result<NamedTempFile> {
    let input_path = Path::new(path);
    if input_path.extension().and_then(|s| s.to_str()) != Some("c") {
        return Err(anyhow::anyhow!(
            "extern module supports .c or .bc files, got: {}",
            path
        ));
    }

    let temp = tempfile::Builder::new().suffix(".bc").tempfile()?;
    let output_path = temp.path().to_str().unwrap();

    let output = Command::new("clang")
        .args(["-c", "-emit-llvm", "-O0"])
        .arg(path)
        .arg("-o")
        .arg(output_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "failed to compile extern module {}: {}",
            path,
            stderr
        ));
    }

    Ok(temp)
}
