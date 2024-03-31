#![allow(deprecated)]
#![allow(dead_code)]
//TODO: address these lints

pub mod bool;
pub mod func;
pub mod list;
pub mod num;
pub mod num64;
pub mod return_type;
pub mod string;
pub mod void;

use llvm_sys::core::LLVMGetValueName;
use std::any::Any;
use std::ffi::CStr;

use dyn_clone::DynClone;
extern crate libc;
use libc::c_char;

extern crate llvm_sys;
use crate::compiler::codegen::{
    int1_ptr_type, int1_type, int32_ptr_type, int32_type, int64_ptr_type, int64_type,
    int8_ptr_type, int8_type,
};
use crate::compiler::context::ASTContext;

use anyhow::anyhow;
use anyhow::Result;
use cyclang_parser::Expression;
use llvm_sys::prelude::*;
use crate::compiler::codegen::builder::LLVMCodegenBuilder;
use crate::compiler::visitor::Visitor;

#[derive(Debug, PartialEq)]
pub enum BaseTypes {
    String,
    Number,
    Number64,
    Bool,
    List(Box<BaseTypes>),
    Func,
    Void,
    Return,
}

pub trait TypeBase: DynClone + Func {
    // TODO: remove on implementation
    #[allow(clippy::all)]
    fn new(_value: Box<dyn Any>, _name: String, _context: &mut ASTContext) -> Box<dyn TypeBase>
    where
        Self: Sized,
    {
        unimplemented!("new has not been implemented for this type");
    }
    fn assign(&mut self, codegen: &mut LLVMCodegenBuilder, _rhs: Box<dyn TypeBase>) -> Result<()> {
        if _rhs.get_type() != self.get_type() {
            return Err(anyhow!(
                "Can't reassign variable {:?} that has type {:?} to type {:?}",
                self.get_name_as_str(),
                self.get_type(),
                _rhs.get_type()
            ));
        }
        codegen.build_load_store(
            _rhs.get_ptr().unwrap(),
            self.get_ptr().unwrap(),
            self.get_llvm_type(),
            self.get_name_as_str(),
        );
        Ok(())
    }
    fn get_name(&self) -> *const c_char {
        unsafe { LLVMGetValueName(self.get_value()) }
    }

    fn get_name_as_str(&self) -> &str {
        unsafe {
            let c_str_ref = CStr::from_ptr(self.get_name());
            c_str_ref.to_str().unwrap()
        }
    }
    fn get_value(&self) -> LLVMValueRef;
    fn get_ptr(&self) -> Option<LLVMValueRef> {
        unimplemented!(
            "get_ptr is not implemented for this type {:?}",
            self.get_type()
        )
    }
    // TODO: make this a raw value
    fn get_str(&self) -> String {
        unimplemented!("{:?} type does not implement get_cstr", self.get_type())
    }

    fn get_value_for_printf(&self, codegen: &mut LLVMCodegenBuilder) -> LLVMValueRef {
        match self.get_ptr() {
            Some(v) => codegen.build_load(v, self.get_llvm_type(), "debug"),
            None => self.get_value(),
        }
    }

    fn print(&self, context: &mut ASTContext, codegen: &mut LLVMCodegenBuilder) -> Result<()> {
        let print_args: Vec<LLVMValueRef> = vec![
            codegen.get_printf_str(self.get_type()),
            self.get_value_for_printf(codegen),
        ];
        let print_func = codegen
            .llvm_func_cache
            .get("printf")
            .ok_or(anyhow!("unable to call print function"))?;
        codegen.build_call(print_func, print_args, 2, "");
        Ok(())
    }

    fn get_type(&self) -> BaseTypes;
    fn get_llvm_type(&self) -> LLVMTypeRef {
        match self.get_type() {
            BaseTypes::String => int8_type(),
            BaseTypes::Bool => int1_type(),
            BaseTypes::Number => int32_type(),
            BaseTypes::Number64 => int64_type(),
            _ => {
                unreachable!("LLVMType for Type {:?} not found", self.get_type())
            }
        }
    }
    fn get_llvm_ptr_type(&self) -> LLVMTypeRef {
        match self.get_type() {
            BaseTypes::String => int8_ptr_type(),
            BaseTypes::Bool => int1_ptr_type(),
            BaseTypes::Number => int32_ptr_type(),
            BaseTypes::Number64 => int64_type(),
            _ => {
                unreachable!("LLVMType for Type {:?} not found", self.get_type())
            }
        }
    }
}

pub trait Func {
    fn call(&self, context: &mut ASTContext, args: Vec<Expression>, visitor: &mut Box<dyn Visitor<Box<dyn TypeBase>>>, codegen: &mut LLVMCodegenBuilder) -> Result<Box<dyn TypeBase>> {
        unimplemented!("type does not implement call")
    }
}

dyn_clone::clone_trait_object!(TypeBase);
