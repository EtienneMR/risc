use std::{error::Error, path::PathBuf};

use clap::{Args, Parser, Subcommand};

use crate::{project::Project, repl::repl, runtime::Runtime};

#[derive(Parser)]
#[command(name = "risc", version, about = "Risc scripting language CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Arguments passed to script
    #[arg(last = true)]
    script_args: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute file
    Run(RunArgs),

    /// Execute code
    Eval(EvalArgs),

    /// Run a project
    Project(ProjectArgs),

    /// Start REPL
    Repl(ReplArgs),
}

#[derive(Args)]
struct RunArgs {
    /// Script file path
    file: String,
}

#[derive(Args)]
struct EvalArgs {
    /// Inline code to execute
    code: String,
}

#[derive(Args)]
struct ProjectArgs {
    /// Path to project directory
    #[arg(short, long, default_value = ".")]
    path: String,

    /// Entry module (optional override)
    #[arg(short, long)]
    entry: Option<String>,
}

#[derive(Args)]
struct ReplArgs;

fn eval_one(content: String, path: String, args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let project = Project::new(PathBuf::from(&path));
    let runtime = Runtime::new(project, args);

    if let Err(e) = runtime.run(path, content) {
        eprintln!("{}", e.display(&runtime.source_map()));
        return Err("evaluation failed".into());
    }
    Ok(())
}

pub fn run_one(path: String, args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let content = std::fs::read_to_string(&path)?;
    eval_one(content, path, args)
}

fn run_project(args: ProjectArgs, script_args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let root = std::path::Path::new(&args.path);
    let project = crate::project::Project::resolve(root)?;

    let entry_path = if let Some(entry) = args.entry {
        root.join(entry)
    } else {
        project.entrypoint().clone()
    };

    let content = std::fs::read_to_string(&entry_path)?;
    let entry_name = entry_path.to_string_lossy().to_string();

    let runtime = Runtime::new(project, script_args);
    if let Err(e) = runtime.run(entry_name, content) {
        eprintln!("{}", e.display(&runtime.source_map()));
        return Err("project execution failed".into());
    }
    Ok(())
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(c) => match c {
            Commands::Run(args) => run_one(args.file, cli.script_args.clone()),
            Commands::Eval(args) => {
                eval_one(args.code, "<eval>".to_owned(), cli.script_args.clone())
            }
            Commands::Project(args) => run_project(args, cli.script_args.clone()),
            Commands::Repl(_args) => repl(cli.script_args.clone()),
        },
        None => repl(cli.script_args.clone()),
    }
}
