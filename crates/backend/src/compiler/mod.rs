mod semantic_analyzer;
mod type_resolver;
mod validation_rules;
mod desugar;

use crate::compiler::semantic_analyzer::SemanticAnalyzer;
use crate::compiler::type_resolver::TypeResolver;
use anyhow::{Context, Result};
use codegen::builder::LLVMCodegenBuilder;
use codegen::code_generator::CodeGenerator;
use parser::Expression;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;
use std::fs;

pub use codegen::CompileOptions;
pub use desugar::desugar_program;

pub fn compile(exprs: Vec<Expression>, options: Option<CompileOptions>) -> Result<String> {
    let (imports, exprs) = extract_imports(exprs);
    let exprs = apply_imports(exprs, &imports)?;
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

fn extract_imports(exprs: Vec<Expression>) -> (Vec<String>, Vec<Expression>) {
    let mut imports = Vec::new();
    let mut filtered = Vec::new();
    for expr in exprs {
        match expr {
            Expression::ImportStmt(module) => imports.push(module),
            other => filtered.push(other),
        }
    }
    (imports, filtered)
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

fn apply_imports(mut exprs: Vec<Expression>, imports: &[String]) -> Result<Vec<Expression>> {
    let mut prelude_exprs = Vec::new();
    let mut system_imported = false;
    for module in imports {
        if module == "System" {
            system_imported = true;
            prelude_exprs.extend(load_system_prelude()?);
        } else {
            return Err(anyhow::anyhow!("Unknown import module: {}", module));
        }
    }

    if system_imported {
        for expr in &mut exprs {
            rewrite_system_calls(expr);
        }
    }

    if !prelude_exprs.is_empty() {
        prelude_exprs.extend(exprs);
        Ok(prelude_exprs)
    } else {
        Ok(exprs)
    }
}

fn load_system_prelude() -> Result<Vec<Expression>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let prelude_path = root.join("preludes/System.cyc");
    let prelude_src = fs::read_to_string(&prelude_path).with_context(|| {
        format!(
            "failed to read System prelude file at {}",
            prelude_path.display()
        )
    })?;

    let mut exprs = parser::parse_cyclo_program(&prelude_src).map_err(|e| {
        anyhow::anyhow!(
            "failed to parse System prelude {}: {}",
            prelude_path.display(),
            e
        )
    })?;

    let base_dir = prelude_path.parent().unwrap_or(Path::new("."));
    rewrite_prelude_paths(&mut exprs, base_dir);

    Ok(exprs)
}

fn rewrite_prelude_paths(exprs: &mut [Expression], base_dir: &Path) {
    for expr in exprs {
        if let Expression::ExternModule(path) = expr {
            let path_buf = PathBuf::from(path.as_str());
            if path_buf.is_relative() {
                let full_path = base_dir.join(path_buf);
                *path = full_path.to_string_lossy().to_string();
            }
        }
    }
}

fn rewrite_system_calls(expr: &mut Expression) {
    match expr {
        Expression::CallStmt(name, args) => {
            if let Some(stripped) = name.strip_prefix("System.") {
                *name = stripped.to_string();
            }
            for arg in args {
                rewrite_system_calls(arg);
            }
        }
        Expression::Binary(lhs, _, rhs) => {
            rewrite_system_calls(lhs);
            rewrite_system_calls(rhs);
        }
        Expression::Grouping(inner) => rewrite_system_calls(inner),
        Expression::LetStmt(_, _, value) => rewrite_system_calls(value),
        Expression::BlockStmt(stmts) => {
            for stmt in stmts {
                rewrite_system_calls(stmt);
            }
        }
        Expression::FuncStmt(_, _, _, body) => rewrite_system_calls(body),
        Expression::IfStmt(cond, then_branch, else_branch) => {
            rewrite_system_calls(cond);
            rewrite_system_calls(then_branch);
            if let Some(else_expr) = else_branch.as_mut().as_mut() {
                rewrite_system_calls(else_expr);
            }
        }
        Expression::WhileStmt(cond, body) => {
            rewrite_system_calls(cond);
            rewrite_system_calls(body);
        }
        Expression::ReturnStmt(value) => rewrite_system_calls(value),
        Expression::ForStmt(_, _, _, _, body) => rewrite_system_calls(body),
        Expression::Print(value) => rewrite_system_calls(value),
        Expression::Len(value) => rewrite_system_calls(value),
        Expression::List(values) => {
            for v in values {
                rewrite_system_calls(v);
            }
        }
        Expression::ListIndex(list, index) => {
            rewrite_system_calls(list);
            rewrite_system_calls(index);
        }
        Expression::ListAssign(_, index, value) => {
            rewrite_system_calls(index);
            rewrite_system_calls(value);
        }
        _ => {}
    }
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
