#[macro_use]
extern crate pest_derive;
use std::fs;
use clap::Parser;
mod compiler;
mod parser;
mod repl;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    file: Option<String>,
}

fn main() {
    let args = Args::parse();
    if let Some(filename) = args.file {
        let contents = fs::read_to_string(filename)
        .expect("Failed to read file");
        match parser::parse_asharp_program(&contents) {
            // loop through expression, if type var then store
            Ok(exprs) => match compiler::compile(exprs) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                }
            },
            Err(e) => println!("Error: {}", e),
        }
    } else {
        repl::run();
    }
}
