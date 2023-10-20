#![allow(dead_code)]
use crate::compiler::llvm::*;
use crate::compiler::types::bool::BoolType;
use crate::compiler::types::func::FuncType;
use crate::compiler::types::num::NumberType;
use crate::compiler::types::string::StringType;
use crate::compiler::types::void::VoidType;
use crate::compiler::types::TypeBase;
use crate::cyclo_error::CycloError;

use std::collections::HashMap;
use std::ffi::CStr;

use crate::compiler::llvm::context::*;
use crate::compiler::llvm::control_flow::new_if_stmt;
use crate::compiler::llvm::functions::*;
use crate::parser::{Expression, Type};

extern crate llvm_sys;
use crate::c_str;
use llvm_sys::core::*;
use llvm_sys::execution_engine::{
    LLVMCreateExecutionEngineForModule, LLVMDisposeExecutionEngine, LLVMGetFunctionAddress,
    LLVMLinkInMCJIT,
};
use llvm_sys::prelude::*;
use llvm_sys::target::{LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget};
use std::process::Command;
use std::ptr;

use self::llvm::control_flow::{new_for_loop, new_while_stmt};
use self::types::return_type::ReturnType;

pub mod llvm;
pub mod types;

fn llvm_compile_to_ir(exprs: Vec<Expression>, is_execution_engine: bool) -> String {
    unsafe {
        LLVMLinkInMCJIT();
        LLVM_InitializeNativeTarget();
        LLVM_InitializeNativeAsmPrinter();

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

        // Run execution engine
        let mut engine = ptr::null_mut();
        let mut error = ptr::null_mut();

        if LLVMCreateExecutionEngineForModule(&mut engine, module, &mut error) != 0 {
            LLVMDisposeMessage(error);
            panic!("Failed to create execution engine");
        }

        let main_func: extern "C" fn() = std::mem::transmute(LLVMGetFunctionAddress(
            engine,
            b"main\0".as_ptr() as *const _,
        ));

        // Call the main function. It should execute its code.
        if is_execution_engine {
            main_func();
        }
        if !is_execution_engine {
            LLVMPrintModuleToFile(module, c_str!("bin/main.ll"), ptr::null_mut());
        }
        // Use Clang to output LLVM IR -> Binary
        // LLVMWriteBitcodeToFile(module, c_str!("bin/main.bc"));

        let module_cstr = CStr::from_ptr(LLVMPrintModuleToString(module));
        let module_string = module_cstr.to_string_lossy().into_owned();

        // clean up
        LLVMDisposeBuilder(builder);
        if is_execution_engine {
            LLVMDisposeExecutionEngine(engine);
        }
        if !is_execution_engine {
            LLVMDisposeModule(module);
        }
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
                    Some(mut val) => {
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
                let mut val: Box<dyn TypeBase> = Box::new(VoidType {});
                for expr in exprs {
                    val = self.match_ast(expr);
                }
                // Delete Variables
                self.var_cache.del_locals(self.get_depth());
                self.decr();
                val
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

                let func = FuncType {
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
                return new_if_stmt(self, *condition, *if_stmt, *else_stmt);
            }
            Expression::WhileStmt(condition, while_block_stmt) => {
                new_while_stmt(self, *condition, *while_block_stmt)
            }
            Expression::ForStmt(var_name, init, length, increment, for_block_expr) => {
                new_for_loop(self, var_name, init, length, increment, *for_block_expr)
            }
            Expression::Print(input) => {
                let expression_value = self.match_ast(*input);
                expression_value.print(self);
                expression_value
            }
            Expression::ReturnStmt(input) => {
                let expression_value = self.match_ast(*input);
                unsafe {
                    LLVMBuildRet(self.builder, expression_value.get_value());
                }
                Box::new(ReturnType {})
            }
        }
    }
}

pub fn compile(input: Vec<Expression>, is_execution_engine: bool) -> Result<String, CycloError> {
    // output LLVM IR

    llvm_compile_to_ir(input, is_execution_engine);
    // compile to binary
    if !is_execution_engine {
        Command::new("clang")
            .arg("bin/main.ll")
            .arg("-o")
            .arg("bin/main")
            .output()?;

        // // //TODO: add this as a debug line
        // println!("main executable generated, running bin/main");
        let output = Command::new("bin/main").output()?;
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }
    // todo handle no command
    Ok("".to_string())
}
