use crate::compiler::codegen::cstr_from_string;
use crate::compiler::codegen::*;
use crate::compiler::context::ASTContext;
use crate::compiler::types::{Arithmetic, Base, BaseTypes, Comparison, Func, TypeBase};
use std::any::Any;

extern crate llvm_sys;
use crate::compiler::types::bool::BoolType;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMIntPredicate::{
    LLVMIntEQ, LLVMIntNE, LLVMIntSGE, LLVMIntSGT, LLVMIntSLE, LLVMIntSLT,
};
#[derive(Debug, Clone)]
pub struct NumberType64 {
    //TODO: remove pub use of these
    pub llvm_value: LLVMValueRef,
    pub llvm_value_pointer: Option<LLVMValueRef>,
    pub name: String,
}

impl TypeBase for NumberType64 {
    fn new(_value: Box<dyn Any>, name: String, context: &mut ASTContext) -> Box<dyn TypeBase> {
        let value_as_i64 = match _value.downcast_ref::<i64>() {
            Some(val) => *val,
            None => panic!("The input value must be an i64"),
        };
        let llvm_value =
            context
                .codegen
                .const_int(int64_type(), value_as_i64.try_into().unwrap(), 0);
        let llvm_value_pointer = Some(context.codegen.build_alloca_store(
            llvm_value,
            int64_ptr_type(),
            &name,
        ));
        Box::new(NumberType64 {
            name,
            llvm_value,
            llvm_value_pointer,
        })
    }
    fn get_value(&self) -> LLVMValueRef {
        self.llvm_value
    }
    fn get_ptr(&self) -> Option<LLVMValueRef> {
        self.llvm_value_pointer
    }
}

impl Base for NumberType64
{ fn get_type(& self) -> BaseTypes { BaseTypes :: Number64 } }

impl Func for NumberType64 {}

