#[macro_use]
extern crate pest_derive;

mod compiler;
mod parser;

fn main() {
    let input = "
    let five = 5;
    let four = 4;
    print(20 / four);
    print(20 + four);
    print(20 - four);
    print(five * four);
    let is_true = true;
    print(is_true);
    let example_string = \"hello\";
    print(example_string + \" world\");
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
