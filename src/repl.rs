//! Interactive REPL with multi-line input, persistent history, and inline syntax hints.
//! ReplValidator detects incomplete input (UnexpectedEOF) to show a continuation prompt.
//! Variables survive between inputs via repl_mode; let re-binds rather than erroring.
//! History is persisted to ~/.risc_history via reedline's FileBackedHistory (capacity 1000).
//! Type "exit" to quit; errors print source context without exiting.

use std::{borrow::Cow, env, error::Error, path::PathBuf};

use nu_ansi_term::{Color, Style};
use reedline::{
    DefaultHinter, FileBackedHistory, Prompt, PromptEditMode, PromptHistorySearch, Reedline,
    Signal, ValidationResult, Validator,
};

use crate::{
    error::LangErrorKind, lexer::Lexer, parser::Parser, project::Project, runtime::Runtime,
    source::SourceMap, value::Value,
};

struct ReplPrompt;

impl Prompt for ReplPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Borrowed("> ")
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed("  ")
    }

    fn render_prompt_history_search_indicator(&self, _search: PromptHistorySearch) -> Cow<'_, str> {
        Cow::Borrowed("(search) ")
    }
}

struct ReplValidator;

impl Validator for ReplValidator {
    fn validate(&self, input: &str) -> ValidationResult {
        if input.ends_with("\n\n") {
            return ValidationResult::Complete;
        }

        let mut probe_map = SourceMap::new();
        let probe_source = probe_map.add("<repl>".to_owned(), input.to_string());
        let probe_result = Parser::new(Lexer::new(probe_source)).parse();
        match probe_result {
            Err(e) => {
                if matches!(e.kind, LangErrorKind::UnexpectedEOF { .. }) {
                    ValidationResult::Incomplete
                } else {
                    ValidationResult::Complete
                }
            }
            _ => ValidationResult::Complete,
        }
    }
}

fn create_history() -> Option<FileBackedHistory> {
    env::var_os("HOME")
        .map(|h| PathBuf::from(h).join(".risc_history"))
        .and_then(|p| FileBackedHistory::with_file(1000, p).ok())
}

pub fn repl(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let project = Project::new(env::current_dir()?);
    let runtime = Runtime::new(project, args);
    let env = runtime.create_module_env();

    let mut line_editor = Reedline::create()
        .with_validator(Box::new(ReplValidator))
        .with_hinter(Box::new(
            DefaultHinter::default().with_style(Style::new().italic().fg(Color::LightGray)),
        ));

    if let Some(histroy) = create_history() {
        line_editor = line_editor.with_history(Box::new(histroy))
    } else {
        eprintln!("History could not be created");
    }

    loop {
        let input = match line_editor.read_line(&ReplPrompt)? {
            Signal::Success(buffer) => buffer,
            _ => return Ok(()),
        };

        if input == "exit" {
            return Ok(());
        }

        match runtime.run_repl(input, env.clone()) {
            Ok(Value::Nil) => {}
            Ok(v) => println!("{}", v),
            Err(e) => {
                eprintln!("{}", e.display(&runtime.source_map()));
            }
        }
    }
}
