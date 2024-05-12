use crate::compiler::cache::VariableCache;
use crate::compiler::codegen::builder::LLVMCodegenBuilder;
use crate::compiler::codegen::context::LLVMFunction;
use crate::compiler::codegen::{
    cstr_from_string, int1_ptr_type, int1_type, int32_ptr_type, int32_type, int64_ptr_type,
    int64_type, int8_ptr_type,
};
use crate::compiler::types::bool::BoolType;
use crate::compiler::types::func::FuncType;
use crate::compiler::types::list::ListType;
use crate::compiler::types::num::NumberType;
use crate::compiler::types::num64::NumberType64;
use crate::compiler::types::return_type::ReturnType;
use crate::compiler::types::string::StringType;
use crate::compiler::types::void::VoidType;
use crate::compiler::types::{BaseTypes, TypeBase};
use crate::compiler::visitor::Visitor;
use crate::compiler::Expression;
use anyhow::anyhow;
use anyhow::Result;
use cyclang_parser::Type;
use libc::c_ulonglong;
use llvm_sys::core::{LLVMBuildCall2, LLVMConstStringInContext, LLVMCountParamTypes};
use std::ffi::CString;

pub struct ASTContext {
    pub var_cache: VariableCache,
    pub func_cache: VariableCache,
    pub depth: i32,
}

impl ASTContext {
    pub fn init() -> Result<ASTContext> {
        let var_cache = VariableCache::new();
        let func_cache = VariableCache::new();
        Ok(ASTContext {
            var_cache,
            func_cache,
            depth: 0,
        })
    }

    pub fn match_ast(
        &mut self,
        input: Expression,
        visitor: &mut Box<dyn Visitor<Box<dyn TypeBase>>>,
        codegen: &mut LLVMCodegenBuilder,
    ) -> Result<Box<dyn TypeBase>> {
        match input {
            Expression::Number(_) => visitor.visit_number(&input, codegen),
            Expression::Number64(_) => visitor.visit_number(&input, codegen),
            Expression::String(_) => visitor.visit_string(&input, codegen),
            Expression::Bool(_) => visitor.visit_bool(&input, codegen),
            Expression::Variable(_) => visitor.visit_variable_expr(&input, codegen, self),
            Expression::List(_) => visitor.visit_list_expr(&input, codegen, self),
            Expression::ListIndex(_, _) => visitor.visit_list_index_expr(&input, codegen, self),
            Expression::ListAssign(_, _, _) => {
                visitor.visit_list_assign_expr(&input, codegen, self)
            }
            Expression::Nil => visitor.visit_nil(),
            Expression::Binary(_, _, _) => visitor.visit_binary_stmt(&input, codegen, self),
            Expression::Grouping(_) => visitor.visit_grouping_stmt(input, codegen, self),
            Expression::LetStmt(_, _, _) => visitor.visit_let_stmt(&input, codegen, self),
            Expression::BlockStmt(_) => visitor.visit_block_stmt(&input, codegen, self),
            Expression::CallStmt(_, _) => visitor.visit_call_stmt(&input, codegen, self),
            Expression::FuncStmt(_, _, _, _) => visitor.visit_func_stmt(&input, codegen, self),
            Expression::IfStmt(_, _, _) => visitor.visit_if_stmt(&input, codegen, self),
            Expression::WhileStmt(_, _) => visitor.visit_while_stmt(&input, codegen, self),
            Expression::ForStmt(_, _, _, _, _) => {
                visitor.visit_for_loop_stmt(&input, codegen, self)
            }
            Expression::Print(_) => visitor.visit_print_stmt(&input, codegen, self),
            Expression::ReturnStmt(_) => visitor.visit_return_stmt(&input, codegen, self),
            _ => Err(anyhow!("this should be unreachable code, for {:?}", input)),
        }
    }

    pub fn get_depth(&self) -> i32 {
        self.depth
    }
    pub fn incr(&mut self) {
        self.depth += 1;
    }
    pub fn decr(&mut self) {
        self.depth -= 1;
    }
}
pub struct LLVMCodegenVisitor {}

