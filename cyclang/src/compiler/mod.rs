#![allow(dead_code)]
use crate::compiler::llvm::*;
use crate::compiler::types::bool::BoolType;
use crate::compiler::types::func::FuncType;
use crate::compiler::types::num::NumberType;
use crate::compiler::types::string::StringType;
use crate::compiler::types::void::VoidType;
use crate::compiler::types::{BaseTypes, TypeBase};
use crate::cyclo_error::CycloError;

use std::collections::HashMap;
use std::ffi::CStr;
use std::io::{Error, ErrorKind};

use crate::compiler::llvm::context::*;
use crate::compiler::llvm::control_flow::new_if_stmt;
use crate::compiler::llvm::functions::*;
use crate::parser::{Expression, Type};

extern crate llvm_sys;
use llvm_sys::core::*;
use llvm_sys::execution_engine::{
    LLVMCreateExecutionEngineForModule, LLVMDisposeExecutionEngine, LLVMGetFunctionAddress,
    LLVMLinkInMCJIT,
};
use llvm_sys::prelude::*;
use llvm_sys::target::{
    LLVMInitializeWebAssemblyAsmPrinter, LLVMInitializeWebAssemblyTarget,
    LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget,
};
use std::process::Command;
use std::ptr;

use self::llvm::control_flow::{new_for_loop, new_while_stmt};
use self::types::return_type::ReturnType;
use crate::compiler::llvm::cstr_from_string;
use crate::compiler::types::list::ListType;
use crate::compiler::types::num64::NumberType64;
use crate::cyclo_error::CycloError::CompileError;

pub mod llvm;
pub mod types;

#[derive(Debug, Clone, Copy)]
pub struct CompileOptions {
    pub is_execution_engine: bool,
    pub target: Option<Target>,
}

#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Target {
    wasm,
    arm32,
    arm64,
    x86_32,
    x86_64,
}

impl Target {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "wasm" => Some(Target::wasm),
            "arm32" => Some(Target::arm32),
            "arm64" => Some(Target::arm64),
            "x86_32" => Some(Target::x86_32),
            "x86_64" => Some(Target::x86_64),
            _ => None,
        }
    }

    pub fn get_llvm_target_name(&self) -> String {
        match self {
            Target::wasm => "wasm32-unknown-unknown-wasm".to_string(),
            Target::arm32 => "arm-unknown-linux-gnueabihf".to_string(),
            Target::arm64 => "aarch64-unknown-linux-gnu".to_string(),
            Target::x86_32 => "i386-unknown-unknown-elf".to_string(),
            Target::x86_64 => "x86_64-unknown-unknown-elf".to_string(),
        }
    }

    pub fn initialize(&self) {
        unsafe {
            match self {
                Target::wasm => {
                    LLVMInitializeWebAssemblyTarget();
                    LLVMInitializeWebAssemblyAsmPrinter();
                }
                Target::arm32 => {
                    unimplemented!("arm32 not implemented yet ")
                }
                Target::arm64 => {
                    unimplemented!("arm64 not implemented yet ")
                }
                Target::x86_32 => {
                    unimplemented!("x86_32 not implemented yet ")
                }
                Target::x86_64 => {
                    unimplemented!("x86_64 not implemented yet ")
                }
            }
        }
    }
}

