extern crate pest;

use pest::Parser;
use std::num::ParseIntError;

#[derive(Parser)]
#[grammar = "../grammar/cyclo.pest"]
struct CycloParser;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    None,
    Int,
    String,
    Bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Number(i32),
    String(String),
    Bool(bool),
    Nil,
    Variable(String),
    Binary(Box<Expression>, String, Box<Expression>),
    Grouping(Box<Expression>),
    LetStmt(String, Box<Expression>),
    BlockStmt(Vec<Expression>),
    FuncArg(String, Type),
    FuncStmt(String, Vec<Expression>, Type, Box<Expression>),
    CallStmt(String, Vec<Expression>),
    IfStmt(Box<Expression>, Box<Expression>, Box<Option<Expression>>),
    WhileStmt(Box<Expression>, Box<Expression>),
    ReturnStmt(Box<Expression>),
    ForStmt(String, i32, i32, i32, Box<Expression>),
    Print(Box<Expression>),
}

impl Expression {
    fn new_number(n: i32) -> Self {
        Self::Number(n)
    }

    fn new_string(s: String) -> Self {
        Self::String(s)
    }

    fn new_binary(left: Expression, op: String, right: Expression) -> Self {
        Self::Binary(Box::new(left), op, Box::new(right))
    }

    fn new_bool(b: bool) -> Self {
        Self::Bool(b)
    }

    fn new_nil() -> Self {
        Self::Nil
    }

    fn new_variable(name: String) -> Self {
        Self::Variable(name)
    }

    fn new_let_stmt(name: String, value: Expression) -> Self {
        Self::LetStmt(name, Box::new(value))
    }

    fn new_block_stmt(exprs: Vec<Expression>) -> Self {
        Self::BlockStmt(exprs)
    }

    fn new_if_stmt(
        condition: Expression,
        if_block_expr: Expression,
        else_block_expr: Option<Expression>,
    ) -> Self {
        Self::IfStmt(
            Box::new(condition),
            Box::new(if_block_expr),
            Box::new(else_block_expr),
        )
    }

    fn new_while_stmt(condition: Expression, while_block_expr: Expression) -> Self {
        Self::WhileStmt(Box::new(condition), Box::new(while_block_expr))
    }

    fn new_for_stmt(
        var_name: String,
        start: i32,
        end: i32,
        step: i32,
        for_block_expr: Expression,
    ) -> Self {
        Self::ForStmt(var_name, start, end, step, Box::new(for_block_expr))
    }

    fn new_func_stmt(
        name: String,
        args: Vec<Expression>,
        return_type: Type,
        body: Expression,
    ) -> Self {
        Self::FuncStmt(name, args, return_type, Box::new(body))
    }

    fn new_func_arg(name: String, arg_type: Type) -> Self {
        Self::FuncArg(name, arg_type)
    }

    fn new_call_stmt(name: String, args: Vec<Expression>) -> Self {
        Self::CallStmt(name, args)
    }

    fn new_print_stmt(value: Expression) -> Self {
        Self::Print(Box::new(value))
    }

    fn new_return_stmt(value: Expression) -> Self {
        Self::ReturnStmt(Box::new(value))
    }
}

