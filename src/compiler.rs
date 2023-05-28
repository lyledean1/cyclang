#![allow(dead_code)]
use crate::types::{BlockType, BoolType, NumberType, StringType, TypeBase};
use std::ffi::CStr;

use crate::context::*;
use std::io::Error;
use std::process::Output;

use crate::parser::Expression;

extern crate llvm_sys;
use llvm_sys::bit_writer::*;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMIntPredicate;
use std::os::raw::c_ulonglong;
use std::process::Command;
use std::ptr;

macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    };
}

const LLVM_FALSE: LLVMBool = 0;
const LLVM_TRUE: LLVMBool = 1;

// Types

fn create_string_type(context: LLVMContextRef) -> LLVMTypeRef {
    unsafe {
        // Create an LLVM 8-bit integer type (i8) to represent a character
        let i8_type = LLVMInt8TypeInContext(context);

        // Create a pointer type to the i8 type to represent a string
        LLVMPointerType(i8_type, 0)
    }
}
/// Convert this integer to LLVM's representation of a constant
/// integer.
unsafe fn int8(val: c_ulonglong) -> LLVMValueRef {
    LLVMConstInt(LLVMInt8Type(), val, LLVM_FALSE)
}

/// Convert this integer to LLVM's representation of a constant
/// integer.
// TODO: this should be a machine word size rather than hard-coding 32-bits.
fn int32(val: c_ulonglong) -> LLVMValueRef {
    unsafe { LLVMConstInt(LLVMInt32Type(), val, LLVM_FALSE) }
}

fn int1_type() -> LLVMTypeRef {
    unsafe { LLVMInt1Type() }
}

fn int8_type() -> LLVMTypeRef {
    unsafe { LLVMInt8Type() }
}

fn int32_type() -> LLVMTypeRef {
    unsafe { LLVMInt32Type() }
}

fn int8_ptr_type() -> LLVMTypeRef {
    unsafe { LLVMPointerType(LLVMInt8Type(), 0) }
}

fn bool_type(context: LLVMContextRef, boolean: bool) -> LLVMValueRef {
    unsafe {
        let bool_type = LLVMInt1TypeInContext(context);

        // Create a LLVM value for the bool
        // Return the LLVMValueRef for the bool
        LLVMConstInt(bool_type, boolean as u64, 0)
    }
}

