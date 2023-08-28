#![allow(dead_code)]
use crate::compiler::llvm::*;
use crate::compiler::types::bool::BoolType;
use crate::compiler::types::func::FuncType;
use crate::compiler::types::num::NumberType;
use crate::compiler::types::string::StringType;
use crate::compiler::types::void::VoidType;
use crate::compiler::types::TypeBase;

use std::collections::HashMap;
use std::ffi::CStr;

use crate::compiler::llvm::context::*;
use crate::compiler::llvm::control_flow::new_if_stmt;
use crate::compiler::llvm::functions::*;

use std::io::Error;
use std::process::Output;

use crate::parser::{Expression, Type};

extern crate llvm_sys;
use crate::c_str;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use std::process::Command;
use std::ptr;

use self::llvm::control_flow::{new_for_loop, new_while_stmt};

pub mod llvm;
pub mod types;

fn llvm_compile_to_ir(exprs: Vec<Expression>) -> String {
    unsafe {
        // setup
        let context = LLVMContextCreate();
        let module = LLVMModuleCreateWithName(c_str!("main"));
        let builder = LLVMCreateBuilderInContext(context);

        // common void type
        let void_type: *mut llvm_sys::LLVMType = LLVMVoidTypeInContext(context);

        // our "main" function which will be the entry point when we run the executable
        let main_func_type = LLVMFunctionType(void_type, ptr::null_mut(), 0, 0);
        let main_func = LLVMAddFunction(module, c_str!("main"), main_func_type);
        let main_block = LLVMAppendBasicBlockInContext(context, main_func, c_str!("main"));
        LLVMPositionBuilderAtEnd(builder, main_block);

        // Define common functions

        let llvm_func_cache = build_helper_funcs(module, context, main_block);

        let var_cache = VariableCache::new();
        let func_cache = VariableCache::new();

        let format_str = "%d\n\0";
        let printf_str_num_value = LLVMBuildGlobalStringPtr(
            builder,
            format_str.as_ptr() as *const i8,
            c_str!("number_printf_val"),
        );
        let printf_str_value =
            LLVMBuildGlobalStringPtr(builder, c_str!("%s\n"), c_str!("str_printf_val"));

        let mut ast_ctx = ASTContext {
            builder,
            module,
            context,
            llvm_func_cache,
            var_cache,
            func_cache,
            current_function: LLVMFunction {
                function: main_func,
                func_type: main_func_type,
                block: main_block,
                entry_block: main_block,
                symbol_table: HashMap::new(),
                args: vec![],
                return_type: Type::None,
            },
            depth: 0,
            printf_str_value,
            printf_str_num_value,
        };
        for expr in exprs {
            ast_ctx.match_ast(expr);
        }
        LLVMBuildRetVoid(builder);
        // write our bitcode file to arm64
        LLVMSetTarget(module, c_str!("arm64"));
        LLVMPrintModuleToFile(module, c_str!("bin/main.ll"), ptr::null_mut());
        // Use Clang to output LLVM IR -> Binary
        // LLVMWriteBitcodeToFile(module, c_str!("bin/main.bc"));
        let module_cstr = CStr::from_ptr(LLVMPrintModuleToString(module));
        let module_string = module_cstr.to_string_lossy().into_owned();

        // clean up
        LLVMDisposeBuilder(builder);
        LLVMDisposeModule(module);
        LLVMContextDispose(context);
        module_string
    }
}

struct ExprContext {
    alloca: Option<LLVMValueRef>,
}

//TODO: remove this and see code warnings
#[allow(unused_variables, unused_assignments)]
impl ASTContext {
    pub unsafe fn set_current_block(&mut self, block: LLVMBasicBlockRef) {
        LLVMPositionBuilderAtEnd(self.builder, block);
        self.current_function.block = block;
    }

    pub fn set_entry_block(&mut self, block: LLVMBasicBlockRef) {
        self.current_function.entry_block = block;
    }

    //TODO: figure a better way to create a named variable in the LLVM IR
    fn try_match_with_var(&mut self, name: String, input: Expression) -> Box<dyn TypeBase> {
        match input {
            Expression::Number(input) => {
                return NumberType::new(
                    Box::new(input),
                    var_type_str(name, "num_var".to_string()),
                    self,
                )
            }
            Expression::String(input) => {
                return StringType::new(
                    Box::new(input),
                    var_type_str(name, "str_var".to_string()),
                    self,
                )
            }
            Expression::Bool(input) => {
                return BoolType::new(
                    Box::new(input),
                    var_type_str(name, "bool_var".to_string()),
                    self,
                )
            }
            _ => {
                // just return without var
                return self.match_ast(input);
            }
        }
    }

