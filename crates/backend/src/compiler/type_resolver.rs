use codegen::typed_ast::{ResolvedType, TypedExpression};
use anyhow::Result;
use parser::Expression;
use std::collections::HashMap;

pub struct TypeResolver {
    // Symbol table: variable name -> type
    // Maps to track scoping (depth -> list of variables at that depth)
    symbol_table: HashMap<String, ResolvedType>,
    locals: HashMap<i32, Vec<String>>,
    depth: i32,
}

impl TypeResolver {
    pub fn new() -> Self {
        TypeResolver {
            symbol_table: HashMap::new(),
            locals: HashMap::new(),
            depth: 0,
        }
    }

    fn set_variable(&mut self, name: &str, ty: ResolvedType) {
        self.symbol_table.insert(name.to_string(), ty);
        self.locals
            .entry(self.depth)
            .or_default()
            .push(name.to_string());
    }

    fn get_variable(&self, name: &str) -> Option<&ResolvedType> {
        self.symbol_table.get(name)
    }

    fn incr_depth(&mut self) {
        self.depth += 1;
    }

    fn decr_depth(&mut self) {
        if let Some(vars) = self.locals.remove(&self.depth) {
            for var in vars {
                self.symbol_table.remove(&var);
            }
        }
        self.depth -= 1;
    }

