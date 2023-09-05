use crate::compiler;
use crate::parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use text_colorizer::*;
use crate::cyclo_error::CycloError;

pub fn run() {
    let version: &str = env!("CARGO_PKG_VERSION");

    println!("{} version: {}", "#".bold(), version.italic());
    println!();

    let mut rl = DefaultEditor::new().unwrap();
    if rl.load_history("history.txt").is_err() {
        println!();
    }
    loop {
        let line = rl.readline(">> ");
        match line {
            Ok(input) => {
                match input.trim() {
                    "exit()" => break,
                    _ => {
                       let _ = rl.add_history_entry(input.as_str());
                        match parse_and_compile(input.to_string()) {
                            Ok(output) => {
                                println!("{}", output)
                            },
                            Err(e) => {
                                eprintln!("{}", e);
                            } 
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Did you want to exit? Type exit()");
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}


fn parse_and_compile(input: String) -> Result<String, CycloError> {
    let exprs = parser::parse_cyclo_program(&input)?;
    let output = compiler::compile(exprs, true)?;
    Ok(output)
}