extern crate pest;

#[macro_use]
extern crate pest_derive;

use pest::Parser;
use std::num::ParseIntError;

#[derive(Parser)]
#[grammar = "gptql.pest"]
struct GptQLParser;

#[derive(Debug)]
enum Expression {
    Number(i32),
    String(String),
    Bool(bool),
    Variable(String),
    Binary(Box<Expression>, char, Box<Expression>),
    LetStmt(String, Box<Expression>, Box<Expression>),
}

fn make_error_message(info: String, message: String) -> String {
    format!("{} {}", info, message)
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

    fn new_variable(name: String) -> Self {
        Self::Variable(name)
    }

    fn new_let_stmt(name: String, value: Expression, expr: Expression) -> Self {
        Self::LetStmt(name, Box::new(value), Box::new(expr))
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
        Rule::let_stmt => {
            let mut inner_pairs = pair.into_inner();
            let name = inner_pairs.next().unwrap().as_str().to_string();
            let value = parse_expression(inner_pairs.next().unwrap())?;
            let expr = parse_expression(inner_pairs.next().unwrap())?;
            Ok(Expression::new_let_stmt(name, value, expr))
        }
        Rule::expression => {
            let mut inner_pairs = pair.into_inner();
            let left = parse_expression(inner_pairs.next().unwrap())?;
            let op = inner_pairs.next().unwrap().as_str().chars().next().unwrap();
            let right = parse_expression(inner_pairs.next().unwrap())?;
            // Precedence handling
            match op {
                '+' | '-' | '*' | '/' | '^' => Ok(Expression::new_binary(left, op, right)),
                '='  => { // add "=="
                    if let Expression::Binary(lhs, _, _) = &left {
                        unreachable!("invalid operator")
                    } else {
                        Ok(Expression::new_binary(left, op, right))
                    }
                }
                _ => {
                    unreachable!("invalid operator")
                },
            }
        }
        Rule::literal => {
            let inner_pair = pair.into_inner().next().unwrap();
            parse_expression(inner_pair)
        }
        _ => Err(pest::error::Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: "Invalid expression".to_string(),
            },
            pair.as_span(),
        )),
    }
}

fn parse_program(pair: pest::iterators::Pair<Rule>) -> Result<(), pest::error::Error<Rule>> {
    for stmt_pair in pair.into_inner() {
        println!("Parsed statement: {}", stmt_pair.as_str());
        match stmt_pair.as_rule() {
            Rule::let_stmt => {
                parse_expression(stmt_pair)?;
            }
            Rule::expression => {
                parse_expression(stmt_pair)?;
            }
            Rule::literal => {
                parse_expression(stmt_pair)?;
            }
            Rule::semicolon => {
                continue;
            }
            _ => {
                return Err(pest::error::Error::new_from_span(
                    pest::error::ErrorVariant::CustomError {
                        message: make_error_message(
                            "Invalid statement".to_string(),
                            stmt_pair.to_string(),
                        ),
                    },
                    stmt_pair.as_span(),
                ));
            }
        }
    }
    Ok(())
}

fn parse_function_program(input: &str) -> Result<(), String> {
    let pairs = match GptQLParser::parse(Rule::expression_list, input) {
        Ok(pairs) => pairs,
        Err(e) => {
            return Err(format!(
                "Failed to decode pairs: {}",
                e.to_string()
            ))
        }
    };

    if let Some(pair) = pairs.into_iter().next() {
        match parse_program(pair) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!(
                "Failed to parse program: {}",
                e.to_string()
            )),
        }
    } else {
        Err("No pairs found in input".to_string())
    }
}

fn main() {
    let input = r#""hello""#;
    match parse_function_program(input) {
        Ok(()) => println!("Parsed successfully!"),
        Err(e) => println!("Error: {}", e),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_string_expression() {
        let input = r#""hello";"#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_string_expression_err() {
        let input = r#"hello";"#;
        assert!(parse_function_program(input).is_err());
    }

    #[test]
    fn test_parse_number_expression() {
        let input = r#"555;"#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_negative_number_expression() {
        let input = r#"-555;"#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_expression_err() {
        let input = r#"555""#;
        assert!(parse_function_program(input).is_err());
    }

    #[test]
    fn test_parse_true_bool() {
        let input = r#"true;"#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_false_bool() {
        let input = r#"false;"#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_string_equals() {
        let input = r#""hello" == "hello";"#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_bool_equals() {
        let input = r#"true == true;"#;
        match parse_function_program(input) {
            Ok(()) => (),
            Err(e) => println!("Error: {}", e),
        }
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_equals() {
        let input = r#"55 == 45;"#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_bool_equals_string() {
        let input = r#"true == "hello";"#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_stmt_only() {
        let input = r#"let value = "hello";"#;
        match parse_function_program(input) {
            Ok(()) => (),
            Err(e) => println!("Error: {}", e),
        }
        assert!(parse_function_program(input).is_ok());
    }
}
