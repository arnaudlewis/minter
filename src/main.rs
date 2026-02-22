use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use clap::{Parser, Subcommand};

use specval::graph::{self, GraphCache};
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
    /// Validate one or more .spec files or directories
    Validate {
        /// Spec files or directories to validate
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Also resolve and validate dependencies
        #[arg(long)]
        deps: bool,
    },
    /// Watch a directory for spec file changes and validate incrementally
    Watch {
        /// Directory to watch
        #[arg(required = true)]
        dir: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Validate { files, deps }) => {
            process::exit(run_validate(&files, deps));
        }
        Some(Commands::Watch { dir }) => {
            process::exit(specval::watch::run_watch(&dir));
        }
        None => {
            use clap::CommandFactory;
            Cli::command().print_help().ok();
            println!();
        }
    }
}

/// Validate files/directories and return exit code (0 = success, 1 = failure).
fn run_validate(files: &[PathBuf], check_deps: bool) -> i32 {
    let mut any_failed = false;
    let mut seen = HashSet::new();

    for file in files {
        if file.is_dir() {
            if !validate_directory(file, check_deps, &mut seen) {
                any_failed = true;
            }
        } else {
            // For single files with --deps, manage graph at CWD
            let mut graph_state: Option<GraphState> = None;
            if check_deps {
                graph_state = Some(load_or_build_graph());
            }

            if !validate_one(file, check_deps, &mut seen, graph_state.as_mut()) {
                any_failed = true;
            }

            // Save graph if modified
            if let Some(state) = graph_state {
                if state.dirty {
                    let graph_path = graph::graph_json_path_cwd();
                    if let Err(e) = state.cache.save(&graph_path) {
                        eprintln!("warning: failed to save graph cache: {}", e);
                    }
                }
            }
        }
    }

    if any_failed { 1 } else { 0 }
}

/// Validate all .spec files in a directory.
fn validate_directory(dir: &Path, check_deps: bool, seen: &mut HashSet<String>) -> bool {
    if !dir.exists() {
        eprintln!("error: directory not found: {}", dir.display());
        return false;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("error: cannot read directory {}: {}", dir.display(), e);
            return false;
        }
    };

    let mut spec_files: Vec<PathBuf> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("spec") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    if spec_files.is_empty() {
        eprintln!("error: no .spec files found in {}", dir.display());
        return false;
    }

    spec_files.sort();

    // Load or build graph cache when --deps is used
    let mut graph_state: Option<GraphState> = None;
    if check_deps {
        graph_state = Some(load_or_build_graph());
    }

    let mut any_failed = false;
    for file in &spec_files {
        if !validate_one(file, check_deps, seen, graph_state.as_mut()) {
            any_failed = true;
        }
    }

    // Prune stale entries from graph (specs that no longer exist on disk)
    if let Some(ref mut state) = graph_state {
        let on_disk: HashSet<String> = spec_files
            .iter()
            .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(String::from))
            .collect();
        let stale: Vec<String> = state
            .cache
            .specs
            .keys()
            .filter(|name| !on_disk.contains(name.as_str()))
            .cloned()
            .collect();
        for name in stale {
            state.cache.specs.remove(&name);
            state.dirty = true;
        }
    }

    // Save graph if it was modified
    if let Some(state) = graph_state {
        if state.dirty {
            let graph_path = graph::graph_json_path_cwd();
            if let Err(e) = state.cache.save(&graph_path) {
                eprintln!("warning: failed to save graph cache: {}", e);
            }
        }
    }

    !any_failed
}

struct GraphState {
    cache: GraphCache,
    dirty: bool,
}

fn load_or_build_graph() -> GraphState {
    let graph_path = graph::graph_json_path_cwd();
    if graph_path.exists() {
        match GraphCache::load(&graph_path) {
            Ok(cache) => {
                return GraphState {
                    cache,
                    dirty: false,
                };
            }
            Err(graph::GraphError::Corrupted(msg)) => {
                eprintln!(
                    "warning: cached graph is corrupt ({}), rebuilding from scratch",
                    msg
                );
            }
            Err(graph::GraphError::SchemaMismatch) => {
                eprintln!(
                    "warning: cached graph has incompatible format, rebuilding from scratch"
                );
            }
            Err(graph::GraphError::Io(_)) => {}
        }
    }
    GraphState {
        cache: GraphCache::new(),
        dirty: true, // new graph needs saving
    }
}

