use crate::compiler::codegen::cstr_from_string;
use crate::compiler::codegen::*;
use crate::compiler::context::ASTContext;
use crate::compiler::types::{Arithmetic, Base, BaseTypes, Comparison, Func, TypeBase};
use std::any::Any;

extern crate llvm_sys;
use llvm_sys::core::*;
use llvm_sys::prelude::*;

#[derive(Debug, Clone)]
pub struct NumberType {
    //TODO: remove pub use of these
    pub llvm_value: LLVMValueRef,
    pub llvm_value_pointer: Option<LLVMValueRef>,
    pub name: String,
}

impl Base for NumberType
{ fn get_type(& self) -> BaseTypes { BaseTypes :: Number } }

impl TypeBase for NumberType {
    fn new(_value: Box<dyn Any>, name: String, context: &mut ASTContext) -> Box<dyn TypeBase> {
        let value_as_i32 = match _value.downcast_ref::<i32>() {
            Some(val) => *val,
            None => panic!("The input value must be an i32"),
        };
        let value = context
            .codegen
            .const_int(int32_type(), value_as_i32.try_into().unwrap(), 0);
        let ptr = context
            .codegen
            .build_alloca_store(value, int32_ptr_type(), &name);
        Box::new(NumberType {
            name,
            llvm_value: value,
            llvm_value_pointer: Some(ptr),
        })
    }
    fn get_value(&self) -> LLVMValueRef {
        self.llvm_value
    }
    fn get_ptr(&self) -> Option<LLVMValueRef> {
        self.llvm_value_pointer
    }
}

impl Func for NumberType {}

impl Arithmetic for NumberType {
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
                            cstr_from_string("addNumberType").as_ptr(),
                        );
                        let alloca = context.codegen.build_alloca_store(
                            result,
                            self.get_llvm_ptr_type(),
                            "param_add",
                        );
                        let name = self.get_name_as_str().to_string();
                        Box::new(NumberType {
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
                        Box::new(NumberType {
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
                            cstr_from_string("addNumberType").as_ptr(),
                        );
                        let alloca = context.codegen.build_alloca_store(
                            result,
                            self.get_llvm_ptr_type(),
                            "param_add",
                        );
                        let name = self.get_name_as_str().to_string();
                        Box::new(NumberType {
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
                        Box::new(NumberType {
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
                            cstr_from_string("addNumberType").as_ptr(),
                        );
                        let alloca = context.codegen.build_alloca_store(
                            result,
                            self.get_llvm_ptr_type(),
                            "param_add",
                        );
                        let name = self.get_name_as_str().to_string();
                        Box::new(NumberType {
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
                        Box::new(NumberType {
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
