extern crate pest;

use pest::Parser;
use std::num::ParseIntError;

#[derive(Parser)]
#[grammar = "../grammar/asharp.pest"]
struct ASharpParser;

#[derive(Debug, Clone)]
pub enum Expression {
    Number(i32),
    String(String),
    Bool(bool),
    Nil,
    Variable(String),
    List(Box<Vec<Expression>>),
    Binary(Box<Expression>, String, Box<Expression>),
    Grouping(Box<Expression>),
    LetStmt(String, Box<Expression>),
    BlockStmt(Box<Vec<Expression>>),
    FuncStmt(String, Vec<String>, Box<Expression>),
    CallStmt(String, Vec<String>),
    IfStmt(Box<Expression>, Box<Expression>, Box<Option<Expression>>),
    WhileStmt(Box<Expression>, Box<Expression>),
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

    fn new_list(exprs: Vec<Expression>) -> Self {
        Self::List(Box::new(exprs))
    }

    fn new_let_stmt(name: String, value: Expression) -> Self {
        Self::LetStmt(name, Box::new(value))
    }

    fn new_block_stmt(exprs: Vec<Expression>) -> Self {
        Self::BlockStmt(Box::new(exprs))
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

    fn new_func_stmt(name: String, args: Vec<String>, body: Expression) -> Self {
        Self::FuncStmt(name, args, Box::new(body))
    }

    fn new_call_stmt(name: String, args: Vec<String>) -> Self {
        Self::CallStmt(name, args)
    }

    fn new_print_stmt(value: Expression) -> Self {
        Self::Print(Box::new(value))
    }
}

fn parse_expression(
    pair: pest::iterators::Pair<Rule>,
) -> Result<Expression, pest::error::Error<Rule>> {
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
            let s = pair.as_str().to_string();
            Ok(Expression::new_variable(s))
        }
        Rule::string => {
            let s = pair.as_str().to_string();
            Ok(Expression::new_string(s))
        }
        Rule::bool => match pair.as_str() {
            "true" => Ok(Expression::new_bool(true)),
            "false" => Ok(Expression::new_bool(false)),
            _ => Err(pest::error::Error::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: "Invalid boolean value".to_string(),
                },
                pair.as_span(),
            )),
        },
        Rule::nil => Ok(Expression::new_nil()),
        Rule::list => {
            let inner_pairs = pair.into_inner();
            let mut exprs = vec![];
            for inner_pair in inner_pairs {
                if inner_pair.as_rule() == Rule::comma {
                    continue;
                }
                exprs.push(parse_expression(inner_pair)?);
            }
            Ok(Expression::new_list(exprs))
        }
        Rule::binary => {
            let mut inner_pairs = pair.into_inner();
            let left = parse_expression(inner_pairs.next().unwrap())?;
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
            let mut args = vec![];
            while inner_pairs
                .peek()
                .map_or(false, |p| p.as_rule() == Rule::comma)
            {
                inner_pairs.next(); // skip the comma
                let arg_name = inner_pairs.next().unwrap().as_str().to_string();
                args.push(arg_name);
            }
            let body = parse_expression(inner_pairs.next().unwrap())?;
            Ok(Expression::new_func_stmt(name, args, body))
        }
        Rule::call_stmt => {
            let mut inner_pairs = pair.into_inner();
            let name = inner_pairs.next().unwrap().as_str().to_string();
            let mut args = vec![];
            while inner_pairs
                .peek()
                .map_or(false, |p| p.as_rule() == Rule::comma)
            {
                inner_pairs.next(); // skip the comma
                let arg_name = inner_pairs.next().unwrap().as_str().to_string();
                args.push(arg_name);
            }
            Ok(Expression::new_call_stmt(name, args))
        }
        Rule::block_stmt => {
            let inner_pairs = pair.into_inner();
            let mut expressions = Vec::new();

            for inner_pair in inner_pairs {
                if inner_pair.as_rule() == Rule::semicolon {
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
            let var_name = var.next().unwrap().as_str().to_string().replace(" ", "");
            let start = var.next().unwrap().as_str().parse::<i32>().unwrap();

            //TODO: Identify > and < signs
            let mut cond_stmt = inner_pairs.next().unwrap().into_inner();
            let _cond_var_name = cond_stmt
                .next()
                .unwrap()
                .as_str()
                .to_string()
                .replace(" ", "");
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
        Rule::while_stmt => {
            let mut inner_pairs = pair.into_inner();
            let cond = parse_expression(inner_pairs.next().unwrap())?;
            let while_block_expr = parse_expression(inner_pairs.next().unwrap())?;
            Ok(Expression::new_while_stmt(cond, while_block_expr))
        }
        _ => Err(pest::error::Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: format!("Invalid expression for rule {:?} or rule not specified for grammar", pair.as_rule()),
            },
            pair.as_span(),
        )),
    }
}