    pub fn resolve_expression(
        &mut self,
        expr: &Expression,
    ) -> Result<(TypedExpression, ResolvedType)> {
        match expr {
            Expression::Number(val) => {
                Ok((
                    TypedExpression::Number32 {
                        value: *val,
                        // span: expr.span()
                    },
                    ResolvedType::I32,
                ))
            }
            Expression::Number64(val) => {
                Ok((
                    TypedExpression::Number64 {
                        value: *val,
                        // span: expr.span() - add error handling
                    },
                    ResolvedType::I64,
                ))
            }
            Expression::String(val) => Ok((
                TypedExpression::String { value: val.clone() },
                ResolvedType::String,
            )),
            Expression::Bool(val) => {
                Ok((TypedExpression::Bool { value: *val }, ResolvedType::Bool))
            }
            Expression::Binary(left, op, right) => {
                let (lhs, lhs_ty) = self.resolve_expression(left)?;
                let (rhs, rhs_ty) = self.resolve_expression(right)?;

                // Infer the result type of the binary operation
                let result_type = match op.as_str() {
                    // Comparison operators always return Bool
                    "==" | "!=" | "<" | "<=" | ">" | ">=" => ResolvedType::Bool,
                    // Arithmetic operators return numeric type
                    "+" | "-" | "*" | "/" => match (&lhs_ty, &rhs_ty) {
                        (ResolvedType::I32, ResolvedType::I32) => ResolvedType::I32,
                        (ResolvedType::I64, ResolvedType::I64) => ResolvedType::I64,
                        (ResolvedType::I32, ResolvedType::I64)
                        | (ResolvedType::I64, ResolvedType::I32) => ResolvedType::I64,
                        _ => ResolvedType::Binary(
                            Box::new(lhs_ty.clone()),
                            op.to_string(),
                            Box::new(rhs_ty.clone()),
                        ),
                    },
                    // Default fallback
                    _ => ResolvedType::Binary(
                        Box::new(lhs_ty.clone()),
                        op.to_string(),
                        Box::new(rhs_ty.clone()),
                    ),
                };

                Ok((
                    TypedExpression::Binary {
                        left: Box::new(lhs),
                        op: op.to_string(),
                        right: Box::new(rhs),
                    },
                    result_type,
                ))
            }
            Expression::FuncStmt(name, args, return_type, body) => {
                // Resolve argument types
                let mut typed_args = Vec::new();
                for arg in args {
                    match arg {
                        Expression::FuncArg(arg_name, arg_type) => {
                            let resolved_arg_type = self.resolve_type(arg_type);
                            typed_args.push((arg_name.clone(), resolved_arg_type));
                        }
                        _ => return Err(anyhow::anyhow!("Expected FuncArg in function arguments")),
                    }
                }

                // Resolve return type
                let resolved_return_type = self.resolve_type(return_type);

                // Create a new scope for the function body
                self.incr_depth();

                // Add function parameters to the symbol table
                for (arg_name, arg_type) in &typed_args {
                    self.set_variable(arg_name, arg_type.clone());
                }

                // Resolve body (can now reference parameters)
                let (typed_body, _body_ty) = self.resolve_expression(body)?;

                // Exit the function scope
                self.decr_depth();

                let func_type = ResolvedType::Function(
                    typed_args.iter().map(|(_, ty)| ty.clone()).collect(),
                    Box::new(resolved_return_type.clone()),
                );

                Ok((
                    TypedExpression::FuncStmt {
                        name: name.clone(),
                        args: typed_args,
                        return_type: resolved_return_type,
                        body: Box::new(typed_body),
                    },
                    func_type,
                ))
            }
            Expression::BlockStmt(statements) => {
                self.incr_depth();
                let mut typed_statements = Vec::new();
                let mut last_type = ResolvedType::Void;

                for stmt in statements {
                    let (typed_stmt, stmt_ty) = self.resolve_expression(stmt)?;
                    typed_statements.push(typed_stmt);
                    last_type = stmt_ty;
                }

                self.decr_depth();

                Ok((
                    TypedExpression::BlockStmt {
                        statements: typed_statements,
                    },
                    last_type,
                ))
            }
            Expression::Variable(name) => {
                let var_type = self
                    .get_variable(name)
                    .ok_or_else(|| anyhow::anyhow!("Undefined variable: {}", name))?
                    .clone();
                Ok((TypedExpression::Variable { name: name.clone() }, var_type))
            }
            Expression::Print(value) => {
                let (typed_value, _) = self.resolve_expression(value)?;
                Ok((
                    TypedExpression::Print {
                        value: Box::new(typed_value),
                    },
                    ResolvedType::Void,
                ))
            }
            Expression::ReturnStmt(value) => {
                let (typed_value, value_ty) = self.resolve_expression(value)?;
                Ok((
                    TypedExpression::ReturnStmt {
                        value: Box::new(typed_value),
                    },
                    value_ty,
                ))
            }
            Expression::CallStmt(name, args) => {
                let mut typed_args = Vec::new();
                for arg in args {
                    let (typed_arg, _) = self.resolve_expression(arg)?;
                    typed_args.push(typed_arg);
                }

                Ok((
                    TypedExpression::CallStmt {
                        callee: Box::new(TypedExpression::Variable { name: name.clone() }),
                        args: typed_args,
                    },
                    ResolvedType::Void, // TODO: lookup function return type
                ))
            }
            Expression::LetStmt(name, var_type, value) => {
                // Resolve the value expression first
                let (typed_value, value_type) = self.resolve_expression(value)?;

                // Check if variable already exists - if so, this is reassignment, not declaration
                if let Some(existing_type) = self.get_variable(name) {
                    // This is reassignment (e.g., "x = 5" without "let")
                    // Type check: new value must match existing variable's type
                    if existing_type != &value_type {
                        return Err(anyhow::anyhow!(
                            "Cannot reassign variable '{}' of type {:?} to value of type {:?}",
                            name,
                            existing_type,
                            value_type
                        ));
                    }

                    // Return AssignStmt instead of LetStmt
                    return Ok((
                        TypedExpression::AssignStmt {
                            name: name.clone(),
                            value: Box::new(typed_value),
                        },
                        ResolvedType::Void, // Assignments don't return values
                    ));
                }

                // This is a new variable declaration
                // Resolve the declared type
                let declared_type = self.resolve_type(var_type);

                // Type checking: ensure value matches declared type
                // For Type::None, we allow type inference
                if declared_type != ResolvedType::Void && declared_type != value_type {
                    return Err(anyhow::anyhow!(
                        "Type mismatch for variable '{}': declared as {:?}, but value is {:?}",
                        name,
                        declared_type,
                        value_type
                    ));
                }

                // Use declared type if not void, otherwise infer from value
                let final_type = if declared_type == ResolvedType::Void {
                    value_type
                } else {
                    declared_type
                };

                // Add to symbol table
                self.set_variable(name, final_type.clone());

                Ok((
                    TypedExpression::LetStmt {
                        name: name.clone(),
                        var_type: Some(final_type.clone()),
                        value: Box::new(typed_value),
                    },
                    final_type,
                ))
            }
            Expression::IfStmt(condition, then_branch, else_branch) => {
                // Resolve condition - should be boolean
                let (typed_condition, cond_type) = self.resolve_expression(condition)?;

                // Type check: condition should be boolean
                if cond_type != ResolvedType::Bool {
                    return Err(anyhow::anyhow!(
                        "If condition must be boolean, got {:?}",
                        cond_type
                    ));
                }

                // Resolve then branch
                let (typed_then, _then_type) = self.resolve_expression(then_branch)?;

                // Resolve else branch if it exists
                // Note: Parser returns Box<Option<Expression>>, not Option<Box<Expression>>
                let typed_else = if let Some(else_expr) = else_branch.as_ref().as_ref() {
                    let (typed_else_branch, _else_type) = self.resolve_expression(else_expr)?;
                    Some(Box::new(typed_else_branch))
                } else {
                    None
                };

                Ok((
                    TypedExpression::IfStmt {
                        condition: Box::new(typed_condition),
                        then_branch: Box::new(typed_then),
                        else_branch: typed_else,
                    },
                    ResolvedType::Void, // If statements don't return values
                ))
            }
            Expression::WhileStmt(condition, body) => {
                // Resolve condition - should be boolean
                let (typed_condition, cond_type) = self.resolve_expression(condition)?;

                // Type check: condition should be boolean
                if cond_type != ResolvedType::Bool {
                    return Err(anyhow::anyhow!(
                        "While condition must be boolean, got {:?}",
                        cond_type
                    ));
                }

                // Resolve body
                let (typed_body, _body_type) = self.resolve_expression(body)?;

                Ok((
                    TypedExpression::WhileStmt {
                        condition: Box::new(typed_condition),
                        body: Box::new(typed_body),
                    },
                    ResolvedType::Void, // While loops don't return values
                ))
            }
            Expression::ForStmt(var_name, start, end, step, body) => {
                // For loops declare a loop variable that's scoped to the loop
                // We need to:
                // 1. Create a new scope
                // 2. Declare the loop variable
                // 3. Resolve the body (which can reference the loop variable)
                // 4. Exit the scope

                self.incr_depth();

                // Declare the loop variable with type i32
                self.set_variable(var_name, ResolvedType::I32);

                // Resolve body
                let (typed_body, _body_type) = self.resolve_expression(body)?;

                self.decr_depth();

                Ok((
                    TypedExpression::ForStmt {
                        var_name: var_name.clone(),
                        start: *start,
                        end: *end,
                        step: *step,
                        body: Box::new(typed_body),
                    },
                    ResolvedType::Void, // For loops don't return values
                ))
            }
            Expression::Grouping(inner) => {
                // Resolve the inner expression and wrap it in Grouping
                let (typed_inner, inner_type) = self.resolve_expression(inner)?;
                Ok((
                    TypedExpression::Grouping {
                        inner: Box::new(typed_inner),
                    },
                    inner_type, // Type passes through from inner expression
                ))
            }
            Expression::List(elements) => {
                // If list is empty, we can't infer the type - error for now
                if elements.is_empty() {
                    return Err(anyhow::anyhow!("Empty lists are not yet supported"));
                }

                // Resolve all elements and infer type from first element
                let mut typed_elements = Vec::new();
                let (first_typed, element_type) = self.resolve_expression(&elements[0])?;
                typed_elements.push(first_typed);

                // Check that all elements have the same type
                for elem in &elements[1..] {
                    let (typed_elem, elem_ty) = self.resolve_expression(elem)?;
                    if elem_ty != element_type {
                        return Err(anyhow::anyhow!(
                            "List elements must all have the same type. Expected {:?}, got {:?}",
                            element_type,
                            elem_ty
                        ));
                    }
                    typed_elements.push(typed_elem);
                }

                let list_type = ResolvedType::List(Box::new(element_type.clone()));

                Ok((
                    TypedExpression::List {
                        elements: typed_elements,
                        element_type: element_type.clone(),
                    },
                    list_type,
                ))
            }
            Expression::ListIndex(list, index) => {
                // Resolve list expression - must be a List type
                let (typed_list, list_type) = self.resolve_expression(list)?;

                // Extract element type from list
                let element_type = match list_type {
                    ResolvedType::List(inner) => *inner,
                    _ => {
                        return Err(anyhow::anyhow!(
                            "Cannot index into non-list type {:?}",
                            list_type
                        ))
                    }
                };

                // Resolve index expression - must be i32
                let (typed_index, index_type) = self.resolve_expression(index)?;
                if index_type != ResolvedType::I32 {
                    return Err(anyhow::anyhow!(
                        "List index must be i32, got {:?}",
                        index_type
                    ));
                }

                Ok((
                    TypedExpression::ListIndex {
                        list: Box::new(typed_list),
                        index: Box::new(typed_index),
                    },
                    element_type, // Return element type
                ))
            }
            Expression::ListAssign(name, index, value) => {
                // Look up the list variable
                let list_type = self
                    .get_variable(name)
                    .ok_or_else(|| anyhow::anyhow!("Undefined variable: {}", name))?
                    .clone();

                // Extract element type from list
                let element_type = match list_type {
                    ResolvedType::List(inner) => *inner,
                    _ => {
                        return Err(anyhow::anyhow!(
                            "Cannot index into non-list type {:?}",
                            list_type
                        ))
                    }
                };

                // Resolve index expression - must be i32
                let (typed_index, index_type) = self.resolve_expression(index)?;
                if index_type != ResolvedType::I32 {
                    return Err(anyhow::anyhow!(
                        "List index must be i32, got {:?}",
                        index_type
                    ));
                }

                // Resolve value expression - must match element type
                let (typed_value, value_type) = self.resolve_expression(value)?;
                if value_type != element_type {
                    return Err(anyhow::anyhow!(
                        "Cannot assign value of type {:?} to list of type {:?}",
                        value_type,
                        element_type
                    ));
                }

                Ok((
                    TypedExpression::ListAssign {
                        name: name.clone(),
                        index: Box::new(typed_index),
                        value: Box::new(typed_value),
                    },
                    ResolvedType::Void, // Assignments don't return values
                ))
            }
            Expression::Len(value) => {
                // Resolve the value expression - must be a List type
                let (typed_value, value_type) = self.resolve_expression(value)?;

                // Check that value is a list
                match value_type {
                    ResolvedType::List(_) => {}
                    _ => {
                        return Err(anyhow::anyhow!(
                            "len() requires a list argument, got {:?}",
                            value_type
                        ))
                    }
                };

                Ok((
                    TypedExpression::Len {
                        value: Box::new(typed_value),
                    },
                    ResolvedType::I32, // len() returns i32
                ))
            }
            // ... other cases
            _ => unreachable!("Not implemented for expression {:?}", expr),
        }
    }

    fn resolve_type(&self, ty: &parser::Type) -> ResolvedType {
        use parser::Type;
        match ty {
            Type::None => ResolvedType::Void,
            Type::i32 => ResolvedType::I32,
            Type::i64 => ResolvedType::I64,
            Type::String => ResolvedType::String,
            Type::Bool => ResolvedType::Bool,
            Type::List(inner) => ResolvedType::List(Box::new(self.resolve_type(inner))),
        }
    }
}