fn parse_expression(
    pair: pest::iterators::Pair<Rule>,
) -> Result<Expression, Box<pest::error::Error<Rule>>> {
    match pair.as_rule() {
        Rule::number => {
            let n = pair.as_str().parse().map_err(|e: ParseIntError| {
                pest::error::Error::new_from_span(
                    pest::error::ErrorVariant::CustomError {
                        message: e.to_string(),
                    },
                    pair.as_span(),
                )
            })?;
            Ok(Expression::new_number(n))
        }
        Rule::name => {
            let s = pair.as_str().to_string().replace(' ', "");
            Ok(Expression::new_variable(s))
        }
        Rule::string => {
            let s = pair.as_str().to_string();
            Ok(Expression::new_string(s))
        }
        Rule::bool => match pair.as_str() {
            "true" => Ok(Expression::new_bool(true)),
            "false" => Ok(Expression::new_bool(false)),
            _ => Err(Box::new(pest::error::Error::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: "Invalid boolean value".to_string(),
                },
                pair.as_span(),
            ))),
        },
        Rule::nil => Ok(Expression::new_nil()),
        Rule::binary => {
            let mut inner_pairs = pair.into_inner();
            let next = inner_pairs.next().unwrap();
            let left = parse_expression(next)?;
            let op = inner_pairs.next().unwrap().as_str().to_string();
            let right = parse_expression(inner_pairs.next().unwrap())?;
            Ok(Expression::new_binary(left, op, right))
        }
        Rule::grouping => {
            let inner_pair = pair.into_inner().next().unwrap();
            parse_expression(inner_pair).map(|expr| Expression::Grouping(Box::new(expr)))
        }
        Rule::let_stmt => {
            let mut inner_pairs = pair.into_inner();
            let name = inner_pairs.next().unwrap().as_str().to_string();
            inner_pairs.next(); // Skip the equal sign
            let value = parse_expression(inner_pairs.next().unwrap())?;
            Ok(Expression::new_let_stmt(name, value))
        }
        Rule::expression => {
            let mut inner_pairs = pair.into_inner();
            let left = parse_expression(inner_pairs.next().unwrap())?;
            let op = inner_pairs.next().unwrap().as_str().to_string();
            let right = parse_expression(inner_pairs.next().unwrap())?;
            Ok(Expression::new_binary(left, op, right))
        }
        Rule::literal => {
            let inner_pair = pair.into_inner().next().unwrap();
            parse_expression(inner_pair)
        }
        Rule::print_stmt => {
            let inner_pair = pair.into_inner().next().unwrap();
            let value = parse_expression(inner_pair)?;
            Ok(Expression::new_print_stmt(value))
        }
        Rule::func_stmt => {
            let mut inner_pairs = pair.into_inner();
            let name = inner_pairs.next().unwrap().as_str().to_string();

            // Does this handle no args?
            let mut func_args = vec![];

            while inner_pairs
                .peek()
                .map_or(false, |p| p.as_rule() == Rule::func_arg)
            {
                let args: pest::iterators::Pair<'_, Rule> = inner_pairs.next().unwrap();
                func_args.push(parse_expression(args)?);
            }

            let mut func_type = Type::None;
            // Get function type or default to none
            while inner_pairs.peek().map_or(false, |p| {
                p.as_rule() == Rule::type_name || p.as_rule() == Rule::arrow
            }) {
                let next: pest::iterators::Pair<'_, Rule> = inner_pairs.next().unwrap();
                if next.as_rule() == Rule::type_name {
                    match next.as_str() {
                        "string" => {
                            func_type = Type::String;
                        }
                        "bool" => {
                            func_type = Type::Bool;
                        }
                        "int" => {
                            func_type = Type::Int;
                        }
                        _ => {
                            func_type = Type::None;
                        }
                    }
                }
            }

            let inner = inner_pairs.next().unwrap();
            let body = parse_expression(inner)?;
            let func = Expression::new_func_stmt(name, func_args, func_type, body);
            Ok(func)
        }
        Rule::func_arg => {
            let mut inner_pairs = pair.into_inner();
            while inner_pairs.peek().map_or(false, |p| {
                p.as_rule() == Rule::comma
                    || p.as_rule() == Rule::name
                    || p.as_rule() == Rule::type_name
            }) {
                let next = inner_pairs.next().unwrap();
                if next.as_rule() == Rule::comma {
                    continue;
                }
                if next.as_rule() == Rule::type_name {
                    let arg_type = next.as_str().to_string();
                    let arg_name = inner_pairs.next().unwrap().as_str().to_string();
                    match arg_type.as_str() {
                        "string" => return Ok(Expression::new_func_arg(arg_name, Type::String)),
                        "bool" => return Ok(Expression::new_func_arg(arg_name, Type::Bool)),
                        "int" => return Ok(Expression::new_func_arg(arg_name, Type::Int)),
                        _ => {
                            unimplemented!(
                                "No rule for arg {:?} with arg type {:?}",
                                arg_name,
                                arg_type
                            );
                        }
                    }
                }
            }
            unreachable!("Unable to parse args {:?}", inner_pairs)
        }
        Rule::call_stmt => {
            let mut inner_pairs = pair.into_inner();
            let name = inner_pairs.next().unwrap().as_str().to_string();
            let mut args = vec![];
            // TODO: fix this so it handles the different cases properly instead of this hack
            while inner_pairs.peek().map_or(false, |p| {
                p.as_rule() == Rule::comma
                    || p.as_rule() == Rule::binary
                    || p.as_rule() == Rule::literal
                    || p.as_rule() == Rule::name
            }) {
                let next = inner_pairs.next().unwrap();
                if next.as_rule() != Rule::comma {
                    let arg_expr = parse_expression(next)?;
                    args.push(arg_expr);
                }
            }
            let call = Expression::new_call_stmt(name, args);
            Ok(call)
        }
        Rule::block_stmt => {
            let inner_pairs = pair.into_inner();
            let mut expressions = Vec::new();

            for inner_pair in inner_pairs {
                let rule = inner_pair.as_rule();
                if rule == Rule::semicolon {
                    continue;
                }
                expressions.push(parse_expression(inner_pair)?);
            }

            Ok(Expression::new_block_stmt(expressions))
        }
        Rule::if_stmt => {
            let mut inner_pairs = pair.into_inner();
            let cond = parse_expression(inner_pairs.next().unwrap())?;
            let if_stmt = parse_expression(inner_pairs.next().unwrap())?;
            let else_stmt = if let Some(else_pair) = inner_pairs.next() {
                Some(parse_expression(else_pair)?)
            } else {
                None
            };
            Ok(Expression::new_if_stmt(cond, if_stmt, else_stmt))
        }
        Rule::for_stmt => {
            //TODO: improve this logic
            let mut inner_pairs = pair.into_inner();
            let mut var = inner_pairs.next().unwrap().into_inner();
            let var_name = var.next().unwrap().as_str().to_string().replace(' ', "");
            let start = var.next().unwrap().as_str().parse::<i32>().unwrap();

            //TODO: Identify > and < signs
            let mut cond_stmt = inner_pairs.next().unwrap().into_inner();
            let _cond_var_name = cond_stmt
                .next()
                .unwrap()
                .as_str()
                .to_string()
                .replace(' ', "");
            let end = cond_stmt.next().unwrap().as_str().parse::<i32>().unwrap();

            let mut step = 1;
            let step_stmt = inner_pairs.next();

            if step_stmt.unwrap().as_str().to_string().contains("--") {
                step = -1;
            }
            let block_stmt = parse_expression(inner_pairs.next().unwrap())?;
            Ok(Expression::new_for_stmt(
                var_name, start, end, step, block_stmt,
            ))
        }
        Rule::return_stmt => {
            let inner_pairs = pair.into_inner().next().unwrap();
            let expr = parse_expression(inner_pairs)?;
            Ok(Expression::new_return_stmt(expr))
        }
        Rule::while_stmt => {
            let mut inner_pairs = pair.into_inner();
            let cond = parse_expression(inner_pairs.next().unwrap())?;
            let while_block_expr = parse_expression(inner_pairs.next().unwrap())?;
            Ok(Expression::new_while_stmt(cond, while_block_expr))
        }
        _ => Err(Box::new(pest::error::Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: format!(
                    "Invalid expression for rule {:?} or rule not specified for grammar",
                    pair.as_rule()
                ),
            },
            pair.as_span(),
        ))),
    }
}

