use anyhow::{anyhow, Result};
use backend::compiler::{CompileOptions, desugar_program};
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
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::process::Command;
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
    let mut persisted: Vec<String> = Vec::new();
    rl.bind_sequence(
        KeyEvent(KeyCode::Down, Modifiers::SHIFT),
        EventHandler::Simple(Cmd::Newline),
    );

    loop {
        let line = rl.readline(">> ");
        match line {
            Ok(input) => {
                let cleaned = if input.starts_with(">>\n") {
                    input.trim_start_matches(">>\n").to_string()
                } else if input.starts_with(">> ") {
                    input.trim_start_matches(">> ").to_string()
                } else {
                    input
                };
                match cleaned.trim() {
                "exit()" => break,
                "" => continue,
                cmd if cmd.starts_with(":load ") => match load_file(cmd) {
                    Ok(source) => {
                        println!(">>\n{source}");
                        let history_entry = format!(">>\n{source}");
                        let _ = rl.add_history_entry(history_entry.as_str());
                        match parse_and_compile_no_state(source) {
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
                    Ok(source) => match parse_and_compile(source, &mut persisted) {
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
                cmd if cmd.starts_with(":emit") => match wrap_emit(cmd) {
                    Ok(source) => match parse_and_emit_ir(source, &mut persisted) {
                        Ok(output) => {
                            if !output.is_empty() {
                                print!("{}", highlight_llvm_ir(&output));
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
                cmd if cmd.starts_with(":opt") => match wrap_opt(cmd) {
                    Ok(source) => match parse_and_opt_ir(source, &mut persisted) {
                        Ok(output) => {
                            if !output.is_empty() {
                                print!("{}", highlight_llvm_ir(&output));
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
                cmd if cmd.starts_with(":asm") => match wrap_asm(cmd) {
                    Ok(source) => match parse_and_asm(source, &mut persisted) {
                        Ok(output) => {
                            if !output.is_empty() {
                                print!("{}", highlight_asm(&output));
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
                cmd if cmd.starts_with(":ast") && !cmd.starts_with(":astd") => match wrap_ast(cmd) {
                    Ok(source) => match parse_and_ast(source, &mut persisted) {
                        Ok(output) => {
                            if !output.is_empty() {
                                print!("{}", highlight_ast(&output));
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
                cmd if cmd.starts_with(":astd") => match wrap_astd(cmd) {
                    Ok(source) => match parse_and_ast_desugar(source, &mut persisted) {
                        Ok(output) => {
                            if !output.is_empty() {
                                print!("{}", highlight_ast(&output));
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
                _ => match parse_and_compile(cleaned.to_string(), &mut persisted) {
                    Ok(output) => {
                        if !output.is_empty() {
                            print!("{output}");
                        }
                    }
                    Err(e) => {
                        println!("{}", e.to_string().red());
                    }
                },
            }
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

fn wrap_emit(cmd: &str) -> Result<String> {
    let expr = cmd.trim_start_matches(":emit").trim();
    if expr.is_empty() {
        return Err(anyhow!("Usage: :emit <statement>"));
    }
    Ok(expr.to_string())
}

fn wrap_opt(cmd: &str) -> Result<String> {
    let expr = cmd.trim_start_matches(":opt").trim();
    if expr.is_empty() {
        return Err(anyhow!("Usage: :opt <statement>"));
    }
    let expr = expr.trim_end_matches(';').trim_end();
    Ok(format!("{expr};"))
}

fn wrap_asm(cmd: &str) -> Result<String> {
    let expr = cmd.trim_start_matches(":asm").trim();
    if expr.is_empty() {
        return Err(anyhow!("Usage: :asm <statement>"));
    }
    let expr = expr.trim_end_matches(';').trim_end();
    Ok(format!("{expr};"))
}

fn wrap_ast(cmd: &str) -> Result<String> {
    let expr = cmd.trim_start_matches(":ast").trim();
    if expr.is_empty() {
        return Err(anyhow!("Usage: :ast <statement>"));
    }
    Ok(expr.to_string())
}

fn wrap_astd(cmd: &str) -> Result<String> {
    let expr = cmd.trim_start_matches(":astd").trim();
    if expr.is_empty() {
        return Err(anyhow!("Usage: :astd <statement>"));
    }
    Ok(expr.to_string())
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
                "fn" | "let" | "if" | "else" | "while" | "for" | "return" | "break" => KW,
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

fn highlight_llvm_ir(input: &str) -> String {
    const RESET: &str = "\x1b[0m";
    const KW: &str = "\x1b[38;5;81m";
    const TY: &str = "\x1b[38;5;75m";
    const NUM: &str = "\x1b[38;5;221m";
    const STR: &str = "\x1b[38;5;114m";
    const COM: &str = "\x1b[38;5;242m";
    const LABEL: &str = "\x1b[38;5;208m";

    let mut out = String::with_capacity(input.len() + 32);
    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(';') {
            out.push_str(COM);
            out.push_str(line);
            out.push_str(RESET);
            out.push('\n');
            continue;
        }

        if trimmed.ends_with(':') {
            out.push_str(LABEL);
            out.push_str(line);
            out.push_str(RESET);
            out.push('\n');
            continue;
        }

        let mut i = 0;
        let bytes = line.as_bytes();
        while i < bytes.len() {
            let c = bytes[i] as char;
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

            if c.is_ascii_alphabetic() || c == '_' || c == '.' {
                let start = i;
                i += 1;
                while i < bytes.len() {
                    let ch = bytes[i] as char;
                    if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
                        i += 1;
                    } else {
                        break;
                    }
                }
                let token = &line[start..i];
                let color = match token {
                    "define" | "declare" | "call" | "ret" | "br" | "load" | "store"
                    | "alloca" | "getelementptr" | "icmp" | "phi" | "switch" | "unreachable"
                    | "tail" | "invoke" | "landingpad" => KW,
                    "i1" | "i8" | "i16" | "i32" | "i64" | "i128" | "void" | "ptr" | "label"
                    | "double" | "float" => TY,
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
        out.push('\n');
    }

    out
}

fn parse_and_compile(input: String, persisted: &mut Vec<String>) -> Result<String> {
    let joined = if persisted.is_empty() {
        String::new()
    } else {
        persisted.join("\n") + "\n"
    };

    let final_string = format!("fn main() {{ {joined}{input} }}");
    let exprs = parse_cyclo_program(&final_string)?;
    let compile_options = Some(CompileOptions {
        is_execution_engine: true,
        emit_llvm_ir: false,
        emit_llvm_ir_main_only: true,
        emit_llvm_ir_with_called: false,
        target: None,
    });
    let output = compiler::compile(exprs.clone(), compile_options)?;

    for expr in parse_cyclo_program(&input)? {
        if let Expression::LetStmt(_, _, _)
        | Expression::FuncStmt(_, _, _, _)
        | Expression::ListAssign(_, _, _) = expr
        {
            persisted.push(input.clone());
        }
    }
    Ok(output)
}

fn parse_and_compile_no_state(input: String) -> Result<String> {
    let final_string = format!("fn main() {{ {input} }}");
    let exprs = parse_cyclo_program(&final_string)?;
    let compile_options = Some(CompileOptions {
        is_execution_engine: true,
        emit_llvm_ir: false,
        emit_llvm_ir_main_only: true,
        emit_llvm_ir_with_called: false,
        target: None,
    });
    compiler::compile(exprs, compile_options)
}

fn parse_and_emit_ir(input: String, persisted: &mut [String]) -> Result<String> {
    let joined = if persisted.is_empty() {
        String::new()
    } else {
        persisted.join("\n") + "\n"
    };

    let final_string = format!("fn main() {{ {joined}{input} }}");
    let exprs = parse_cyclo_program(&final_string)?;
    let compile_options = Some(CompileOptions {
        is_execution_engine: false,
        emit_llvm_ir: true,
        emit_llvm_ir_main_only: true,
        emit_llvm_ir_with_called: true,
        target: None,
    });
    compiler::compile(exprs, compile_options)
}

fn parse_and_opt_ir(input: String, persisted: &mut [String]) -> Result<String> {
    let joined = if persisted.is_empty() {
        String::new()
    } else {
        persisted.join("\n") + "\n"
    };

    let final_string = format!("fn main() {{ {joined}{input} }}");
    let exprs = parse_cyclo_program(&final_string)?;
    let compile_options = Some(CompileOptions {
        is_execution_engine: false,
        emit_llvm_ir: true,
        emit_llvm_ir_main_only: false,
        emit_llvm_ir_with_called: false,
        target: None,
    });
    let module_ir = compiler::compile(exprs, compile_options)?;
    run_opt_on_ir(&module_ir)
}

fn parse_and_asm(input: String, persisted: &mut [String]) -> Result<String> {
    let joined = if persisted.is_empty() {
        String::new()
    } else {
        persisted.join("\n") + "\n"
    };

    let final_string = format!("fn main() {{ {joined}{input} }}");
    let exprs = parse_cyclo_program(&final_string)?;
    let compile_options = Some(CompileOptions {
        is_execution_engine: false,
        emit_llvm_ir: true,
        emit_llvm_ir_main_only: false,
        emit_llvm_ir_with_called: false,
        target: None,
    });
    let module_ir = compiler::compile(exprs, compile_options)?;
    run_llc_on_ir(&module_ir)
}

fn parse_and_ast(input: String, persisted: &mut [String]) -> Result<String> {
    let joined = if persisted.is_empty() {
        String::new()
    } else {
        persisted.join("\n") + "\n"
    };
    let final_string = format!("fn main() {{ {joined}{input} }}");
    let exprs = parse_cyclo_program(&final_string)?;
    Ok(format_ast(&exprs))
}

fn parse_and_ast_desugar(input: String, persisted: &mut [String]) -> Result<String> {
    let joined = if persisted.is_empty() {
        String::new()
    } else {
        persisted.join("\n") + "\n"
    };
    let final_string = format!("fn main() {{ {joined}{input} }}");
    let exprs = parse_cyclo_program(&final_string)?;
    let desugared = desugar_program(exprs);
    Ok(format_ast(&desugared))
}

fn run_opt_on_ir(ir: &str) -> Result<String> {
    let mut child = Command::new("opt")
        .arg("-O2")
        .arg("-S")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("Failed to run opt: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(ir.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(anyhow!(
            "opt failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let optimized = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(extract_main_ir(&optimized).unwrap_or(optimized))
}

fn run_llc_on_ir(ir: &str) -> Result<String> {
    let mut child = Command::new("llc")
        .arg("-O2")
        .arg("-filetype=asm")
        .arg("-o")
        .arg("-")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("Failed to run llc: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(ir.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(anyhow!(
            "llc failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let asm = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(extract_main_asm(&asm).unwrap_or(asm))
}

fn extract_main_asm(asm: &str) -> Option<String> {
    let lines = asm.lines().peekable();
    let mut buf = String::new();
    let mut in_fn = false;
    for line in lines {
        let trimmed = line.trim_start();
        if !in_fn {
            if trimmed.starts_with("_main:") || trimmed.starts_with("main:") {
                in_fn = true;
                buf.push_str(line);
                buf.push('\n');
                continue;
            }
        } else {
            if trimmed.starts_with(".globl") || trimmed.starts_with(".section") {
                break;
            }
            buf.push_str(line);
            buf.push('\n');
        }
    }
    if buf.is_empty() { None } else { Some(buf) }
}

fn highlight_asm(input: &str) -> String {
    const RESET: &str = "\x1b[0m";
    const MN: &str = "\x1b[38;5;81m";
    const REG: &str = "\x1b[38;5;75m";
    const NUM: &str = "\x1b[38;5;221m";
    const COM: &str = "\x1b[38;5;242m";
    const LABEL: &str = "\x1b[38;5;208m";

    let mut out = String::with_capacity(input.len() + 32);
    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            out.push_str(COM);
            out.push_str(line);
            out.push_str(RESET);
            out.push('\n');
            continue;
        }
        if trimmed.ends_with(':') {
            out.push_str(LABEL);
            out.push_str(line);
            out.push_str(RESET);
            out.push('\n');
            continue;
        }

        let mut i = 0;
        let bytes = line.as_bytes();
        while i < bytes.len() {
            let c = bytes[i] as char;
            if c == '#' {
                out.push_str(COM);
                out.push_str(&line[i..]);
                out.push_str(RESET);
                i = bytes.len();
                continue;
            }
            if c == '%' {
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
                out.push_str(REG);
                out.push_str(&line[start..i]);
                out.push_str(RESET);
                continue;
            }
            if c.is_ascii_digit() || (c == '-' && i + 1 < bytes.len() && (bytes[i + 1] as char).is_ascii_digit()) {
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
            if c.is_ascii_alphabetic() || c == '_' || c == '.' {
                let start = i;
                i += 1;
                while i < bytes.len() {
                    let ch = bytes[i] as char;
                    if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
                        i += 1;
                    } else {
                        break;
                    }
                }
                let token = &line[start..i];
                // Color the mnemonic if it's the first token after indentation
                if line[..start].trim().is_empty() {
                    out.push_str(MN);
                    out.push_str(token);
                    out.push_str(RESET);
                } else {
                    out.push_str(token);
                }
                continue;
            }
            out.push(c);
            i += 1;
        }
        out.push('\n');
    }
    out
}

fn format_ast(exprs: &[Expression]) -> String {
    let mut out = String::new();
    out.push_str("Program\n");
    for (i, expr) in exprs.iter().enumerate() {
        let is_last = i + 1 == exprs.len();
        format_expr_tree(expr, "", is_last, &mut out);
    }
    out
}

fn highlight_ast(input: &str) -> String {
    const RESET: &str = "\x1b[0m";
    const BLOCK: &str = "\x1b[38;5;75m";
    const FLOW: &str = "\x1b[38;5;81m";
    const FUNC: &str = "\x1b[38;5;135m";
    const OP: &str = "\x1b[38;5;208m";
    const LIT: &str = "\x1b[38;5;114m";
    const VAR: &str = "\x1b[38;5;222m";
    const SECTION: &str = "\x1b[38;5;244m";

    let mut out = String::with_capacity(input.len() + 32);
    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }

        let (prefix, label) = {
            let prefix_len = line.len() - trimmed.len();
            (&line[..prefix_len], trimmed)
        };

        let clean_label = label
            .trim_start_matches("├─ ")
            .trim_start_matches("└─ ")
            .trim();

        let color = if clean_label.starts_with("BlockStmt")
            || clean_label.starts_with("LetStmt")
            || clean_label.starts_with("List")
            || clean_label.starts_with("ListIndex")
            || clean_label.starts_with("ListAssign")
        {
            BLOCK
        } else if clean_label.starts_with("FuncStmt") || clean_label.starts_with("FuncArg") {
            FUNC
        } else if clean_label.starts_with("IfStmt")
            || clean_label.starts_with("WhileStmt")
            || clean_label.starts_with("ForStmt")
            || clean_label.starts_with("ReturnStmt")
            || clean_label.starts_with("BreakStmt")
        {
            FLOW
        } else if clean_label.starts_with("Binary")
            || clean_label.starts_with("Grouping")
            || clean_label.starts_with("CallStmt")
            || clean_label.starts_with("Print")
            || clean_label.starts_with("Len")
        {
            OP
        } else if clean_label.starts_with("Number")
            || clean_label.starts_with("Number64")
            || clean_label.starts_with("String")
            || clean_label.starts_with("Bool")
            || clean_label.starts_with("Nil")
        {
            LIT
        } else if clean_label.starts_with("Variable") {
            VAR
        } else if clean_label == "Args"
            || clean_label == "Body"
            || clean_label == "Condition"
            || clean_label == "Then"
            || clean_label == "Else"
            || clean_label == "Program"
        {
            SECTION
        } else {
            FLOW
        };

        out.push_str(prefix);
        out.push_str(color);
        out.push_str(label);
        out.push_str(RESET);
        out.push('\n');
    }
    out
}

fn format_expr_tree(expr: &Expression, prefix: &str, is_last: bool, out: &mut String) {
    use Expression::*;
    let branch = if is_last { "└─ " } else { "├─ " };
    let next_prefix = if is_last { "   " } else { "│  " };

    let label = match expr {
        Number(n) => format!("Number({n})"),
        Number64(n) => format!("Number64({n})"),
        String(s) => format!("String({s})"),
        Bool(b) => format!("Bool({b})"),
        Nil => "Nil".to_string(),
        Variable(name) => format!("Variable({name})"),
        Binary(_, op, _) => format!("Binary({op})"),
        Grouping(_) => "Grouping".to_string(),
        LetStmt(name, ty, _) => format!("LetStmt({name}: {})", format_type(ty)),
        BlockStmt(_) => "BlockStmt".to_string(),
        FuncArg(name, ty) => format!("FuncArg({name}: {})", format_type(ty)),
        FuncStmt(name, _, ret_ty, _) => format!("FuncStmt({name} -> {})", format_type(ret_ty)),
        ExternFuncStmt(name, _, ret_ty) => {
            format!("ExternFuncStmt({name} -> {})", format_type(ret_ty))
        }
        ExternModule(path) => format!("ExternModule({path})"),
        CallStmt(name, _) => format!("CallStmt({name})"),
        IfStmt(_, _, _) => "IfStmt".to_string(),
        WhileStmt(_, _) => "WhileStmt".to_string(),
        ReturnStmt(_) => "ReturnStmt".to_string(),
        BreakStmt => "BreakStmt".to_string(),
        ForStmt(name, start, end, step, _) => {
            format!("ForStmt({name} = {start}..{end} step {step})")
        }
        Print(_) => "Print".to_string(),
        Len(_) => "Len".to_string(),
        List(_) => "List".to_string(),
        ListIndex(_, _) => "ListIndex".to_string(),
        ListAssign(name, _, _) => format!("ListAssign({name})"),
    };

    out.push_str(prefix);
    out.push_str(branch);
    out.push_str(&label);
    out.push('\n');

    let child_prefix = format!("{prefix}{next_prefix}");
    match expr {
        Binary(lhs, _, rhs) => {
            format_expr_tree(lhs, &child_prefix, false, out);
            format_expr_tree(rhs, &child_prefix, true, out);
        }
        Grouping(inner) => {
            format_expr_tree(inner, &child_prefix, true, out);
        }
        LetStmt(_, _, value) => {
            format_expr_tree(value, &child_prefix, true, out);
        }
        BlockStmt(stmts) => {
            for (i, stmt) in stmts.iter().enumerate() {
                let last = i + 1 == stmts.len();
                format_expr_tree(stmt, &child_prefix, last, out);
            }
        }
        FuncStmt(_, args, _, body) => {
            out.push_str(&child_prefix);
            out.push_str("├─ Args\n");
            let args_prefix = format!("{child_prefix}│  ");
            for (i, arg) in args.iter().enumerate() {
                let last = i + 1 == args.len();
                format_expr_tree(arg, &args_prefix, last, out);
            }
            out.push_str(&child_prefix);
            out.push_str("└─ Body\n");
            let body_prefix = format!("{child_prefix}   ");
            format_expr_tree(body, &body_prefix, true, out);
        }
        ExternFuncStmt(_, args, _) => {
            out.push_str(&child_prefix);
            out.push_str("└─ Args\n");
            let args_prefix = format!("{child_prefix}   ");
            for (i, arg) in args.iter().enumerate() {
                let last = i + 1 == args.len();
                format_expr_tree(arg, &args_prefix, last, out);
            }
        }
        ExternModule(_) => {}
        CallStmt(_, args) => {
            for (i, arg) in args.iter().enumerate() {
                let last = i + 1 == args.len();
                format_expr_tree(arg, &child_prefix, last, out);
            }
        }
        IfStmt(cond, then_block, else_block) => {
            out.push_str(&child_prefix);
            out.push_str("├─ Condition\n");
            let cond_prefix = format!("{child_prefix}│  ");
            format_expr_tree(cond, &cond_prefix, true, out);

            out.push_str(&child_prefix);
            if else_block.is_some() {
                out.push_str("├─ Then\n");
                let then_prefix = format!("{child_prefix}│  ");
                format_expr_tree(then_block, &then_prefix, true, out);

                out.push_str(&child_prefix);
                out.push_str("└─ Else\n");
                let else_prefix = format!("{child_prefix}   ");
                if let Some(else_expr) = else_block.as_ref() {
                    format_expr_tree(else_expr, &else_prefix, true, out);
                }
            } else {
                out.push_str("└─ Then\n");
                let then_prefix = format!("{child_prefix}   ");
                format_expr_tree(then_block, &then_prefix, true, out);
            }
        }
        WhileStmt(cond, body) => {
            out.push_str(&child_prefix);
            out.push_str("├─ Condition\n");
            let cond_prefix = format!("{child_prefix}│  ");
            format_expr_tree(cond, &cond_prefix, true, out);

            out.push_str(&child_prefix);
            out.push_str("└─ Body\n");
            let body_prefix = format!("{child_prefix}   ");
            format_expr_tree(body, &body_prefix, true, out);
        }
        ReturnStmt(value) => {
            format_expr_tree(value, &child_prefix, true, out);
        }
        BreakStmt => {}
        ForStmt(_, _, _, _, body) => {
            format_expr_tree(body, &child_prefix, true, out);
        }
        Print(expr) | Len(expr) => {
            format_expr_tree(expr, &child_prefix, true, out);
        }
        List(values) => {
            for (i, v) in values.iter().enumerate() {
                let last = i + 1 == values.len();
                format_expr_tree(v, &child_prefix, last, out);
            }
        }
        ListIndex(list, index) => {
            format_expr_tree(list, &child_prefix, false, out);
            format_expr_tree(index, &child_prefix, true, out);
        }
        ListAssign(_, index, value) => {
            format_expr_tree(index, &child_prefix, false, out);
            format_expr_tree(value, &child_prefix, true, out);
        }
        _ => {}
    }
}

fn format_type(ty: &parser::Type) -> String {
    match ty {
        parser::Type::None => "None".to_string(),
        parser::Type::i32 => "i32".to_string(),
        parser::Type::i64 => "i64".to_string(),
        parser::Type::String => "string".to_string(),
        parser::Type::Bool => "bool".to_string(),
        parser::Type::List(inner) => format!("List<{}>", format_type(inner)),
    }
}

fn extract_main_ir(module_ir: &str) -> Option<String> {
    let main_ir = extract_main_only(module_ir)?;
    let mut out = String::new();
    out.push_str(&main_ir);
    let calls = collect_called_functions(&main_ir);
    for name in calls {
        if let Some(def) = extract_function_def(module_ir, &name) {
            out.push('\n');
            out.push_str(&def);
        }
    }
    Some(out)
}

fn extract_main_only(module_ir: &str) -> Option<String> {
    let lines = module_ir.lines();
    let mut buf = String::new();
    let mut in_main = false;
    let mut brace_depth = 0i32;

    for line in lines {
        if !in_main {
            let trimmed = line.trim_start();
            if trimmed.starts_with("define ") && trimmed.contains("@main") {
                in_main = true;
                buf.push_str(line);
                buf.push('\n');
                brace_depth += line.matches('{').count() as i32;
                brace_depth -= line.matches('}').count() as i32;
                if brace_depth <= 0 && line.contains('}') {
                    return Some(buf);
                }
                continue;
            }
        } else {
            buf.push_str(line);
            buf.push('\n');
            brace_depth += line.matches('{').count() as i32;
            brace_depth -= line.matches('}').count() as i32;
            if brace_depth <= 0 {
                return Some(buf);
            }
        }
    }
    None
}

fn collect_called_functions(ir: &str) -> HashSet<String> {
    let mut names = HashSet::new();
    for line in ir.lines() {
        if !line.contains("call") {
            continue;
        }
        let mut i = 0;
        let bytes = line.as_bytes();
        while i < bytes.len() {
            if bytes[i] as char == '@' {
                let start = i + 1;
                let mut end = start;
                while end < bytes.len() {
                    let ch = bytes[end] as char;
                    if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
                        end += 1;
                    } else {
                        break;
                    }
                }
                if end > start {
                    let name = &line[start..end];
                    if !name.starts_with("llvm.") && name != "printf" {
                        names.insert(name.to_string());
                    }
                }
                i = end;
            } else {
                i += 1;
            }
        }
    }
    names
}

fn extract_function_def(module_ir: &str, name: &str) -> Option<String> {
    let needle = format!("@{name}");
    let lines = module_ir.lines();
    let mut buf = String::new();
    let mut in_fn = false;
    let mut brace_depth = 0i32;

    for line in lines {
        if !in_fn {
            let trimmed = line.trim_start();
            if trimmed.starts_with("define ") && trimmed.contains(&needle) {
                in_fn = true;
                buf.push_str(line);
                buf.push('\n');
                brace_depth += line.matches('{').count() as i32;
                brace_depth -= line.matches('}').count() as i32;
                if brace_depth <= 0 && line.contains('}') {
                    return Some(buf);
                }
                continue;
            }
        } else {
            buf.push_str(line);
            buf.push('\n');
            brace_depth += line.matches('{').count() as i32;
            brace_depth -= line.matches('}').count() as i32;
            if brace_depth <= 0 {
                return Some(buf);
            }
        }
    }
    None
}
