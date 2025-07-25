use crate::compiler::{self, CompileOptions};
use anyhow::Result;
use art_parser::{parse_cyclo_program, Expression};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use rustyline::{Cmd, EventHandler, KeyCode, KeyEvent, Modifiers};
use text_colorizer::*;

pub fn run() {
    let version: &str = env!("CARGO_PKG_VERSION");

    println!("{} {}", "art".italic(), version.italic());
    let mut rl = DefaultEditor::new().unwrap();
    rl.bind_sequence(
        KeyEvent(KeyCode::Down, Modifiers::SHIFT),
        EventHandler::Simple(Cmd::Newline),
    );

    loop {
        let line = rl.readline(">> ");
        match line {
            Ok(input) => match input.trim() {
                "exit()" => break,
                _ => match parse_and_compile(input.to_string(), &mut rl) {
                    Ok(output) => {
                        if !output.is_empty() {
                            println!("{output:?}")
                        }
                    }
                    Err(e) => {
                        println!("{}", e.to_string().red());
                    }
                },
            },
            Err(ReadlineError::Interrupted) => {
                println!("Did you want to exit? Type exit()");
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {err:?}");
                break;
            }
        }
    }
}

fn parse_and_compile(input: String, rl: &mut DefaultEditor) -> Result<String> {
    let joined_history = rl
        .history()
        .iter()
        .map(|s| &**s) // Convert &String to &str
        .collect::<Vec<&str>>()
        .join("\n");

    let final_string = format!("{joined_history}{input}");
    let exprs = parse_cyclo_program(&final_string)?;
    let compile_options = Some(CompileOptions {
        is_execution_engine: true,
        target: None,
    });
    let output = compiler::compile(exprs.clone(), compile_options)?;

    for expr in parse_cyclo_program(&input)? {
        if let Expression::LetStmt(_, _, _) | Expression::FuncStmt(_, _, _, _) = expr {
            let _ = rl.add_history_entry(input.as_str());
        }
    }
    Ok(output)
}
