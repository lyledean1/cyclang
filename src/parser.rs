extern crate pest;

use pest::Parser;
use std::num::ParseIntError;

#[derive(Parser)]
#[grammar = "asharp.pest"]
struct GptQLParser;

#[derive(Debug)]
pub enum Expression {
    Number(i32),
    String(String),
    Bool(bool),
    Nil,
    Variable(String),
    Binary(Box<Expression>, char, Box<Expression>),
    Grouping(Box<Expression>),
    LetStmt(String, Box<Expression>),
    Print(Box<Expression>),
}

impl Expression {
    fn new_number(n: i32) -> Self {
        Self::Number(n)
    }

    fn new_string(s: String) -> Self {
        Self::String(s)
    }

    fn new_binary(left: Expression, op: char, right: Expression) -> Self {
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
        Rule::binary => {
            let mut inner_pairs = pair.into_inner();
            let left = parse_expression(inner_pairs.next().unwrap())?;
            let op = inner_pairs.next().unwrap().as_str().chars().next().unwrap();
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
            let op = inner_pairs.next().unwrap().as_str().chars().next().unwrap();
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
        _ => Err(pest::error::Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: format!("Invalid expression for rule {:?}", pair.as_rule()),
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
            Rule::semicolon | Rule::EOI => {
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

pub fn parse_gptql_program(input: &str) -> Result<Vec<Expression>, pest::error::Error<Rule>> {
    match GptQLParser::parse(Rule::expression_list, input) {
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
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_string_expression_err() {
        let input = r#"hello";"#;
        assert!(parse_gptql_program(input).is_err());
    }

    #[test]
    fn test_parse_number_expression() {
        let input = r#"555;"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_expression_add() {
        let input = r#"555 + 555 + 555;"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_expression_add_grouping() {
        let input = r#"(555 + 555) + (555 + 555);"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_negative_number_expression() {
        let input = r#"-555 - 555;"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_expression_err() {
        let input = r#"555""#;
        assert!(parse_gptql_program(input).is_err());
    }

    #[test]
    fn test_parse_nil() {
        let input = r#"nil;"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_true_bool() {
        let input = r#"true;"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_false_bool() {
        let input = r#"false;"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_string_equals() {
        let input = r#""hello" == "hello";"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_bool_equals() {
        let input = r#"true == true;"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_nil_equals() {
        let input = r#"nil == nil;"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_equals() {
        let input = r#"55 == 45;"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_bool_equals_string() {
        let input = r#"true == "hello";"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_string() {
        let input = r#"let value = "hello";"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_bool() {
        let input = r#"let value = true;"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_bool_without_comma() {
        let input = r#"let value = true"#;
        assert!(parse_gptql_program(input).is_err());
    }

    #[test]
    fn test_parse_let_stmt_number() {
        let input = r#"let value = 555;"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_nil() {
        let input = r#"let value = nil;"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_grouping() {
        let input = r#"let value = (true == true);"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_assign() {
        let input = r#"let value = other_value;"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_bool_assign() {
        let input = r#"let value = (other_value == first_value);"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_print_stmt_bool_assign() {
        let input = r#"print(other_value == first_value);"#;
        assert!(parse_gptql_program(input).is_ok());
    }

    #[test]
    fn test_parse_multi_line_stmt() {
        let input = "
        let one = true;
        let two = false;
        let three = (two == one);
        ";
        assert!(parse_gptql_program(input).is_ok());
    }
}