    pub fn match_ast(&mut self, input: Expression) -> Box<dyn TypeBase> {
        match input {
            Expression::Number(input) => {
                return NumberType::new(Box::new(input), "num".to_string(), self);
            }
            Expression::String(input) => {
                return StringType::new(Box::new(input), "str".to_string(), self)
            }
            Expression::Bool(input) => BoolType::new(Box::new(input), "bool".to_string(), self),
            Expression::Variable(input) => match self.current_function.symbol_table.get(&input) {
                Some(val) => val.clone(),
                None => {
                    // check if variable is in function
                    // TODO: should this be reversed i.e check func var first then global
                    match self.var_cache.get(&input) {
                        Some(val) => val,
                        None => {
                            unreachable!()
                        }
                    }
                }
            },
            Expression::Nil => {
                unimplemented!()
            }
            Expression::Binary(lhs, op, rhs) => match op.as_str() {
                "+" => {
                    let lhs = self.match_ast(*lhs);
                    let rhs = self.match_ast(*rhs);
                    lhs.add(self, rhs)
                }
                "-" => {
                    let lhs = self.match_ast(*lhs);
                    let rhs = self.match_ast(*rhs);
                    lhs.sub(self, rhs)
                }
                "/" => {
                    let lhs = self.match_ast(*lhs);
                    let rhs = self.match_ast(*rhs);
                    lhs.div(self, rhs)
                }
                "*" => {
                    let lhs = self.match_ast(*lhs);
                    let rhs = self.match_ast(*rhs);
                    lhs.mul(self, rhs)
                }
                "^" => {
                    unimplemented!()
                }
                "==" => {
                    let lhs = self.match_ast(*lhs);
                    let rhs = self.match_ast(*rhs);
                    lhs.eqeq(self, rhs)
                }
                "!=" => {
                    let lhs = self.match_ast(*lhs);
                    let rhs = self.match_ast(*rhs);
                    lhs.ne(self, rhs)
                }
                "<" => {
                    let lhs = self.match_ast(*lhs);
                    let rhs = self.match_ast(*rhs);
                    lhs.lt(self, rhs)
                }
                "<=" => {
                    let lhs = self.match_ast(*lhs);
                    let rhs = self.match_ast(*rhs);
                    lhs.lte(self, rhs)
                }
                ">" => {
                    let lhs = self.match_ast(*lhs);
                    let rhs = self.match_ast(*rhs);
                    lhs.gt(self, rhs)
                }
                ">=" => {
                    let lhs = self.match_ast(*lhs);
                    let rhs = self.match_ast(*rhs);
                    lhs.gte(self, rhs)
                }
                _ => {
                    unimplemented!()
                }
            },
            Expression::Grouping(_input) => self.match_ast(*_input),
            Expression::LetStmt(var, lhs) => {
                match self.var_cache.get(&var) {
                    Some(val) => {
                        // Check Variables are the same Type
                        // Then Update the value of the old variable
                        // reassign variable

                        // Assign a temp variable to the stack
                        let lhs: Box<dyn TypeBase> = self.match_ast(*lhs);

                        // Assign this new value
                        val.assign(self, lhs);
                        val
                    }
                    _ => {
                        let lhs = self.try_match_with_var(var.clone(), *lhs);
                        self.var_cache.set(&var.clone(), lhs.clone(), self.depth);
                        //TODO: figure out best way to handle a let stmt return
                        lhs
                    }
                }
            }
            Expression::BlockStmt(exprs) => {
                // Set Variable Depth
                // Each Block Stmt, Incr and Decr
                // Clearing all the "Local" Variables That Have Been Assigned
                self.incr();
                for expr in exprs {
                    self.match_ast(expr);
                }
                // Delete Variables
                self.var_cache.del_locals(self.get_depth());
                self.decr();
                Box::new(VoidType {})
            }
            Expression::CallStmt(name, args) => match self.func_cache.get(&name) {
                Some(val) => {
                    let call_val = val.call(self, args);
                    self.var_cache
                        .set(name.as_str(), call_val.clone(), self.depth);
                    call_val
                }
                _ => {
                    unreachable!("call does not exists for function {:?}", name);
                }
            },
            Expression::FuncStmt(name, args, _return_type, body) => unsafe {
                let llvm_func = LLVMFunction::new(
                    self,
                    name.clone(),
                    args.clone(),
                    _return_type.clone(),
                    *body.clone(),
                    self.current_function.block,
                );

                let mut func = FuncType {
                    llvm_type: llvm_func.func_type,
                    llvm_func: llvm_func.function,
                    return_type: _return_type,
                };
                // Set Func as a variable
                self.func_cache
                    .set(&name, Box::new(func.clone()), self.depth);
                Box::new(func)
            },
            Expression::FuncArg(arg_name, arg_type) => {
                unimplemented!()
            }
            Expression::IfStmt(condition, if_stmt, else_stmt) => {
                new_if_stmt(self, condition, if_stmt, else_stmt);
                Box::new(VoidType {})
            }
            Expression::WhileStmt(condition, while_block_stmt) => {
                new_while_stmt(self, condition, while_block_stmt)
            }
            Expression::ForStmt(var_name, init, length, increment, for_block_expr) => {
                new_for_loop(self, var_name, init, length, increment, for_block_expr)
            }
            Expression::Print(input) => {
                let expression_value = self.match_ast(*input);
                expression_value.print(self);
                expression_value
            }
            Expression::ReturnStmt(input) => {
                let expression_value = self.match_ast(*input);
                unsafe {
                    LLVMBuildRet(self.builder, expression_value.get_ptr());
                }
                expression_value
            }
        }
    }
}

pub fn compile(input: Vec<Expression>) -> Result<Output, Error> {
    // output LLVM IR
    llvm_compile_to_ir(input);
    // compile to binary

    let output = Command::new("clang")
        .arg("bin/main.ll")
        .arg("-o")
        .arg("bin/main")
        .output();

    match output {
        Ok(_ok) => {}
        Err(e) => return Err(e),
    }

    // // //TODO: add this as a debug line
    // println!("main executable generated, running bin/main");
    Command::new("bin/main").output()
}
