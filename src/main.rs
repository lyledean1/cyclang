#[macro_use]
extern crate pest_derive;

mod compiler;
mod parser;

fn main() {
    let input = "
    print(\"this is a test\");
    print(55);
    print(63);
    ";
    match parser::parse_gptql_program(input) {
        Ok(exprs) => {
            for expr in exprs {
                println!("Parsed expression successfully {:?}", expr)
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
