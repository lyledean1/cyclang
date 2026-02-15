use anyhow::{anyhow, Result};
use backend::compiler::CompileOptions;
use backend::compiler;
use parser::{parse_cyclo_program, Expression};
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Helper};
use rustyline::history::DefaultHistory;
use rustyline::Editor;
use rustyline::{Cmd, EventHandler, KeyCode, KeyEvent, Modifiers};
use std::borrow::Cow;
use std::fs;
use text_colorizer::*;

struct CyclangHelper;

impl CyclangHelper {
    fn new() -> Self {
        Self
    }
}

impl Helper for CyclangHelper {}

impl Completer for CyclangHelper {
    type Candidate = String;

    fn complete(
        &self,
        _line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        Ok((pos, Vec::new()))
    }
}

impl Hinter for CyclangHelper {
    type Hint = String;

    fn hint(
        &self,
        _line: &str,
        _pos: usize,
        _ctx: &Context<'_>,
    ) -> Option<Self::Hint> {
        None
    }
}

impl Validator for CyclangHelper {
    fn validate(
        &self,
        _ctx: &mut ValidationContext<'_>,
    ) -> Result<ValidationResult, ReadlineError> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Highlighter for CyclangHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Owned(highlight_line(line))
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Cow::Owned(format!("\x1b[38;5;214m{}\x1b[0m", prompt))
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        false
    }
}

pub fn run() {
    let version: &str = env!("CARGO_PKG_VERSION");

    println!("{} {}", "cyclang".italic(), version.italic());
    let mut rl = Editor::<CyclangHelper, DefaultHistory>::new().unwrap();
    rl.set_helper(Some(CyclangHelper::new()));
    rl.bind_sequence(
        KeyEvent(KeyCode::Down, Modifiers::SHIFT),
        EventHandler::Simple(Cmd::Newline),
    );

    loop {
        let line = rl.readline(">> ");
        match line {
            Ok(input) => match input.trim() {
                "exit()" => break,
                "" => continue,
                cmd if cmd.starts_with(":load ") => match load_file(cmd) {
                    Ok(source) => {
                        let _ = rl.add_history_entry(source.as_str());
                        match parse_and_compile(source, &mut rl) {
                        Ok(output) => {
                            if !output.is_empty() {
                                print!("{output}");
                            }
                        }
                        Err(e) => {
                            println!("{}", e.to_string().red());
                        }
                    }}
                    Err(e) => {
                        println!("{}", e.to_string().red());
                    }
                },
                cmd if cmd.starts_with(":print ") => match wrap_print(cmd) {
                    Ok(source) => match parse_and_compile(source, &mut rl) {
                        Ok(output) => {
                            if !output.is_empty() {
                                print!("{output}");
                            }
                        }
                        Err(e) => {
                            println!("{}", e.to_string().red());
                        }
                    },
                    Err(e) => {
                        println!("{}", e.to_string().red());
                    }
                },
                _ => match parse_and_compile(input.to_string(), &mut rl) {
                    Ok(output) => {
                        if !output.is_empty() {
                            print!("{output}");
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

fn load_file(cmd: &str) -> Result<String> {
    let path = cmd.trim_start_matches(":load").trim();
    if path.is_empty() {
        return Err(anyhow!("Usage: :load <path>"));
    }
    Ok(fs::read_to_string(path)?)
}

fn wrap_print(cmd: &str) -> Result<String> {
    let expr = cmd.trim_start_matches(":print").trim();
    if expr.is_empty() {
        return Err(anyhow!("Usage: :print <expression>"));
    }
    let expr = expr.trim_end_matches(';').trim_end();
    Ok(format!("print({});", expr))
}

fn highlight_line(line: &str) -> String {
    const RESET: &str = "\x1b[0m";
    const KW: &str = "\x1b[38;5;81m";
    const TY: &str = "\x1b[38;5;75m";
    const STR: &str = "\x1b[38;5;114m";
    const NUM: &str = "\x1b[38;5;221m";
    const COM: &str = "\x1b[38;5;242m";

    let bytes = line.as_bytes();
    let mut out = String::with_capacity(line.len() + 16);
    let mut i = 0;

    while i < bytes.len() {
        let c = bytes[i] as char;

        if c == '/' && i + 1 < bytes.len() && bytes[i + 1] as char == '/' {
            out.push_str(COM);
            out.push_str(&line[i..]);
            out.push_str(RESET);
            break;
        }

        if c == '"' {
            let start = i;
            i += 1;
            while i < bytes.len() {
                let ch = bytes[i] as char;
                if ch == '"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            out.push_str(STR);
            out.push_str(&line[start..i]);
            out.push_str(RESET);
            continue;
        }

        if c.is_ascii_digit()
            || (c == '-' && i + 1 < bytes.len() && (bytes[i + 1] as char).is_ascii_digit())
        {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i] as char).is_ascii_digit() {
                i += 1;
            }
            out.push_str(NUM);
            out.push_str(&line[start..i]);
            out.push_str(RESET);
            continue;
        }

        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            i += 1;
            while i < bytes.len() {
                let ch = bytes[i] as char;
                if ch.is_ascii_alphanumeric() || ch == '_' {
                    i += 1;
                } else {
                    break;
                }
            }
            let token = &line[start..i];
            let color = match token {
                "fn" | "let" | "if" | "else" | "while" | "for" | "return" => KW,
                "print" | "len" => KW,
                "true" | "false" | "nil" => STR,
                "i32" | "i64" | "bool" | "string" | "List" => TY,
                _ => "",
            };
            if color.is_empty() {
                out.push_str(token);
            } else {
                out.push_str(color);
                out.push_str(token);
                out.push_str(RESET);
            }
            continue;
        }

        out.push(c);
        i += 1;
    }

    out
}

fn parse_and_compile(input: String, rl: &mut Editor<CyclangHelper, DefaultHistory>) -> Result<String> {
    let joined_history = rl
        .history()
        .iter()
        .map(|s| &**s) // Convert &String to &str
        .collect::<Vec<&str>>()
        .join("\n");

    let final_string = format!("fn main() {{ {joined_history}{input} }}");
    let exprs = parse_cyclo_program(&final_string)?;
    let compile_options = Some(CompileOptions {
        is_execution_engine: true,
        target: None,
    });
    let output = compiler::compile(exprs.clone(), compile_options)?;

    for expr in parse_cyclo_program(&input)? {
        if let Expression::LetStmt(_, _, _)
        | Expression::FuncStmt(_, _, _, _)
        | Expression::ListAssign(_, _, _) = expr
        {
            let _ = rl.add_history_entry(input.as_str());
        }
    }
    Ok(output)
}