fn parse_program(
    pair: pest::iterators::Pair<Rule>,
) -> Result<Vec<Expression>, Box<pest::error::Error<Rule>>> {
    let mut expr_vec = vec![];
    for stmt_pair in pair.into_inner() {
        match stmt_pair.as_rule() {
            Rule::semicolon | Rule::EOI | Rule::comma => {
                continue;
            }
            _ => {
                let expr = parse_expression(stmt_pair)?;
                expr_vec.push(expr);
            }
        }
    }
    Ok(expr_vec)
}

pub fn parse_cyclo_program(input: &str) -> Result<Vec<Expression>, Box<pest::error::Error<Rule>>> {
    match CycloParser::parse(Rule::expression_list, input) {
        Ok(mut pairs) => {
            // TODO: only returns first pair
            // should this iterate through all pairs?
            if let Some(pair) = pairs.next() {
                return parse_program(pair);
            }
        }
        Err(e) => return Err(Box::new(e)),
    };
    unreachable!("parse function program")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::Expression::{FuncArg, Variable};
    #[test]
    fn test_parse_string_expression() {
        let input = r#""hello";"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_string_expression_err() {
        let input = r#"hello";"#;
        assert!(parse_cyclo_program(input).is_err());
    }

    #[test]
    fn test_parse_digit() {
        let input = r#"5;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_digit_err() {
        let input = r#"5"#;
        assert!(parse_cyclo_program(input).is_err());
    }

    #[test]
    fn test_parse_number_expression() {
        let input = r#"555;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_expression_add() {
        let input = r#"555 + 555 + 555;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_expression_add_grouping() {
        let input = r#"(555 + 555) + (555 + 555);"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_minus_negative_number_expression() {
        let input = r#"-555 - 555;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_minus_negative_two_number_expression() {
        let input = r#"-555 - -555;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_negative_digit_expression() {
        let input = r#"-5 - 5;"#;
        match parse_cyclo_program(input) {
            Err(e) => {
                eprintln!("{}", e);
            }
            _ => {}
        }
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_expression_err() {
        let input = r#"555""#;
        assert!(parse_cyclo_program(input).is_err());
    }

    #[test]
    fn test_parse_nil() {
        let input = r#"nil;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_true_bool() {
        let input = r#"true;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_false_bool() {
        let input = r#"false;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_string_equals() {
        let input = r#""hello" == "hello";"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_bool_equals() {
        let input = r#"true == true;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_nil_equals() {
        let input = r#"nil == nil;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_char_equals() {
        let input = r#""h" == "h";"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_str_equals() {
        let input = r#""hello" == "hello";"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_equals_digit() {
        let input = r#"5 == 5;"#;
        match parse_cyclo_program(input) {
            Err(e) => {
                eprintln!("{}", e);
            }
            _ => {}
        }
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_equals_digit_rhs() {
        let input = r#"55 == 5;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_not_equals_digit() {
        let input = r#"5 != 5;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_lt_digit() {
        let input = r#"5 < 5;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_lte_digit() {
        let input = r#"5 <= 5;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_gt_digit() {
        let input = r#"5 > 5;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_gte_digit() {
        let input = r#"5 >= 5;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_equals() {
        let input = r#"55 == 45;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_less_than() {
        let input = r#"55 < 45;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_less_than_equal() {
        let input = r#"55 <= 45;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_more_than() {
        let input = r#"55 > 45;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_more_than_equal() {
        let input = r#"55 >= 45;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_not_equal() {
        let input = r#"55 != 45;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_bool_equals_string() {
        let input = r#"true == "hello";"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_string() {
        let input = r#"let value = "hello";"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_bool() {
        let input = r#"let value = true;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_bool_without_comma() {
        let input = r#"let value = true"#;
        assert!(parse_cyclo_program(input).is_err());
    }

    #[test]
    fn test_parse_let_stmt_digit() {
        let input = r#"let value = 5;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_number() {
        let input = r#"let value = 555;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_nil() {
        let input = r#"let value = nil;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_grouping() {
        let input = r#"let value = (true == true);"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_assign() {
        let input = r#"let value = other_value;"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_bool_assign() {
        let input = r#"let value = (other_value == first_value);"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_print_stmt_bool_assign() {
        let input = r#"print(other_value == first_value);"#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_parse_multi_line_stmt() {
        let input = "
        let one = true;
        let two = false;
        let three = (two == one);
        ";
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_empty_block_stmt() {
        let input = "
        {
            
        }
        ";
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_block_stmt() {
        let input = "
        {
            let b = 5;
            {
                {
                    fn example(int arg1, int arg2) {
                        print(arg1 + arg2);
                    }
                    example(5,5);
                }
                a = 5;
            }
        }
        ";
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_func_no_return() {
        let input = r#"
        fn example() {
            print(1);
        }
        fn hello() {
            print("hello");
        }
        "#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_fn_return_int_num() {
        let input = r#"
        fn get_ten() -> int {
            return 10;
        }
        let var = get_ten();
        "#;
        let output: Result<Vec<Expression>, Box<pest::error::Error<Rule>>> =
            parse_cyclo_program(input);
        let func_expr = build_basic_func_ast(
            "get_ten".into(),
            [].to_vec(),
            Type::Int,
            vec![Expression::ReturnStmt(Box::new(Expression::Number(10)))],
        );
        assert!(output.is_ok());
        assert!(output.unwrap().contains(&func_expr))
    }

    #[test]
    fn test_fn_return_int_value() {
        let input = r#"
        fn get_value(int value) -> int {
            return value;
        }
        "#;
        let output: Result<Vec<Expression>, Box<pest::error::Error<Rule>>> =
            parse_cyclo_program(input);
        let func_expr = build_basic_func_ast(
            "get_value".into(),
            [FuncArg("value".into(), Type::Int)].to_vec(),
            Type::Int,
            vec![Expression::ReturnStmt(Box::new(Expression::Variable(
                "value".into(),
            )))],
        );
        assert!(output.is_ok());
        assert!(output.unwrap().contains(&func_expr))
    }

    #[test]
    fn test_fn_return_string_value() {
        let input = r#"
        fn get_value(string value) -> string {
            return value;
        }
        "#;
        let output: Result<Vec<Expression>, Box<pest::error::Error<Rule>>> =
            parse_cyclo_program(input);
        let func_expr = build_basic_func_ast(
            "get_value".into(),
            [FuncArg("value".into(), Type::String)].to_vec(),
            Type::String,
            vec![Expression::ReturnStmt(Box::new(Expression::Variable(
                "value".into(),
            )))],
        );
        assert!(output.is_ok());
        // Return stmt not returning correct ast
        // Returning Binaray(Expr)
        // Instead of ReturnStmt(Binary(Expr))
        assert!(output.unwrap().contains(&func_expr))
    }

    #[test]
    fn test_fn_return_int_add() {
        let input = r#"
        fn add(int x, int y) -> int {
            return x + y;
        }
        "#;
        let output: Result<Vec<Expression>, Box<pest::error::Error<Rule>>> =
            parse_cyclo_program(input);
        let func_expr = build_basic_func_ast(
            "add".into(),
            [
                FuncArg("x".into(), Type::Int),
                FuncArg("y".into(), Type::Int),
            ]
            .to_vec(),
            Type::Int,
            vec![
                Expression::LetStmt(
                    "value".into(),
                    Box::new(Expression::Binary(
                        Box::new(Variable("x".into())),
                        "+".into(),
                        Box::new(Variable("y".into())),
                    )),
                ),
                Expression::ReturnStmt(Box::new(Expression::Variable("value".into()))),
            ],
        );
        println!("{:?}", output);
        assert!(output.is_ok());
        assert!(output.unwrap().contains(&func_expr))
    }

    // func to validate that the return statement AST is correct
    fn build_basic_func_ast(
        name: String,
        args: Vec<Expression>,
        return_type: Type,
        block_stmt: Vec<Expression>,
    ) -> Expression {
        let body = Expression::BlockStmt(block_stmt);
        Expression::new_func_stmt(name, args, return_type, body)
    }

    #[test]
    fn test_fn_return_string() {
        let input = r#"
        fn hello_world() -> string {
            return "hello world";
        }
        let val = hello_world();
        "#;
        let output: Result<Vec<Expression>, Box<pest::error::Error<Rule>>> =
            parse_cyclo_program(input);
        let func_expr = build_basic_func_ast(
            "hello_world".into(),
            [].to_vec(),
            Type::String,
            vec![Expression::ReturnStmt(Box::new(Expression::String(
                "\"hello world\"".into(),
            )))],
        );
        assert!(output.is_ok());
        assert!(output.unwrap().contains(&func_expr))
    }

    #[test]
    fn test_fn_return_bool() {
        let input = r#"
        fn hello_bool() -> bool {
            return true;
        }
        let val = hello_world();
        "#;
        let output: Result<Vec<Expression>, Box<pest::error::Error<Rule>>> =
            parse_cyclo_program(input);
        let func_expr = build_basic_func_ast(
            "hello_bool".into(),
            [].to_vec(),
            Type::Bool,
            vec![
                Expression::ReturnStmt(Box::new(Expression::Bool(true))),
            ],
        );
        assert!(output.is_ok());
        println!("{:?}", output);
        assert!(output.unwrap().contains(&func_expr))
    }

    #[test]
    fn test_fibonacci_fn() {
        let input = r#"
        fn fib(int val, int two) -> int {
            if (val < 2) {
                return 0;
            }
            return fib(val-1) + fib(val-2);
        }
        fib(20);
        "#;
        let output: Result<Vec<Expression>, Box<pest::error::Error<Rule>>> =
        parse_cyclo_program(input);
        // TODO: fix this so call stmts return correctly
        println!("{:?}", output);
        assert!(output.is_ok());
    }

    #[test]
    fn test_call_func() {
        let input = r#"
        fn hello() {
            print("hello");
        }
        hello();
        "#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_if_stmt() {
        let input = r#"
        if (value)
        {
            print("hello");
        }
        "#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_if_stmt_expression_value_comp() {
        let input = r#"
        if (value == other_value)
        {
            print("hello");
        }
        "#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_if_stmt_expression() {
        let input = r#"
        if (1 == 1)
        {
            print("hello");
        }
        "#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_if_else_stmt() {
        let input = r#"
        if (value)
        {
            print("hello");
        } 
        else {
            print("else");
        }
        "#;
        assert!(parse_cyclo_program(input).is_ok());
    }
    #[test]
    fn test_while_stmt() {
        let input = r#"
        while (value)
        {
            print("hello");
            let i = 1;
        }
        "#;
        assert!(parse_cyclo_program(input).is_ok());
    }
    #[test]
    fn test_for_loop_stmt() {
        let input = r#"
        for (let i = 0; i < 20; i++)
        {
            print(i);
        }
        "#;
        assert!(parse_cyclo_program(input).is_ok());
    }

    #[test]
    fn test_for_loop_stmt_reverse() {
        let input = r#"
        for (let i = 40; i < 10; i--)
        {
            print(i);
        }
        "#;
        assert!(parse_cyclo_program(input).is_ok());
    }
}
