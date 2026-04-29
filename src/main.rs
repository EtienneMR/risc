//! Entry point: run the REPL when called with no arguments, or a script with one argument.
//! Script errors print source context and exit with code 1; the REPL loops until EOF or "exit".
//! run_one is a thin wrapper: read the file, hand it to Runtime::run, format any LangError.
//! No flags are parsed at this level; scripts use @std/cli for their own argument handling.
//! Embedding: instantiate Runtime directly and call run() / run_repl() from your own binary.

mod ast;
mod cli;
mod corelib;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod project;
mod repl;
mod runtime;
mod source;
mod value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::run()
}

#[cfg(test)]
mod tests {
    include!(concat!(env!("OUT_DIR"), "/ri_tests.rs"));
}
