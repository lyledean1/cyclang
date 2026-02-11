use crate::compiler::validation_rules::{RequireMainFunction, ValidationRule};
use codegen::typed_ast::{ResolvedType, TypedExpression};
use anyhow::{Context, Result};

pub struct SemanticAnalyzer {
    validation_rules: Vec<Box<dyn ValidationRule>>,
}

impl SemanticAnalyzer {
    /// Creates a new SemanticAnalyzer with default validation rules
    pub fn new() -> Self {
        Self::default_rules()
    }

    /// Creates a SemanticAnalyzer with the default set of validation rules
    pub fn default_rules() -> Self {
        Self {
            validation_rules: vec![
                Box::new(RequireMainFunction),
                // Easy to add more rules here!
            ],
        }
    }

    /// Validates the entire program by running all validation rules
    pub fn validate_program(&self, typed_exprs: &[(TypedExpression, ResolvedType)]) -> Result<()> {
        for rule in &self.validation_rules {
            rule.validate(typed_exprs)
                .with_context(|| format!("Validation rule '{}' failed", rule.name()))?;
        }
        Ok(())
    }

    pub fn analyze(&mut self, typed_expr: &TypedExpression) -> Result<()> {
        match typed_expr {
            TypedExpression::Number32 { value: _, .. } => self.analyze_number(typed_expr),
            TypedExpression::Number64 { value: _, .. } => self.analyze_number(typed_expr),
            TypedExpression::String { value: _ } => Ok(()),
            TypedExpression::Bool { value: _ } => Ok(()),
            TypedExpression::Binary { left, op, right } => self.analyse_binary(left, op, right),
            TypedExpression::CallStmt { callee, args: _ } => self.analyze_call(callee),
            TypedExpression::FuncStmt {
                name: _,
                args: _,
                return_type: _,
                body,
            } => {
                self.analyze(body)?;
                Ok(())
            }
            TypedExpression::BlockStmt { statements } => {
                for stmt in statements {
                    self.analyze(stmt)?;
                }
                Ok(())
            }
            TypedExpression::Variable { name: _ } => Ok(()),
            TypedExpression::Print { value } => self.analyze(value),
            TypedExpression::ReturnStmt { value } => self.analyze(value),
            TypedExpression::LetStmt {
                name: _,
                var_type: _,
                value,
            } => self.analyze(value),
            TypedExpression::IfStmt {
                condition,
                then_branch,
                else_branch,
            } => {
                self.analyze(condition)?;
                self.analyze(then_branch)?;
                if let Some(else_expr) = else_branch {
                    self.analyze(else_expr)?;
                }
                Ok(())
            }
            TypedExpression::WhileStmt { condition, body } => {
                self.analyze(condition)?;
                self.analyze(body)?;
                Ok(())
            }
            TypedExpression::AssignStmt { name: _, value } => {
                self.analyze(value)?;
                Ok(())
            }
            TypedExpression::Grouping { inner } => {
                // Analyze the inner expression
                self.analyze(inner)
            }
            TypedExpression::List {
                elements,
                element_type: _,
            } => {
                // Analyze all list elements
                for elem in elements {
                    self.analyze(elem)?;
                }
                Ok(())
            }
            TypedExpression::ListIndex { list, index } => {
                // Analyze the list and index expressions
                self.analyze(list)?;
                self.analyze(index)?;
                Ok(())
            }
            TypedExpression::ListAssign {
                name: _,
                index,
                value,
            } => {
                // Analyze the index and value expressions
                self.analyze(index)?;
                self.analyze(value)?;
                Ok(())
            }
            TypedExpression::Len { value } => {
                // Analyze the value expression
                self.analyze(value)
            }
        }
    }
    pub fn analyze_number(&mut self, typed_expr: &TypedExpression) -> Result<()> {
        match typed_expr {
            TypedExpression::Number32 { value: _ } => {
                // Could check for specific constraints
                // if *value < 0 && self.context.expects_unsigned() {
                //     return Err(SemanticError::NegativeUnsigned {
                //         value: *value,
                //         span: *span
                //     });
                // }
                Ok(())
            }
            TypedExpression::Number64 { value: _ } => Ok(()),
            _ => unreachable!(),
        }
    }

    pub fn analyze_call(&mut self, _typed_expr: &TypedExpression) -> Result<()> {
        Ok(())
    }

    pub fn analyse_binary(
        &mut self,
        _left: &TypedExpression,
        _op: &String,
        _right: &TypedExpression,
    ) -> Result<()> {
        Ok(())
    }
}