fn llvm_compile_to_ir(
    exprs: Vec<Expression>,
    compile_options: Option<CompileOptions>,
) -> Result<String, CycloError> {
    unsafe {
        let mut is_execution_engine = false;
        let mut is_default_target: bool = true;

        if let Some(compile_options) = compile_options {
            is_execution_engine = compile_options.is_execution_engine;
            is_default_target = compile_options.target.is_none();
        }

        if is_execution_engine {
            LLVMLinkInMCJIT();
        }

        if is_default_target {
            LLVM_InitializeNativeTarget();
            LLVM_InitializeNativeAsmPrinter();
        }
        if !is_default_target {
            compile_options.unwrap().target.unwrap().initialize();
        }

        let context = LLVMContextCreate();
        let module = LLVMModuleCreateWithName(cstr_from_string("main").as_ptr());
        let builder = LLVMCreateBuilderInContext(context);
        if !is_default_target {
            LLVMSetTarget(
                module,
                cstr_from_string("wasm32-unknown-unknown-wasm").as_ptr(),
            );
        }
        // common void type
        let void_type: *mut llvm_sys::LLVMType = LLVMVoidTypeInContext(context);

        // our "main" function which will be the entry point when we run the executable
        let main_func_type = LLVMFunctionType(void_type, ptr::null_mut(), 0, 0);
        let main_func = LLVMAddFunction(module, cstr_from_string("main").as_ptr(), main_func_type);
        let main_block =
            LLVMAppendBasicBlockInContext(context, main_func, cstr_from_string("main").as_ptr());
        LLVMPositionBuilderAtEnd(builder, main_block);

        // Define common functions

        let llvm_func_cache = build_helper_funcs(module, context, main_block);

        let var_cache = VariableCache::new();
        let func_cache = VariableCache::new();

        let format_str = "%d\n\0";
        let printf_str_num_value = LLVMBuildGlobalStringPtr(
            builder,
            format_str.as_ptr() as *const i8,
            cstr_from_string("number_printf_val").as_ptr(),
        );
        let printf_str_num64_value = LLVMBuildGlobalStringPtr(
            builder,
            cstr_from_string("%llu\n").as_ptr(),
            cstr_from_string("number64_printf_val").as_ptr(),
        );
        let printf_str_value = LLVMBuildGlobalStringPtr(
            builder,
            cstr_from_string("%s\n").as_ptr(),
            cstr_from_string("str_printf_val").as_ptr(),
        );

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
            printf_str_num64_value,
        };
        for expr in exprs {
            ast_ctx.match_ast(expr)?;
        }
        LLVMBuildRetVoid(builder);

        // Run execution engine
        let mut engine = ptr::null_mut();
        let mut error = ptr::null_mut();

        // Call the main function. It should execute its code.
        if is_execution_engine {
            if LLVMCreateExecutionEngineForModule(&mut engine, module, &mut error) != 0 {
                LLVMDisposeMessage(error);
                panic!("Failed to create execution engine");
            }
            let main_func: extern "C" fn() = std::mem::transmute(LLVMGetFunctionAddress(
                engine,
                b"main\0".as_ptr() as *const _,
            ));
            main_func();
        }

        if !is_execution_engine {
            LLVMPrintModuleToFile(
                module,
                cstr_from_string("bin/main.ll").as_ptr(),
                ptr::null_mut(),
            );
        }
        // Use Clang to output LLVM IR -> Binary
        // LLVMWriteBitcodeToFile(module, cstr_from_string("bin/main.bc"));

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
        Ok(module_string)
    }
}

struct ExprContext {
    alloca: Option<LLVMValueRef>,
}

//TODO: remove this and see code warnings
#[allow(unused_variables, unused_assignments)]
impl ASTContext {
    pub fn set_current_block(&mut self, block: LLVMBasicBlockRef) {
        self.position_builder_at_end(block);
        self.current_function.block = block;
    }

    pub fn set_entry_block(&mut self, block: LLVMBasicBlockRef) {
        self.current_function.entry_block = block;
    }