/// Validate a single file. Returns true if valid, false if invalid.
fn validate_one(
    path: &Path,
    check_deps: bool,
    seen: &mut HashSet<String>,
    graph_state: Option<&mut GraphState>,
) -> bool {
    let filename = path.display();

    // Check if path exists
    if !path.exists() {
        eprintln!("error: cannot read {}: No such file or directory", filename);
        return false;
    }

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
        print_failure(&spec);
        for e in &errors {
            eprintln!("{}: {}", filename, e);
        }
        return false;
    }

    // Dependency resolution
    if check_deps {
        let dir = path.parent().unwrap_or(Path::new("."));
        let siblings = discover_siblings(dir, path);
        let mut resolved: HashMap<String, ResolvedDep> = HashMap::new();
        let mut errors = Vec::new();
        let mut stack: Vec<String> = vec![spec.name.clone()];

        resolve_and_collect(
            &spec.dependencies,
            dir,
            &siblings,
            &mut resolved,
            &mut stack,
            &mut errors,
        );

        // Update graph cache with this spec and its resolved deps
        if let Some(state) = graph_state {
            let hash = graph::content_hash(&source);
            let dep_names: Vec<String> = spec
                .dependencies
                .iter()
                .map(|d| d.spec_name.clone())
                .collect();

            if state.cache.is_changed(&spec.name, &hash) {
                state.cache.upsert(
                    spec.name.clone(),
                    hash,
                    spec.version.clone(),
                    spec.behaviors.len(),
                    errors.is_empty(),
                    dep_names,
                );
                state.dirty = true;
            }

            // Also cache resolved deps
            for (dep_name, rd) in &resolved {
                if let Some(dep_path) = siblings.get(dep_name) {
                    if let Ok(dep_source) = fs::read_to_string(dep_path) {
                        let dep_hash = graph::content_hash(&dep_source);
                        if state.cache.is_changed(dep_name, &dep_hash) {
                            let dep_dep_names: Vec<String> = rd
                                .spec
                                .dependencies
                                .iter()
                                .map(|d| d.spec_name.clone())
                                .collect();
                            state.cache.upsert(
                                dep_name.clone(),
                                dep_hash,
                                rd.spec.version.clone(),
                                rd.spec.behaviors.len(),
                                rd.valid,
                                dep_dep_names,
                            );
                            state.dirty = true;
                        }
                    }
                }
            }
        }

        // Skip entirely if already shown in another spec's tree
        if !seen.contains(&spec.name) {
            print_success(&spec);
            seen.insert(spec.name.clone());
            let shallowest = compute_shallowest_depths(&spec.dependencies, &resolved);
            print_dep_tree(&spec.dependencies, &resolved, seen, "", 1, &shallowest);
        }

        if !errors.is_empty() {
            for e in &errors {
                eprintln!("{}", e);
            }
            return false;
        }

        return true;
    }

    // Success — print result line
    print_success(&spec);
    true
}

fn behavior_count_label(count: usize) -> String {
    if count == 1 {
        "1 behavior".to_string()
    } else {
        format!("{} behaviors", count)
    }
}

fn print_success(spec: &Spec) {
    println!(
        "✓ {} v{} ({})",
        spec.name,
        spec.version,
        behavior_count_label(spec.behaviors.len()),
    );
}

fn print_failure(spec: &Spec) {
    println!("✗ {} v{}", spec.name, spec.version);
}

// ═══════════════════════════════════════════════════════════════
// Dependency tree resolution and display
// ═══════════════════════════════════════════════════════════════

struct ResolvedDep {
    spec: Spec,
    valid: bool,
}

