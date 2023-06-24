#![allow(dead_code)]
use crate::types::block::BlockType;
use crate::types::bool::BoolType;
use crate::types::func::FuncType;
use crate::types::llvm::*;
use crate::types::num::NumberType;
use crate::types::string::StringType;
use crate::types::TypeBase;

use std::collections::HashMap;
use std::ffi::CStr;

use crate::context::*;
use std::io::Error;
use std::process::Output;

use crate::parser::Expression;

extern crate llvm_sys;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMIntPredicate;
use std::process::Command;
use std::ptr;

macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    };
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

        let bool_to_str_func = build_bool_to_str_func(module, context);
        llvm_func_cache.set("bool_to_str", bool_to_str_func);

        //printf
        let print_func_type = LLVMFunctionType(void_type, [int8_ptr_type()].as_mut_ptr(), 1, 1);
        let print_func = LLVMAddFunction(module, c_str!("printf"), print_func_type);
        llvm_func_cache.set(
            "printf",
            LLVMFunction {
                function: print_func,
                func_type: print_func_type,
                block: main_block,
                entry_block: main_block,
                symbol_table: HashMap::new(),
                args: vec![],
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
                entry_block: main_block,
                symbol_table: HashMap::new(),
                args: vec![],
            },
        );

        let var_cache = VariableCache::new();
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
            current_function: LLVMFunction {
                function: main_func,
                func_type: main_func_type,
                block: main_block,
                entry_block: main_block,
                symbol_table: HashMap::new(),
                args: vec![],
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
            Expression::Variable(input) => match self.var_cache.get(&input) {
                Some(val) => val,
                None => {
                    panic!("var {:?} not found", input)
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
            Expression::List(list) => {
                unimplemented!()
            }
            Expression::BlockStmt(exprs) => {
                // Set Variable Depth
                // Each Block Stmt, Incr and Decr
                // Clearing all the "Local" Variables That Have Been Assigned
                self.incr();
                for expr in exprs.clone() {
                    self.match_ast(expr);
                }
                // Delete Variables
                self.var_cache.del_locals(self.get_depth());
                self.decr();
                Box::new(BlockType { values: exprs })
            }
            Expression::CallStmt(name, args) => match self.var_cache.get(&name) {
                Some(val) => {
                    val.call(self, args);
                    val
                }
                _ => {
                    unreachable!("call does not exists for function {:?}", name);
                }
            },
            Expression::FuncStmt(name, args, body) => unsafe {
                let llvm_func =
                    LLVMFunction::new(self, name.clone(), args.clone(), *body.clone(), self.current_function.block);

                let mut func = FuncType {
                    body: *body,
                    args: args.clone(),
                    llvm_type: llvm_func.func_type,
                    llvm_func: llvm_func.function,
                    llvm_func_ref: llvm_func,
                };
                func.set_args(args);
                // Set Func as a variable
                self.var_cache.set(&name, Box::new(func.clone()), self.depth);
                Box::new(func)
            },
            Expression::IfStmt(condition, if_stmt, else_stmt) => unsafe {
                let function = self.current_function.function;
                let if_entry_block: *mut llvm_sys::LLVMBasicBlock = self.current_function.block;

                LLVMPositionBuilderAtEnd(self.builder, if_entry_block);

                let cond: Box<dyn TypeBase> = self.match_ast(*condition);
                // Build If Block
                let then_block = LLVMAppendBasicBlock(function, c_str!("then_block"));
                let merge_block = LLVMAppendBasicBlock(function, c_str!("merge_block"));

                self.set_current_block(then_block);

                self.match_ast(*if_stmt);

                // Each
                LLVMBuildBr(self.builder, merge_block); // Branch to merge_block

                // Build Else Block
                let else_block = LLVMAppendBasicBlock(function, c_str!("else_block"));
                self.set_current_block(else_block);

                match *else_stmt {
                    Some(v_stmt) => {
                        self.match_ast(v_stmt);
                        LLVMBuildBr(self.builder, merge_block); // Branch to merge_block
                    }
                    _ => {
                        LLVMPositionBuilderAtEnd(self.builder, else_block);
                        LLVMBuildBr(self.builder, merge_block); // Branch to merge_block
                    }
                }

                // E
                LLVMPositionBuilderAtEnd(self.builder, merge_block);
                self.set_current_block(merge_block);

                self.set_current_block(if_entry_block);
                LLVMBuildCondBr(self.builder, cond.get_value(), then_block, else_block);

                self.set_current_block(merge_block);
                Box::new(BlockType { values: vec![] })
            },
            Expression::WhileStmt(condition, while_block_stmt) => {
                unsafe {
                    let while_entry_block: *mut llvm_sys::LLVMBasicBlock =
                        self.current_function.block;
                    let function = self.current_function.function;

                    let loop_cond_block = LLVMAppendBasicBlock(function, c_str!("loop_cond"));
                    let loop_body_block = LLVMAppendBasicBlock(function, c_str!("loop_body"));
                    let loop_exit_block = LLVMAppendBasicBlock(function, c_str!("loop_exit"));

                    // Set bool type in entry block
                    let var_name = c_str!("while_value_bool_var");
                    let bool_type_ptr = LLVMBuildAlloca(self.builder, int1_type(), var_name);
                    let value_condition = self.match_ast(*condition);

                    LLVMBuildStore(self.builder, value_condition.get_value(), bool_type_ptr);

                    LLVMBuildBr(self.builder, loop_cond_block);

                    self.set_current_block(loop_body_block);
                    // Check if the global variable already exists

                    self.match_ast(*while_block_stmt);

                    LLVMBuildBr(self.builder, loop_cond_block); // Jump back to loop condition

                    self.set_current_block(loop_cond_block);
                    // Build loop condition block
                    let value_cond_load = LLVMBuildLoad2(
                        self.builder,
                        int1_type(),
                        value_condition.get_ptr(),
                        c_str!("while_value_bool_var"),
                    );

                    LLVMBuildCondBr(
                        self.builder,
                        value_cond_load,
                        loop_body_block,
                        loop_exit_block,
                    );

                    // Position builder at loop exit block
                    self.set_current_block(loop_exit_block);
                    value_condition
                }
            }
            Expression::ForStmt(var_name, init, length, increment, for_block_expr) => {
                unsafe {
                    // Create basic blocks
                    let function = self.current_function.function;
                    let for_block = self.current_function.block;

                    self.set_current_block(for_block);
                    let loop_cond_block =
                        LLVMAppendBasicBlock(self.current_function.function, c_str!("loop_cond"));
                    let loop_body_block =
                        LLVMAppendBasicBlock(self.current_function.function, c_str!("loop_body"));
                    // is this not needed?
                    // let loop_incr_block =
                    //     LLVMAppendBasicBlock(self.current_function.function, c_str!("loop_incr"));
                    let loop_exit_block =
                        LLVMAppendBasicBlock(self.current_function.function, c_str!("loop_exit"));

                    let i: Box<dyn TypeBase> =
                        NumberType::new(Box::new(init), "i".to_string(), self);

                    let value = i.get_value();
                    let ptr = i.get_ptr();
                    self.var_cache.set(&var_name, i, self.depth);

                    LLVMBuildStore(self.builder, value, ptr);

                    // Branch to loop condition block
                    LLVMBuildBr(self.builder, loop_cond_block);

                    // Build loop condition block
                    self.set_current_block(loop_cond_block);

                    // TODO: improve this logic for identifying for and reverse fors
                    let mut op = LLVMIntPredicate::LLVMIntSLT;
                    if increment < 0 {
                        op = LLVMIntPredicate::LLVMIntSGT;
                    }

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
                    self.set_current_block(loop_body_block);
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
                    self.set_current_block(loop_exit_block);

                    for_block_cond
                }
            }
            Expression::Print(input) => {
                let expression_value = self.match_ast(*input);
                expression_value.print(self);
                expression_value
            }
            Expression::ReturnStmt(input) => {
                unimplemented!()
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
        Ok(_ok) => {
            println!("{:?}", _ok);
        }
        Err(e) => return Err(e),
    }

    // // //TODO: add this as a debug line
    // println!("main executable generated, running bin/main");
    Command::new("bin/main").output()
}