    //TODO: figure a better way to create a named variable in the LLVM IR
    fn try_match_with_var(
        &mut self,
        name: String,
        input: Expression,
    ) -> Result<Box<dyn TypeBase>, CycloError> {
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
            BaseTypes::Number => self.printf_str_num_value,
            BaseTypes::Number64 => self.printf_str_num64_value,
            BaseTypes::Bool => self.printf_str_value,
            BaseTypes::String => self.printf_str_value,
            _ => {
                unreachable!("get_printf_str not implemented for type {:?}", val)
            }
        }
    }

    pub fn match_ast(&mut self, input: Expression) -> Result<Box<dyn TypeBase>, CycloError> {
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
                match self.current_function.symbol_table.get(&input) {
                    Some(val) => Ok(val.clone()),
                    None => {
                        // check if variable is in function
                        // TODO: should this be reversed i.e check func var first then global
                        match self.var_cache.get(&input) {
                            Some(val) => Ok(val),
                            None => {
                                let error_message = format!("Unknown variable {}", input);
                                Err(CompileError(Error::new(
                                    ErrorKind::Unsupported,
                                    error_message,
                                )))
                            }
                        }
                    }
                }
            }
            Expression::List(v) => {
                let mut vec_expr = vec![];
                let previous_type: BaseTypes = BaseTypes::Void;
                for x in v {
                    let expr = self.match_ast(x)?;
                    let current_type = expr.get_type();
                    // if previous_type != BaseTypes::Void && previous_type != current_type {
                    //     unreachable!("add error for mismatching types")
                    // }
                    vec_expr.push(expr)
                }
                let first_element = vec_expr.get(0).unwrap();
                let mut elements = vec![];
                for x in vec_expr.iter() {
                    elements.push(x.get_value());
                }

                let array_type = first_element.get_llvm_type();
                let array_len = vec_expr.len() as u64;
                let llvm_array_value =
                    self.const_array(array_type, elements.as_mut_ptr(), array_len);

                let llvm_array_type = self.array_type(array_type, array_len);
                let array_ptr = self.build_alloca_store(
                    llvm_array_value,
                    llvm_array_type,
                    cstr_from_string("array").as_ptr(),
                );
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
                let zero_index = self.const_int(int64_type(), 0, 0);
                let build_load_index = self.build_load(index.get_ptr().unwrap(), index.get_llvm_type(), cstr_from_string("example").as_ptr());
                let mut indices = [zero_index, build_load_index];
                let val = self.build_gep(
                    val.get_llvm_type(),
                    val.get_ptr().unwrap(),
                    indices.as_mut_ptr(),
                    2_u32,
                    name,
                );
                Ok(Box::new(NumberType {
                    llmv_value: val,
                    llmv_value_pointer: Some(val),
                    name: "".to_string(),
                    cname: name,
                }))
            }
            Expression::ListAssign(var, i, rhs) => match self.var_cache.get(&var) {
                Some(val) => {
                    let name = cstr_from_string("access_array").as_ptr();
                    let lhs: Box<dyn TypeBase> = self.match_ast(*rhs)?;
                    let ptr = val.get_ptr().unwrap();
                    let index = self.match_ast(*i)?;
                    let zero_index = self.const_int(int64_type(), 0, 0);
                    let build_load_index = self.build_load(index.get_ptr().unwrap(), index.get_llvm_type(), cstr_from_string("example").as_ptr());
                    let mut indices = [zero_index, build_load_index];
                    let element_ptr = self.build_gep(
                        val.get_llvm_type(),
                        val.get_ptr().unwrap(),
                        indices.as_mut_ptr(),
                        2_u32,
                        name,
                    );
                    self.build_store(lhs.get_value(), element_ptr);
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
                "^" => Err(CompileError(Error::new(
                    ErrorKind::Unsupported,
                    "^ is not implemented yet".to_string(),
                ))),
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
                    let error_message =
                        format!("Invalid operator found for {:?} {} {:?}", lhs, op, rhs);
                    Err(CompileError(Error::new(
                        ErrorKind::Unsupported,
                        error_message,
                    )))
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
                        val.assign(self, lhs);
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
                    let error_message = format!("call does not exist for function {:?}", name);
                    Err(CompileError(Error::new(
                        ErrorKind::Unsupported,
                        error_message,
                    )))
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
                let error_message = format!("this should be unreachable code, for Expression::FuncArg arg_name:{} arg_type:{:?}", arg_name, arg_type);
                Err(CompileError(Error::new(
                    ErrorKind::Unsupported,
                    error_message,
                )))
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
                expression_value.print(self);
                Ok(expression_value)
            }
            Expression::ReturnStmt(input) => {
                let expression_value = self.match_ast(*input)?;
                self.build_ret(expression_value.get_value());
                Ok(Box::new(ReturnType {}))
            }
        }
    }
}

pub fn compile(
    input: Vec<Expression>,
    compile_options: Option<CompileOptions>,
) -> Result<String, CycloError> {
    // output LLVM IR

    llvm_compile_to_ir(input, compile_options)?;
    // compile to binary
    if let Some(compile_options) = compile_options {
        if !compile_options.is_execution_engine {
            Command::new("clang")
                .arg("bin/main.ll")
                .arg("-o")
                .arg("bin/main")
                .output()?;
            let output = Command::new("bin/main").output()?;
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }
    }
    Ok("".to_string())
}
