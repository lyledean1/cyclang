use crate::context::LLVMFunction;
use crate::parser::Expression;
extern crate llvm_sys;
use llvm_sys::core::LLVMBuildCall2;
use crate::types::{Arithmetic, Base, BaseTypes, Comparison, Debug, TypeBase, Func};
use llvm_sys::prelude::*;

use super::string::StringType;

macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    };
}

//TODO: create new functon
#[derive(Debug, Clone)]
pub struct FuncType {
    pub body: Expression,
    pub args: Vec<String>,
    pub llvm_type: LLVMTypeRef,
    pub llvm_func: LLVMValueRef,
    pub llvm_func_ref: LLVMFunction,
}

impl Base for FuncType {
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Func
    }
}

impl Arithmetic for FuncType {}

impl Comparison for FuncType {}

impl Debug for FuncType {}

impl Func for FuncType {
    fn call(&self, _context: &mut crate::context::ASTContext, _args: Vec<Expression>) {
        unsafe { 
        let args = &mut vec![];
        if _args.len() > 0 {
            let value = StringType::new(Box::new("example".to_string()), "hello world".to_string(), _context);
            args.push(value.get_value())
        }
        println!("{:?}", args);
        LLVMBuildCall2(
            _context.builder,
            self.get_llvm_type(),
            self.get_value(),
            args.as_mut_ptr(),
            self.llvm_func_ref.args.len() as u32,
            c_str!(""),
        );
        println!("here again");
    }
    }
}

impl TypeBase for FuncType {
    fn get_value(&self) -> LLVMValueRef {
        self.llvm_func
    }
    fn get_llvm_type(&self) -> LLVMTypeRef {
        self.llvm_type
    }
    fn set_args(&mut self, args: Vec<String>) {
        self.args = args;
    }
    fn get_args(&self) -> Vec<String> {
        self.args.clone()
    }
}
