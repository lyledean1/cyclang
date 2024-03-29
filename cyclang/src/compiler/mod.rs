#![allow(dead_code)]
use crate::compiler::codegen::*;
use crate::compiler::types::bool::BoolType;
use crate::compiler::types::func::FuncType;
use crate::compiler::types::num::NumberType;
use crate::compiler::types::string::StringType;
use crate::compiler::types::void::VoidType;
use crate::compiler::types::{BaseTypes, TypeBase};
use anyhow::anyhow;
use anyhow::Result;

use crate::compiler::codegen::context::*;
use crate::compiler::codegen::control_flow::new_if_stmt;
use crate::compiler::codegen::target::Target;
use crate::parser::Expression;

extern crate llvm_sys;
use llvm_sys::prelude::*;
use std::process::Command;

use self::codegen::control_flow::{new_for_loop, new_while_stmt};
use self::types::return_type::ReturnType;
use crate::compiler::codegen::builder::LLVMCodegenBuilder;
use crate::compiler::codegen::cstr_from_string;
use crate::compiler::types::list::ListType;
use crate::compiler::types::num64::NumberType64;

pub mod codegen;
pub mod types;

#[derive(Debug, Clone, Copy)]
pub struct CompileOptions {
    pub is_execution_engine: bool,
    pub target: Option<Target>,
}
struct ExprContext {
    alloca: Option<LLVMValueRef>,
}

impl ASTContext {
    pub fn init(compile_options: Option<CompileOptions>) -> Result<ASTContext> {
        let var_cache = VariableCache::new();
        let func_cache = VariableCache::new();
        let codegen = LLVMCodegenBuilder::init(compile_options)?;
        Ok(ASTContext {
            var_cache,
            func_cache,
            depth: 0,
            codegen,
        })
    }
    pub fn set_current_block(&mut self, block: LLVMBasicBlockRef) {
        self.codegen.position_builder_at_end(block);
        self.codegen.current_function.block = block;
    }

    pub fn set_entry_block(&mut self, block: LLVMBasicBlockRef) {
        self.codegen.current_function.entry_block = block;
    }

    //TODO: figure a better way to create a named variable in the LLVM IR
    fn try_match_with_var(&mut self, name: String, input: Expression) -> Result<Box<dyn TypeBase>> {
        match input {
            Expression::Number(input) => Ok(NumberType::new(Box::new(input), name, self)),
            Expression::String(input) => Ok(StringType::new(
                Box::new(input),
                var_type_str(name, "str_var".to_string()),
                self,
            )),
            Expression::Bool(input) => Ok(BoolType::new(
                Box::new(input),
                var_type_str(name, "bool_var".to_string()),
                self,
            )),
            _ => {
                // just return without var
                self.match_ast(input)
            }
        }
    }

    fn get_printf_str(&mut self, val: BaseTypes) -> LLVMValueRef {
        match val {
            BaseTypes::Number => self.codegen.printf_str_num_value,
            BaseTypes::Number64 => self.codegen.printf_str_num64_value,
            BaseTypes::Bool => self.codegen.printf_str_value,
            BaseTypes::String => self.codegen.printf_str_value,
            _ => {
                unreachable!("get_printf_str not implemented for type {:?}", val)
            }
        }
    }

