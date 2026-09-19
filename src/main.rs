use std::io::{self, BufRead, Write};
use std::process::ExitCode;

use sscheme::Interpreter;

fn main() -> ExitCode {
    let mut interpreter = Interpreter::new();
    let result = match std::env::args().nth(1) {
        Some(path) => run_file(&mut interpreter, &path),
        None => repl(&mut interpreter),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Evaluates the file at `path`, stopping at the first error.
fn run_file(interpreter: &mut Interpreter, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path).map_err(|err| format!("{path}: {err}"))?;
    interpreter.eval(&source)?;
    Ok(())
}

/// Reads expressions from stdin and prints their values until end of input.
fn repl(interpreter: &mut Interpreter) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdin = io::stdin().lock();
    let mut input = String::new();
    loop {
        print!("{}", if input.is_empty() { "> " } else { ". " });
        io::stdout().flush()?;
        if stdin.read_line(&mut input)? == 0 {
            println!();
            return Ok(());
        }
        if open_parens(&input) > 0 {
            continue;
        }
        if !input.trim().is_empty() {
            match interpreter.eval(&input) {
                Ok(value) => println!("{value}"),
                Err(err) => eprintln!("error: {err}"),
            }
        }
        input.clear();
    }
}

/// Counts parentheses that are still open, so an expression can span
/// several lines.
fn open_parens(source: &str) -> isize {
    source.chars().fold(0, |depth, c| match c {
        '(' => depth + 1,
        ')' => depth - 1,
        _ => depth,
    })
}
