use crate::c_str;
use crate::compiler::llvm::*;
use std::any::Any;

use crate::compiler::llvm::context::ASTContext;
use crate::compiler::types::bool::BoolType;
use crate::compiler::types::{Arithmetic, Base, BaseTypes, Comparison, Debug, Func, TypeBase};

extern crate llvm_sys;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMIntPredicate;

#[derive(Debug, Clone)]
pub struct NumberType {
    //TODO: remove pub use of these
    pub llmv_value: LLVMValueRef,
    pub llmv_value_pointer: Option<LLVMValueRef>,
    pub name: String,
    pub cname: *const i8,
}

impl Base for NumberType {
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Number
    }
}

impl Arithmetic for NumberType {
    fn add(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                match self.llmv_value_pointer {
                    Some(_p) => {
                        let lhs_value = LLVMBuildLoad2(
                            context.builder,
                            int32_type(),
                            self.get_ptr().unwrap(),
                            self.get_name(),
                        );
                        let rhs_value: *mut llvm_sys::LLVMValue = LLVMBuildLoad2(
                            context.builder,
                            int32_type(),
                            _rhs.get_ptr().unwrap(),
                            _rhs.get_name(),
                        );
                        let result =
                            LLVMBuildAdd(context.builder, lhs_value, rhs_value, c_str!("add_num"));
                        LLVMBuildStore(context.builder, result, self.get_ptr().unwrap());
                        //TODO: fix the new instruction
                        Box::new(NumberType {
                            name: self.name.clone(),
                            llmv_value: result,
                            llmv_value_pointer: self.llmv_value_pointer,
                            cname: self.get_name(),
                        })
                    }
                    None => {
                        let result = LLVMBuildAdd(
                            context.builder,
                            self.get_value(),
                            _rhs.get_value(),
                            c_str!("add_num"),
                        );
                        let alloca =
                            LLVMBuildAlloca(context.builder, int32_ptr_type(), c_str!("param_add"));
                        LLVMBuildStore(context.builder, result, alloca);
                        //TODO: fix the new instruction
                        Box::new(NumberType {
                            name: self.name.clone(),
                            llmv_value: result,
                            llmv_value_pointer: Some(alloca),
                            cname: self.get_name(),
                        })
                    }
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

    fn sub(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                match self.llmv_value_pointer {
                    Some(_p) => {
                        let lhs_value = LLVMBuildLoad2(
                            context.builder,
                            int32_type(),
                            self.get_ptr().unwrap(),
                            self.get_name(),
                        );
                        let rhs_value: *mut llvm_sys::LLVMValue = LLVMBuildLoad2(
                            context.builder,
                            int32_type(),
                            _rhs.get_ptr().unwrap(),
                            _rhs.get_name(),
                        );
                        let result =
                            LLVMBuildSub(context.builder, lhs_value, rhs_value, c_str!("add_num"));
                        LLVMBuildStore(context.builder, result, self.get_ptr().unwrap());
                        //TODO: fix the new instruction
                        Box::new(NumberType {
                            name: self.name.clone(),
                            llmv_value: result,
                            llmv_value_pointer: self.llmv_value_pointer,
                            cname: self.get_name(),
                        })
                    }
                    None => {
                        let result = LLVMBuildSub(
                            context.builder,
                            self.get_value(),
                            _rhs.get_value(),
                            c_str!("add_num"),
                        );
                        let alloca =
                            LLVMBuildAlloca(context.builder, int32_ptr_type(), c_str!("param_add"));
                        LLVMBuildStore(context.builder, result, alloca);
                        //TODO: fix the new instruction
                        Box::new(NumberType {
                            name: self.name.clone(),
                            llmv_value: result,
                            llmv_value_pointer: Some(alloca),
                            cname: self.get_name(),
                        })
                    }
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

    fn mul(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                match self.llmv_value_pointer {
                    Some(_p) => {
                        let lhs_value = LLVMBuildLoad2(
                            context.builder,
                            int32_type(),
                            self.get_ptr().unwrap(),
                            self.get_name(),
                        );
                        let rhs_value: *mut llvm_sys::LLVMValue = LLVMBuildLoad2(
                            context.builder,
                            int32_type(),
                            _rhs.get_ptr().unwrap(),
                            _rhs.get_name(),
                        );
                        let result =
                            LLVMBuildMul(context.builder, lhs_value, rhs_value, c_str!("add_num"));
                        LLVMBuildStore(context.builder, result, self.get_ptr().unwrap());
                        //TODO: fix the new instruction
                        Box::new(NumberType {
                            name: self.name.clone(),
                            llmv_value: result,
                            llmv_value_pointer: self.llmv_value_pointer,
                            cname: self.get_name(),
                        })
                    }
                    None => {
                        let result = LLVMBuildMul(
                            context.builder,
                            self.get_value(),
                            _rhs.get_value(),
                            c_str!("add_num"),
                        );
                        let alloca =
                            LLVMBuildAlloca(context.builder, int32_ptr_type(), c_str!("param_add"));
                        LLVMBuildStore(context.builder, result, alloca);
                        //TODO: fix the new instruction
                        Box::new(NumberType {
                            name: self.name.clone(),
                            llmv_value: result,
                            llmv_value_pointer: Some(alloca),
                            cname: self.get_name(),
                        })
                    }
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

    fn div(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                let lhs_value = LLVMBuildLoad2(
                    context.builder,
                    int32_type(),
                    self.get_ptr().unwrap(),
                    self.get_name(),
                );
                let rhs_value: *mut llvm_sys::LLVMValue = LLVMBuildLoad2(
                    context.builder,
                    int32_type(),
                    _rhs.get_ptr().unwrap(),
                    _rhs.get_name(),
                );
                let result = LLVMBuildSDiv(context.builder, lhs_value, rhs_value, c_str!("result"));
                LLVMBuildStore(context.builder, result, self.get_ptr().unwrap());
                //TODO: fix the new instruction
                Box::new(NumberType {
                    name: self.name.clone(),
                    llmv_value: result,
                    llmv_value_pointer: self.llmv_value_pointer,
                    cname: self.get_name(),
                })
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

impl Debug for NumberType {
    fn print(&self, ast_context: &mut ASTContext) {
        unsafe {
            // Load Value from Value Index Ptr
            match self.get_ptr() {
                Some(e) => {
                    let val = LLVMBuildLoad2(
                        ast_context.builder,
                        int32_ptr_type(),
                        e,
                        c_str!("num_printf_ptr_val"),
                    );

                    let print_args = [ast_context.printf_str_num_value, val].as_mut_ptr();
                    match ast_context.llvm_func_cache.get("printf") {
                        Some(print_func) => {
                            LLVMBuildCall2(
                                ast_context.builder,
                                print_func.func_type,
                                print_func.function,
                                print_args,
                                2,
                                c_str!(""),
                            );
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
                None => {
                    // let alloca = LLVMBuildAlloca(ast_context.builder, int32_ptr_type(), c_str!("print_num_ptr"));
                    // LLVMBuildStore(ast_context.builder, self.get_value(), alloca);
                    let print_args =
                        [ast_context.printf_str_num_value, self.get_value()].as_mut_ptr();
                    match ast_context.llvm_func_cache.get("printf") {
                        Some(print_func) => {
                            LLVMBuildCall2(
                                ast_context.builder,
                                print_func.func_type,
                                print_func.function,
                                print_args,
                                2,
                                c_str!(""),
                            );
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
            }
        }
    }
}

impl TypeBase for NumberType {
    fn new(_value: Box<dyn Any>, _name: String, _context: &mut ASTContext) -> Box<dyn TypeBase> {
        let value_as_i32 = match _value.downcast_ref::<i32>() {
            Some(val) => *val,
            None => panic!("The input value must be an i32"),
        };
        unsafe {
            let value = LLVMConstInt(int32_type(), value_as_i32.try_into().unwrap(), 0);
            let ptr = LLVMBuildAlloca(_context.builder, int32_ptr_type(), c_str(_name.as_str()));
            LLVMBuildStore(_context.builder, value, ptr);
            let cname = c_str!("var_num_var");
            Box::new(NumberType {
                name: _name,
                llmv_value: value,
                llmv_value_pointer: Some(ptr),
                cname,
            })
        }
    }
    unsafe fn get_name(&self) -> *const i8 {
        self.cname
    }
    fn assign(&mut self, _ast_context: &mut ASTContext, _rhs: Box<dyn TypeBase>) {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                let alloca = self.get_ptr().unwrap();
                let name = LLVMGetValueName(self.get_value());
                let new_value = LLVMBuildLoad2(_ast_context.builder, int32_type(), alloca, name);
                LLVMBuildStore(_ast_context.builder, new_value, alloca);
            },
            _ => {
                unreachable!(
                    "Can't reassign variable {:?} that has type {:?} to type {:?}",
                    self.name,
                    self.get_type(),
                    _rhs.get_type()
                )
            }
        }
    }

    fn get_value(&self) -> LLVMValueRef {
        self.llmv_value
    }
    fn get_ptr(&self) -> Option<LLVMValueRef> {
        self.llmv_value_pointer
    }
}

unsafe fn get_int32_arg_for_comp_self(
    val: NumberType,
    rhs: Box<dyn TypeBase>,
) -> [LLVMValueRef; 2] {
    match val.get_ptr() {
        Some(v) => {
            [v, rhs.get_ptr().unwrap()]
        }
        None => [val.get_value(), rhs.get_value()],
    }
}

impl Comparison for NumberType {
    fn eqeq(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                let compare_int32_args = [self.get_value(), _rhs.get_value()].as_mut_ptr();
                match context.llvm_func_cache.get("compare_int32") {
                    Some(compare_int32_func) => {
                        let cmp = LLVMBuildCall2(
                            context.builder,
                            compare_int32_func.func_type,
                            compare_int32_func.function,
                            compare_int32_args,
                            2,
                            c_str!(""),
                        );
                        let alloca =
                            LLVMBuildAlloca(context.builder, int1_type(), c_str!("bool_cmp"));
                        LLVMBuildStore(context.builder, cmp, alloca);
                        Box::new(BoolType {
                            name: String::from("eqeq_bool"),
                            builder: context.builder,
                            llmv_value: cmp,
                            llmv_value_pointer: alloca,
                        })
                    }
                    _ => {
                        unreachable!()
                    }
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

    fn ne(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                return get_comparison_number_type(
                    self.name.clone(),
                    context,
                    _rhs.get_ptr(),
                    self.get_ptr(),
                    LLVMIntPredicate::LLVMIntNE,
                );
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

    fn gt(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                return get_comparison_number_type(
                    self.name.clone(),
                    context,
                    _rhs.get_ptr(),
                    self.get_ptr(),
                    LLVMIntPredicate::LLVMIntSGT,
                );
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

    fn gte(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                return get_comparison_number_type(
                    self.name.clone(),
                    context,
                    _rhs.get_ptr(),
                    self.get_ptr(),
                    LLVMIntPredicate::LLVMIntSGE,
                );
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

    fn lt(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                return get_comparison_number_type(
                    self.name.clone(),
                    context,
                    _rhs.get_ptr(),
                    self.get_ptr(),
                    LLVMIntPredicate::LLVMIntSLT,
                );
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

    fn lte(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                return get_comparison_number_type(
                    self.name.clone(),
                    context,
                    _rhs.get_ptr(),
                    self.get_ptr(),
                    LLVMIntPredicate::LLVMIntSLE,
                );
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

impl Func for NumberType {}

unsafe fn get_comparison_number_type(
    _name: String,
    _context: &mut ASTContext,
    rhs: Option<LLVMValueRef>,
    lhs: Option<LLVMValueRef>,
    comparison: LLVMIntPredicate,
) -> Box<dyn TypeBase> {
    let lhs_val = LLVMBuildLoad2(
        _context.builder,
        int8_type(),
        lhs.unwrap(),
        c_str!("lhs_bool"),
    );
    let rhs_val = LLVMBuildLoad2(
        _context.builder,
        int8_type(),
        rhs.unwrap(),
        c_str!("rhs_bool"),
    );
    let cmp = LLVMBuildICmp(
        _context.builder,
        comparison,
        lhs_val,
        rhs_val,
        c_str!("result"),
    );
    let alloca = LLVMBuildAlloca(_context.builder, int1_type(), c_str!("bool_cmp"));
    LLVMBuildStore(_context.builder, cmp, alloca);
    Box::new(BoolType {
        name: _name,
        builder: _context.builder,
        llmv_value: cmp,
        llmv_value_pointer: alloca,
    })
}