impl Visitor<Box<dyn TypeBase>> for LLVMCodegenVisitor {
    fn visit_number(
        &mut self,
        left: &Expression,
        codegen: &LLVMCodegenBuilder,
    ) -> Result<Box<dyn TypeBase>> {
        match left {
            Expression::Number(val) => {
                let name = "num32";
                let c_val = *val as c_ulonglong;
                let value = codegen.const_int(int32_type(), c_val, 0);
                let ptr = codegen.build_alloca_store(value, int32_ptr_type(), name);
                Ok(Box::new(NumberType {
                    name: name.to_string(),
                    llvm_value: value,
                    llvm_value_pointer: Some(ptr),
                }))
            }
            Expression::Number64(val) => {
                let name = "num64";
                let c_val = *val as c_ulonglong;
                let value = codegen.const_int(int64_type(), c_val, 0);
                let ptr = codegen.build_alloca_store(value, int64_ptr_type(), name);
                Ok(Box::new(NumberType64 {
                    name: name.to_string(),
                    llvm_value: value,
                    llvm_value_pointer: Some(ptr),
                }))
            }
            _ => Err(anyhow!("type is not a number (i32,i64)")),
        }
    }

    fn visit_string(
        &mut self,
        left: &Expression,
        codegen: &LLVMCodegenBuilder,
    ) -> Result<Box<dyn TypeBase>> {
        if let Expression::String(val) = left {
            let name = "str_val";
            let val = val.replace('"', "");
            let string: CString = CString::new(val.clone()).unwrap();
            unsafe {
                let value = LLVMConstStringInContext(
                    codegen.context,
                    string.as_ptr(),
                    string.as_bytes().len() as u32,
                    0,
                );
                let string_init_func_llvm = codegen.llvm_func_cache.get("stringInit").unwrap();
                let string_ptr =
                    codegen.build_alloca_store(value, int8_ptr_type(), "stringPtrExample");

                let return_value = codegen.build_call(
                    string_init_func_llvm.clone(),
                    vec![string_ptr],
                    1,
                    "stringInitExample",
                );

                let mut len_value: usize = string.as_bytes().len();
                let ptr: *mut usize = (&mut len_value) as *mut usize;
                return Ok(Box::new(StringType {
                    name: name.to_string(),
                    length: ptr,
                    llvm_value: return_value,
                    llvm_value_pointer: Some(return_value),
                    str_value: val.to_string(), // fix
                }));
            }
        }
        Err(anyhow!("type is not a string"))
    }

    fn visit_bool(
        &mut self,
        left: &Expression,
        codegen: &LLVMCodegenBuilder,
    ) -> Result<Box<dyn TypeBase>> {
        if let Expression::Bool(val) = left {
            let value = *val;
            let name = "bool_value";
            let bool_value = codegen.const_int(int1_type(), value.into(), 0);
            let alloca = codegen.build_alloca_store(bool_value, int1_type(), name);
            return Ok(Box::new(BoolType {
                name: name.parse()?,
                builder: codegen.builder,
                llvm_value: bool_value,
                llvm_value_pointer: alloca,
            }));
        }
        Err(anyhow!("type is not a bool"))
    }

    fn visit_variable_expr(
        &mut self,
        left: &Expression,
        codegen: &LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        if let Expression::Variable(input) = left {
            return match codegen.current_function.symbol_table.get(input) {
                Some(val) => Ok(val.clone()),
                None => {
                    // check if variable is in function
                    // TODO: should this be reversed i.e check func var first then global
                    match context.var_cache.get(input) {
                        Some(val) => Ok(val),
                        None => Err(anyhow!(format!("Unknown variable {}", input))),
                    }
                }
            };
        }
        Err(anyhow!("type is not an i32"))
    }

