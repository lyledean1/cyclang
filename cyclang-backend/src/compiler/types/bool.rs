use crate::compiler::codegen::*;
use crate::compiler::context::ASTContext;
use std::any::Any;

extern crate llvm_sys;
use super::Arithmetic;
use crate::compiler::types::{Base, BaseTypes, Comparison, Func, TypeBase};
use anyhow::anyhow;
use anyhow::Result;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMIntPredicate::{
    LLVMIntEQ, LLVMIntNE, LLVMIntSGE, LLVMIntSGT, LLVMIntSLE, LLVMIntSLT,
};
#[derive(Clone)]
pub struct BoolType {
    pub builder: LLVMBuilderRef,
    pub llvm_value: LLVMValueRef,
    pub llvm_value_pointer: LLVMValueRef,
    pub name: String,
}

impl Base for BoolType
{ fn get_type(& self) -> BaseTypes { BaseTypes :: Bool } }

impl Arithmetic for BoolType {}

fn get_value_for_print_argument(
    context: &mut ASTContext,
    name: &str,
    value: BoolType,
) -> Vec<LLVMValueRef> {
    match value.get_ptr() {
        Some(v) => vec![context.codegen.build_load(v, int1_type(), name)],
        None => vec![value.get_value()],
    }
}

impl TypeBase for BoolType {
    fn new(_value: Box<dyn Any>, _name: String, context: &mut ASTContext) -> Box<dyn TypeBase>
    where
        Self: Sized,
    {
        let value_as_bool = match _value.downcast_ref::<bool>() {
            Some(val) => *val,
            None => panic!("The input value must be a bool"),
        };
        let bool_value = context
            .codegen
            .const_int(int1_type(), value_as_bool.into(), 0);
        let alloca = context
            .codegen
            .build_alloca_store(bool_value, int1_type(), &_name);
        Box::new(BoolType {
            name: _name,
            builder: context.codegen.builder,
            llvm_value: bool_value,
            llvm_value_pointer: alloca,
        })
    }
    fn get_value(&self) -> LLVMValueRef {
        self.llvm_value
    }
    fn get_ptr(&self) -> Option<LLVMValueRef> {
        Some(self.llvm_value_pointer)
    }
    fn print(&self, astcontext: &mut ASTContext) -> Result<()> {
        let bool_func_args = get_value_for_print_argument(astcontext, "", self.clone());

        let bool_to_string_func = astcontext
            .codegen
            .llvm_func_cache
            .get("bool_to_str")
            .ok_or(anyhow!("unable to find bool_to_str function"))?;
        let str_value = astcontext
            .codegen
            .build_call(bool_to_string_func, bool_func_args, 1, "");
        let print_args: Vec<LLVMValueRef> = vec![str_value];
        let print_func = astcontext
            .codegen
            .llvm_func_cache
            .get("printf")
            .ok_or(anyhow!("unable to find printf function"))?;
        astcontext.codegen.build_call(print_func, print_args, 1, "");
        Ok(())
    }
}

impl Func for BoolType {}

