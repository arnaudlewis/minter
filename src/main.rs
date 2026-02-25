use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "minter", version, about = "Spec compiler & validator")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate one or more .spec files or directories
    Validate {
        /// Spec files or directories to validate
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Also resolve and validate dependencies
        #[arg(long)]
        deep: bool,
    },
    /// Watch a file or directory for spec file changes and validate incrementally
    Watch {
        /// File or directory to watch
        #[arg(required = true)]
        path: PathBuf,
    },
    /// Display the DSL grammar reference
    Format {
        /// Arguments: spec type (fr, nfr)
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Generate a skeleton .spec file
    Scaffold {
        /// Arguments: spec type (fr, nfr), optional category
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Display structured metadata for a spec file
    Inspect {
        /// Spec file to inspect
        #[arg(required = true)]
        file: PathBuf,
    },
    /// Print the spec-driven development methodology reference
    Explain,
    /// Display the dependency graph
    Graph {
        /// Directory containing spec files
        #[arg(required = true)]
        dir: PathBuf,

        /// Show reverse dependencies of a named spec
        #[arg(long)]
        impacted: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Validate { files, deep }) => {
            process::exit(minter::validate::run_validate(&files, deep));
        }
        Some(Commands::Watch { path }) => {
            process::exit(minter::watch::run_watch(&path));
        }
        Some(Commands::Format { args }) => {
            process::exit(minter::format::run_format(&args));
        }
        Some(Commands::Scaffold { args }) => {
            process::exit(minter::scaffold::run_scaffold(&args));
        }
        Some(Commands::Inspect { file }) => {
            process::exit(minter::inspect::run_inspect(&file));
        }
        Some(Commands::Explain) => {
            process::exit(minter::explain::run_explain());
        }
        Some(Commands::Graph { dir, impacted }) => {
            process::exit(minter::graph_cmd::run_graph(&dir, impacted.as_deref()));
        }
        None => {
            use clap::CommandFactory;
            Cli::command().print_help().ok();
            println!();
        }
    }
}
