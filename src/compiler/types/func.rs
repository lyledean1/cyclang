use crate::parser::{Expression, Type};
extern crate llvm_sys;
use crate::compiler::llvm::{cstr_from_string, int1_ptr_type, int32_ptr_type};
use crate::compiler::types::bool::BoolType;
use crate::compiler::types::num::NumberType;
use crate::compiler::types::void::VoidType;
use crate::compiler::types::{Arithmetic, Base, BaseTypes, Comparison, Debug, Func, TypeBase};
use llvm_sys::core::{LLVMBuildAlloca, LLVMBuildCall2, LLVMBuildStore, LLVMCountParamTypes};
use llvm_sys::prelude::*;
use crate::cyclo_error::CycloError;

// FuncType -> Exposes the Call Func (i.e after function has been executed)
// So can provide the return type to be used after execution
#[derive(Clone)]
pub struct FuncType {
    pub return_type: Type,
    pub llvm_type: LLVMTypeRef,
    pub llvm_func: LLVMValueRef,
}

impl Base for FuncType {
    fn get_llvm_type(&self) -> LLVMTypeRef {
        self.llvm_type
    }
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
    ) -> Result<Box<dyn TypeBase>, CycloError> {
        unsafe {
            // need to build up call with actual LLVMValue

            let call_args = &mut vec![];
            for arg in args.iter() {
                call_args.push(_context.match_ast(arg.clone())?.get_value());
            }
            let call_value = LLVMBuildCall2(
                _context.builder,
                self.get_llvm_type(),
                self.get_value(),
                call_args.as_mut_ptr(),
                LLVMCountParamTypes(self.get_llvm_type()),
                cstr_from_string("").as_ptr(),
            );
            match self.return_type {
                Type::Int => {
                    let ptr = LLVMBuildAlloca(
                        _context.builder,
                        int32_ptr_type(),
                        cstr_from_string("call_value_int").as_ptr(),
                    );
                    LLVMBuildStore(_context.builder, call_value, ptr);

                    return Ok(Box::new(NumberType {
                        llmv_value: call_value,
                        llmv_value_pointer: None,
                        name: "call_value".into(),
                        cname: cstr_from_string("call_value").as_ptr(),
                    }));
                }
                Type::Bool => {
                    let ptr = LLVMBuildAlloca(
                        _context.builder,
                        int1_ptr_type(),
                        cstr_from_string("call_value_int").as_ptr(),
                    );
                    LLVMBuildStore(_context.builder, call_value, ptr);
                    return Ok(Box::new(BoolType {
                        builder: _context.builder,
                        llmv_value: call_value,
                        llmv_value_pointer: ptr,
                        name: "call_value".into(),
                    }));
                }
                Type::String => {}
                Type::None => {
                    //Return void
                    return Ok(Box::new(VoidType {}));
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
}
