use std::{error::Error, path::PathBuf};

use clap::{ArgGroup, Parser};

use crate::{project::Project, repl::repl, runtime::Runtime};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    group(
        ArgGroup::new("mode")
            .args(["file", "project", "expr"])
            .multiple(false)
    )
)]
struct Cli {
    /// File to run
    file: Option<PathBuf>,

    /// Project to run
    #[arg(
        short = 'p',
        long,
        num_args(0..=1),
        default_missing_value = "."
    )]
    project: Option<PathBuf>,

    /// Raw expression to run
    #[arg(short = 'e', long)]
    expr: Option<String>,

    /// Script arguments
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
    args: Vec<String>,
}

#[derive(Debug)]
enum Mode {
    File(PathBuf),
    Project {
        path: PathBuf,
        entrypoint: Option<PathBuf>,
    },
    Expr(String),
    Repl,
}

impl Cli {
    fn resolve_mode(self) -> (Mode, Vec<String>) {
        (
            if let Some(path) = self.file {
                Mode::File(path)
            } else if let Some(path) = self.project {
                Mode::Project {
                    path,
                    entrypoint: self.file,
                }
            } else if let Some(code) = self.expr {
                Mode::Expr(code)
            } else {
                Mode::Repl
            },
            self.args,
        )
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let (mode, args) = Cli::parse().resolve_mode();

    match mode {
        Mode::File(path) => run_one(path, args),
        Mode::Expr(code) => eval_one(code, "<eval>".to_owned(), args),
        Mode::Project { path, entrypoint } => run_project(path, entrypoint, args),
        Mode::Repl => repl(args),
    }
}

fn eval_one(content: String, path: String, args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let project_root = std::env::current_dir()?;
    let project = Project::new(project_root);
    let runtime = Runtime::new(project, args);

    if let Err(e) = runtime.run(path, content) {
        eprintln!("{}", e.display(&runtime.source_map()));
        return Err("evaluation failed".into());
    }

    Ok(())
}

pub fn run_one(path: PathBuf, args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let content = std::fs::read_to_string(&path)?;
    eval_one(content, path.to_string_lossy().to_string(), args)
}

fn run_project(
    path: PathBuf,
    entrypoint: Option<PathBuf>,
    args: Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let project = Project::resolve(&path)?;

    let entry_path = match entrypoint {
        Some(ep) => ep.canonicalize()?,
        None => project.entrypoint().clone(),
    };

    let content = std::fs::read_to_string(&entry_path)?;
    let entry_name = entry_path.to_string_lossy().to_string();

    let runtime = Runtime::new(project, args);
    if let Err(e) = runtime.run(entry_name, content) {
        eprintln!("{}", e.display(&runtime.source_map()));
        return Err("project execution failed".into());
    }

    Ok(())
}