impl Comparison for NumberType64 {
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
impl Arithmetic for NumberType64 {
    fn add(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number | BaseTypes::Number64 => unsafe {
                match (self.get_ptr(), _rhs.get_ptr()) {
                    (Some(ptr), Some(rhs_ptr)) => {
                        let mut lhs_val =
                            context.codegen.build_load(ptr, self.get_llvm_type(), "rhs");
                        let mut rhs_val =
                            context
                                .codegen
                                .build_load(rhs_ptr, _rhs.get_llvm_type(), "lhs");
                        lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                        rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                        let result = LLVMBuildAdd(
                            context.codegen.builder,
                            lhs_val,
                            rhs_val,
                            cstr_from_string("addNumberType64").as_ptr(),
                        );
                        let alloca = context.codegen.build_alloca_store(
                            result,
                            self.get_llvm_ptr_type(),
                            "param_add",
                        );
                        let name = self.get_name_as_str().to_string();
                        Box::new(NumberType64 {
                            name,
                            llvm_value: result,
                            llvm_value_pointer: Some(alloca),
                        })
                    }
                    _ => {
                        let mut lhs_val = self.get_value();
                        let mut rhs_val = _rhs.get_value();
                        lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                        rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                        let result = LLVMBuildAdd(
                            context.codegen.builder,
                            lhs_val,
                            rhs_val,
                            cstr_from_string("add").as_ptr(),
                        );
                        let alloca = context.codegen.build_alloca_store(
                            result,
                            self.get_llvm_ptr_type(),
                            "param_add",
                        );
                        let name = self.get_name_as_str().to_string();
                        Box::new(NumberType64 {
                            name,
                            llvm_value: result,
                            llvm_value_pointer: Some(alloca),
                        })
                    }
                }
            },
            _ => {
                unreachable!(
                    "Can't {} type {:?} and type {:?}",
                    stringify!("add"),
                    self.get_type(),
                    _rhs.get_type()
                )
            }
        }
    }
    fn sub(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number | BaseTypes::Number64 => unsafe {
                match (self.get_ptr(), _rhs.get_ptr()) {
                    (Some(ptr), Some(rhs_ptr)) => {
                        let mut lhs_val =
                            context.codegen.build_load(ptr, self.get_llvm_type(), "rhs");
                        let mut rhs_val =
                            context
                                .codegen
                                .build_load(rhs_ptr, _rhs.get_llvm_type(), "lhs");
                        lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                        rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                        let result = LLVMBuildSub(
                            context.codegen.builder,
                            lhs_val,
                            rhs_val,
                            cstr_from_string("addNumberType64").as_ptr(),
                        );
                        let alloca = context.codegen.build_alloca_store(
                            result,
                            self.get_llvm_ptr_type(),
                            "param_add",
                        );
                        let name = self.get_name_as_str().to_string();
                        Box::new(NumberType64 {
                            name,
                            llvm_value: result,
                            llvm_value_pointer: Some(alloca),
                        })
                    }
                    _ => {
                        let mut lhs_val = self.get_value();
                        let mut rhs_val = _rhs.get_value();
                        lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                        rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                        let result = LLVMBuildSub(
                            context.codegen.builder,
                            lhs_val,
                            rhs_val,
                            cstr_from_string("sub").as_ptr(),
                        );
                        let alloca = context.codegen.build_alloca_store(
                            result,
                            self.get_llvm_ptr_type(),
                            "param_add",
                        );
                        let name = self.get_name_as_str().to_string();
                        Box::new(NumberType64 {
                            name,
                            llvm_value: result,
                            llvm_value_pointer: Some(alloca),
                        })
                    }
                }
            },
            _ => {
                unreachable!(
                    "Can't {} type {:?} and type {:?}",
                    stringify!("sub"),
                    self.get_type(),
                    _rhs.get_type()
                )
            }
        }
    }
    fn mul(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number | BaseTypes::Number64 => unsafe {
                match (self.get_ptr(), _rhs.get_ptr()) {
                    (Some(ptr), Some(rhs_ptr)) => {
                        let mut lhs_val =
                            context.codegen.build_load(ptr, self.get_llvm_type(), "rhs");
                        let mut rhs_val =
                            context
                                .codegen
                                .build_load(rhs_ptr, _rhs.get_llvm_type(), "lhs");
                        lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                        rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                        let result = LLVMBuildMul(
                            context.codegen.builder,
                            lhs_val,
                            rhs_val,
                            cstr_from_string("addNumberType64").as_ptr(),
                        );
                        let alloca = context.codegen.build_alloca_store(
                            result,
                            self.get_llvm_ptr_type(),
                            "param_add",
                        );
                        let name = self.get_name_as_str().to_string();
                        Box::new(NumberType64 {
                            name,
                            llvm_value: result,
                            llvm_value_pointer: Some(alloca),
                        })
                    }
                    _ => {
                        let mut lhs_val = self.get_value();
                        let mut rhs_val = _rhs.get_value();
                        lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                        rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                        let result = LLVMBuildMul(
                            context.codegen.builder,
                            lhs_val,
                            rhs_val,
                            cstr_from_string("mul").as_ptr(),
                        );
                        let alloca = context.codegen.build_alloca_store(
                            result,
                            self.get_llvm_ptr_type(),
                            "param_add",
                        );
                        let name = self.get_name_as_str().to_string();
                        Box::new(NumberType64 {
                            name,
                            llvm_value: result,
                            llvm_value_pointer: Some(alloca),
                        })
                    }
                }
            },
            _ => {
                unreachable!(
                    "Can't {} type {:?} and type {:?}",
                    stringify!("mul"),
                    self.get_type(),
                    _rhs.get_type()
                )
            }
        }
    }
    fn div(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number | BaseTypes::Number64 => unsafe {
                match (self.get_ptr(), _rhs.get_ptr()) {
                    (Some(ptr), Some(rhs_ptr)) => {
                        let mut lhs_val =
                            context.codegen.build_load(ptr, self.get_llvm_type(), "rhs");
                        let mut rhs_val =
                            context
                                .codegen
                                .build_load(rhs_ptr, _rhs.get_llvm_type(), "lhs");
                        lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                        rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                        let result = LLVMBuildSDiv(
                            context.codegen.builder,
                            lhs_val,
                            rhs_val,
                            cstr_from_string("addNumberType64").as_ptr(),
                        );
                        let alloca = context.codegen.build_alloca_store(
                            result,
                            self.get_llvm_ptr_type(),
                            "param_add",
                        );
                        let name = self.get_name_as_str().to_string();
                        Box::new(NumberType64 {
                            name,
                            llvm_value: result,
                            llvm_value_pointer: Some(alloca),
                        })
                    }
                    _ => {
                        let mut lhs_val = self.get_value();
                        let mut rhs_val = _rhs.get_value();
                        lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                        rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);
                        let result = LLVMBuildSDiv(
                            context.codegen.builder,
                            lhs_val,
                            rhs_val,
                            cstr_from_string("div").as_ptr(),
                        );
                        let alloca = context.codegen.build_alloca_store(
                            result,
                            self.get_llvm_ptr_type(),
                            "param_add",
                        );
                        let name = self.get_name_as_str().to_string();
                        Box::new(NumberType64 {
                            name,
                            llvm_value: result,
                            llvm_value_pointer: Some(alloca),
                        })
                    }
                }
            },
            _ => {
                unreachable!(
                    "Can't {} type {:?} and type {:?}",
                    stringify!("div"),
                    self.get_type(),
                    _rhs.get_type()
                )
            }
        }
    }
}
