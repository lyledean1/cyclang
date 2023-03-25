#[macro_use]
extern crate pest_derive;

mod compiler;
mod parser;

fn main() {
    let input = "
    let one = true;
    let two = false;
    let three = (two == one);
    print(three);
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
