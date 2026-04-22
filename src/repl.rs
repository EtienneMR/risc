//! Interactive REPL with multi-line input and persistent session state.
//! Uses a throw-away SourceMap probe to detect incomplete input (missing tokens)
//! and show a continuation prompt (..) instead of an error.
//! Variables survive between inputs; let re-binds rather than errors (repl_mode).
//! Type "exit" or send EOF (Ctrl-D) to quit.

use std::io::{Write, stdin, stdout};

use crate::{
    interpreter::Interpreter,
    lexer::Lexer,
    parser::Parser,
    source::SourceMap,
    value::{Signal, SignalKind, Value},
};

pub fn repl() {
    let mut interpreter = Interpreter::new_repl();

    let sin = stdin();
    let mut sout = stdout();

    let mut buffer = String::new();
    let mut probe_map = SourceMap::new();

    loop {
        let prompt = if buffer.is_empty() { b"> " } else { b". " };
        sout.write_all(prompt).unwrap();
        sout.flush().unwrap();

        let mut line = String::new();
        match sin.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("repl: read error: {e}");
                break;
            }
        }

        if buffer.is_empty() && line.trim() == "exit" {
            break;
        }

        if !buffer.is_empty() {
            buffer.push('\n');
        }
        buffer.push_str(&line);

        {
            let probe_source = probe_map.add("<probe>".to_owned(), buffer.clone());
            let probe_result = Parser::new(Lexer::new(probe_source)).parse();
            match probe_result {
                Err(ref e) if e.is_incomplete() => continue,
                _ => {}
            }
        }

        let input = std::mem::take(&mut buffer);
        probe_map = SourceMap::new();

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
