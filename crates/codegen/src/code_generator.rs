use crate::builder::LLVMCodegenBuilder;
use crate::{
    cstr_from_string, int1_type, int32_type, int64_type, int8_ptr_type,
};
use crate::typed_ast::{ResolvedType, TypedExpression};
use anyhow::{anyhow, Result};
use llvm_sys::core::{
    LLVMAddFunction, LLVMConstStringInContext2, LLVMFunctionType, LLVMGetParam, LLVMVoidType,
};
use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMTypeRef, LLVMValueRef};
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_ulonglong;

pub struct CodeGenerator<'a> {
    builder: &'a mut LLVMCodegenBuilder,
    // Symbol table: variable name -> (LLVMValueRef, pointer, type)
    symbol_table: HashMap<String, GeneratedValue>,
    // Function cache: function name -> (LLVMValueRef, type, return_type)
    function_cache: HashMap<String, FunctionInfo>,
    // Track local variables by depth for scoping
    locals: HashMap<i32, Vec<String>>,
    depth: i32,
    // Track the previous function when entering a new function
    previous_block: Option<LLVMBasicBlockRef>,
}

#[derive(Clone)]
pub struct FunctionInfo {
    pub function: LLVMValueRef,
    pub func_type: LLVMTypeRef,
    pub return_type: ResolvedType,
}

pub struct GeneratedValue {
    pub value: LLVMValueRef,
    pub pointer: Option<LLVMValueRef>,
    pub ty: ResolvedType,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(builder: &'a mut LLVMCodegenBuilder) -> Self {
        Self {
            builder,
            symbol_table: HashMap::new(),
            function_cache: HashMap::new(),
            locals: HashMap::new(),
            depth: 0,
            previous_block: None,
        }
    }

    fn set_variable(&mut self, name: &str, value: GeneratedValue) {
        self.symbol_table.insert(name.to_string(), value);
        self.locals
            .entry(self.depth)
            .or_default()
            .push(name.to_string());
    }

