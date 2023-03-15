extern crate pest;

#[macro_use]
extern crate pest_derive;
use pest::Parser;

#[derive(Parser)]
#[grammar = "gptql.pest"]
struct GptQLParser;

#[derive(Debug)]
enum Expression {
    Number(i32),
    String(String),
    Bool(bool),
    Binary(Box<Expression>, char, Box<Expression>),
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
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> Expression {
    match pair.as_rule() {
        Rule::number => Expression::new_number(pair.as_str().parse().unwrap()),
        Rule::string => Expression::new_string(pair.as_str().to_string()),
        Rule::bool => {
            match pair.as_str() {
                "true" => Expression::new_bool(true),
                "false" => Expression::new_bool(false),
                _ => {
                    unreachable!()
                }
                
            }
        }
        Rule::expression => {
            let mut inner_pairs = pair.into_inner();
            let left = parse_expression(inner_pairs.next().unwrap());
            let op = inner_pairs.next().unwrap().as_str().chars().next().unwrap();
            let right = parse_expression(inner_pairs.next().unwrap());
            Expression::new_binary(left, op, right)
        },
        Rule::digits => Expression::new_number(pair.as_str().parse().unwrap()),
        _ => {
            unreachable!("{}", pair)
        }
    }
}

fn parse_program(pair: pest::iterators::Pair<Rule>) {
    for expr_pair in pair.into_inner() {
        parse_expression(expr_pair);
    }
}


fn parse_function_program(input: &str) -> Result<(), pest::error::Error<Rule>> {
    let pairs = GptQLParser::parse(Rule::expression, input);
    match pairs {
        Ok(pairs) => {
            if let Some(pair) = pairs.into_iter().next() {
                parse_program(pair);
            }
        },
        Err(e) => {
            println!("{:?}", e);
            return Err(e)
        }
        
    }
    Ok(())
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
        let input = r#""hello""#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_expression() {
        let input = r#"555"#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_true_bool() {
        let input = r#"true"#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_false_bool() {
        let input = r#"false"#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_nil() {
        let input = r#"nil"#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_string_equals() {
        let input = r#""hello" == "hello""#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_bool_equals() {
        let input = r#"true == true"#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_number_equals() {
        let input = r#"55 == 45"#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_bool_equals_string() {
        let input = r#"true == "hello""#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_operator_error() {
        let input = r#"*"#;
        assert!(parse_function_program(input).is_err());
    }
}