fn llvm_compile_to_ir(exprs: Vec<Expression>) -> String {
    unsafe {
        // setup
        let context = LLVMContextCreate();
        let module = LLVMModuleCreateWithName(c_str!("main"));
        let builder = LLVMCreateBuilderInContext(context);

        // common void type
        let void_type = LLVMVoidTypeInContext(context);
        let mut llvm_func_cache = LLVMFunctionCache::new();

        // our "main" function which will be the entry point when we run the executable
        let main_func_type = LLVMFunctionType(void_type, ptr::null_mut(), 0, 0);
        let main_func = LLVMAddFunction(module, c_str!("main"), main_func_type);
        let main_block = LLVMAppendBasicBlockInContext(context, main_func, c_str!("main"));
        LLVMPositionBuilderAtEnd(builder, main_block);

        // Define common functions

        //printf
        let print_func_type = LLVMFunctionType(void_type, [int8_ptr_type()].as_mut_ptr(), 1, 1);
        let print_func = LLVMAddFunction(module, c_str!("printf"), print_func_type);
        llvm_func_cache.set(
            "printf",
            LLVMFunction {
                function: print_func,
                func_type: print_func_type,
                block: main_block,
            },
        );
        //sprintf
        let mut arg_types = [
            LLVMPointerType(LLVMInt8TypeInContext(context), 0),
            LLVMPointerType(LLVMInt8TypeInContext(context), 0),
            LLVMPointerType(LLVMInt8TypeInContext(context), 0),
            LLVMPointerType(LLVMInt8TypeInContext(context), 0),
        ];
        let ret_type = LLVMPointerType(LLVMInt8TypeInContext(context), 0);
        let sprintf_type =
            LLVMFunctionType(ret_type, arg_types.as_mut_ptr(), arg_types.len() as u32, 1);
        let sprintf = LLVMAddFunction(module, "sprintf\0".as_ptr() as *const i8, sprintf_type);
        llvm_func_cache.set(
            "sprintf",
            LLVMFunction {
                function: sprintf,
                func_type: sprintf_type,
                block: main_block,
            },
        );

        let var_cache = VariableCache::new();
        let mut ast_ctx = ASTContext {
            builder,
            module,
            context,
            llvm_func_cache,
            var_cache,
            current_function: LLVMFunction {
                function: main_func,
                func_type: main_func_type,
                block: main_block,
            },
            current_block: main_block,
        };
        for expr in exprs {
            ast_ctx.match_ast(expr);
        }
        LLVMBuildRetVoid(builder);
        // write our bitcode file to arm64
        LLVMSetTarget(module, c_str!("arm64"));
        LLVMPrintModuleToFile(module, c_str!("bin/main.ll"), ptr::null_mut());
        LLVMWriteBitcodeToFile(module, c_str!("bin/main.bc"));
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
    fn match_ast(&mut self, input: Expression) -> Box<dyn TypeBase> {
        match input {
            Expression::Number(input) => {
                return NumberType::new(Box::new(input), self);
            }
            Expression::String(input) => return StringType::new(Box::new(input), self),
            Expression::Bool(input) => BoolType::new(Box::new(input), self),
            Expression::Variable(input) => match self.var_cache.get(&input) {
                Some(val) => val,
                None => {
                    panic!("var not found")
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
                        // combine alloca to reassign variable
                        let mut lhs = self.match_ast(*lhs);
                        self.var_cache.set(&var.clone(), lhs.clone());
                        //TODO: figure out best way to handle a let stmt return
                        unsafe {
                            let alloca = val.get_ptr();
                            lhs.set_ptr(alloca);
                            let build_store = LLVMBuildStore(self.builder, lhs.get_value(), alloca);
                            let new_value = LLVMBuildLoad2(
                                self.builder,
                                int1_type(),
                                alloca,
                                c_str!("bool_type"),
                            );
                            lhs.set_value(new_value);

                            lhs
                        }
                    }
                    _ => {
                        let lhs = self.match_ast(*lhs);
                        self.var_cache.set(&var.clone(), lhs.clone());
                        //TODO: figure out best way to handle a let stmt return
                        lhs
                    }
                }
            }
            Expression::List(list) => {
                unimplemented!()
            }
            Expression::FuncStmt(name, args, body) => {
                // save to call stack
                unimplemented!()
            }
            Expression::CallStmt(name, args) => {
                unimplemented!()
            }
            Expression::BlockStmt(exprs) => {
                for expr in exprs.clone() {
                    self.match_ast(expr);
                }
                Box::new(BlockType {
                    values: exprs,
                })
            }
            Expression::IfStmt(condition, if_stmt, else_stmt) => {
                let cond = self.match_ast(*condition);
                unsafe {
                    // Build If Block
                    let then_block =
                        LLVMAppendBasicBlock(self.current_function.function, c_str!("then_block"));
                    let else_block =
                        LLVMAppendBasicBlock(self.current_function.function, c_str!("else_block"));
                    let merge_block =
                        LLVMAppendBasicBlock(self.current_function.function, c_str!("merge_block"));

                    LLVMBuildCondBr(self.builder, cond.get_value(), then_block, else_block);

                    LLVMPositionBuilderAtEnd(self.builder, then_block);
                    let then_result = self.match_ast(*if_stmt);
                    LLVMBuildBr(self.builder, merge_block); // Branch to merge_block

                    // Build Else Block
                    LLVMPositionBuilderAtEnd(self.builder, else_block);
                    match *else_stmt {
                        Some(v_stmt) => {
                            let else_result = self.match_ast(v_stmt);
                            LLVMBuildBr(self.builder, merge_block); // Branch to merge_block
                        }
                        _ => {
                            LLVMPositionBuilderAtEnd(self.builder, else_block);
                            LLVMBuildBr(self.builder, merge_block); // Branch to merge_block
                        }
                    }
                    LLVMPositionBuilderAtEnd(self.builder, merge_block);
                }
                cond
            }
            Expression::WhileStmt(condition, while_block_stmt) => {
                unsafe {
                    // build new function
                    let function_name = c_str!("while_loop");
                    let main_function = self.current_function.function;
                    let main_func_type = self.current_function.func_type;
                    let while_function_type =
                        LLVMFunctionType(LLVMVoidType(), ptr::null_mut(), 0, 0);
                    let while_function =
                        LLVMAddFunction(self.module, function_name, while_function_type);
                    self.current_function.function = while_function;
                    self.current_function.func_type = while_function_type;

                    let entry_block = LLVMAppendBasicBlock(while_function, c_str!("entry"));
                    let loop_cond_block = LLVMAppendBasicBlock(while_function, c_str!("loop_cond"));
                    let loop_body_block = LLVMAppendBasicBlock(while_function, c_str!("loop_body"));
                    let loop_exit_block = LLVMAppendBasicBlock(while_function, c_str!("loop_exit"));

                    LLVMPositionBuilderAtEnd(self.builder, entry_block);
                    // Set bool type in entry block
                    let var_name = c_str!("bool_type");
                    let bool_type_ptr = LLVMBuildAlloca(self.builder, int1_type(), var_name);
                    let value_condition = self.match_ast(*condition);
                    let build_store =
                        LLVMBuildStore(self.builder, value_condition.get_value(), bool_type_ptr);
                    let bool_type_val =
                        LLVMBuildLoad2(self.builder, int1_type(), bool_type_ptr, var_name);

                    LLVMBuildBr(self.builder, loop_cond_block);

                    LLVMBuildRetVoid(self.builder);

                    LLVMPositionBuilderAtEnd(self.builder, loop_body_block);

                    // Check if the global variable already exists

                    self.match_ast(*while_block_stmt);

                    // Build loop condition block
                    LLVMPositionBuilderAtEnd(self.builder, loop_cond_block);
                    LLVMBuildCondBr(
                        self.builder,
                        bool_type_val,
                        loop_body_block,
                        loop_exit_block,
                    );

                    // Build loop body block
                    LLVMPositionBuilderAtEnd(self.builder, loop_body_block);

                    LLVMBuildBr(self.builder, loop_cond_block); // Jump back to loop condition

                    // Position builder at loop exit block
                    LLVMPositionBuilderAtEnd(self.builder, loop_exit_block);
                    LLVMBuildRetVoid(self.builder);

                    //call main function
                    self.current_function.function = main_function;
                    self.current_function.func_type = main_func_type;
                    LLVMPositionBuilderAtEnd(self.builder, self.current_function.block);
                    LLVMBuildCall2(
                        self.builder,
                        while_function_type,
                        while_function,
                        [].as_mut_ptr(),
                        0,
                        c_str!(""),
                    );
                    value_condition
                }
            }
            Expression::ForStmt(var_name, init, length, increment, for_block_expr) => {
                unsafe {
                    // Create basic blocks
                    let loop_cond_block =
                        LLVMAppendBasicBlock(self.current_function.function, c_str!("loop_cond"));
                    let loop_body_block =
                        LLVMAppendBasicBlock(self.current_function.function, c_str!("loop_body"));
                    let loop_incr_block =
                        LLVMAppendBasicBlock(self.current_function.function, c_str!("loop_incr"));
                    let loop_exit_block =
                        LLVMAppendBasicBlock(self.current_function.function, c_str!("loop_exit"));

                    let i: Box<dyn TypeBase> = NumberType::new(Box::new(init), self);

                    let value = i.clone().get_value();
                    let ptr = i.clone().get_ptr();
                    self.var_cache.set(&var_name, i);

                    LLVMBuildStore(self.builder, value, ptr);

                    // Branch to loop condition block
                    LLVMBuildBr(self.builder, loop_cond_block);

                    // Build loop condition block
                    LLVMPositionBuilderAtEnd(self.builder, loop_cond_block);

                    let op = LLVMIntPredicate::LLVMIntSLT;
                    let op_lhs = ptr;
                    let op_rhs = length;
                    let loop_condition = LLVMBuildICmp(
                        self.builder,
                        op,
                        LLVMBuildLoad2(
                            self.builder,
                            LLVMInt32TypeInContext(self.context),
                            op_lhs,
                            c_str!(""),
                        ),
                        LLVMConstInt(
                            LLVMInt32TypeInContext(self.context),
                            op_rhs.try_into().unwrap(),
                            0,
                        ),
                        c_str!(""),
                    );
                    LLVMBuildCondBr(
                        self.builder,
                        loop_condition,
                        loop_body_block,
                        loop_exit_block,
                    );

                    // Build loop body block
                    LLVMPositionBuilderAtEnd(self.builder, loop_body_block);
                    let for_block_cond = self.match_ast(*for_block_expr);

                    let new_value = LLVMBuildAdd(
                        self.builder,
                        LLVMBuildLoad2(
                            self.builder,
                            LLVMInt32TypeInContext(self.context),
                            ptr,
                            c_str!(""),
                        ),
                        LLVMConstInt(LLVMInt32TypeInContext(self.context), increment as u64, 0),
                        c_str!(""),
                    );
                    LLVMBuildStore(self.builder, new_value, ptr);
                    LLVMBuildBr(self.builder, loop_cond_block); // Jump back to loop condition

                    // Position builder at loop exit block
                    LLVMPositionBuilderAtEnd(self.builder, loop_exit_block);
                    LLVMBuildRetVoid(self.builder);
                    for_block_cond
                }
            }
            Expression::Print(input) => {
                let expression_value = self.match_ast(*input);
                expression_value.print(self);
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
        .arg("bin/main.bc")
        .arg("-o")
        .arg("bin/main")
        .output();

    match output {
        Ok(_ok) => {
            println!("{:?}", _ok);
        }
        Err(e) => return Err(e),
    }

    // // //TODO: add this as a debug line
    // println!("main executable generated, running bin/main");
    Command::new("bin/main").output()
}
