use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use text_colorizer::*;
use crate::compiler;
use crate::parser;

pub fn run() {
    let version: &str = env!("CARGO_PKG_VERSION");

    println!("{} version: {}", "a#".bold(), version.italic());
    println!("");


    let mut rl = DefaultEditor::new().unwrap();
    if rl.load_history("history.txt").is_err() {
        println!("");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(input) => {
                let history_err = rl.add_history_entry(input.as_str());
                match history_err {
                    //TODO: decide how to handle
                    _ => {}
                }
                match input.trim() {
                    "exit()" => break,
                    _ => {
                        //TODO: to
                        match parser::parse_gptql_program(&input) {
                            // add each 
                            Ok(exprs) => match compiler::compile(exprs) {
                                Ok(output) => {
                                    println!("{}", String::from_utf8_lossy(&output.stdout))

                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                }
                            },
                            Err(e) => println!("Error: {}", e),
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