fn parse_program(
    pair: pest::iterators::Pair<Rule>,
) -> Result<Vec<Expression>, pest::error::Error<Rule>> {
    let mut expr_vec = vec![];
    for stmt_pair in pair.into_inner() {
        match stmt_pair.as_rule() {
            Rule::semicolon | Rule::EOI | Rule::comma | Rule::comment => {
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

pub fn parse_asharp_program(input: &str) -> Result<Vec<Expression>, pest::error::Error<Rule>> {
    match ASharpParser::parse(Rule::expression_list, input) {
        Ok(pairs) => {
            for pair in pairs {
                match parse_program(pair) {
                    Ok(pair) => {
                        return Ok(pair);
                    }
                    Err(e) => return Err(e),
                }
            }
        }
        Err(e) => return Err(e),
    };
    unreachable!("parse function program")
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_string_expression() {
        let input = r#""hello";"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_string_expression_err() {
        let input = r#"hello";"#;
        assert!(parse_asharp_program(input).is_err());
    }

    #[test]
    fn test_parse_digit() {
        let input = r#"5;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_digit_err() {
        let input = r#"5"#;
        assert!(parse_asharp_program(input).is_err());
    }

    #[test]
    fn test_parse_number_expression() {
        let input = r#"555;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_expression_add() {
        let input = r#"555 + 555 + 555;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_expression_add_grouping() {
        let input = r#"(555 + 555) + (555 + 555);"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_minus_negative_number_expression() {
        let input = r#"-555 - 555;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_minus_negative_two_number_expression() {
        let input = r#"-555 - -555;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_negative_digit_expression() {
        let input = r#"-5 - 5;"#;
        match parse_asharp_program(input) {
            Err(e) => {
                eprintln!("{}", e);
            }
            _ => {}
        }
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_expression_err() {
        let input = r#"555""#;
        assert!(parse_asharp_program(input).is_err());
    }

    #[test]
    fn test_parse_nil() {
        let input = r#"nil;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_true_bool() {
        let input = r#"true;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_false_bool() {
        let input = r#"false;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_string_equals() {
        let input = r#""hello" == "hello";"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_bool_equals() {
        let input = r#"true == true;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_nil_equals() {
        let input = r#"nil == nil;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_char_equals() {
        let input = r#""h" == "h";"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_str_equals() {
        let input = r#""hello" == "hello";"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_equals_digit() {
        let input = r#"5 == 5;"#;
        match parse_asharp_program(input) {
            Err(e) => {
                eprintln!("{}", e);
            }
            _ => {}
        }
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_equals_digit_rhs() {
        let input = r#"55 == 5;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_not_equals_digit() {
        let input = r#"5 != 5;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_lt_digit() {
        let input = r#"5 < 5;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_lte_digit() {
        let input = r#"5 <= 5;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_gt_digit() {
        let input = r#"5 > 5;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_gte_digit() {
        let input = r#"5 >= 5;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_equals() {
        let input = r#"55 == 45;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_less_than() {
        let input = r#"55 < 45;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_less_than_equal() {
        let input = r#"55 <= 45;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_more_than() {
        let input = r#"55 > 45;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_more_than_equal() {
        let input = r#"55 >= 45;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_not_equal() {
        let input = r#"55 != 45;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_list() {
        let input = r#"[1, true, false, "three", nil];"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    // TODO: Add Map Type
    // #[test]
    // fn test_parse_map() {
    //     let input = r#"Map(1 -> 2, "three" -> 4, "five" -> 6);"#;
    //     match parse_asharp_program(input) {
    //         Err(e) => {
    //             eprintln!("{}", e);
    //         }
    //         _ => {

    //         }
    //     }
    //     assert!(parse_asharp_program(input).is_ok());
    // }

    #[test]
    fn test_parse_bool_equals_string() {
        let input = r#"true == "hello";"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_string() {
        let input = r#"let value = "hello";"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_bool() {
        let input = r#"let value = true;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_bool_without_comma() {
        let input = r#"let value = true"#;
        assert!(parse_asharp_program(input).is_err());
    }

    #[test]
    fn test_parse_let_stmt_digit() {
        let input = r#"let value = 5;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_number() {
        let input = r#"let value = 555;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_nil() {
        let input = r#"let value = nil;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_grouping() {
        let input = r#"let value = (true == true);"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_assign() {
        let input = r#"let value = other_value;"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_bool_assign() {
        let input = r#"let value = (other_value == first_value);"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_print_stmt_bool_assign() {
        let input = r#"print(other_value == first_value);"#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_parse_multi_line_stmt() {
        let input = "
        let one = true;
        let two = false;
        let three = (two == one);
        ";
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_block_stmt() {
        let input = "
        {
            let b = 5;
            {
                {
                    fn example(arg1, arg2) {
                        print(arg1 + arg2);
                    }
                    example(5,5);
                }
                a = 5;
            }
        }
        ";
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_func() {
        let input = r#"
        fn example(arg1, arg2) {
            print(1);
        }
        fn example_two(arg1, arg2) {
            let a = 5;
            print(a);
        }
        fn hello() {
            print("hello");
        }
        "#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_call_func() {
        let input = r#"
        fn hello() {
            print("hello");
        }
        hello();
        "#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_if_stmt() {
        let input = r#"
        if (value)
        {
            print("hello");
        }
        "#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_if_stmt_expression_value_comp() {
        let input = r#"
        if (value == other_value)
        {
            print("hello");
        }
        "#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_if_stmt_expression() {
        let input = r#"
        if (1 == 1)
        {
            print("hello");
        }
        "#;
        match parse_asharp_program(input) {
            Err(e) => {
                eprintln!("{}", e);
            }
            _ => {}
        }
        assert!(parse_asharp_program(input).is_ok());
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
        assert!(parse_asharp_program(input).is_ok());
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
        assert!(parse_asharp_program(input).is_ok());
    }
    #[test]
    fn test_for_loop_stmt() {
        let input = r#"
        for (let i = 0; i < 20; i++)
        {
            print(i);
        }
        "#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_for_loop_stmt_reverse() {
        let input = r#"
        for (let i = 40; i < 10; i--)
        {
            print(i);
        }
        "#;
        assert!(parse_asharp_program(input).is_ok());
    }

    #[test]
    fn test_for_loop_stmt_reverse_with_comment() {
        let input = r#"
        /* this is a comment */
        for (let i = 40; i < 10; i--)
        {
            print(i);
        }
        "#;
        assert!(parse_asharp_program(input).is_ok());
    }
}
