#[macro_use]
extern crate pest_derive;

use clap::Parser;
use std::fmt;
use std::process::exit;
use std::{fs, process::Output};
mod parser;
mod repl;

#[macro_use]
mod compiler;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    file: Option<String>,
}

#[derive(Debug)]
struct ParserError {
    message: String,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MyError: {}", self.message)
    }
}

fn compile_output_from_string(contents: String) -> Output {
    match parser::parse_cyclo_program(&contents) {
        // loop through expression, if type var then store
        Ok(exprs) => match compiler::compile(exprs) {
            Ok(output) => output,
            Err(e) => {
                eprintln!("unable to compile contents due to error: {}", e);
                exit(1)
            }
        },
        Err(e) => {
            eprintln!("unable to parse contents due to error: {}", e);
            exit(1)
        }
    }
}

fn main() {
    let args = Args::parse();
    if let Some(filename) = args.file {
        let contents = fs::read_to_string(filename).expect("Failed to read file");
        compile_output_from_string(contents);
    } else {
        repl::run();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    //Note: Integration tests for parsing and compiling output
    #[test]
    fn test_compile_print_number_expression() {
        let input = r#"print(12);"#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "12\n");
    }

    #[test]
    fn test_compile_print_string_expression() {
        let input = r#"print("example blah blah blah");"#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "\"example blah blah blah\"\n");
    }

    #[test]
    fn test_compile_print_add_string_expression() {
        let input = r#"print("hello" + " world");"#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "hello world\n");
    }

    #[test]
    fn test_compile_print_bool_expression() {
        let input = r#"print(true);"#;
        // call print statement for str
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "true\n");
    }

    #[test]
    fn test_compile_variable_bool() {
        let input = r#"
        let variable = true;
        print(variable);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "true\n");
    }

    #[test]
    fn test_compile_variable_number() {
        let input = r#"
        let variable = 2;
        print(variable);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "2\n");
    }

    #[test]
    fn test_compile_variable_number_and_add() {
        let input = r#"
        let number = 2;
        number = number + 1;
        print(number);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "3\n");
    }

    #[test]
    fn test_compile_variable_string() {
        let input = r#"
        let variable = "hello";
        print(variable);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "\"hello\"\n");
    }

    #[test]
    fn test_compile_grouping() {
        let input = r#"
        let value = (1 == 1);
        print(value);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "true\n");
    }

    #[test]
    fn test_compile_addition() {
        let input = r#"
        print(2 + 4);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "6\n");
    }

    #[test]
    fn test_compile_subtraction() {
        let input = r#"
        print(6 - 4);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "2\n");
    }

    #[test]
    fn test_compile_multiplication() {
        let input = r#"
        print(5 * 4);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "20\n");
    }

    #[test]
    fn test_compile_division() {
        let input = r#"
        print(20/4);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "5\n");
    }

    #[test]
    fn test_compile_eqeq_true_number() {
        let input = r#"
        print(4 == 4);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "true\n");
    }

    #[test]
    fn test_compile_eqeq_false_number() {
        let input = r#"
        print(4 == 5);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "false\n");
    }

    #[test]
    fn test_compile_eqeq_true_string() {
        let input = r#"
        print("4" == "4");
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "true\n");
    }

    #[test]
    fn test_compile_eqeq_false_string() {
        let input = r#"
        print("4" == "5");
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "false\n");
    }

    #[test]
    fn test_compile_eqeq_bool_false() {
        let input = r#"
        print(true == false);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "false\n");
    }

    #[test]
    fn test_compile_eqeq_bool_true() {
        let input = r#"
        print(true == true);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "true\n");
    }

    #[test]
    fn test_compile_ne_bool_false() {
        let input = r#"
        print(true != true);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "false\n");
    }

    #[test]
    fn test_compile_ne_bool_true() {
        let input = r#"
        print(true != false);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "true\n");
    }

    #[test]
    fn test_if_stmt_with_let_stmt() {
        let input = r#"
        let is_value = true;
        if (is_value)
        {
            print("hello");
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "\"hello\"\n");
    }

    #[test]
    fn test_if_stmt_with_eqeq_stmt_number() {
        let input = r#"
        if (1 == 1)
        {
            print("hello");
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "\"hello\"\n");
    }

    #[test]
    fn test_if_stmt_with_ne_stmt_bool() {
        let input = r#"
        if (1 != 1)
        {
            print("not hello");
        } else {
            print("hello");
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "\"hello\"\n");
    }

    #[test]
    fn test_if_else_stmt() {
        let input = r#"
        let value = false;
        if (value)
        {
            print("not hello");
        } else {
            print("hello");
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "\"hello\"\n");
    }

    #[test]
    fn test_nested_if_stmts() {
        let input = r#"
        if (true) {
            if (true) {
                print("yep");
            } else {
                print("nope");
            }
        } else {
            print("don't print this");
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "\"yep\"\n");
    }

    #[test]
    fn test_nested_if_stmts_with_print_after() {
        let input = r#"
        if (true) {
            if (true) {
                print("yep");
            } else {
                print("nope");
            }
            print("yep");
        } else {
            print("don't print this");
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "\"yep\"\n\"yep\"\n");
    }

    #[test]
    fn test_nested_if_stmts_deeper() {
        let input = r#"
        if (true) {
            if (true) {
                print(1);
                if (false) {
                    print("error");
                } else {
                    print(2);
                    if (true) {
                        print(3);
                    } else {
                        print("nothing");
                    }
                }
            }
            print(4);
        } else {
            print("don't print this");
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "1\n2\n3\n4\n");
    }

    #[test]
    fn test_nested_if_stmts_with_top_level_var() {
        let input = r#"
        let var = 3;
        if (true) {
            if (true) {
                print(1);
                if (false) {
                    print("error");
                } else {
                    print(2);
                    if (true) {
                        print(var);
                        var = var + 1;
                        print(var);
                        var = var + 1;
                    } else {
                        print("nope");
                    }
                }
            }
        } else {
            print("don't print this");
        }
        print(var);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "1\n2\n3\n4\n5\n");
    }

    #[test]
    fn test_compile_while_stmt_one_pass() {
        let input = r#"
        let value = true;
        while(value) {
            value = false;
            print(value);
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "false\n");
    }

    #[test]
    fn test_compile_while_stmt_with_if_true() {
        let input = r#"
        let value = true;
        while(value) {
            if (value == true) {
                print(value);
            }
            value = false;
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "true\n");
    }

    #[test]
    fn test_compile_while_stmt_one_pass_grouping_string() {
        let input = r#"
        let value = true;
        while(value) {
            value = false;
            print("here");
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(stderr, "");
        assert_eq!(stdout, "\"here\"\n");
    }

    #[test]
    fn test_compile_while_stmt_one_pass_grouping() {
        let input = r#"
        let value = true;
        while(value) {
            print(value);
            value = false;
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "true\n");
    }

    #[test]
    fn test_compile_while_stmt_false() {
        let input = r#"
        let value = false;
        while(value) {
            print(value);
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "");
    }

    #[test]
    fn test_compile_while_stmt_with_if() {
        let input = r#"
        let value = true;
        let number = 0;
        while(value) {
            let other_value = true;
            let value = false;
            print(number + 1);
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "1\n");
    }

    #[test]
    fn test_compile_for_loop() {
        let input = r#"
        for (let i = 0; i < 10; i++)
        {  
            print(i);
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n");
    }

    #[test]
    fn test_compile_block_stmt_bool() {
        let input = r#"
        let is_true = false;
        {
            is_true = true;
        }
        print(is_true);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "true\n");
    }

    #[test]
    fn test_compile_function_stmt_no_args() {
        let input = r#"
        fn hello_world() {
            print("hello world");
        }
        hello_world();
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "\"hello world\"\n");
    }

    #[test]
    fn test_compile_function_stmt_no_args_with_if() {
        let input = r#"
        fn hello_world() {
            print("hello world");
        }
        fn not_executed() {
            print("not executed");
        }
        if (true) {
            hello_world();
        } else {
            not_executed();
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "\"hello world\"\n");
    }

    #[test]
    fn test_compile_function_stmt_print_if() {
        let input = r#"
        fn hello_world() {
            let value = true;
            if (value) {
                print(value);
            }
        }
        hello_world();
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "true\n");
    }

    #[test]
    fn test_compile_for_loop_with_num() {
        let input = r#"
        let val = 0;
        for (let i = 0; i < 10; i++)
        {  
            val = val + i;
            print(val);
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "0\n1\n3\n6\n10\n15\n21\n28\n36\n45\n");
    }

    #[test]
    fn test_compile_for_loop_reverse_with_num() {
        let input = r#"
        let val = 0;
        for (let i = 10; i > 0; i--)
        {
            val = val + i;
            print(val);
        }
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "10\n19\n27\n34\n40\n45\n49\n52\n54\n55\n");
    }

    #[test]
    fn test_compile_function_return_int() {
        let input = r#"
        fn get_int() -> int {
            return 5;
        }
        let val = get_int();
        print(get_int());
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "5\n");
    }

    #[test]
    fn test_compile_function_with_two_args_and_ignore_top_level_var() {
        let input = r#"
        let var = 0;
        fn add(int x, int y) {
            print(x + y);
        }
        add(10, 10);
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "20\n");
    }

    #[test]
    fn test_compile_fn_return_int_value() {
        let input = r#"
        fn add(int x, int y) -> int {
            return x + y;
        }
        print(add(5,5));
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "10\n");
    }

    #[test]
    fn test_compile_fn_return_int_value_mul() {
        let input = r#"
        fn mul(int x, int y) -> int {
            return x * y;
        }
        print(mul(5,5));
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "25\n");
    }

    #[test]
    fn test_compile_fn_return_int_value_with_call_stmts() {
        let input = r#"
        fn add(int x, int y) -> int {
            return x + y;
        }
        fn add_together() -> int {
            return add(5,10) + add(10,4);
        }
        print(add_together());
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "29\n");
    }

    #[test]
    fn test_compile_fn_return_bool_value() {
        let input = r#"
        fn compare(bool x, bool y) -> bool {
            return (x == y);
        }
        print(compare(true,false));
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "false\n");
    }

    #[test]
    fn test_compile_fn_return_bool_value_cmp_ints() {
        let input = r#"
        fn compare_ints(int x, int y) -> bool {
            return (x == y);
        }
        print(compare_ints(1000,1000));
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "true\n");
    }

    #[test]
    fn test_compile_fn_return_bool_true_value_cmp_ints_in_another_fn() {
        let input = r#"
        fn compare(int x, int y) -> bool {
            return (x == y);
        }
        fn expect_true() -> bool {
            return (compare(1,1) == compare(2,2));
        }
        print(expect_true());
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "true\n");
    }

    #[test]
    fn test_compile_fn_return_bool_false_value_cmp_ints_in_another_fn() {
        let input = r#"
        fn compare(int x, int y) -> bool {
            return (x == y);
        }
        fn expect_false() -> bool {
            return (compare(1,2) == compare(1,1));
        }
        print(expect_false());
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "false\n");
    }

    #[test]
    fn test_factorial_recursive_function() {
        let input = r#"
        fn factorial(int n) -> int {
            if (n == 0) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        print(factorial(5));
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "true\n");
    }

    #[test]
    fn test_recursive_fn() {
        let input = r#"
        fn fib(int n) -> int {
            if (n == 0) {
                return 0;
            }
            if (n == 1) {
                return 1;
            }
            return fib(n - 1) + fib(n - 2);
        }
        print(fib(20));
        "#;
        let output = compile_output_from_string(input.to_string());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "true\n");
    }
}
