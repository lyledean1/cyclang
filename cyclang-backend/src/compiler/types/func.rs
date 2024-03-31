extern crate llvm_sys;
use crate::compiler::codegen::{cstr_from_string, int1_ptr_type, int32_ptr_type, int64_ptr_type};
use crate::compiler::context::ASTContext;
use crate::compiler::types::bool::BoolType;
use crate::compiler::types::num::NumberType;
use crate::compiler::types::void::VoidType;
use crate::compiler::types::{BaseTypes, Func, TypeBase};
use anyhow::Result;
use cyclang_parser::{Expression, Type};
use llvm_sys::core::{LLVMBuildCall2, LLVMCountParamTypes};
use llvm_sys::prelude::*;

// FuncType -> Exposes the Call Func (i.e after function has been executed)
// So can provide the return type to be used after execution
#[derive(Clone)]
pub struct FuncType {
    pub return_type: Type,
    pub llvm_type: LLVMTypeRef,
    pub llvm_func: LLVMValueRef,
}
impl Func for FuncType {
    fn call(&self, context: &mut ASTContext, args: Vec<Expression>) -> Result<Box<dyn TypeBase>> {
        unsafe {
            // need to build up call with actual LLVMValue

            let call_args = &mut vec![];
            for arg in args.iter() {
                // build load args i.e if variable
                let ast_value = context.match_ast(arg.clone())?;
                if let Some(ptr) = ast_value.get_ptr() {
                    let loaded_value =
                        context
                            .codegen
                            .build_load(ptr, ast_value.get_llvm_type(), "call_arg");
                    call_args.push(loaded_value);
                } else {
                    call_args.push(ast_value.get_value());
                }
            }
            let llvm_type = self.get_llvm_type();
            let value = self.get_value();
            let call_value = LLVMBuildCall2(
                context.codegen.builder,
                llvm_type,
                value,
                call_args.as_mut_ptr(),
                LLVMCountParamTypes(llvm_type),
                cstr_from_string("").as_ptr(),
            );
            match self.return_type {
                Type::i32 => {
                    let _ptr = context.codegen.build_alloca_store(
                        call_value,
                        int32_ptr_type(),
                        "call_value_int32",
                    );
                    Ok(Box::new(NumberType {
                        llvm_value: call_value,
                        llvm_value_pointer: None,
                        name: "call_value".into(),
                    }))
                }
                Type::i64 => {
                    let _ptr = context.codegen.build_alloca_store(
                        call_value,
                        int64_ptr_type(),
                        "call_value_int64",
                    );
                    Ok(Box::new(NumberType {
                        llvm_value: call_value,
                        llvm_value_pointer: None,
                        name: "call_value".into(),
                    }))
                }
                Type::Bool => {
                    let ptr = context.codegen.build_alloca_store(
                        call_value,
                        int1_ptr_type(),
                        "bool_value",
                    );
                    Ok(Box::new(BoolType {
                        builder: context.codegen.builder,
                        llvm_value: call_value,
                        llvm_value_pointer: ptr,
                        name: "call_value".into(),
                    }))
                }
                Type::String => {
                    unimplemented!("String types haven't been implemented yet for functions")
                }
                Type::List(_) => {
                    unimplemented!("List types haven't been implemented yet for functions")
                }
                Type::None => {
                    //Return void
                    Ok(Box::new(VoidType {}))
                }
            }
        }
    }

}

impl TypeBase for FuncType {
    fn get_value(&self) -> LLVMValueRef {
        self.llvm_func
    }
    fn get_type(& self) -> BaseTypes { BaseTypes :: Func }

    fn get_llvm_type(&self) -> LLVMTypeRef {
        self.llvm_type
    }
}
