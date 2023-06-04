#![allow(deprecated)]
#![allow(dead_code)]
//TODO: address these lints

pub mod block;
pub mod bool;
pub mod func;
pub mod llvm;
pub mod num;
pub mod string;

//TODO: Upgrade to LLVMGetValueName2
use llvm_sys::core::LLVMGetValueName;
use std::any::Any;

use crate::context::ASTContext;
use dyn_clone::DynClone;

extern crate llvm_sys;
use llvm_sys::prelude::*;

#[derive(Debug)]
pub enum BaseTypes {
    String,
    Number,
    Bool,
    Block,
    Func,
}

pub trait TypeBase: DynClone {
    // TODO: remove on implementation
    #[allow(clippy::all)]
    fn new(_value: Box<dyn Any>, _name: String, _context: &mut ASTContext) -> Box<dyn TypeBase>
    where
        Self: Sized,
    {
        unimplemented!("new has not been implemented for this type");
    }
    fn print(&self, ast_context: &mut ASTContext);
    fn assign(&self, _ast_context: &mut ASTContext, _rhs: Box<dyn TypeBase>) {
        unimplemented!("{:?} type does not implement assign", self.get_type())
    }
    fn get_type(&self) -> BaseTypes;
    fn get_name(&self) -> *const i8 {
        unsafe { LLVMGetValueName(self.get_value()) }
    }
    fn get_llvm_type(&self) -> LLVMTypeRef {
        unimplemented!(
            "{:?} type does not implement get_llvm_type",
            self.get_type()
        )
    }
    fn get_value(&self) -> LLVMValueRef;
    fn set_value(&mut self, _value: LLVMValueRef) {
        unimplemented!("{:?} type does not implement set_value", self.get_type())
    }
    fn get_ptr(&self) -> LLVMValueRef {
        unimplemented!(
            "get_ptr is not implemented for this type {:?}",
            self.get_type()
        )
    }

    // TODO: make this a raw value
    fn get_str(&self) -> String {
        unimplemented!("{:?} type does not implement get_cstr", self.get_type())
    }
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

pub trait Arithmetic: DynClone + TypeBase {
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

pub trait Comparison: DynClone + TypeBase {
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

dyn_clone::clone_trait_object!(TypeBase);
