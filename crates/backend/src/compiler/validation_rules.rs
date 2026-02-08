use codegen::typed_ast::{ResolvedType, TypedExpression};
use anyhow::{anyhow, Result};

/// Trait for validation rules that can be applied to a program
pub trait ValidationRule {
    /// Returns the name of this validation rule
    fn name(&self) -> &str;

    /// Validates the program and returns an error if validation fails
    fn validate(&self, program: &[(TypedExpression, ResolvedType)]) -> Result<()>;
}

/// Rule: Program must contain a 'main' function
pub struct RequireMainFunction;

impl ValidationRule for RequireMainFunction {
    fn name(&self) -> &str {
        "require-main-function"
    }

    fn validate(&self, program: &[(TypedExpression, ResolvedType)]) -> Result<()> {
        let has_main = program.iter().any(
            |(expr, _)| matches!(expr, TypedExpression::FuncStmt { name, .. } if name == "main"),
        );

        if !has_main {
            return Err(anyhow!(
                "Program must contain a 'main' function as the entry point"
            ));
        }

        Ok(())
    }
}
