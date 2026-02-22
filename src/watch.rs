use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};

use crate::graph::{self, GraphCache};
use crate::model::Spec;
use crate::{parser, semantic};

// ANSI color codes
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

/// Run watch mode on a directory. Returns exit code.
pub fn run_watch(dir: &Path) -> i32 {
    if !dir.exists() || !dir.is_dir() {
        eprintln!("error: {} is not a directory", dir.display());
        return 1;
    }

    // Initial validation + graph build
    let mut cache = initial_validate(dir);

    // Save initial graph
    let graph_path = graph::graph_json_path_cwd();
    if let Err(e) = cache.save(&graph_path) {
        eprintln!("warning: failed to save initial graph: {}", e);
    }

    println!("{}watching {}{}", CYAN, dir.display(), RESET);
    let _ = std::io::stdout().flush();

    // Set up SIGINT handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("failed to set SIGINT handler");

    // Set up file watcher with debouncer
    let (tx, rx) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(300), tx)
        .expect("failed to create file watcher");

    debouncer
        .watcher()
        .watch(dir, notify::RecursiveMode::NonRecursive)
        .expect("failed to watch directory");

    // Main event loop
    while running.load(Ordering::SeqCst) {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(events)) => {
                let spec_events: Vec<_> = events
                    .iter()
                    .filter(|e| {
                        e.kind == DebouncedEventKind::Any
                            && e.path
                                .extension()
                                .and_then(|ext| ext.to_str())
                                .is_some_and(|ext| ext == "spec")
                    })
                    .collect();

                if spec_events.is_empty() {
                    continue;
                }

                // Determine what changed
                let mut changed_files = HashSet::new();
                let mut deleted_files = HashSet::new();
                let mut new_files = HashSet::new();

                for event in &spec_events {
                    let path = &event.path;
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    if name.is_empty() {
                        continue;
                    }

                    if !path.exists() {
                        deleted_files.insert(name);
                    } else if cache.specs.contains_key(&name) {
                        changed_files.insert(name);
                    } else {
                        new_files.insert(name);
                    }
                }

                // Handle deleted files
                for name in &deleted_files {
                    println!("{}deleted: {}.spec{}", RED, name, RESET);
                    let _ = std::io::stdout().flush();
                    cache.specs.remove(name);

                    // Check for broken dependencies
                    let dependents: Vec<String> = cache
                        .specs
                        .iter()
                        .filter(|(_, entry)| entry.dependencies.contains(name))
                        .map(|(n, _)| n.clone())
                        .collect();

                    for dep_name in &dependents {
                        eprintln!(
                            "broken dependency: {} depends on missing spec {}",
                            dep_name, name
                        );
                        let _ = std::io::stderr().flush();
                    }
                }

                // Handle new files
                for name in &new_files {
                    let path = dir.join(format!("{}.spec", name));
                    println!("{}detected new file: {}.spec{}", CYAN, name, RESET);
                    let _ = std::io::stdout().flush();
                    validate_and_cache_spec(&path, name, dir, &mut cache);
                }

                // Handle changed files
                for name in &changed_files {
                    let path = dir.join(format!("{}.spec", name));
                    println!("{}changed: {}.spec{}", YELLOW, name, RESET);
                    let _ = std::io::stdout().flush();
                    validate_and_cache_spec(&path, name, dir, &mut cache);

                    // Re-validate dependents
                    let dependents: Vec<String> = cache
                        .specs
                        .iter()
                        .filter(|(n, entry)| {
                            *n != name && entry.dependencies.contains(name)
                        })
                        .map(|(n, _)| n.clone())
                        .collect();

                    for dep_name in &dependents {
                        let dep_path = dir.join(format!("{}.spec", dep_name));
                        if dep_path.exists() {
                            validate_and_cache_spec(
                                &dep_path,
                                &dep_name,
                                dir,
                                &mut cache,
                            );
                        }
                    }
                }

                let _ = std::io::stdout().flush();
                let _ = std::io::stderr().flush();

                // Save graph after processing events
                if let Err(e) = cache.save(&graph_path) {
                    eprintln!("warning: failed to save graph: {}", e);
                }
            }
            Ok(Err(errors)) => {
                eprintln!("watch error: {}", errors);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    // Graceful shutdown: save graph before exit
    if let Err(e) = cache.save(&graph_path) {
        eprintln!("warning: failed to save graph on shutdown: {}", e);
    }

    0
}

