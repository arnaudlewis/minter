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
        /// Arguments: spec type (spec, nfr)
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Generate a skeleton .spec file
    Scaffold {
        /// Arguments: spec type (spec, nfr), optional category
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Display structured metadata for a spec file
    Inspect {
        /// Spec file to inspect
        #[arg(required = true)]
        file: PathBuf,
    },
    /// Display a spec-driven development guide by topic
    Guide {
        /// Guide topic (omit to list available topics)
        #[arg(value_enum, hide_possible_values = true)]
        topic: Option<minter::model::GuideTopic>,
    },
    /// Compute test coverage of spec behaviors
    Coverage {
        /// Spec file or directory path
        spec_path: Option<PathBuf>,

        /// Directories to scan for @minter tags (default: current directory)
        #[arg(long)]
        scan: Vec<PathBuf>,

        /// Output format (default: human, json)
        #[arg(long)]
        format: Option<String>,

        /// Show individual behaviors even when specs are fully covered
        #[arg(long)]
        verbose: bool,
    },
    /// Display the dependency graph
    Graph {
        /// Directory containing spec files
        dir: Option<PathBuf>,

        /// Show reverse dependencies of a named spec
        #[arg(long)]
        impacted: Option<String>,
    },
    /// Generate a minter.lock integrity snapshot
    Lock,
    /// Verify project integrity against the lock file
    Ci,
    /// Launch the interactive dashboard
    Ui {
        /// Port to serve on
        #[arg(long, default_value = "4321")]
        port: u16,
        /// Do not open the browser automatically
        #[arg(long)]
        no_open: bool,
    },
}

/// Load config from the current working directory, exiting on failure.
fn load_config_or_exit() -> minter::core::config::ProjectConfig {
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: cannot determine working directory: {}", e);
            process::exit(1);
        }
    };
    match minter::core::config::load_config(&cwd) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Validate { files, deep }) => {
            if files.is_empty() {
                let config = load_config_or_exit();
                if let Err(e) = minter::core::config::require_specs(&config) {
                    eprintln!("error: {}", e);
                    process::exit(1);
                }
                process::exit(minter::core::commands::validate::run_validate(
                    &[config.specs],
                    deep,
                ));
            } else {
                process::exit(minter::core::commands::validate::run_validate(&files, deep));
            }
        }
        Some(Commands::Watch { path }) => {
            process::exit(minter::core::commands::watch::run_watch(&path));
        }
        Some(Commands::Format { args }) => {
            process::exit(minter::core::commands::format::run_format(&args));
        }
        Some(Commands::Scaffold { args }) => {
            process::exit(minter::core::commands::scaffold::run_scaffold(&args));
        }
        Some(Commands::Inspect { file }) => {
            process::exit(minter::core::commands::inspect::run_inspect(&file));
        }
        Some(Commands::Guide { topic }) => {
            process::exit(match topic {
                Some(t) => minter::core::commands::guide::run_guide(&t),
                None => minter::core::commands::guide::list_topics(),
            });
        }
        Some(Commands::Coverage {
            spec_path,
            scan,
            format,
            verbose,
        }) => {
            let (resolved_spec_path, resolved_scan) = match spec_path {
                Some(p) => (p, scan),
                None => {
                    let config = load_config_or_exit();
                    if let Err(e) = minter::core::config::require_specs(&config) {
                        eprintln!("error: {}", e);
                        process::exit(1);
                    }
                    let scan_dirs = if scan.is_empty() { config.tests } else { scan };
                    (config.specs, scan_dirs)
                }
            };
            process::exit(minter::core::commands::coverage::run_coverage(
                &resolved_spec_path,
                &resolved_scan,
                format.as_deref(),
                verbose,
            ));
        }
        Some(Commands::Graph { dir, impacted }) => {
            let resolved_dir = match dir {
                Some(d) => d,
                None => {
                    let config = load_config_or_exit();
                    if let Err(e) = minter::core::config::require_specs(&config) {
                        eprintln!("error: {}", e);
                        process::exit(1);
                    }
                    config.specs
                }
            };
            process::exit(minter::core::commands::graph::run_graph(
                &resolved_dir,
                impacted.as_deref(),
            ));
        }
        Some(Commands::Lock) => {
            let config = load_config_or_exit();
            if let Err(e) = minter::core::config::require_specs(&config) {
                eprintln!("error: {}", e);
                process::exit(1);
            }
            process::exit(minter::core::commands::lock::run_lock(&config));
        }
        Some(Commands::Ci) => {
            let config = load_config_or_exit();
            process::exit(minter::core::commands::ci::run_ci(&config));
        }
        Some(Commands::Ui { port, no_open }) => {
            let cwd = std::env::current_dir().unwrap_or_else(|e| {
                eprintln!("error: cannot determine working directory: {}", e);
                process::exit(1);
            });
            let rt = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
                eprintln!("error: cannot create async runtime: {}", e);
                process::exit(1);
            });
            let code = rt.block_on(minter::core::web::server::run_server(cwd, port, no_open));
            process::exit(code);
        }
        None => {
            use clap::CommandFactory;
            Cli::command().print_help().ok();
            println!();
        }
    }
}
