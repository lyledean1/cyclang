#![allow(deprecated)]
#![allow(dead_code)]
//TODO: address these lints

pub mod bool;
pub mod func;
pub mod num;
pub mod return_type;
pub mod string;
pub mod void;

//TODO: Upgrade to LLVMGetValueName2
use llvm_sys::core::LLVMGetValueName;
use std::any::Any;

use crate::{compiler::llvm::context::ASTContext, parser::Expression};
use dyn_clone::DynClone;
extern crate libc;
use libc::c_char;

extern crate llvm_sys;
use crate::compiler::llvm::{
    int1_ptr_type, int1_type, int32_ptr_type, int32_type, int8_ptr_type, int8_type,
};

use llvm_sys::prelude::*;
use crate::cyclo_error::CycloError;

#[derive(Debug)]
pub enum BaseTypes {
    String,
    Number,
    Bool,
    Func,
    Void,
    Return,
}
pub trait Base: DynClone {
    fn get_type(&self) -> BaseTypes;
    fn get_llvm_type(&self) -> LLVMTypeRef {
        match self.get_type() {
            BaseTypes::String => int8_type(),
            BaseTypes::Bool => int1_type(),
            BaseTypes::Number => int32_type(),
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
            _ => {
                unreachable!("LLVMType for Type {:?} not found", self.get_type())
            }
        }
    }
}

type LLVMArithmeticFn = unsafe extern "C" fn(
    arg1: LLVMBuilderRef,
    LHS: LLVMValueRef,
    RHS: LLVMValueRef,
    Name: *const c_char,
) -> LLVMValueRef;

pub trait TypeBase: DynClone + Base + Arithmetic + Comparison + Debug + Func {
    // TODO: remove on implementation
    #[allow(clippy::all)]
    fn new(_value: Box<dyn Any>, _name: String, _context: &mut ASTContext) -> Box<dyn TypeBase>
    where
        Self: Sized,
    {
        unimplemented!("new has not been implemented for this type");
    }
    fn assign(&mut self, _ast_context: &mut ASTContext, _rhs: Box<dyn TypeBase>) {
        unimplemented!("{:?} type does not implement assign", self.get_type())
    }
    unsafe fn get_name(&self) -> *const i8 {
        LLVMGetValueName(self.get_value())
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
}

pub trait Debug: Base {
    fn print(&self, _ast_context: &mut ASTContext) {
        unimplemented!("{:?} type does not implement print", self.get_type())
    }
}

pub trait Arithmetic: Base {
    fn add(&self, _ast_context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement add", self.get_type())
    }
    fn sub(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement sub", self.get_type())
    }
    fn mul(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement mul", self.get_type())
    }
    fn div(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement div", self.get_type())
    }
}

pub trait Comparison: Base {
    fn eqeq(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement eqeq", self.get_type())
    }
    fn ne(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement eqeq", self.get_type())
    }
    fn gt(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement gt", self.get_type())
    }
    fn gte(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement gte", self.get_type())
    }
    fn lt(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement lt", self.get_type())
    }
    fn lte(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement lte", self.get_type())
    }
}

pub trait Func: Base {
    fn call(
        &self,
        _context: &mut ASTContext,
        _call_arguments: Vec<Expression>,
    ) -> Result<Box<dyn TypeBase>, CycloError> {
        unimplemented!("{:?} type does not implement call", self.get_type())
    }
}

dyn_clone::clone_trait_object!(TypeBase);