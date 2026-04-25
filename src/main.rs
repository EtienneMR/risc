//! Entry point: run the REPL when called with no arguments, or a script with one argument.
//! Script errors print source context and exit with code 1; the REPL loops until EOF or "exit".
//! run_one is a thin wrapper: read the file, hand it to Runtime::run, format any LangError.
//! No flags are parsed at this level; scripts use @std/cli for their own argument handling.
//! Embedding: instantiate Runtime directly and call run() / run_repl() from your own binary.

use crate::{repl::repl, runtime::Runtime};

mod ast;
mod corelib;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod repl;
mod runtime;
mod source;
mod value;

fn run_one(path: &String) {
    let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("error: cannot read '{path}': {e}");
        std::process::exit(1);
    });

    let mut runtime = Runtime::new();

    if let Err(e) = runtime.run(path.clone(), content) {
        eprintln!("{}", e.display(runtime.source_map()));
        std::process::exit(1);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        repl();
    } else {
        run_one(&args[1]);
    }
}