    fn get_variable(&self, name: &str) -> Option<&GeneratedValue> {
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

    pub fn generate_expression(&mut self, typed_expr: &TypedExpression) -> Result<GeneratedValue> {
        match typed_expr {
            TypedExpression::Number32 { value, .. } => self.generate_i32(*value),
            TypedExpression::Number64 { value, .. } => self.generate_i64(*value),
            TypedExpression::String { value } => self.generate_string(value),
            TypedExpression::Bool { value } => self.generate_bool(*value),
            TypedExpression::Binary { left, op, right } => self.generate_binary(left, op, right),
            TypedExpression::CallStmt { callee, args } => self.generate_call(callee, args),
            TypedExpression::FuncStmt {
                name,
                args,
                return_type,
                body,
            } => self.generate_function(name, args, return_type, body),
            TypedExpression::BlockStmt { statements } => self.generate_block(statements),
            TypedExpression::Variable { name } => self.generate_variable(name),
            TypedExpression::Print { value } => self.generate_print(value),
            TypedExpression::ReturnStmt { value } => self.generate_return(value),
            TypedExpression::LetStmt {
                name,
                var_type,
                value,
            } => self.generate_let_stmt(name, var_type, value),
            TypedExpression::IfStmt {
                condition,
                then_branch,
                else_branch,
            } => self.generate_if(
                condition,
                then_branch,
                else_branch.as_ref().map(|b| b.as_ref()),
            ),
            TypedExpression::WhileStmt { condition, body } => self.generate_while(condition, body),
            TypedExpression::AssignStmt { name, value } => self.generate_assign(name, value),
            TypedExpression::Grouping { inner } => {
                // Grouping just generates the inner expression
                self.generate_expression(inner)
            }
            TypedExpression::List {
                elements,
                element_type,
            } => self.generate_list(elements, element_type),
            TypedExpression::ListIndex { list, index } => self.generate_list_index(list, index),
            TypedExpression::ListAssign { name, index, value } => {
                self.generate_list_assign(name, index, value)
            }
            TypedExpression::Len { value } => self.generate_len(value),
        }
    }

    fn generate_i32(&mut self, value: i32) -> Result<GeneratedValue> {
        let llvm_value = self
            .builder
            .const_int(int32_type(), value as c_ulonglong, 0);

        Ok(GeneratedValue {
            value: llvm_value,
            pointer: None,
            ty: ResolvedType::I32,
        })
    }

    fn generate_i64(&mut self, value: i64) -> Result<GeneratedValue> {
        let llvm_value = self
            .builder
            .const_int(int64_type(), value as c_ulonglong, 0);

        Ok(GeneratedValue {
            value: llvm_value,
            pointer: None,
            ty: ResolvedType::I64,
        })
    }

    fn generate_string(&mut self, value: &str) -> Result<GeneratedValue> {
        unsafe {
            // Remove quotes from string
            let val = value.replace('"', "");
            let string = CString::new(val.clone()).unwrap();

            // Create LLVM constant string
            let llvm_string = LLVMConstStringInContext2(
                self.builder.context,
                string.as_ptr(),
                string.as_bytes().len(),
                0,
            );

            // Allocate and store the string
            let string_ptr =
                self.builder
                    .build_alloca_store(llvm_string, int8_ptr_type(), "string_ptr");

            // Call stringInit helper function to create a proper string object
            let string_init_func = self
                .builder
                .llvm_func_cache
                .get("stringInit")
                .ok_or_else(|| anyhow!("stringInit function not found in cache"))?;

            let return_value =
                self.builder
                    .build_call(string_init_func, vec![string_ptr], 1, "string_init_call");

            Ok(GeneratedValue {
                value: return_value,
                pointer: Some(return_value),
                ty: ResolvedType::String,
            })
        }
    }

    fn generate_bool(&mut self, value: bool) -> Result<GeneratedValue> {
        let bool_value = self.builder.const_int(int1_type(), value as c_ulonglong, 0);

        Ok(GeneratedValue {
            value: bool_value,
            pointer: None,
            ty: ResolvedType::Bool,
        })
    }

    fn generate_binary(
        &mut self,
        left: &TypedExpression,
        op: &String,
        right: &TypedExpression,
    ) -> Result<GeneratedValue> {
        let lhs = self.generate_expression(left)?;
        let rhs = self.generate_expression(right)?;
        match op.as_str() {
            "+" | "-" | "/" | "*" => self.builder.arithmetic_v2(&lhs, &rhs, op),
            "^" => Err(anyhow!("^ is not implemented yet")),
            "==" | "!=" | "<" | "<=" | ">" | ">=" => {
                // Use the existing cmp method from builder
                // Convert GeneratedValue back to TypeBase for compatibility
                use crate::types::bool::BoolType;
                use crate::types::num::NumberType;
                use crate::types::string::StringType;
                use crate::types::TypeBase;

                let lhs_base: Box<dyn TypeBase> = match &lhs.ty {
                    ResolvedType::I32 => Box::new(NumberType {
                        name: "lhs".to_string(),
                        llvm_value: lhs.value,
                        llvm_value_pointer: lhs.pointer,
                    }),
                    ResolvedType::Bool => Box::new(BoolType {
                        name: "lhs".to_string(),
                        builder: self.builder.builder,
                        llvm_value: lhs.value,
                        llvm_value_pointer: lhs.pointer,
                    }),
                    ResolvedType::String => Box::new(StringType {
                        name: "lhs".to_string(),
                        llvm_value: lhs.value,
                        llvm_value_pointer: lhs.pointer,
                    }),
                    _ => return Err(anyhow!("Comparison not implemented for type: {:?}", lhs.ty)),
                };

                let rhs_base: Box<dyn TypeBase> = match &rhs.ty {
                    ResolvedType::I32 => Box::new(NumberType {
                        name: "rhs".to_string(),
                        llvm_value: rhs.value,
                        llvm_value_pointer: rhs.pointer,
                    }),
                    ResolvedType::Bool => Box::new(BoolType {
                        name: "rhs".to_string(),
                        builder: self.builder.builder,
                        llvm_value: rhs.value,
                        llvm_value_pointer: rhs.pointer,
                    }),
                    ResolvedType::String => Box::new(StringType {
                        name: "rhs".to_string(),
                        llvm_value: rhs.value,
                        llvm_value_pointer: rhs.pointer,
                    }),
                    _ => return Err(anyhow!("Comparison not implemented for type: {:?}", rhs.ty)),
                };

                let result = self.builder.cmp(lhs_base, rhs_base, op.to_string())?;
                Ok(GeneratedValue {
                    value: result.get_value(),
                    pointer: result.get_ptr(),
                    ty: ResolvedType::Bool,
                })
            }

            _ => Err(anyhow!("Operator: {} not implement", op.clone())),
        }
    }

    fn generate_call(
        &mut self,
        callee: &TypedExpression,
        args: &Vec<TypedExpression>,
    ) -> Result<GeneratedValue> {
        // Get the function name from the callee (should be a Variable)
        let func_name = match callee {
            TypedExpression::Variable { name } => name,
            _ => return Err(anyhow!("Callee must be a variable (function name)")),
        };

        // Look up the function in the cache
        let func_info = self
            .function_cache
            .get(func_name)
            .ok_or_else(|| anyhow!("Undefined function: {}", func_name))?
            .clone();

        // Generate code for each argument
        let mut arg_values = Vec::new();
        for arg in args {
            let arg_val = self.generate_expression(arg)?;
            arg_values.push(arg_val.value);
        }

        // Build the call
        // IMPORTANT: Void functions can't have named results in LLVM IR
        let call_name = if func_info.return_type == ResolvedType::Void {
            ""
        } else {
            &format!("{}_call", func_name)
        };

        let call_result = self.builder.build_call(
            crate::context::LLVMCallFn {
                function: func_info.function,
                func_type: func_info.func_type,
            },
            arg_values,
            args.len() as u32,
            call_name,
        );

        // Return the call result
        Ok(GeneratedValue {
            value: call_result,
            pointer: None,
            ty: func_info.return_type.clone(),
        })
    }

    fn resolved_type_to_llvm(&self, ty: &ResolvedType) -> LLVMTypeRef {
        unsafe {
            use llvm_sys::core::{LLVMGetTypeByName2, LLVMPointerType};

            match ty {
                ResolvedType::I32 => int32_type(),
                ResolvedType::I64 => int64_type(),
                ResolvedType::Bool => int1_type(),
                ResolvedType::Void => LLVMVoidType(),
                ResolvedType::String => {
                    // Strings are represented as struct.StringType* pointers
                    let string_struct_name =
                        CString::new("struct.StringType").expect("CString::new failed");
                    let string_type =
                        LLVMGetTypeByName2(self.builder.context, string_struct_name.as_ptr());
                    LLVMPointerType(string_type, 0)
                }
                ResolvedType::List(_inner) => {
                    // Lists are represented as pointers (i32* for i32 lists, struct.StringType** for string lists)
                    // We'll use int8_ptr_type() as a generic pointer type since lists are opaque pointers
                    int8_ptr_type()
                }
                _ => unimplemented!("Type conversion not implemented for {:?}", ty),
            }
        }
    }

    fn generate_function(
        &mut self,
        name: &str,
        args: &[(String, ResolvedType)],
        return_type: &ResolvedType,
        body: &TypedExpression,
    ) -> Result<GeneratedValue> {
        unsafe {
            // 1. Build parameter types
            let mut param_types: Vec<LLVMTypeRef> = args
                .iter()
                .map(|(_, ty)| self.resolved_type_to_llvm(ty))
                .collect();

            // 2. Create function type
            let ret_type = self.resolved_type_to_llvm(return_type);
            let function_type =
                LLVMFunctionType(ret_type, param_types.as_mut_ptr(), args.len() as u32, 0);

            // 3. Add function to module
            let function = LLVMAddFunction(
                self.builder.module,
                cstr_from_string(name).as_ptr(),
                function_type,
            );

            // 4. Store function in cache
        self.function_cache.insert(
            name.to_owned(),
            FunctionInfo {
                function,
                func_type: function_type,
                return_type: return_type.clone(),
            },
        );

            // 5. Create entry block
            let entry_block = self.builder.append_basic_block(function, "entry");

            // Save current function and block
            let previous_function = self.builder.current_function.clone();
            self.previous_block = Some(previous_function.block);

            // Update current_function to point to the new function being generated
            self.builder.current_function.function = function;
            self.builder.current_function.func_type = function_type;
            self.builder.current_function.block = entry_block;
            self.builder.current_function.entry_block = entry_block;

            // Position builder in the new function
            self.builder.position_builder_at_end(entry_block);

            // 6. Map function arguments to symbol table
            self.incr_depth();
            for (i, (arg_name, arg_type)) in args.iter().enumerate() {
                let param_value = LLVMGetParam(function, i as u32);
                let param_ptr = self.builder.build_alloca_store(
                    param_value,
                    self.resolved_type_to_llvm(arg_type),
                    &format!("{}_ptr", arg_name),
                );

                self.set_variable(
                    arg_name,
                    GeneratedValue {
                        value: param_value,
                        pointer: Some(param_ptr),
                        ty: arg_type.clone(),
                    },
                );
            }

            // 7. Generate function body
            self.generate_expression(body)?;

            // 8. Add implicit return if void
            if *return_type == ResolvedType::Void {
                self.builder.build_ret_void();
            }

            // 9. Clean up local variables and restore previous function/block
            self.decr_depth();
            if let Some(prev_block) = self.previous_block {
                self.builder.current_function = previous_function;
                self.builder.set_current_block(prev_block);
            }

            // Return function as a value
            Ok(GeneratedValue {
                value: function,
                pointer: None,
                ty: ResolvedType::Function(
                    args.iter().map(|(_, ty)| ty.clone()).collect(),
                    Box::new(return_type.clone()),
                ),
            })
        }
    }

    fn generate_block(&mut self, statements: &Vec<TypedExpression>) -> Result<GeneratedValue> {
        self.incr_depth();
        let mut last_value = None;
        for stmt in statements {
            last_value = Some(self.generate_expression(stmt)?);
        }
        self.decr_depth();
        // Return the last statement's value or void
        last_value.ok_or_else(|| anyhow!("Empty block"))
    }

    fn generate_variable(&mut self, name: &String) -> Result<GeneratedValue> {
        let var = self
            .get_variable(name)
            .ok_or_else(|| anyhow!("Undefined variable: {}", name))?;

        // If the variable has a pointer, we need to load the current value from memory
        // This is important for variable reassignment to work correctly
        // Exception: Lists and Strings are already pointers, so we don't load them
        let current_value = match (&var.ty, var.pointer) {
            (ResolvedType::List(_), _) | (ResolvedType::String, _) => {
                // Lists and Strings are already pointer values, don't load
                var.value
            }
            (_, Some(ptr)) => {
                // Load the current value from the pointer
                let llvm_type = self.resolved_type_to_llvm(&var.ty);
                self.builder
                    .build_load(ptr, llvm_type, &format!("{}_load", name))
            }
            (_, None) => var.value,
        };

        Ok(GeneratedValue {
            value: current_value,
            pointer: var.pointer,
            ty: var.ty.clone(),
        })
    }

    fn generate_print(&mut self, value: &TypedExpression) -> Result<GeneratedValue> {
        let generated_value = self.generate_expression(value)?;

        match &generated_value.ty {
            ResolvedType::String => {
                // For strings, use stringPrint helper function
                let string_print_func = self
                    .builder
                    .llvm_func_cache
                    .get("stringPrint")
                    .ok_or_else(|| anyhow!("stringPrint function not found in cache"))?;
                let args = vec![generated_value.value];
                self.builder.build_call(string_print_func, args, 1, "");
            }
            ResolvedType::Bool => {
                // For booleans, convert to string first using bool_to_str, then print
                let bool_to_str_func = self
                    .builder
                    .llvm_func_cache
                    .get("bool_to_str")
                    .ok_or_else(|| anyhow!("bool_to_str function not found in cache"))?;
                let str_value = self.builder.build_call(
                    bool_to_str_func,
                    vec![generated_value.value],
                    1,
                    "bool_to_str_call",
                );

                let printf_func = self
                    .builder
                    .llvm_func_cache
                    .get("printf")
                    .ok_or_else(|| anyhow!("printf function not found in cache"))?;
                self.builder.build_call(printf_func, vec![str_value], 1, "");
            }
            ResolvedType::I32 | ResolvedType::I64 => {
                // For numeric types, use printf with format string
                let base_type = match &generated_value.ty {
                    ResolvedType::I32 => crate::types::BaseTypes::Number,
                    ResolvedType::I64 => crate::types::BaseTypes::Number64,
                    _ => unreachable!(),
                };
                let fmt_str = self.builder.get_printf_str(base_type);
                let printf_func = self
                    .builder
                    .llvm_func_cache
                    .get("printf")
                    .ok_or_else(|| anyhow!("printf function not found in cache"))?;
                let print_args = vec![fmt_str, generated_value.value];
                self.builder.build_call(printf_func, print_args, 2, "");
            }
            ResolvedType::Binary(left_ty, _, _) => {
                // For binary operations, use the left operand type
                match **left_ty {
                    ResolvedType::I32 | ResolvedType::I64 => {
                        let base_type = match **left_ty {
                            ResolvedType::I32 => crate::types::BaseTypes::Number,
                            ResolvedType::I64 => crate::types::BaseTypes::Number64,
                            _ => unreachable!(),
                        };
                        let fmt_str = self.builder.get_printf_str(base_type);
                        let printf_func = self
                            .builder
                            .llvm_func_cache
                            .get("printf")
                            .ok_or_else(|| anyhow!("printf function not found in cache"))?;
                        let print_args = vec![fmt_str, generated_value.value];
                        self.builder.build_call(printf_func, print_args, 2, "");
                    }
                    _ => {
                        return Err(anyhow!(
                            "Print not implemented for binary type: {:?}",
                            generated_value.ty
                        ))
                    }
                }
            }
            ResolvedType::List(element_type) => {
                // For lists, use the appropriate print function
                let print_func_name = match **element_type {
                    ResolvedType::I32 => "printInt32List",
                    ResolvedType::String => "printStringList",
                    _ => {
                        return Err(anyhow!(
                            "Print not implemented for list of type: {:?}",
                            element_type
                        ))
                    }
                };

                let print_func = self
                    .builder
                    .llvm_func_cache
                    .get(print_func_name)
                    .ok_or_else(|| anyhow!("{} function not found in cache", print_func_name))?;
                self.builder
                    .build_call(print_func, vec![generated_value.value], 1, "");
            }
            _ => {
                return Err(anyhow!(
                    "Print not implemented for type: {:?}",
                    generated_value.ty
                ))
            }
        }

        // Print returns void
        Ok(GeneratedValue {
            value: generated_value.value, // Keep the value for potential chaining
            pointer: None,
            ty: ResolvedType::Void,
        })
    }

    fn generate_return(&mut self, value: &TypedExpression) -> Result<GeneratedValue> {
        let return_value = self.generate_expression(value)?;
        self.builder.build_ret(return_value.value);
        Ok(return_value)
    }

    fn generate_let_stmt(
        &mut self,
        name: &str,
        _var_type: &Option<ResolvedType>,
        value: &TypedExpression,
    ) -> Result<GeneratedValue> {
        // Generate the value expression
        let generated_value = self.generate_expression(value)?;

        let ptr = match generated_value.pointer {
            Some(ptr) => ptr,
            None => {
                let llvm_ty = self.resolved_type_to_llvm(&generated_value.ty);
                self.builder
                    .build_alloca_store(generated_value.value, llvm_ty, name)
            }
        };

        // Store in symbol table
        self.set_variable(
            name,
            GeneratedValue {
                value: generated_value.value,
                pointer: Some(ptr),
                ty: generated_value.ty.clone(),
            },
        );

        // Return the generated value
        Ok(generated_value)
    }

    fn generate_if(
        &mut self,
        condition: &TypedExpression,
        then_branch: &TypedExpression,
        else_branch: Option<&TypedExpression>,
    ) -> Result<GeneratedValue> {
        let function = self.builder.current_function.function;
        let if_entry_block = self.builder.current_function.block;

        // Position at entry and generate condition
        self.builder.position_builder_at_end(if_entry_block);
        let cond_value = self.generate_expression(condition)?;

        // Create basic blocks
        let then_block = self.builder.append_basic_block(function, "then_block");
        let merge_block = self.builder.append_basic_block(function, "merge_block");

        // Generate then branch
        self.builder.set_current_block(then_block);
        self.generate_expression(then_branch)?;
        let then_end_block = self.builder.current_function.block;
        if !self.builder.block_has_terminator(then_end_block) {
            self.builder.build_br(merge_block);
        }

        // Create and generate else block
        let else_block = self.builder.append_basic_block(function, "else_block");
        self.builder.set_current_block(else_block);
        if let Some(else_expr) = else_branch {
            self.generate_expression(else_expr)?;
        }
        let else_end_block = self.builder.current_function.block;
        if !self.builder.block_has_terminator(else_end_block) {
            self.builder.build_br(merge_block);
        }

        // Position at merge block
        self.builder.position_builder_at_end(merge_block);
        self.builder.set_current_block(merge_block);

        // Now go back to entry block and add the conditional branch
        // This must be done AFTER all other blocks are set up
        self.builder.set_current_block(if_entry_block);
        let cond_val = if let Some(ptr) = cond_value.pointer {
            self.builder.build_load(ptr, int1_type(), "cond")
        } else {
            cond_value.value
        };
        self.builder.build_cond_br(cond_val, then_block, else_block);

        // Finally position at merge block for subsequent code
        self.builder.set_current_block(merge_block);

        Ok(GeneratedValue {
            value: cond_value.value,
            pointer: None,
            ty: ResolvedType::Void,
        })
    }

    fn generate_while(
        &mut self,
        condition: &TypedExpression,
        body: &TypedExpression,
    ) -> Result<GeneratedValue> {
        let function = self.builder.current_function.function;

            // Create basic blocks
            let loop_cond_block = self.builder.append_basic_block(function, "loop_cond");
            let loop_body_block = self.builder.append_basic_block(function, "loop_body");
            let loop_exit_block = self.builder.append_basic_block(function, "loop_exit");

            // Jump to condition block
            self.builder.build_br(loop_cond_block);

            // Generate loop body
            self.builder.set_current_block(loop_body_block);
            self.generate_expression(body)?;
            self.builder.build_br(loop_cond_block); // Jump back to condition

            // Generate loop condition
            self.builder.set_current_block(loop_cond_block);
            let cond_value = self.generate_expression(condition)?;

            // Load condition value
            let cond_val = if let Some(ptr) = cond_value.pointer {
                self.builder.build_load(ptr, int1_type(), "while_cond")
            } else {
                cond_value.value
            };

            // Branch based on condition
            self.builder
                .build_cond_br(cond_val, loop_body_block, loop_exit_block);

            // Position at exit block
            self.builder.set_current_block(loop_exit_block);

            // While loops return void
        Ok(GeneratedValue {
            value: cond_value.value, // Dummy value
            pointer: None,
            ty: ResolvedType::Void,
        })
    }

    fn generate_assign(&mut self, name: &str, value: &TypedExpression) -> Result<GeneratedValue> {
        // Look up the variable - it must exist from a previous LetStmt
        let var = self
            .get_variable(name)
            .ok_or_else(|| anyhow!("[CodeGen] Undefined variable: {}", name))?;

        // The variable has a pointer from when it was declared
        let var_ptr = var
            .pointer
            .ok_or_else(|| anyhow!("Variable {} has no pointer", name))?;

        // Generate the new value
        let new_value = self.generate_expression(value)?;

        // Store the new value at the existing pointer location
        self.builder.build_store(new_value.value, var_ptr);

        // Update the symbol table with the new value (but same pointer!)
        // IMPORTANT: Use direct insert instead of set_variable to avoid adding to locals again
        self.symbol_table.insert(
            name.to_string(),
            GeneratedValue {
                value: new_value.value,
                pointer: Some(var_ptr),
                ty: new_value.ty.clone(),
            },
        );

        // Assignments return void
        Ok(GeneratedValue {
            value: new_value.value,
            pointer: Some(var_ptr),
            ty: ResolvedType::Void,
        })
    }

    fn generate_list(
        &mut self,
        elements: &[TypedExpression],
        element_type: &ResolvedType,
    ) -> Result<GeneratedValue> {
        // Call the appropriate create list function based on element type
        let (create_func_name, set_func_name) = match element_type {
            ResolvedType::I32 => ("create_int32_tList", "set_int32_tValue"),
            ResolvedType::String => ("createStringList", "setStringValue"),
            _ => {
                return Err(anyhow!(
                    "Lists of type {:?} are not yet supported",
                    element_type
                ))
            }
        };

        // Get the create function from cache
        let create_func = self
            .builder
            .llvm_func_cache
            .get(create_func_name)
            .ok_or_else(|| anyhow!("{} function not found in cache", create_func_name))?;

        // Create list with size
        let size_value = self
            .builder
            .const_int(int32_type(), elements.len() as c_ulonglong, 0);
        let list_ptr = self
            .builder
            .build_call(create_func, vec![size_value], 1, "list_create");

        // Get the set function from cache
        let set_func = self
            .builder
            .llvm_func_cache
            .get(set_func_name)
            .ok_or_else(|| anyhow!("{} function not found in cache", set_func_name))?
            .clone();

        // Populate the list with elements
        for (i, elem) in elements.iter().enumerate() {
            let elem_value = self.generate_expression(elem)?;
            let index_value = self.builder.const_int(int32_type(), i as c_ulonglong, 0);

            // Call set function: set_func(list_ptr, value, index) for i32
            // or set_func(list_ptr, value, index) for string
            self.builder.build_call(
                set_func.clone(),
                vec![list_ptr, elem_value.value, index_value],
                3,
                "",
            );
        }

        Ok(GeneratedValue {
            value: list_ptr,
            pointer: Some(list_ptr),
            ty: ResolvedType::List(Box::new(element_type.clone())),
        })
    }

    fn generate_list_index(
        &mut self,
        list: &TypedExpression,
        index: &TypedExpression,
    ) -> Result<GeneratedValue> {
        // Generate the list expression
        let list_value = self.generate_expression(list)?;

        // Extract element type from list
        let element_type = match &list_value.ty {
            ResolvedType::List(inner) => inner.as_ref().clone(),
            _ => return Err(anyhow!("Cannot index into non-list type")),
        };

        // Generate the index expression
        let index_value = self.generate_expression(index)?;

        // Call the appropriate get function based on element type
        let get_func_name = match element_type {
            ResolvedType::I32 => "get_int32_tValue",
            ResolvedType::String => "getStringValue",
            _ => {
                return Err(anyhow!(
                    "List indexing for type {:?} not yet supported",
                    element_type
                ))
            }
        };

        let get_func = self
            .builder
            .llvm_func_cache
            .get(get_func_name)
            .ok_or_else(|| anyhow!("{} function not found in cache", get_func_name))?;

        // Call get function: get_func(list_ptr, index)
        let result = self.builder.build_call(
            get_func,
            vec![list_value.value, index_value.value],
            2,
            "list_index",
        );

        // get_int32_tValue returns the i32 value directly, not a pointer
        // getStringValue returns a string pointer
        let (final_value, final_pointer) = match element_type {
            ResolvedType::I32 => (result, None), // i32 value is returned directly
            ResolvedType::String => (result, None), // String returns pointer value
            _ => unreachable!(),
        };

        Ok(GeneratedValue {
            value: final_value,
            pointer: final_pointer,
            ty: element_type,
        })
    }

    fn generate_list_assign(
        &mut self,
        name: &String,
        index: &TypedExpression,
        value: &TypedExpression,
    ) -> Result<GeneratedValue> {
        // Look up the list variable and clone what we need
        let (list_ptr, element_type) = {
            let list_var = self
                .get_variable(name)
                .ok_or_else(|| anyhow!("Undefined variable: {}", name))?;

            let elem_type = match &list_var.ty {
                ResolvedType::List(inner) => inner.as_ref().clone(),
                _ => return Err(anyhow!("Cannot index into non-list type")),
            };

            (list_var.value, elem_type)
        };

        // Generate index and value expressions
        let index_value = self.generate_expression(index)?;
        let new_value = self.generate_expression(value)?;

        // Call the appropriate set function based on element type
        let set_func_name = match element_type {
            ResolvedType::I32 => "set_int32_tValue",
            ResolvedType::String => "setStringValue",
            _ => {
                return Err(anyhow!(
                    "List assignment for type {:?} not yet supported",
                    element_type
                ))
            }
        };

        let set_func = self
            .builder
            .llvm_func_cache
            .get(set_func_name)
            .ok_or_else(|| anyhow!("{} function not found in cache", set_func_name))?
            .clone();

        // Call set function: set_func(list_ptr, value, index)
        self.builder.build_call(
            set_func,
            vec![list_ptr, new_value.value, index_value.value],
            3,
            "list_assign",
        );

        Ok(GeneratedValue {
            value: new_value.value,
            pointer: None,
            ty: ResolvedType::Void,
        })
    }

    fn generate_len(&mut self, value: &TypedExpression) -> Result<GeneratedValue> {
        // Generate the list expression
        let list_value = self.generate_expression(value)?;

        // Extract element type from list
        let element_type = match &list_value.ty {
            ResolvedType::List(inner) => inner.as_ref().clone(),
            _ => return Err(anyhow!("len() requires a list argument")),
        };

        // Call the appropriate len function based on element type
        let len_func_name = match element_type {
            ResolvedType::I32 => "lenInt32List",
            ResolvedType::String => "lenStringList",
            _ => {
                return Err(anyhow!(
                    "len() for type {:?} not yet supported",
                    element_type
                ))
            }
        };

        let len_func = self
            .builder
            .llvm_func_cache
            .get(len_func_name)
            .ok_or_else(|| anyhow!("{} function not found in cache", len_func_name))?;

        // Call len function: len_func(list_ptr)
        let result = self
            .builder
            .build_call(len_func, vec![list_value.value], 1, "list_len");

        Ok(GeneratedValue {
            value: result,
            pointer: None,
            ty: ResolvedType::I32, // len() always returns i32
        })
    }
}
