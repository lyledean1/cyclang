use crate::compiler::llvm::context::LLVMFunction;

use crate::parser::{Expression, Type};
extern crate llvm_sys;
use crate::c_str;
use crate::compiler::types::bool::BoolType;
use crate::compiler::llvm::c_str;
use crate::compiler::types::num::NumberType;
use crate::compiler::types::void::VoidType;
use crate::compiler::types::{Arithmetic, Base, BaseTypes, Comparison, Debug, Func, TypeBase};
use llvm_sys::core::LLVMBuildCall2;
use llvm_sys::prelude::*;

// FuncType -> Exposes the Call Func (i.e after function has been executed)
// So can provide the return type to be used after execution
#[derive(Clone)]
pub struct FuncType {
    pub body: Expression,
    pub args: Vec<Expression>,
    pub return_type: Type,
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
    fn call(
        &self,
        _context: &mut crate::compiler::llvm::context::ASTContext,
        args: Vec<Expression>,
    ) -> Box<dyn TypeBase> {
        unsafe {
            // need to build up call with actual LLVMValue

            let call_args = &mut vec![];
            for arg in args.iter() {
                call_args.push(_context.match_ast(arg.clone()).get_ptr());
            }
            let call_value = LLVMBuildCall2(
                _context.builder,
                self.get_llvm_type(),
                self.get_value(),
                call_args.as_mut_ptr(),
                self.llvm_func_ref.args.len() as u32,
                c_str!(""),
            );
            match self.return_type {
                Type::Int => {
                    return Box::new(NumberType {
                        llmv_value: call_value,
                        llmv_value_pointer: call_value,
                        name: "call_value".into(),
                        cname: c_str("call_value"),
                    });
                }
                Type::Bool => {
                    return Box::new(BoolType {
                        builder: _context.builder,
                        llmv_value: call_value,
                        llmv_value_pointer: call_value,
                        name: "call_value".into(),
                    })
                }
                Type::String => {}
                Type::None => {
                    //Return void
                    return Box::new(VoidType {});
                }
            }
            unreachable!("type {:?} not found", self.return_type);
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
    fn set_args(&mut self, args: Vec<Expression>) {
        self.args = args;
    }
    fn get_args(&self) -> Vec<Expression> {
        self.args.clone()
    }
}
