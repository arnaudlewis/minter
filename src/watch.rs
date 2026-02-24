use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};

use crate::discover;
use crate::display::{self, CYAN, GREEN, RED, RESET, YELLOW};
use crate::graph::{self, CachedEntry, GraphCache};
use crate::{parser, semantic};

/// Run watch mode on a directory. Returns exit code.
pub fn run_watch(dir: &Path) -> i32 {
    if !dir.exists() || !dir.is_dir() {
        eprintln!("error: {} is not a directory", dir.display());
        return 1;
    }

    let mut cache = initial_validate(dir);

    let graph_path = graph::graph_json_path_cwd();
    if let Err(e) = cache.save(&graph_path) {
        eprintln!("warning: failed to save initial graph: {}", e);
    }

    println!("{}watching {}{}", CYAN, dir.display(), RESET);
    let _ = std::io::stdout().flush();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("failed to set SIGINT handler");

    let (tx, rx) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(300), tx)
        .expect("failed to create file watcher");

    debouncer
        .watcher()
        .watch(dir, notify::RecursiveMode::Recursive)
        .expect("failed to watch directory");

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

                if !spec_events.is_empty() {
                    handle_watch_events(&spec_events, dir, &mut cache);

                    if let Err(e) = cache.save(&graph_path) {
                        eprintln!("warning: failed to save graph: {}", e);
                    }
                }
            }
            Ok(Err(errors)) => {
                eprintln!("watch error: {}", errors);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    if let Err(e) = cache.save(&graph_path) {
        eprintln!("warning: failed to save graph on shutdown: {}", e);
    }

    0
}

fn handle_watch_events(
    spec_events: &[&notify_debouncer_mini::DebouncedEvent],
    dir: &Path,
    cache: &mut GraphCache,
) {
    let (changed, deleted, new) = classify_events(spec_events, cache);
    handle_deletions(&deleted, cache);
    handle_new_files(&new, dir, cache);
    handle_changes(&changed, dir, cache);
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
}

fn classify_events(
    spec_events: &[&notify_debouncer_mini::DebouncedEvent],
    cache: &GraphCache,
) -> (HashMap<String, std::path::PathBuf>, HashSet<String>, HashMap<String, std::path::PathBuf>) {
    let mut changed_files: HashMap<String, std::path::PathBuf> = HashMap::new();
    let mut deleted_files = HashSet::new();
    let mut new_files: HashMap<String, std::path::PathBuf> = HashMap::new();

    for event in spec_events {
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
            changed_files.insert(name, path.clone());
        } else {
            new_files.insert(name, path.clone());
        }
    }

    (changed_files, deleted_files, new_files)
}

fn handle_deletions(deleted: &HashSet<String>, cache: &mut GraphCache) {
    for name in deleted {
        println!("{}deleted: {}.spec{}", RED, name, RESET);
        let _ = std::io::stdout().flush();
        cache.specs.remove(name);

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
}

fn handle_new_files(new: &HashMap<String, std::path::PathBuf>, dir: &Path, cache: &mut GraphCache) {
    for (name, path) in new {
        println!("{}detected new file: {}.spec{}", CYAN, name, RESET);
        let _ = std::io::stdout().flush();
        validate_and_cache_spec(path, name, dir, cache);
    }
}

fn handle_changes(
    changed: &HashMap<String, std::path::PathBuf>,
    dir: &Path,
    cache: &mut GraphCache,
) {
    for (name, path) in changed {
        println!("{}changed: {}.spec{}", YELLOW, name, RESET);
        let _ = std::io::stdout().flush();
        validate_and_cache_spec(path, name, dir, cache);

        let dependents: Vec<(String, String)> = cache
            .specs
            .iter()
            .filter(|(n, entry)| {
                *n != name && entry.dependencies.contains(name)
            })
            .map(|(n, entry)| (n.clone(), entry.path.clone()))
            .collect();

        for (dep_name, dep_path_str) in &dependents {
            let dep_path = if !dep_path_str.is_empty() {
                std::path::PathBuf::from(dep_path_str)
            } else {
                dir.join(format!("{}.spec", dep_name))
            };
            if dep_path.exists() {
                validate_and_cache_spec(&dep_path, dep_name, dir, cache);
            }
        }
    }
}

/// Perform initial validation of all specs in the directory (recursively) and build graph.
fn initial_validate(dir: &Path) -> GraphCache {
    let mut cache = GraphCache::load(&graph::graph_json_path_cwd()).unwrap_or_default();

    let spec_files: Vec<_> = walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("spec"))
        .map(|e| e.path().to_path_buf())
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

    if !cache.is_changed(name, &hash) {
        return;
    }

    let spec = match parser::parse(&source) {
        Ok(s) => s,
        Err(errors) => {
            println!("{}\u{2717}{} {}", RED, RESET, name);
            let _ = std::io::stdout().flush();
            for e in &errors {
                eprintln!("{}: {}", path.display(), e);
            }
            cache.upsert(
                name.to_string(),
                CachedEntry {
                    content_hash: hash,
                    version: String::new(),
                    behavior_count: 0,
                    valid: false,
                    dependencies: vec![],
                    path: path.display().to_string(),
                },
            );
            return;
        }
    };

    let valid = semantic::validate(&spec).is_ok();
    if !valid {
        display::print_failure_colored(&spec);
    } else {
        print_success_with_deps(&spec, dir);
    }

    let dep_names: Vec<String> = spec
        .dependencies
        .iter()
        .map(|d| d.spec_name.clone())
        .collect();

    cache.upsert(
        name.to_string(),
        CachedEntry {
            content_hash: hash,
            version: spec.version.clone(),
            behavior_count: spec.behaviors.len(),
            valid,
            dependencies: dep_names,
            path: path.display().to_string(),
        },
    );
}

/// Print success with colored output and shallow 1-level dep tree (watch-specific).
fn print_success_with_deps(spec: &crate::model::Spec, dir: &Path) {
    println!(
        "{}\u{2713}{} {} v{} ({})",
        GREEN,
        RESET,
        spec.name,
        spec.version,
        display::behavior_count_label(spec.behaviors.len()),
    );

    if !spec.dependencies.is_empty() {
        let siblings = discover::discover_specs(dir, None);
        let mut seen = HashSet::new();
        seen.insert(spec.name.clone());
        for (i, dep) in spec.dependencies.iter().enumerate() {
            let is_last = i == spec.dependencies.len() - 1;
            let connector = if is_last { "\u{2514}\u{2500}\u{2500} " } else { "\u{251c}\u{2500}\u{2500} " };
            if let Some(dep_path) = siblings.get(&dep.spec_name) {
                if let Ok(dep_source) = fs::read_to_string(dep_path)
                    && let Ok(dep_spec) = parser::parse(&dep_source) {
                        let valid = semantic::validate(&dep_spec).is_ok();
                        let mark = if valid {
                            format!("{}\u{2713}{}", GREEN, RESET)
                        } else {
                            format!("{}\u{2717}{}", RED, RESET)
                        };
                        println!(
                            "{}{} {} v{} ({})",
                            connector,
                            mark,
                            dep_spec.name,
                            dep_spec.version,
                            display::behavior_count_label(dep_spec.behaviors.len()),
                        );
                    }
            } else {
                println!(
                    "{}{}\u{2717}{} {} (unresolved)",
                    connector, RED, RESET, dep.spec_name
                );
            }
        }
    }
    let _ = std::io::stdout().flush();
}
