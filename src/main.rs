use crate::{interpreter::Interpreter, lexer::Lexer, parser::Parser, value::SignalKind};

mod ast;
mod corelib;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod repl;
mod session;
mod source;
mod value;

fn run_one(path: &String) {
    let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("error: cannot read '{path}': {e}");
        std::process::exit(1);
    });

    let mut interpreter = Interpreter::new();

    let program = {
        let source = interpreter.session.source_map.add(path.clone(), content);
        match Parser::new(Lexer::new(source)).parse() {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("{}", interpreter.session.source_map.with_context(e.span, e));
                std::process::exit(1);
            }
        }
    };

    match interpreter.run(program) {
        Ok(val) => println!("{val:?}"),
        Err(s) => match s.kind {
            SignalKind::Error { kind, message } => {
                eprintln!(
                    "{}",
                    interpreter
                        .session
                        .source_map
                        .with_context(s.span, format!("{kind}: {message}"))
                );
                std::process::exit(1);
            }
            _ => unreachable!(),
        },
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        repl::repl();
    } else {
        run_one(&args[1]);
    }
}