fn resolve_and_collect(
    deps: &[specval::model::Dependency],
    dir: &Path,
    siblings: &HashMap<String, PathBuf>,
    resolved: &mut HashMap<String, ResolvedDep>,
    stack: &mut Vec<String>,
    errors: &mut Vec<String>,
) {
    for dep in deps {
        if stack.contains(&dep.spec_name) {
            errors.push(format!(
                "dependency cycle detected: {} → {}",
                stack.join(" → "),
                dep.spec_name
            ));
            continue;
        }

        if resolved.contains_key(&dep.spec_name) {
            check_version_constraint(dep, &resolved[&dep.spec_name].spec, errors);
            continue;
        }

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

        let valid = semantic::validate(&dep_spec).is_ok();
        if !valid {
            errors.push(format!(
                "dependency '{}' has validation errors",
                dep.spec_name
            ));
        }

        check_version_constraint(dep, &dep_spec, errors);

        let sub_deps = dep_spec.dependencies.clone();
        resolved.insert(dep.spec_name.clone(), ResolvedDep { spec: dep_spec, valid });
        stack.push(dep.spec_name.clone());

        resolve_and_collect(&sub_deps, dir, siblings, resolved, stack, errors);

        stack.pop();
    }
}

/// Compute the shallowest depth at which each dependency name appears in the tree.
fn compute_shallowest_depths(
    deps: &[specval::model::Dependency],
    resolved: &HashMap<String, ResolvedDep>,
) -> HashMap<String, usize> {
    let mut depths: HashMap<String, usize> = HashMap::new();
    let mut visited = HashSet::new();
    compute_depths_recursive(deps, resolved, 1, &mut depths, &mut visited);
    depths
}

fn compute_depths_recursive(
    deps: &[specval::model::Dependency],
    resolved: &HashMap<String, ResolvedDep>,
    depth: usize,
    depths: &mut HashMap<String, usize>,
    visited: &mut HashSet<String>,
) {
    for dep in deps {
        let entry = depths.entry(dep.spec_name.clone()).or_insert(depth);
        if depth < *entry {
            *entry = depth;
        }
        if visited.contains(&dep.spec_name) {
            continue;
        }
        visited.insert(dep.spec_name.clone());
        if let Some(rd) = resolved.get(&dep.spec_name) {
            if !rd.spec.dependencies.is_empty() {
                compute_depths_recursive(
                    &rd.spec.dependencies,
                    resolved,
                    depth + 1,
                    depths,
                    visited,
                );
            }
        }
    }
}

fn print_dep_tree(
    deps: &[specval::model::Dependency],
    resolved: &HashMap<String, ResolvedDep>,
    seen: &mut HashSet<String>,
    prefix: &str,
    depth: usize,
    shallowest: &HashMap<String, usize>,
) {
    for (i, dep) in deps.iter().enumerate() {
        let is_last = i == deps.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };

        if let Some(rd) = resolved.get(&dep.spec_name) {
            let mark = if rd.valid { "✓" } else { "✗" };
            let is_shallowest = shallowest.get(&dep.spec_name).copied().unwrap_or(depth) == depth;

            if is_shallowest && !seen.contains(&dep.spec_name) {
                seen.insert(dep.spec_name.clone());
                println!(
                    "{}{}{} {} v{} ({})",
                    prefix,
                    connector,
                    mark,
                    rd.spec.name,
                    rd.spec.version,
                    behavior_count_label(rd.spec.behaviors.len()),
                );
                if !rd.spec.dependencies.is_empty() {
                    print_dep_tree(&rd.spec.dependencies, resolved, seen, &child_prefix, depth + 1, shallowest);
                }
            } else {
                println!(
                    "{}{}{} \x1b[2m{} v{}\x1b[0m",
                    prefix, connector, mark, rd.spec.name, rd.spec.version
                );
            }
        } else {
            println!("{}{}✗ {} (unresolved)", prefix, connector, dep.spec_name);
        }
    }
}

fn check_version_constraint(
    dep: &specval::model::Dependency,
    dep_spec: &Spec,
    errors: &mut Vec<String>,
) {
    let constraint = &dep.version_constraint;
    let required = constraint.trim_start_matches(">=").trim();

    let req = match semver::Version::parse(required) {
        Ok(v) => v,
        Err(_) => return,
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
