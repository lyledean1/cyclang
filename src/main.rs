#[macro_use]
extern crate pest_derive;

mod compiler;
mod parser;

fn main() {
    let input = "
    print(\"this is a test\");
    print(\"second call\");
    print(5525);
    print(68 + 10);
    print(68 - 10);
    print(99 * 2);
    print(100 / 2);
    print(true);
    print(false);
    ";
    match parser::parse_gptql_program(input) {
        Ok(exprs) => match compiler::compile(exprs) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
            }
        },
        Err(e) => println!("Error: {}", e),
    }
}
