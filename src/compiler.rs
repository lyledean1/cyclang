#![allow(dead_code)]

use std::collections::HashMap;
use std::ffi::CString;
use std::io::Error;
use std::process::Output;
use std::rc::Rc;

use crate::parser::Expression;

extern crate llvm_sys;
use dyn_clone::DynClone;
use llvm_sys::bit_writer::*;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
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
        let bool_value = unsafe { LLVMConstInt(bool_type, boolean as u64, 0) };

        // Return the LLVMValueRef for the bool
        bool_value
    }
}

fn llvm_compile(exprs: Vec<Expression>) -> Result<Output, Error> {
    unsafe {
        // setup
        let context = LLVMContextCreate();
        let module = LLVMModuleCreateWithName(c_str!("main"));
        let builder = LLVMCreateBuilderInContext(context);

        // common void type
        let void_type = LLVMVoidTypeInContext(context);

        // our "main" function which will be the entry point when we run the executable
        let main_func_type = LLVMFunctionType(void_type, ptr::null_mut(), 0, 0);
        let main_func = LLVMAddFunction(module, c_str!("main"), main_func_type);
        let main_block = LLVMAppendBasicBlockInContext(context, main_func, c_str!("main"));
        LLVMPositionBuilderAtEnd(builder, main_block);

        let print_func_type = LLVMFunctionType(void_type, [int8_ptr_type()].as_mut_ptr(), 1, 1);
        let print_func = LLVMAddFunction(module, c_str!("printf"), print_func_type);
        let mut var_cache = VariableCache::new();

        for expr in exprs {
            match_ast(builder, context, print_func, &mut var_cache, expr);
        }
        LLVMBuildRetVoid(builder);
        // write our bitcode file to arm64
        LLVMSetTarget(module, c_str!("arm64"));
        LLVMWriteBitcodeToFile(module, c_str!("/Users/lyledean/gpt/gptql/bin/main.bc"));

        // clean up
        LLVMDisposeBuilder(builder);
        LLVMDisposeModule(module);
        LLVMContextDispose(context);
    }

    // Run clang
    let output = Command::new("clang")
        .arg("/Users/lyledean/gpt/gptql/bin/main.bc")
        .arg("-o")
        .arg("/Users/lyledean/gpt/gptql/bin/main")
        .output();

    match output {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    println!("main executable generated, running bin/main");
    let output = Command::new("bin/main").output();
    return output;
}

fn unbox<T>(value: Box<T>) -> T {
    *value
}

fn match_ast(
    builder: LLVMBuilderRef,
    context: LLVMContextRef,
    print_func: LLVMValueRef,
    var_cache: &mut VariableCache,
    input: Expression,
) -> Box<dyn TypeBase> {
    // LLVMAddFunction(M, Name, FunctionTy)
    match input {
        Expression::Number(input) => unsafe {
            let value = LLVMConstInt(
                LLVMInt32TypeInContext(context),
                input.try_into().unwrap(),
                0,
            );
            return Box::new(NumberType {
                llmv_value: value,
                llmv_value_pointer: None,
            });
        },
        Expression::String(input) => {
            let string = CString::new(input).unwrap();
            unsafe {
                let value = LLVMConstStringInContext(
                    context,
                    string.as_ptr(),
                    string.as_bytes().len() as u32,
                    0,
                );
                let mut len_value: usize = string.as_bytes().len() as usize;
                let ptr: *mut usize = (&mut len_value) as *mut usize;
                return Box::new(StringType {
                    length: ptr,
                    llmv_value: value,
                    llmv_value_pointer: None,
                });
            }
        }
        Expression::Bool(input) => {
            let mut num = 0;
            match input {
                true => num = 1,
                _ => {}
            }
            unsafe {
                let bool_value = LLVMConstInt(int1_type(), num, 0);
                return Box::new(BoolType {
                    value: input,
                    llmv_value: bool_value,
                    llmv_value_pointer: None,
                });
            }
        }
        Expression::Variable(input) => match var_cache.get(&input) {
            Some(val) => {
                // Load Var from Cache and Return
                val          
            }
            None => {
                panic!("var not found")
            }
        },
        Expression::Nil => {
            unimplemented!()
        }
        Expression::Binary(lhs, op, rhs) => match op {
            '+' => {
                let lhs = match_ast(builder, context, print_func, var_cache, unbox(lhs));
                let rhs = match_ast(builder, context, print_func, var_cache, unbox(rhs));
                unsafe {
                    let result =
                        LLVMBuildAdd(builder, lhs.get_value(), rhs.get_value(), c_str!("result"));
                    // let result_str = LLVMBuildIntToPtr(builder, result, int8_ptr_type(), c_str!(""));
                    return Box::new(NumberType {
                        llmv_value: result,
                        llmv_value_pointer: None,
                    });
                }
            }
            '-' => {
                let lhs = match_ast(builder, context, print_func, var_cache, unbox(lhs));
                let rhs = match_ast(builder, context, print_func, var_cache, unbox(rhs));
                unsafe {
                    let result =
                        LLVMBuildSub(builder, lhs.get_value(), rhs.get_value(), c_str!("result"));
                    return Box::new(NumberType {
                        llmv_value: result,
                        llmv_value_pointer: None,
                    });
                }
            }
            '/' => {
                let lhs = match_ast(builder, context, print_func, var_cache, unbox(lhs));
                let rhs = match_ast(builder, context, print_func, var_cache, unbox(rhs));
                unsafe {
                    let result =
                        LLVMBuildFDiv(builder, lhs.get_value(), rhs.get_value(), c_str!("result"));
                    return Box::new(NumberType {
                        llmv_value: result,
                        llmv_value_pointer: None,
                    });
                }
            }
            '*' => {
                let lhs = match_ast(builder, context, print_func, var_cache, unbox(lhs));
                let rhs = match_ast(builder, context, print_func, var_cache, unbox(rhs));
                unsafe {
                    let result =
                        LLVMBuildMul(builder, lhs.get_value(), rhs.get_value(), c_str!("result"));
                    return Box::new(NumberType {
                        llmv_value: result,
                        llmv_value_pointer: None,
                    });
                }
            }
            _ => {
                unimplemented!()
            }
        },
        Expression::Grouping(input) => {
            unimplemented!()
        }
        Expression::LetStmt(var, lhs) => {
            let lhs = match_ast(builder, context, print_func, var_cache, unbox(lhs));
            var_cache.set(&var.clone(), lhs);
            unreachable!()
        }
        Expression::Print(input) => {
            let expression_value = match_ast(builder, context, print_func, var_cache, unbox(input));
            expression_value.print(builder, print_func);
            return expression_value;
        }
        _ => {
            unreachable!("No match for in match_ast")
        }
    }
}

pub fn compile(input: Vec<Expression>) -> Result<Output, Error> {
    llvm_compile(input)
}
// Types
trait TypeBase : DynClone {
    fn print(&self, builder: LLVMBuilderRef, print_func: LLVMValueRef);
    fn get_value(&self) -> LLVMValueRef;
}

dyn_clone::clone_trait_object!(TypeBase);

#[derive(Debug, Clone)]
struct StringType {
    llmv_value: LLVMValueRef,
    length: *mut usize,
    llmv_value_pointer: Option<LLVMValueRef>,
}

impl TypeBase for StringType {
    fn print(&self, builder: LLVMBuilderRef, print_func: LLVMValueRef) {
        unsafe {
            // Set Value
            // create string vairables and then function
            // This is the Main Print Func
            let llvm_value_to_cstr = LLVMGetAsString(self.llmv_value, self.length);

            let value_is_str = LLVMBuildGlobalStringPtr(builder, c_str!("%s\n"), c_str!(""));

            // Load Value from Value Index Ptr
            let val = LLVMBuildGlobalStringPtr(builder, llvm_value_to_cstr, llvm_value_to_cstr);

            let print_args = [value_is_str, val].as_mut_ptr();
            LLVMBuildCall(builder, print_func, print_args, 2, c_str!(""));
        }
    }

    fn get_value(&self) -> LLVMValueRef {
        self.llmv_value
    }
}

#[derive(Debug, Clone)]
struct NumberType {
    llmv_value: LLVMValueRef,
    llmv_value_pointer: Option<LLVMValueRef>,
}

impl TypeBase for NumberType {
    fn print(&self, builder: LLVMBuilderRef, print_func: LLVMValueRef) {
        unsafe {
            let value_index_ptr = LLVMBuildAlloca(builder, int32_type(), c_str!("value"));
            // First thing is to set initial value

            LLVMBuildStore(builder, self.llmv_value, value_index_ptr);

            // Set Value
            // create string vairables and then function
            // This is the Main Print Func

            let value_is_str = LLVMBuildGlobalStringPtr(builder, c_str!("%d\n"), c_str!(""));
            // Load Value from Value Index Ptr
            let val = LLVMBuildLoad(builder, value_index_ptr, c_str!("value"));

            let print_args = [value_is_str, val].as_mut_ptr();
            LLVMBuildCall(builder, print_func, print_args, 2, c_str!(""));
        }
    }

    fn get_value(&self) -> LLVMValueRef {
        self.llmv_value
    }
}

#[derive(Debug, Clone)]
struct BoolType {
    value: bool,
    llmv_value: LLVMValueRef,
    llmv_value_pointer: Option<LLVMValueRef>,
}

impl TypeBase for BoolType {
    fn print(&self, builder: LLVMBuilderRef, print_func: LLVMValueRef) {
        unsafe {
            let mut llvm_value_str =
                LLVMBuildGlobalStringPtr(builder, c_str!("true"), c_str!("true_str"));
            match self.value {
                false => {
                    llvm_value_str =
                        LLVMBuildGlobalStringPtr(builder, c_str!("false"), c_str!("false_str"));
                }
                _ => {}
            }
            let value_is_str = LLVMBuildGlobalStringPtr(builder, c_str!("%s\n"), c_str!(""));
            let print_args = [value_is_str, llvm_value_str].as_mut_ptr();
            LLVMBuildCall(builder, print_func, print_args, 2, c_str!(""));
        }
    }

    fn get_value(&self) -> LLVMValueRef {
        self.llmv_value
    }
}


#[derive(Clone)]
struct Container {
    trait_object: Box<dyn TypeBase>,
}
struct VariableCache {
    map: HashMap<String, Container>,
}

impl VariableCache {
    fn new() -> Self {
        VariableCache {
            map: HashMap::new(),
        }
    }

    fn set(&mut self, key: &str, value: Box<dyn TypeBase>) {
        self.map.insert(key.to_string(), Container { trait_object: value });
    }

    fn get(&self, key: &str) -> Option<Box<dyn TypeBase>> {
        match self.map.get(key) {
            Some(v) => {
                Some(dyn_clone::clone_box(&*v.trait_object))
            }
            None => { None}

        }
    }
}

#[cfg(test)]
mod test {
    use crate::parser::Expression;

    use super::*;
    #[test]
    fn test_compile_print_number_expression() {
        let input = vec![Expression::Print(Box::new(Expression::Number(12)))];
        assert!(compile(input).is_ok());
    }

    #[test]
    fn test_compile_print_string_expression() {
        let input = vec![Expression::Print(Box::new(Expression::String(
            String::from("example blah blah blah"),
        )))];
        assert!(compile(input).is_ok());
    }
}
