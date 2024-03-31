use crate::compiler::codegen::builder::LLVMCodegenBuilder;
use crate::compiler::codegen::context::LLVMFunction;
use crate::compiler::codegen::{
    cstr_from_string, int1_type, int32_ptr_type, int32_type, int64_ptr_type, int64_type,
};
use crate::compiler::types::bool::BoolType;
use crate::compiler::types::func::FuncType;
use crate::compiler::types::list::ListType;
use crate::compiler::types::num::NumberType;
use crate::compiler::types::num64::NumberType64;
use crate::compiler::types::return_type::ReturnType;
use crate::compiler::types::string::StringType;
use crate::compiler::types::void::VoidType;
use crate::compiler::types::TypeBase;
use crate::compiler::visitor::Visitor;
use crate::compiler::Expression;
use anyhow::anyhow;
use anyhow::Result;
use libc::c_ulonglong;
use llvm_sys::core::{
    LLVMBuildPointerCast, LLVMConstStringInContext, LLVMInt8Type, LLVMPointerType,
};
use std::collections::HashMap;
use std::ffi::CString;

pub struct ASTContext {
    pub var_cache: VariableCache,
    pub func_cache: VariableCache,
    pub depth: i32,
}

impl ASTContext {
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

#[derive(Clone)]
struct Container {
    pub trait_object: Box<dyn TypeBase>,
}
pub struct VariableCache {
    map: HashMap<String, Container>,
    local: HashMap<i32, Vec<String>>,
}

impl Default for VariableCache {
    fn default() -> Self {
        Self::new()
    }
}

impl VariableCache {
    pub fn new() -> Self {
        VariableCache {
            map: HashMap::new(),
            local: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, trait_object: Box<dyn TypeBase>, depth: i32) {
        let mut locals: HashMap<i32, bool> = HashMap::new();
        locals.insert(depth, true);
        self.map.insert(key.to_string(), Container { trait_object });
        match self.local.get(&depth) {
            Some(val) => {
                let mut val_clone = val.clone();
                val_clone.push(key.to_string());
                self.local.insert(depth, val_clone);
            }
            None => {
                self.local.insert(depth, vec![key.to_string()]);
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<Box<dyn TypeBase>> {
        match self.map.get(key) {
            Some(v) => Some(dyn_clone::clone_box(&*v.trait_object)),
            None => None,
        }
    }

    #[allow(dead_code)]
    fn del(&mut self, key: &str) {
        self.map.remove(key);
    }

    pub fn del_locals(&mut self, depth: i32) {
        if let Some(v) = self.local.get(&depth) {
            for local in v.iter() {
                self.map.remove(&local.to_string());
            }
            self.local.remove(&depth);
        }
    }
}

impl ASTContext {
    pub fn init() -> Result<ASTContext> {
        let var_cache = VariableCache::new();
        let func_cache = VariableCache::new();
        //TODO: remove
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
            Expression::Variable(_) => {
                visitor.visit_variable_expr(&input, codegen, &self.var_cache)
            }
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
}

pub struct LLVMCodegenVisitor {
    // codegen: LLVMCodegenBuilder,
    // var_cache: VariableCache,
}

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
            let string: CString = CString::new(val.clone()).unwrap();
            unsafe {
                let value = LLVMConstStringInContext(
                    codegen.context,
                    string.as_ptr(),
                    string.as_bytes().len() as u32,
                    0,
                );
                let mut len_value: usize = string.as_bytes().len();
                let ptr: *mut usize = (&mut len_value) as *mut usize;
                let buffer_ptr = LLVMBuildPointerCast(
                    codegen.builder,
                    value,
                    LLVMPointerType(LLVMInt8Type(), 0),
                    cstr_from_string(name).as_ptr(),
                );
                return Ok(Box::new(StringType {
                    name: name.to_string(),
                    length: ptr,
                    llvm_value: value,
                    llvm_value_pointer: Some(buffer_ptr),
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
        var_cache: &VariableCache,
    ) -> Result<Box<dyn TypeBase>> {
        if let Expression::Variable(input) = left {
            return match codegen.current_function.symbol_table.get(input) {
                Some(val) => Ok(val.clone()),
                None => {
                    // check if variable is in function
                    // TODO: should this be reversed i.e check func var first then global
                    match var_cache.get(input) {
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
            let mut vec_expr = vec![];
            let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor {});
            for x in v {
                let expr = context.match_ast(x.clone(), &mut visitor, codegen)?;
                vec_expr.push(expr)
            }
            let first_element = vec_expr.first().unwrap();
            let mut elements = vec![];
            for x in vec_expr.iter() {
                elements.push(x.get_value());
            }

            let array_type = first_element.get_llvm_type();
            let array_len = vec_expr.len() as u64;
            let llvm_array_value =
                codegen.const_array(array_type, elements.as_mut_ptr(), array_len);

            let llvm_array_type = codegen.array_type(array_type, array_len);
            let array_ptr = codegen.build_alloca_store(llvm_array_value, llvm_array_type, "array");
            return Ok(Box::new(ListType {
                llvm_value: llvm_array_value,
                llvm_value_ptr: array_ptr,
                llvm_type: llvm_array_type,
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
            let name = cstr_from_string("access_array").as_ptr();
            let val = context.match_ast(*v.clone(), &mut visitor, codegen)?;
            let index = context.match_ast(*i.clone(), &mut visitor, codegen)?;
            let zero_index = codegen.const_int(int64_type(), 0, 0);
            let build_load_index =
                codegen.build_load(index.get_ptr().unwrap(), index.get_llvm_type(), "example");
            let mut indices = [zero_index, build_load_index];
            let val = codegen.build_gep(
                val.get_llvm_type(),
                val.get_ptr().unwrap(),
                indices.as_mut_ptr(),
                2_u32,
                name,
            );
            return Ok(Box::new(NumberType {
                llvm_value: val,
                llvm_value_pointer: Some(val),
                name: "".to_string(),
            }));
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
            match context.var_cache.get(var) {
                Some(val) => {
                    let name = cstr_from_string("access_array").as_ptr();
                    let lhs: Box<dyn TypeBase> =
                        context.match_ast(*rhs.clone(), &mut visitor, codegen)?;
                    let index = context.match_ast(*i.clone(), &mut visitor, codegen)?;
                    let zero_index = codegen.const_int(int64_type(), 0, 0);
                    let build_load_index = codegen.build_load(
                        index.get_ptr().unwrap(),
                        index.get_llvm_type(),
                        "example",
                    );
                    let mut indices = [zero_index, build_load_index];
                    let element_ptr = codegen.build_gep(
                        val.get_llvm_type(),
                        val.get_ptr().unwrap(),
                        indices.as_mut_ptr(),
                        2_u32,
                        name,
                    );
                    codegen.build_store(lhs.get_value(), element_ptr);
                    return Ok(val);
                }
                _ => {
                    unreachable!("can't assign as var doesn't exist")
                }
            }
        }
        Err(anyhow!("unable to assign variable for list"))
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

    fn visit_nil(&mut self) -> Result<Box<dyn TypeBase>> {
        todo!()
    }

    fn visit_let_stmt(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor {});
        if let Expression::LetStmt(var, _, lhs) = left {
            match context.var_cache.get(var) {
                Some(mut val) => {
                    // Check Variables are the same Type
                    // Then Update the value of the old variable
                    // reassign variable

                    // Assign a temp variable to the stack
                    let lhs: Box<dyn TypeBase> =
                        context.match_ast(*lhs.clone(), &mut visitor, codegen)?;
                    // Assign this new value
                    val.assign(codegen, lhs)?;
                    return Ok(val);
                }
                _ => {
                    let lhs: Box<dyn TypeBase> =
                        context.match_ast(*lhs.clone(), &mut visitor, codegen)?;
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
                    let call_val = val.call(context, args.clone(), &mut visitor, codegen)?;
                    context
                        .var_cache
                        .set(name.as_str(), call_val.clone(), context.depth);
                    Ok(call_val)
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
            return LLVMCodegenBuilder::new_if_stmt(
                context,
                cond,
                *if_stmt.clone(),
                *else_stmt.clone(),
                &mut visitor,
                codegen,
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
            return LLVMCodegenBuilder::new_while_stmt(
                context,
                cond,
                *while_block_stmt.clone(),
                &mut visitor,
                codegen,
            );
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
            return LLVMCodegenBuilder::new_for_loop(
                context,
                var_name.to_string(),
                *init,
                *length,
                *increment,
                *for_block_expr.clone(),
                codegen,
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
            return Ok(Box::new(ReturnType {}))
        }
        Err(anyhow!("unable to visit print stmt"))
    }
}