    pub fn match_ast(&mut self, input: Expression) -> Result<Box<dyn TypeBase>> {
        match input {
            Expression::Number(input) => {
                Ok(NumberType::new(Box::new(input), "num".to_string(), self))
            }
            Expression::Number64(input) => Ok(NumberType64::new(
                Box::new(input),
                "num64".to_string(),
                self,
            )),
            Expression::String(input) => {
                Ok(StringType::new(Box::new(input), "str".to_string(), self))
            }
            Expression::Bool(input) => Ok(BoolType::new(Box::new(input), "bool".to_string(), self)),
            Expression::Variable(input) => {
                match self.codegen.current_function.symbol_table.get(&input) {
                    Some(val) => Ok(val.clone()),
                    None => {
                        // check if variable is in function
                        // TODO: should this be reversed i.e check func var first then global
                        match self.var_cache.get(&input) {
                            Some(val) => Ok(val),
                            None => {
                                Err(anyhow!(format!("Unknown variable {}", input)))
                            }
                        }
                    }
                }
            }
            Expression::List(v) => {
                let mut vec_expr = vec![];
                for x in v {
                    let expr = self.match_ast(x)?;
                    vec_expr.push(expr)
                }
                let first_element = vec_expr.first().unwrap();
                let mut elements = vec![];
                for x in vec_expr.iter() {
                    elements.push(x.get_value());
                }

                let array_type = first_element.get_llvm_type();
                let array_len = vec_expr.len() as u64;
                let llvm_array_value =
                    self.codegen.const_array(array_type, elements.as_mut_ptr(), array_len);

                let llvm_array_type = self.codegen.array_type(array_type, array_len);
                let array_ptr = self.codegen.build_alloca_store(llvm_array_value, llvm_array_type, "array");
                Ok(Box::new(ListType {
                    llvm_value: llvm_array_value,
                    llvm_value_ptr: array_ptr,
                    llvm_type: llvm_array_type,
                }))
            }
            Expression::ListIndex(v, i) => {
                let name = cstr_from_string("access_array").as_ptr();
                let val = self.match_ast(*v)?;
                let index = self.match_ast(*i)?;
                let zero_index = self.codegen.const_int(int64_type(), 0, 0);
                let build_load_index =
                    self.codegen.build_load(index.get_ptr().unwrap(), index.get_llvm_type(), "example");
                let mut indices = [zero_index, build_load_index];
                let val = self.codegen.build_gep(
                    val.get_llvm_type(),
                    val.get_ptr().unwrap(),
                    indices.as_mut_ptr(),
                    2_u32,
                    name,
                );
                Ok(Box::new(NumberType {
                    llvm_value: val,
                    llvm_value_pointer: Some(val),
                    name: "".to_string(),
                }))
            }
            Expression::ListAssign(var, i, rhs) => match self.var_cache.get(&var) {
                Some(val) => {
                    let name = cstr_from_string("access_array").as_ptr();
                    let lhs: Box<dyn TypeBase> = self.match_ast(*rhs)?;
                    let index = self.match_ast(*i)?;
                    let zero_index = self.codegen.const_int(int64_type(), 0, 0);
                    let build_load_index =
                        self.codegen.build_load(index.get_ptr().unwrap(), index.get_llvm_type(), "example");
                    let mut indices = [zero_index, build_load_index];
                    let element_ptr = self.codegen.build_gep(
                        val.get_llvm_type(),
                        val.get_ptr().unwrap(),
                        indices.as_mut_ptr(),
                        2_u32,
                        name,
                    );
                    self.codegen.build_store(lhs.get_value(), element_ptr);
                    Ok(val)
                }
                _ => {
                    unreachable!("can't assign as var doesn't exist")
                }
            },
            Expression::Nil => {
                unimplemented!()
            }
            Expression::Binary(lhs, op, rhs) => match op.as_str() {
                "+" => {
                    let lhs = self.match_ast(*lhs)?;
                    let rhs = self.match_ast(*rhs)?;
                    Ok(lhs.add(self, rhs))
                }
                "-" => {
                    let lhs = self.match_ast(*lhs)?;
                    let rhs = self.match_ast(*rhs)?;
                    Ok(lhs.sub(self, rhs))
                }
                "/" => {
                    let lhs = self.match_ast(*lhs)?;
                    let rhs = self.match_ast(*rhs)?;
                    Ok(lhs.div(self, rhs))
                }
                "*" => {
                    let lhs = self.match_ast(*lhs)?;
                    let rhs = self.match_ast(*rhs)?;
                    Ok(lhs.mul(self, rhs))
                }
                "^" => Err(anyhow!("^ is not implemented yet")),
                "==" => {
                    let lhs = self.match_ast(*lhs)?;
                    let rhs = self.match_ast(*rhs)?;
                    Ok(lhs.eqeq(self, rhs))
                }
                "!=" => {
                    let lhs = self.match_ast(*lhs)?;
                    let rhs = self.match_ast(*rhs)?;
                    Ok(lhs.ne(self, rhs))
                }
                "<" => {
                    let lhs = self.match_ast(*lhs)?;
                    let rhs = self.match_ast(*rhs)?;
                    Ok(lhs.lt(self, rhs))
                }
                "<=" => {
                    let lhs = self.match_ast(*lhs)?;
                    let rhs = self.match_ast(*rhs)?;
                    Ok(lhs.lte(self, rhs))
                }
                ">" => {
                    let lhs = self.match_ast(*lhs)?;
                    let rhs = self.match_ast(*rhs)?;
                    Ok(lhs.gt(self, rhs))
                }
                ">=" => {
                    let lhs = self.match_ast(*lhs)?;
                    let rhs = self.match_ast(*rhs)?;
                    Ok(lhs.gte(self, rhs))
                }
                _ => {
                    Err(anyhow!("Invalid operator found for {:?} {} {:?}", lhs, op, rhs))
                }
            },
            Expression::Grouping(_input) => self.match_ast(*_input),
            Expression::LetStmt(var, _, lhs) => {
                match self.var_cache.get(&var) {
                    Some(mut val) => {
                        // Check Variables are the same Type
                        // Then Update the value of the old variable
                        // reassign variable

                        // Assign a temp variable to the stack
                        let lhs: Box<dyn TypeBase> = self.match_ast(*lhs)?;
                        // Assign this new value
                        val.assign(self, lhs)?;
                        Ok(val)
                    }
                    _ => {
                        let lhs = self.try_match_with_var(var.clone(), *lhs)?;
                        self.var_cache.set(&var.clone(), lhs.clone(), self.depth);
                        Ok(lhs)
                    }
                }
            }
            Expression::BlockStmt(exprs) => {
                // Set Variable Depth
                // Each Block Stmt, Incr and Decr
                // Clearing all the "Local" Variables That Have Been Assigned
                self.incr();
                let mut val: Box<dyn TypeBase> = Box::new(VoidType {});
                for expr in exprs {
                    val = self.match_ast(expr)?;
                }
                // Delete Variables
                self.var_cache.del_locals(self.get_depth());
                self.decr();
                Ok(val)
            }
            Expression::CallStmt(name, args) => match self.func_cache.get(&name) {
                Some(val) => {
                    let call_val = val.call(self, args)?;
                    self.var_cache
                        .set(name.as_str(), call_val.clone(), self.depth);
                    Ok(call_val)
                }
                _ => {
                    Err(anyhow!("call does not exist for function {:?}", name))
                }
            },
            Expression::FuncStmt(name, args, _return_type, body) => unsafe {
                let llvm_func = LLVMFunction::new(
                    self,
                    name.clone(),
                    args.clone(),
                    _return_type.clone(),
                    *body.clone(),
                    self.codegen.current_function.block,
                )?;

                let func = FuncType {
                    llvm_type: llvm_func.func_type,
                    llvm_func: llvm_func.function,
                    return_type: _return_type,
                };
                // Set Func as a variable
                self.func_cache
                    .set(&name, Box::new(func.clone()), self.depth);
                Ok(Box::new(func))
            },
            Expression::FuncArg(arg_name, arg_type) => {
                Err(anyhow!("this should be unreachable code, for Expression::FuncArg arg_name:{} arg_type:{:?}", arg_name, arg_type))
            }
            Expression::IfStmt(condition, if_stmt, else_stmt) => {
                new_if_stmt(self, *condition, *if_stmt, *else_stmt)
            }
            Expression::WhileStmt(condition, while_block_stmt) => {
                new_while_stmt(self, *condition, *while_block_stmt)
            }
            Expression::ForStmt(var_name, init, length, increment, for_block_expr) => {
                new_for_loop(self, var_name, init, length, increment, *for_block_expr)
            }
            Expression::Print(input) => {
                let expression_value = self.match_ast(*input)?;
                expression_value.print(self)?;
                Ok(expression_value)
            }
            Expression::ReturnStmt(input) => {
                let expression_value = self.match_ast(*input)?;
                self.codegen.build_ret(expression_value.get_value());
                Ok(Box::new(ReturnType {}))
            }
        }
    }

    pub fn dispose_and_get_module_str(&self) -> Result<String> {
        //TODO: should this live here?
        self.codegen.dispose_and_get_module_str()
    }
}

pub fn compile(exprs: Vec<Expression>, compile_options: Option<CompileOptions>) -> Result<String> {
    // output LLVM IR
    let mut ast_ctx = ASTContext::init(compile_options)?;
    for expr in exprs {
        ast_ctx.match_ast(expr)?;
    }
    ast_ctx.dispose_and_get_module_str()
}