    fn visit_list_expr(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        if let Expression::List(v) = left {
            // get elements
            let mut vec_expr = vec![];
            let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor {});
            for x in v {
                let expr = context.match_ast(x.clone(), &mut visitor, codegen)?;
                vec_expr.push(expr)
            }

            let first_type = vec_expr.first().unwrap().get_type();

            // todo: refactor this
            let list_init_func_name = Self::get_list_init_func_name(&first_type);

            let list_init_func = codegen.llvm_func_cache.get(list_init_func_name).unwrap();


            let length = self.visit_number(&Expression::Number(vec_expr.len() as i32), codegen);
            let list = codegen.build_call(list_init_func, vec![length.unwrap().get_value()], 1, "");

            let set_int32_func = codegen.llvm_func_cache.get("setInt32Value").unwrap();
            let set_string_func = codegen.llvm_func_cache.get("setStringValue").unwrap();

            for (i, x) in vec_expr.iter().enumerate() {
                let index = self.visit_number(&Expression::Number(i as i32), codegen);
                let func_args = vec![list, x.get_value(), index.unwrap().get_value()];
                match x.get_type() {
                    BaseTypes::Number => {
                        codegen.build_call(set_int32_func.clone(), func_args, 3, "");
                    }
                    BaseTypes::String => {
                        codegen.build_call(set_string_func.clone(), func_args, 3, "");
                    }
                    _ => {
                        unimplemented!("type {:?} is unimplemented", x.get_type())
                    }
                }
            }
            let list_ptr_value = codegen.build_load(list, int32_ptr_type(), "");
            return Ok(Box::new(ListType {
                llvm_value: list,
                llvm_value_ptr: list_ptr_value,
                llvm_type: int32_ptr_type(),
                inner_type: first_type,
            }));
        }
        Err(anyhow!("unable to visit list"))
    }

    fn visit_list_index_expr(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor {});
        if let Expression::ListIndex(v, i) = left {
            let val = context.match_ast(*v.clone(), &mut visitor, codegen)?;
            let index = context.match_ast(*i.clone(), &mut visitor, codegen)?;
            let get_index_value_args = vec![val.get_value(), index.get_value()];
            if let BaseTypes::List(inner) = val.get_type() {
                match *inner {
                    BaseTypes::Number => {
                        let get_int32_value_func =
                            codegen.llvm_func_cache.get("getInt32Value").unwrap();
                        let i_val =
                            codegen.build_call(get_int32_value_func, get_index_value_args, 2, "");
                        let i_val_ptr = codegen.build_alloca_store(i_val, int32_ptr_type(), "");
                        return Ok(Box::new(NumberType {
                            llvm_value: i_val,
                            llvm_value_pointer: Some(i_val_ptr),
                            name: "".to_string(),
                        }));
                    }
                    _ => unreachable!("not implement for {:?}", inner),
                }
            }
        }
        Err(anyhow!("not a list index"))
    }

    fn visit_list_assign_expr(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor {});
        if let Expression::ListAssign(var, i, rhs) = left {
            if let Some(val) = context.var_cache.get(var) {
                let lhs: Box<dyn TypeBase> =
                    context.match_ast(*rhs.clone(), &mut visitor, codegen)?;
                let index = context.match_ast(*i.clone(), &mut visitor, codegen)?;
                if let BaseTypes::List(inner) = val.get_type() {
                    match *inner {
                        BaseTypes::Number => {
                            let set_int32_value_func =
                                codegen.llvm_func_cache.get("setInt32Value").unwrap();
                            let set_int32_args =
                                vec![val.get_value(), lhs.get_value(), index.get_value()];
                            codegen.build_call(set_int32_value_func, set_int32_args, 3, "");
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                    return Ok(val);
                }
            }
        }
        Err(anyhow!("unable to assign variable for list"))
    }

    fn visit_nil(&mut self) -> Result<Box<dyn TypeBase>> {
        todo!()
    }

    fn visit_binary_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor {});
        if let Expression::Binary(lhs, op, rhs) = left {
            let lhs = context.match_ast(*lhs.clone(), &mut visitor, codegen)?;
            let rhs = context.match_ast(*rhs.clone(), &mut visitor, codegen)?;
            return match op.as_str() {
                "+" | "-" | "/" | "*" => codegen.arithmetic(lhs, rhs, op.to_string()),
                "^" => Err(anyhow!("^ is not implemented yet")),
                "==" | "!=" | "<" | "<=" | ">" | ">=" => codegen.cmp(lhs, rhs, op.to_string()),

                _ => Err(anyhow!("Operator: {} not implement", op.clone())),
            };
        }
        Err(anyhow!("unable to apply binary operation"))
    }

    fn visit_grouping_stmt(
        &mut self,
        left: Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor {});
        if let Expression::Grouping(val) = left {
            return context.match_ast(*val, &mut visitor, codegen);
        }
        Err(anyhow!("unable to apply grouping"))
    }

    fn visit_let_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor {});
        if let Expression::LetStmt(var, _, lhs) = left {
            let lhs: Box<dyn TypeBase> = context.match_ast(*lhs.clone(), &mut visitor, codegen)?;
            match context.var_cache.get(var) {
                Some(val) => {
                    return codegen.assign(val.clone(), lhs);
                }
                _ => {
                    context
                        .var_cache
                        .set(&var.clone(), lhs.clone(), context.depth);
                    return Ok(lhs);
                }
            }
        }
        Err(anyhow!("unable to visit let statement"))
    }

    fn visit_block_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor {});
        if let Expression::BlockStmt(exprs) = left {
            // Set Variable Depth
            // Each Block Stmt, Incr and Decr
            // Clearing all the "Local" Variables That Have Been Assigned
            context.incr();
            let mut val: Box<dyn TypeBase> = Box::new(VoidType {});
            for expr in exprs {
                val = context.match_ast(expr.clone(), &mut visitor, codegen)?;
            }
            // Delete Variables
            context.var_cache.del_locals(context.get_depth());
            context.decr();
            return Ok(val);
        }
        Err(anyhow!("unable to visit block stmt"))
    }

    fn visit_call_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor {});
        if let Expression::CallStmt(name, args) = left {
            return match context.func_cache.get(name) {
                Some(val) => {
                    unsafe {
                        // need to build up call with actual LLVMValue
                        let call_args = &mut vec![];
                        for arg in args.iter() {
                            // build load args i.e if variable
                            let ast_value =
                                context.match_ast(arg.clone(), &mut visitor, codegen)?;

                            if let Some(ptr) = ast_value.get_ptr() {
                                let loaded_value =
                                    codegen.build_load(ptr, ast_value.get_llvm_type(), "call_arg");
                                call_args.push(loaded_value);
                            } else {
                                call_args.push(ast_value.get_value());
                            }
                        }
                        let llvm_type = val.get_llvm_type();
                        let value = val.get_value();
                        let call_value = LLVMBuildCall2(
                            codegen.builder,
                            llvm_type,
                            value,
                            call_args.as_mut_ptr(),
                            LLVMCountParamTypes(llvm_type),
                            cstr_from_string("").as_ptr(),
                        );
                        match val.get_return_type() {
                            Type::i32 => {
                                let _ptr = codegen.build_alloca_store(
                                    call_value,
                                    int32_ptr_type(),
                                    "call_value_int32",
                                );
                                let call_val = Box::new(NumberType {
                                    llvm_value: call_value,
                                    llvm_value_pointer: None,
                                    name: "call_value".into(),
                                });
                                context.var_cache.set(
                                    name.as_str(),
                                    call_val.clone(),
                                    context.depth,
                                );
                                Ok(call_val)
                            }
                            Type::i64 => {
                                let _ptr = codegen.build_alloca_store(
                                    call_value,
                                    int64_ptr_type(),
                                    "call_value_int64",
                                );
                                let call_val = Box::new(NumberType {
                                    llvm_value: call_value,
                                    llvm_value_pointer: None,
                                    name: "call_value".into(),
                                });
                                context.var_cache.set(
                                    name.as_str(),
                                    call_val.clone(),
                                    context.depth,
                                );
                                Ok(call_val)
                            }
                            Type::Bool => {
                                let ptr = codegen.build_alloca_store(
                                    call_value,
                                    int1_ptr_type(),
                                    "bool_value",
                                );
                                let call_val = Box::new(BoolType {
                                    builder: codegen.builder,
                                    llvm_value: call_value,
                                    llvm_value_pointer: ptr,
                                    name: "call_value".into(),
                                });
                                context.var_cache.set(
                                    name.as_str(),
                                    call_val.clone(),
                                    context.depth,
                                );
                                Ok(call_val)
                            }
                            Type::String => {
                                unimplemented!(
                                    "String types haven't been implemented yet for functions"
                                )
                            }
                            Type::List(_) => {
                                unimplemented!(
                                    "List types haven't been implemented yet for functions"
                                )
                            }
                            Type::None => {
                                //Return void
                                let call_val = Box::new(VoidType {});
                                context.var_cache.set(
                                    name.as_str(),
                                    call_val.clone(),
                                    context.depth,
                                );
                                Ok(call_val)
                            }
                        }
                    }
                }
                _ => Err(anyhow!("call does not exist for function {:?}", name)),
            };
        }
        Err(anyhow!("unable to visit call stmt"))
    }

    fn visit_func_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        if let Expression::FuncStmt(name, args, _return_type, body) = left {
            let llvm_func = LLVMFunction::new(
                context,
                name.clone(),
                args.clone(),
                _return_type.clone(),
                *body.clone(),
                codegen.current_function.block,
                codegen,
            )?;

            let func = FuncType {
                llvm_type: llvm_func.func_type,
                llvm_func: llvm_func.function,
                return_type: _return_type.clone(),
            };
            // Set Func as a variable
            context
                .func_cache
                .set(name, Box::new(func.clone()), context.depth);
            return Ok(Box::new(func));
        }
        Err(anyhow!("unable to visit func stmt"))
    }

    fn visit_if_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor {});
        if let Expression::IfStmt(condition, if_stmt, else_stmt) = left {
            //TODO: fix this so its an associated function
            let cond = *condition.clone();
            return codegen.new_if_stmt(
                context,
                cond,
                *if_stmt.clone(),
                *else_stmt.clone(),
                &mut visitor,
            );
        }
        Err(anyhow!("unable to visit if stmt"))
    }

    fn visit_while_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor {});
        if let Expression::WhileStmt(condition, while_block_stmt) = left {
            //TODO: fix this so its an associated function
            let cond = *condition.clone();
            return codegen.new_while_stmt(context, cond, *while_block_stmt.clone(), &mut visitor);
        }
        Err(anyhow!("unable to visit while stmt"))
    }

    fn visit_for_loop_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        if let Expression::ForStmt(var_name, init, length, increment, for_block_expr) = left {
            //TODO: fix this so its an associated function
            return codegen.new_for_loop(
                context,
                var_name.to_string(),
                *init,
                *length,
                *increment,
                *for_block_expr.clone(),
            );
        }
        Err(anyhow!("unable to visit for loop"))
    }

    fn visit_print_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor {});
        if let Expression::Print(input) = left {
            let expression_value = context.match_ast(*input.clone(), &mut visitor, codegen)?;
            expression_value.print(codegen)?;
            return Ok(expression_value);
        }
        Err(anyhow!("unable to visit print stmt"))
    }

    fn visit_return_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor {});
        if let Expression::ReturnStmt(input) = left {
            let expression_value = context.match_ast(*input.clone(), &mut visitor, codegen)?;
            codegen.build_ret(expression_value.get_value());
            return Ok(Box::new(ReturnType {}));
        }
        Err(anyhow!("unable to visit print stmt"))
    }
}

impl LLVMCodegenVisitor {
    fn get_list_init_func_name(first_type: &BaseTypes) -> &str {
        match first_type {
            BaseTypes::String => {
                "createStringList"
            }
            BaseTypes::Number => {
                "createInt32List"
            }
            _ => {
                unimplemented!("type {:?} is unimplemented", first_type)
            }
        }
    }
}
