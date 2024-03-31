use crate::compiler::context::ASTContext;
use crate::compiler::types::{BaseTypes, Func, TypeBase};

extern crate llvm_sys;
use crate::compiler::codegen::cstr_from_string;
use anyhow::Result;

use llvm_sys::core::*;
use llvm_sys::prelude::*;

#[derive(Debug, Clone)]
pub struct StringType {
    pub name: String,
    pub llvm_value: LLVMValueRef,
    pub length: *mut usize,
    pub llvm_value_pointer: Option<LLVMValueRef>,
    pub str_value: String,
}
impl TypeBase for StringType {
    fn assign(&mut self, _ast_context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Result<()> {
        // TODO - add string implementation for assigning variable
        unimplemented!()
    }
    fn get_value(&self) -> LLVMValueRef {
        self.llvm_value
    }
    fn get_ptr(&self) -> Option<LLVMValueRef> {
        match self.llvm_value_pointer {
            Some(v) => Some(v),
            None => {
                unreachable!("No pointer for this value")
            }
        }
    }
    fn get_str(&self) -> String {
        self.str_value.clone()
    }
    fn print(&self, ast_context: &mut ASTContext) -> Result<()> {
        unsafe {
            // Set Value
            // create string vairables and then function
            // This is the Main Print Func
            let llvm_value_to_cstr = LLVMGetAsString(self.llvm_value, self.length);
            // Load Value from Value Index Ptr
            let val = LLVMBuildGlobalStringPtr(
                ast_context.codegen.builder,
                llvm_value_to_cstr,
                llvm_value_to_cstr,
            );

            // let mut print_args = [ast_context.printf_str_value, val].as_mut_ptr();
            let mut print_args: Vec<LLVMValueRef> = vec![ast_context.codegen.printf_str_value, val];
            match ast_context.codegen.llvm_func_cache.get("printf") {
                Some(print_func) => {
                    LLVMBuildCall2(
                        ast_context.codegen.builder,
                        print_func.func_type,
                        print_func.function,
                        print_args.as_mut_ptr(),
                        2,
                        cstr_from_string("").as_ptr(),
                    );
                }
                _ => {
                    unreachable!()
                }
            }
        }
        Ok(())
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::String
    }
}

impl Func for StringType {}
