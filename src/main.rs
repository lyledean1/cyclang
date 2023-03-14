extern crate pest;

#[macro_use]
extern crate pest_derive;
use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct FunctionParser;

#[derive(Debug)]
enum Expression {
    Number(i32),
    String(String),
    Function(String, Vec<Expression>),
    Binary(Box<Expression>, char, Box<Expression>),
}

impl Expression {
    fn new_number(n: i32) -> Self {
        Self::Number(n)
    }

    fn new_string(s: String) -> Self {
        Self::String(s)
    }

    fn new_function(name: String, args: Vec<Expression>) -> Self {
        Self::Function(name, args)
    }

    fn new_binary(left: Expression, op: char, right: Expression) -> Self {
        Self::Binary(Box::new(left), op, Box::new(right))
    }
}

fn parse_parameter_list(pair: pest::iterators::Pair<Rule>) -> Vec<Expression> {
    let mut args = Vec::new();
    for expr_pair in pair.into_inner() {
        args.push(parse_expression(expr_pair));
    }
    args
}

fn parse_function(pair: pest::iterators::Pair<Rule>) -> Expression {
    let mut inner_pairs = pair.into_inner();
    let name = inner_pairs.next().unwrap().as_span().as_str().to_string();
    let arg_list_pair = inner_pairs.next().unwrap();
    let args = parse_parameter_list(arg_list_pair);
    let code_block_pair = inner_pairs.next().unwrap();
    let _ = parse_expression(code_block_pair);
    Expression::new_function(name, args)
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> Expression {
    match pair.as_rule() {
        Rule::number => Expression::new_number(pair.as_str().parse().unwrap()),
        Rule::string => Expression::new_string(pair.as_str().to_string()),
        Rule::function => parse_function(pair),
        Rule::expression => {
            let mut inner_pairs = pair.into_inner();
            let left = parse_expression(inner_pairs.next().unwrap());
            let op = inner_pairs.next().unwrap().as_str().chars().next().unwrap();
            let right = parse_expression(inner_pairs.next().unwrap());
            Expression::new_binary(left, op, right)
        },
        Rule::assignment => {
            let mut inner_pairs = pair.into_inner();
            inner_pairs.next(); // Skip "let"
            inner_pairs.next(); // Skip the name
            inner_pairs.next(); // Skip "="
            parse_expression(inner_pairs.next().unwrap()) // Process the assigned expression
        },
        Rule::digits => Expression::new_number(pair.as_str().parse().unwrap()),
        _ => {
            unreachable!("{}", pair)
        }
    }
}

fn parse_function_block(pair: pest::iterators::Pair<Rule>) {
    for expr_pair in pair.into_inner() {
        match expr_pair.as_rule() {
            Rule::expression | Rule::assignment => {
                let _ = parse_expression(expr_pair);
            }
            _ => unreachable!(),
        }
    }
}

fn parse_function_def(pair: pest::iterators::Pair<Rule>) {
    let mut inner_pairs = pair.into_inner();
    let name = inner_pairs.next().unwrap().as_span().as_str().to_string();
    let arg_list_pair = inner_pairs.next().unwrap();
    let args = parse_parameter_list(arg_list_pair);
    let code_block_pair = inner_pairs.next().unwrap();
    parse_function_block(code_block_pair);
    let _ = Expression::new_function(name, args);
}

fn parse_assignment(pair: pest::iterators::Pair<Rule>) {
    let mut inner_pairs = pair.into_inner();
    let _ = inner_pairs.next().unwrap().as_span().as_str().to_string();
    let _ = inner_pairs.next().unwrap().as_span().as_str().to_string();
    let expr_pair = inner_pairs.next().unwrap();
    let _ = parse_expression(expr_pair);
}


fn parse_program(pair: pest::iterators::Pair<Rule>) {
    for expr_pair in pair.into_inner() {
        match expr_pair.as_rule() {
            Rule::function => parse_function_def(expr_pair),
            Rule::stmt => {
                let stmt_pair = expr_pair.into_inner().next().unwrap();
                match stmt_pair.as_rule() {
                    Rule::assignment => {
                        let _ = parse_assignment(stmt_pair);
                    }
                    Rule::expression => {
                        let _ = parse_expression(stmt_pair);
                    }
                    _ => unreachable!("{}", stmt_pair),
                }
            }
            _ => {
                unreachable!("{}", expr_pair)
            }
        }
    }
}


fn parse_function_program(input: &str) -> Result<(), pest::error::Error<Rule>> {
    let pairs = FunctionParser::parse(Rule::program, input)?;
    if let Some(pair) = pairs.into_iter().next() {
        parse_program(pair);
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
    fn test_parse_let_expression_number() {
        let input = r#"let value = 2;"#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_let_expression_string() {
        let input = r#"let value = "string";"#;
        assert!(parse_function_program(input).is_ok());
    }

    #[test]
    fn test_parse_function_expression() {
        let input = r#"fn hello() { "hello" }"#;
        assert!(parse_function_program(input).is_ok());
    }
}