/// Perform initial validation of all specs in the directory and build graph.
fn initial_validate(dir: &Path) -> GraphCache {
    let mut cache = match GraphCache::load(&graph::graph_json_path_cwd()) {
        Ok(c) => c,
        Err(_) => GraphCache::new(),
    };

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return cache,
    };

    let spec_files: Vec<_> = entries
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

    for path in &spec_files {
        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
            validate_and_cache_spec(path, name, dir, &mut cache);
        }
    }

    cache
}

/// Validate a single spec file and update the cache.
fn validate_and_cache_spec(
    path: &Path,
    name: &str,
    dir: &Path,
    cache: &mut GraphCache,
) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error reading {}: {}", path.display(), e);
            return;
        }
    };

    let hash = graph::content_hash(&source);

    // Skip if unchanged
    if !cache.is_changed(name, &hash) {
        return;
    }

    let spec = match parser::parse(&source) {
        Ok(s) => s,
        Err(errors) => {
            println!("{}✗{} {}", RED, RESET, name);
            let _ = std::io::stdout().flush();
            for e in &errors {
                eprintln!("{}: {}", path.display(), e);
            }
            // Store the hash so the next change is detected
            cache.upsert(
                name.to_string(),
                hash,
                String::new(),
                0,
                false,
                vec![],
            );
            return;
        }
    };

    let valid = semantic::validate(&spec).is_ok();
    if !valid {
        print_failure(&spec);
    } else {
        print_success(&spec, dir);
    }

    let dep_names: Vec<String> = spec
        .dependencies
        .iter()
        .map(|d| d.spec_name.clone())
        .collect();

    cache.upsert(
        name.to_string(),
        hash,
        spec.version.clone(),
        spec.behaviors.len(),
        valid,
        dep_names,
    );
}

fn print_success(spec: &Spec, dir: &Path) {
    let count = spec.behaviors.len();
    let label = if count == 1 {
        "1 behavior".to_string()
    } else {
        format!("{} behaviors", count)
    };
    println!("{}✓{} {} v{} ({})", GREEN, RESET, spec.name, spec.version, label);

    // Print dependency tree for deps that exist
    if !spec.dependencies.is_empty() {
        let siblings = discover_siblings(dir);
        let mut seen = HashSet::new();
        seen.insert(spec.name.clone());
        for (i, dep) in spec.dependencies.iter().enumerate() {
            let is_last = i == spec.dependencies.len() - 1;
            let connector = if is_last { "└── " } else { "├── " };
            if let Some(dep_path) = siblings.get(&dep.spec_name) {
                if let Ok(dep_source) = fs::read_to_string(dep_path) {
                    if let Ok(dep_spec) = parser::parse(&dep_source) {
                        let valid = semantic::validate(&dep_spec).is_ok();
                        let mark = if valid {
                            format!("{}✓{}", GREEN, RESET)
                        } else {
                            format!("{}✗{}", RED, RESET)
                        };
                        let dep_count = dep_spec.behaviors.len();
                        let dep_label = if dep_count == 1 {
                            "1 behavior".to_string()
                        } else {
                            format!("{} behaviors", dep_count)
                        };
                        println!(
                            "{}{} {} v{} ({})",
                            connector, mark, dep_spec.name, dep_spec.version, dep_label
                        );
                    }
                }
            } else {
                println!("{}{}✗{} {} (unresolved)", connector, RED, RESET, dep.spec_name);
            }
        }
    }
    let _ = std::io::stdout().flush();
}

fn print_failure(spec: &Spec) {
    println!("{}✗{} {} v{}", RED, RESET, spec.name, spec.version);
    let _ = std::io::stdout().flush();
}

fn discover_siblings(dir: &Path) -> HashMap<String, std::path::PathBuf> {
    let mut map = HashMap::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("spec") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    map.insert(stem.to_string(), path);
                }
            }
        }
    }
    map
}