impl Comparison for BoolType {
    fn eqeq(&self, context: &mut ASTContext, rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match rhs.get_type() {
            BaseTypes::Number | BaseTypes::Bool => {}
            _ => {
                unreachable!(
                    "Can't do operation type {:?} and type {:?}",
                    self.get_type(),
                    rhs.get_type()
                )
            }
        }
        unsafe {
            match (self.get_ptr(), self.get_type()) {
                (Some(lhs_ptr), BaseTypes::Number) => {
                    let mut lhs_val = context.codegen.build_load(
                        lhs_ptr,
                        self.get_llvm_type(),
                        self.get_name_as_str(),
                    );
                    let mut rhs_val = context.codegen.build_load(
                        rhs.get_ptr().unwrap(),
                        rhs.get_llvm_type(),
                        rhs.get_name_as_str(),
                    );
                    lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                    rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                    let cmp = LLVMBuildICmp(
                        context.codegen.builder,
                        LLVMIntEQ,
                        lhs_val,
                        rhs_val,
                        cstr_from_string("result").as_ptr(),
                    );
                    let alloca = context
                        .codegen
                        .build_alloca_store(cmp, int1_type(), "bool_cmp");
                    Box::new(BoolType {
                        name: self.get_name_as_str().to_string(),
                        builder: context.codegen.builder,
                        llvm_value: cmp,
                        llvm_value_pointer: alloca,
                    })
                }
                _ => {
                    let mut lhs_val = self.get_value();
                    let mut rhs_val = rhs.get_value();
                    lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                    rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                    let cmp = LLVMBuildICmp(
                        context.codegen.builder,
                        LLVMIntEQ,
                        lhs_val,
                        rhs_val,
                        cstr_from_string("result").as_ptr(),
                    );
                    let alloca = context
                        .codegen
                        .build_alloca_store(cmp, int1_type(), "bool_cmp");
                    Box::new(BoolType {
                        name: self.get_name_as_str().to_string(),
                        builder: context.codegen.builder,
                        llvm_value: cmp,
                        llvm_value_pointer: alloca,
                    })
                }
            }
        }
    }
    fn ne(&self, context: &mut ASTContext, rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match rhs.get_type() {
            BaseTypes::Number | BaseTypes::Bool => {}
            _ => {
                unreachable!(
                    "Can't do operation type {:?} and type {:?}",
                    self.get_type(),
                    rhs.get_type()
                )
            }
        }
        unsafe {
            match (self.get_ptr(), self.get_type()) {
                (Some(lhs_ptr), BaseTypes::Number) => {
                    let mut lhs_val = context.codegen.build_load(
                        lhs_ptr,
                        self.get_llvm_type(),
                        self.get_name_as_str(),
                    );
                    let mut rhs_val = context.codegen.build_load(
                        rhs.get_ptr().unwrap(),
                        rhs.get_llvm_type(),
                        rhs.get_name_as_str(),
                    );
                    lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                    rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                    let cmp = LLVMBuildICmp(
                        context.codegen.builder,
                        LLVMIntNE,
                        lhs_val,
                        rhs_val,
                        cstr_from_string("result").as_ptr(),
                    );
                    let alloca = context
                        .codegen
                        .build_alloca_store(cmp, int1_type(), "bool_cmp");
                    Box::new(BoolType {
                        name: self.get_name_as_str().to_string(),
                        builder: context.codegen.builder,
                        llvm_value: cmp,
                        llvm_value_pointer: alloca,
                    })
                }
                _ => {
                    let mut lhs_val = self.get_value();
                    let mut rhs_val = rhs.get_value();
                    lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                    rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                    let cmp = LLVMBuildICmp(
                        context.codegen.builder,
                        LLVMIntNE,
                        lhs_val,
                        rhs_val,
                        cstr_from_string("result").as_ptr(),
                    );
                    let alloca = context
                        .codegen
                        .build_alloca_store(cmp, int1_type(), "bool_cmp");
                    Box::new(BoolType {
                        name: self.get_name_as_str().to_string(),
                        builder: context.codegen.builder,
                        llvm_value: cmp,
                        llvm_value_pointer: alloca,
                    })
                }
            }
        }
    }
    fn gt(&self, context: &mut ASTContext, rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match rhs.get_type() {
            BaseTypes::Number | BaseTypes::Bool => {}
            _ => {
                unreachable!(
                    "Can't do operation type {:?} and type {:?}",
                    self.get_type(),
                    rhs.get_type()
                )
            }
        }
        unsafe {
            match (self.get_ptr(), self.get_type()) {
                (Some(lhs_ptr), BaseTypes::Number) => {
                    let mut lhs_val = context.codegen.build_load(
                        lhs_ptr,
                        self.get_llvm_type(),
                        self.get_name_as_str(),
                    );
                    let mut rhs_val = context.codegen.build_load(
                        rhs.get_ptr().unwrap(),
                        rhs.get_llvm_type(),
                        rhs.get_name_as_str(),
                    );
                    lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                    rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                    let cmp = LLVMBuildICmp(
                        context.codegen.builder,
                        LLVMIntSGT,
                        lhs_val,
                        rhs_val,
                        cstr_from_string("result").as_ptr(),
                    );
                    let alloca = context
                        .codegen
                        .build_alloca_store(cmp, int1_type(), "bool_cmp");
                    Box::new(BoolType {
                        name: self.get_name_as_str().to_string(),
                        builder: context.codegen.builder,
                        llvm_value: cmp,
                        llvm_value_pointer: alloca,
                    })
                }
                _ => {
                    let mut lhs_val = self.get_value();
                    let mut rhs_val = rhs.get_value();
                    lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                    rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                    let cmp = LLVMBuildICmp(
                        context.codegen.builder,
                        LLVMIntSGT,
                        lhs_val,
                        rhs_val,
                        cstr_from_string("result").as_ptr(),
                    );
                    let alloca = context
                        .codegen
                        .build_alloca_store(cmp, int1_type(), "bool_cmp");
                    Box::new(BoolType {
                        name: self.get_name_as_str().to_string(),
                        builder: context.codegen.builder,
                        llvm_value: cmp,
                        llvm_value_pointer: alloca,
                    })
                }
            }
        }
    }
    fn gte(&self, context: &mut ASTContext, rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match rhs.get_type() {
            BaseTypes::Number | BaseTypes::Bool => {}
            _ => {
                unreachable!(
                    "Can't do operation type {:?} and type {:?}",
                    self.get_type(),
                    rhs.get_type()
                )
            }
        }
        unsafe {
            match (self.get_ptr(), self.get_type()) {
                (Some(lhs_ptr), BaseTypes::Number) => {
                    let mut lhs_val = context.codegen.build_load(
                        lhs_ptr,
                        self.get_llvm_type(),
                        self.get_name_as_str(),
                    );
                    let mut rhs_val = context.codegen.build_load(
                        rhs.get_ptr().unwrap(),
                        rhs.get_llvm_type(),
                        rhs.get_name_as_str(),
                    );
                    lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                    rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                    let cmp = LLVMBuildICmp(
                        context.codegen.builder,
                        LLVMIntSGE,
                        lhs_val,
                        rhs_val,
                        cstr_from_string("result").as_ptr(),
                    );
                    let alloca = context
                        .codegen
                        .build_alloca_store(cmp, int1_type(), "bool_cmp");
                    Box::new(BoolType {
                        name: self.get_name_as_str().to_string(),
                        builder: context.codegen.builder,
                        llvm_value: cmp,
                        llvm_value_pointer: alloca,
                    })
                }
                _ => {
                    let mut lhs_val = self.get_value();
                    let mut rhs_val = rhs.get_value();
                    lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                    rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                    let cmp = LLVMBuildICmp(
                        context.codegen.builder,
                        LLVMIntSGE,
                        lhs_val,
                        rhs_val,
                        cstr_from_string("result").as_ptr(),
                    );
                    let alloca = context
                        .codegen
                        .build_alloca_store(cmp, int1_type(), "bool_cmp");
                    Box::new(BoolType {
                        name: self.get_name_as_str().to_string(),
                        builder: context.codegen.builder,
                        llvm_value: cmp,
                        llvm_value_pointer: alloca,
                    })
                }
            }
        }
    }
    fn lt(&self, context: &mut ASTContext, rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match rhs.get_type() {
            BaseTypes::Number | BaseTypes::Bool => {}
            _ => {
                unreachable!(
                    "Can't do operation type {:?} and type {:?}",
                    self.get_type(),
                    rhs.get_type()
                )
            }
        }
        unsafe {
            match (self.get_ptr(), self.get_type()) {
                (Some(lhs_ptr), BaseTypes::Number) => {
                    let mut lhs_val = context.codegen.build_load(
                        lhs_ptr,
                        self.get_llvm_type(),
                        self.get_name_as_str(),
                    );
                    let mut rhs_val = context.codegen.build_load(
                        rhs.get_ptr().unwrap(),
                        rhs.get_llvm_type(),
                        rhs.get_name_as_str(),
                    );
                    lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                    rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                    let cmp = LLVMBuildICmp(
                        context.codegen.builder,
                        LLVMIntSLT,
                        lhs_val,
                        rhs_val,
                        cstr_from_string("result").as_ptr(),
                    );
                    let alloca = context
                        .codegen
                        .build_alloca_store(cmp, int1_type(), "bool_cmp");
                    Box::new(BoolType {
                        name: self.get_name_as_str().to_string(),
                        builder: context.codegen.builder,
                        llvm_value: cmp,
                        llvm_value_pointer: alloca,
                    })
                }
                _ => {
                    let mut lhs_val = self.get_value();
                    let mut rhs_val = rhs.get_value();
                    lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                    rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                    let cmp = LLVMBuildICmp(
                        context.codegen.builder,
                        LLVMIntSLT,
                        lhs_val,
                        rhs_val,
                        cstr_from_string("result").as_ptr(),
                    );
                    let alloca = context
                        .codegen
                        .build_alloca_store(cmp, int1_type(), "bool_cmp");
                    Box::new(BoolType {
                        name: self.get_name_as_str().to_string(),
                        builder: context.codegen.builder,
                        llvm_value: cmp,
                        llvm_value_pointer: alloca,
                    })
                }
            }
        }
    }
    fn lte(&self, context: &mut ASTContext, rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match rhs.get_type() {
            BaseTypes::Number | BaseTypes::Bool => {}
            _ => {
                unreachable!(
                    "Can't do operation type {:?} and type {:?}",
                    self.get_type(),
                    rhs.get_type()
                )
            }
        }
        unsafe {
            match (self.get_ptr(), self.get_type()) {
                (Some(lhs_ptr), BaseTypes::Number) => {
                    let mut lhs_val = context.codegen.build_load(
                        lhs_ptr,
                        self.get_llvm_type(),
                        self.get_name_as_str(),
                    );
                    let mut rhs_val = context.codegen.build_load(
                        rhs.get_ptr().unwrap(),
                        rhs.get_llvm_type(),
                        rhs.get_name_as_str(),
                    );
                    lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                    rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                    let cmp = LLVMBuildICmp(
                        context.codegen.builder,
                        LLVMIntSLE,
                        lhs_val,
                        rhs_val,
                        cstr_from_string("result").as_ptr(),
                    );
                    let alloca = context
                        .codegen
                        .build_alloca_store(cmp, int1_type(), "bool_cmp");
                    Box::new(BoolType {
                        name: self.get_name_as_str().to_string(),
                        builder: context.codegen.builder,
                        llvm_value: cmp,
                        llvm_value_pointer: alloca,
                    })
                }
                _ => {
                    let mut lhs_val = self.get_value();
                    let mut rhs_val = rhs.get_value();
                    lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                    rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                    let cmp = LLVMBuildICmp(
                        context.codegen.builder,
                        LLVMIntSLE,
                        lhs_val,
                        rhs_val,
                        cstr_from_string("result").as_ptr(),
                    );
                    let alloca = context
                        .codegen
                        .build_alloca_store(cmp, int1_type(), "bool_cmp");
                    Box::new(BoolType {
                        name: self.get_name_as_str().to_string(),
                        builder: context.codegen.builder,
                        llvm_value: cmp,
                        llvm_value_pointer: alloca,
                    })
                }
            }
        }
    }
}
