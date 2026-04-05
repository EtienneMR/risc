mod interpreter;
mod parser;
mod source;
mod tokenizer;

use std::{env, fs, process};

use interpreter::Interpreter;
use parser::parse;
use source::{Source, SourceId};
use tokenizer::Tokenizer;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("usage: {} <script.ri>", args[0]);
        process::exit(1);
    }

    let path = &args[1];
    let content = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("error: cannot read '{path}': {e}");
        process::exit(1);
    });

    let source = Source::new(SourceId(0), path.clone(), content);

    let block = parse(Tokenizer::new(&source)).unwrap_or_else(|e| {
        eprintln!("{}", source.with_context(e.span, &e.message));
        process::exit(1);
    });

    let mut interpreter = Interpreter::new();
    if let Err((message, span)) = interpreter.run(&block) {
        eprintln!("{}", source.with_context(span, &message));
        process::exit(1);
    }
}
