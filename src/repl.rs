use std::io::{stdin, stdout, Write};

use crate::{
    interpreter::Interpreter,
    lexer::Lexer,
    parser::Parser,
    value::{Signal, SignalKind, Value},
};

pub fn repl() {
    let mut interpreter = Interpreter::new();

    let sin = stdin();
    let mut sout = stdout();

    loop {
        sout.write_all(b"> ").unwrap();
        sout.flush().unwrap();

        let mut input = String::new();
        sin.read_line(&mut input).unwrap();

        if input.trim() == "exit" {
            break;
        }

        let program = {
            let source = interpreter
                .session
                .source_map
                .add("<repl>".to_owned(), input);
            match Parser::new(Lexer::new(source)).parse() {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("{}", interpreter.session.source_map.with_context(e.span, e));
                    continue;
                }
            }
        };

        match interpreter.run(program) {
            Ok(Value::Nil) => {}
            Ok(v) => println!("{}", v),
            Err(Signal {
                kind: SignalKind::Error { kind, message },
                span,
            }) => {
                eprintln!(
                    "{}",
                    interpreter
                        .session
                        .source_map
                        .with_context(span, format!("{kind}: {message}"))
                )
            }
            _ => unreachable!(),
        }
    }
}
