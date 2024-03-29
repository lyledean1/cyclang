use crate::compiler::codegen::context::ASTContext;
use crate::compiler::types::bool::BoolType;
use crate::compiler::types::{Arithmetic, Base, BaseTypes, Comparison, Func, TypeBase};

use cyclang_macros::BaseMacro;
use std::any::Any;
use std::ffi::CString;

extern crate llvm_sys;
use crate::compiler::codegen::cstr_from_string;
use anyhow::Result;

use llvm_sys::core::*;
use llvm_sys::prelude::*;

#[derive(Debug, Clone, BaseMacro)]
#[base_type("BaseTypes::String")]
pub struct StringType {
    name: String,
    llvm_value: LLVMValueRef,
    length: *mut usize,
    llvm_value_pointer: Option<LLVMValueRef>,
    str_value: String,
}

impl Comparison for StringType {
    fn eqeq(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::String => {
                let value = self.get_str() == _rhs.get_str();
                BoolType::new(Box::new(value), self.name.clone(), _context)
            }
            _ => {
                unreachable!(
                    "Can't compare == on dtype {:?} and type {:?}",
                    self.get_type(),
                    _rhs.get_type()
                )
            }
        }
    }

    fn ne(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::String => {
                let value = self.get_str() != _rhs.get_str();
                BoolType::new(Box::new(value), self.name.clone(), _context)
            }
            _ => {
                unreachable!(
                    "Can't compare != on type {:?} and type {:?}",
                    self.get_type(),
                    _rhs.get_type()
                )
            }
        }
    }
}

impl Arithmetic for StringType {
    fn add(&self, _ast_context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::String => match _ast_context.codegen.llvm_func_cache.get("sprintf") {
                Some(_sprintf_func) => unsafe {
                    // TODO: Use sprintf to concatenate two strings
                    // Remove extra quotes
                    let new_string =
                        format!("{}{}", self.get_str(), _rhs.get_str()).replace('\"', "");

                    let string = CString::new(new_string.clone()).unwrap();
                    let value = LLVMConstStringInContext(
                        _ast_context.codegen.context,
                        string.as_ptr(),
                        string.as_bytes().len() as u32,
                        0,
                    );
                    let mut len_value: usize = string.as_bytes().len();
                    let ptr: *mut usize = (&mut len_value) as *mut usize;
                    let buffer_ptr = LLVMBuildPointerCast(
                        _ast_context.codegen.builder,
                        value,
                        LLVMPointerType(LLVMInt8Type(), 0),
                        cstr_from_string("buffer_ptr").as_ptr(),
                    );
                    Box::new(StringType {
                        name: self.name.clone(),
                        length: ptr,
                        llvm_value: value,
                        llvm_value_pointer: Some(buffer_ptr),
                        str_value: new_string,
                    })
                },
                _ => {
                    unreachable!()
                }
            },
            _ => {
                unreachable!(
                    "Can't add type {:?} and type {:?}",
                    self.get_type(),
                    _rhs.get_type()
                )
            }
        }
    }
}

impl TypeBase for StringType {
    fn new(_value: Box<dyn Any>, _name: String, _context: &mut ASTContext) -> Box<dyn TypeBase>
    where
        Self: Sized,
    {
        let value_as_string = match _value.downcast_ref::<String>() {
            Some(val) => val.to_string(),
            None => panic!("The input value must be a string"),
        };
        let string: CString = CString::new(value_as_string.clone()).unwrap();
        unsafe {
            let value = LLVMConstStringInContext(
                _context.codegen.context,
                string.as_ptr(),
                string.as_bytes().len() as u32,
                0,
            );
            let mut len_value: usize = string.as_bytes().len();
            let ptr: *mut usize = (&mut len_value) as *mut usize;
            let buffer_ptr = LLVMBuildPointerCast(
                _context.codegen.builder,
                value,
                LLVMPointerType(LLVMInt8Type(), 0),
                cstr_from_string(_name.as_str()).as_ptr(),
            );
            Box::new(StringType {
                name: _name,
                length: ptr,
                llvm_value: value,
                llvm_value_pointer: Some(buffer_ptr),
                str_value: value_as_string, // fix
            })
        }
    }
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
}

impl Func for StringType {}
