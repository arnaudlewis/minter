use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use clap::{Parser, Subcommand};

use specval::model::Spec;
use specval::parser;
use specval::semantic;

#[derive(Parser)]
#[command(name = "specval", version, about = "Spec compiler & validator")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate one or more .spec files
    Validate {
        /// Spec files to validate
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Also resolve and validate dependencies
        #[arg(long)]
        deps: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Validate { files, deps }) => {
            process::exit(run_validate(&files, deps));
        }
        None => {
            // Print help when no subcommand given
            use clap::CommandFactory;
            Cli::command().print_help().ok();
            println!();
        }
    }
}

/// Validate files and return exit code (0 = success, 1 = failure).
fn run_validate(files: &[PathBuf], check_deps: bool) -> i32 {
    let mut any_failed = false;

    for file in files {
        if !validate_one(file, check_deps) {
            any_failed = true;
        }
    }

    if any_failed { 1 } else { 0 }
}

/// Validate a single file. Returns true if valid, false if invalid.
fn validate_one(path: &Path, check_deps: bool) -> bool {
    let filename = path.display();

    // Check extension
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "spec" {
        eprintln!("error: {} does not have a .spec extension", filename);
        return false;
    }

    // Read file
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("ermission") {
                eprintln!("error: permission denied reading {}", filename);
            } else {
                eprintln!("error: cannot read {}: {}", filename, e);
            }
            return false;
        }
    };

    // Parse
    let spec = match parser::parse(&source) {
        Ok(spec) => spec,
        Err(errors) => {
            for e in &errors {
                eprintln!("{}: {}", filename, e);
            }
            return false;
        }
    };

    // Semantic validation
    if let Err(errors) = semantic::validate(&spec) {
        for e in &errors {
            eprintln!("{}: {}", filename, e);
        }
        return false;
    }

    // Dependency resolution
    if check_deps {
        if !resolve_deps(path, &spec) {
            return false;
        }
    }

    // Success — print summary
    print_summary(&spec);
    true
}

fn print_summary(spec: &Spec) {
    println!(
        "{} v{} — valid ({} behaviors, {} dependencies)",
        spec.name,
        spec.version,
        spec.behaviors.len(),
        spec.dependencies.len(),
    );
    if spec.dependencies.is_empty() {
        println!("  no dependencies");
    }
}

/// Resolve dependencies for a spec by looking for sibling .spec files.
fn resolve_deps(spec_path: &Path, spec: &Spec) -> bool {
    if spec.dependencies.is_empty() {
        return true;
    }

    let dir = spec_path.parent().unwrap_or(Path::new("."));

    // Build map of available sibling specs (name → path)
    let siblings = discover_siblings(dir, spec_path);

    let mut errors = Vec::new();
    let mut resolved: HashMap<String, Spec> = HashMap::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = Vec::new();

    visited.insert(spec.name.clone());
    stack.push(spec.name.clone());
    resolved.insert(spec.name.clone(), spec.clone());

    resolve_recursive(
        &spec.dependencies,
        dir,
        &siblings,
        &mut resolved,
        &mut visited,
        &mut stack,
        &mut errors,
    );

    stack.pop();

    if errors.is_empty() {
        println!("  all dependencies resolved");
        true
    } else {
        for e in &errors {
            eprintln!("{}", e);
        }
        false
    }
}

fn resolve_recursive(
    deps: &[specval::model::Dependency],
    dir: &Path,
    siblings: &HashMap<String, PathBuf>,
    resolved: &mut HashMap<String, Spec>,
    visited: &mut HashSet<String>,
    stack: &mut Vec<String>,
    errors: &mut Vec<String>,
) {
    for dep in deps {
        // Check for cycle
        if stack.contains(&dep.spec_name) {
            errors.push(format!(
                "dependency cycle detected: {} → {}",
                stack.join(" → "),
                dep.spec_name
            ));
            continue;
        }

        // Already resolved successfully
        if resolved.contains_key(&dep.spec_name) {
            // Still check version
            check_version_constraint(dep, &resolved[&dep.spec_name], errors);
            continue;
        }

        // Find the sibling file
        let sibling_path = match siblings.get(&dep.spec_name) {
            Some(p) => p.clone(),
            None => {
                errors.push(format!(
                    "dependency '{}' not found (no {}.spec in sibling directory)",
                    dep.spec_name, dep.spec_name
                ));
                continue;
            }
        };

        // Parse + validate the dependency
        let source = match fs::read_to_string(&sibling_path) {
            Ok(s) => s,
            Err(e) => {
                errors.push(format!(
                    "cannot read dependency '{}': {}",
                    dep.spec_name, e
                ));
                continue;
            }
        };

        let dep_spec = match parser::parse(&source) {
            Ok(s) => s,
            Err(_) => {
                errors.push(format!(
                    "dependency '{}' has parse errors",
                    dep.spec_name
                ));
                continue;
            }
        };

        if let Err(_) = semantic::validate(&dep_spec) {
            errors.push(format!(
                "dependency '{}' has validation errors",
                dep.spec_name
            ));
            continue;
        }

        // Check version constraint
        check_version_constraint(dep, &dep_spec, errors);

        // Record and recurse
        let sub_deps = dep_spec.dependencies.clone();
        resolved.insert(dep.spec_name.clone(), dep_spec);
        visited.insert(dep.spec_name.clone());
        stack.push(dep.spec_name.clone());

        resolve_recursive(&sub_deps, dir, siblings, resolved, visited, stack, errors);

        stack.pop();
    }
}

fn check_version_constraint(
    dep: &specval::model::Dependency,
    dep_spec: &Spec,
    errors: &mut Vec<String>,
) {
    // Parse the constraint ">=X.Y.Z" — extract version from constraint string
    let constraint = &dep.version_constraint;
    let required = constraint.trim_start_matches(">=").trim();

    let req = match semver::Version::parse(required) {
        Ok(v) => v,
        Err(_) => return, // Can't check, skip
    };

    let actual = match semver::Version::parse(&dep_spec.version) {
        Ok(v) => v,
        Err(_) => return,
    };

    if actual < req {
        errors.push(format!(
            "dependency '{}' requires >= {} but found {}",
            dep.spec_name, required, dep_spec.version
        ));
    }
}

fn discover_siblings(dir: &Path, exclude: &Path) -> HashMap<String, PathBuf> {
    let mut map = HashMap::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path == exclude {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) == Some("spec") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    map.insert(stem.to_string(), path);
                }
            }
        }
    }
    map
}
