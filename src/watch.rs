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
use crate::model::Spec;
use crate::{parser, semantic};

/// Run watch mode on a file or directory. Returns exit code.
pub fn run_watch(path: &Path) -> i32 {
    if !path.exists() {
        eprintln!("error: {} does not exist", path.display());
        return 1;
    }

    // Check read permissions
    if path.is_dir() {
        if fs::read_dir(path).is_err() {
            eprintln!("error: permission denied: {}", path.display());
            return 1;
        }
    } else if path.is_file() {
        if fs::read_to_string(path).is_err() {
            eprintln!("error: permission denied: {}", path.display());
            return 1;
        }
    }

    // Determine the watch directory and optional single-file filter
    let (dir, single_file) = if path.is_file() {
        let parent = path.parent().unwrap_or(Path::new("."));
        (parent.to_path_buf(), Some(path.to_path_buf()))
    } else {
        (path.to_path_buf(), None)
    };

    let mut cache = initial_validate(&dir);

    // Check for empty directory (no spec files found)
    if cache.specs.is_empty() {
        eprintln!("error: no spec files found in {}", path.display());
        return 1;
    }

    let graph_path = graph::graph_json_path_cwd();
    if let Err(e) = cache.save(&graph_path) {
        eprintln!("warning: failed to save initial graph: {}", e);
    }

    if single_file.is_some() {
        println!("{}watching {}{}", CYAN, path.display(), RESET);
    } else {
        println!("{}watching {}{}", CYAN, dir.display(), RESET);
    }
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
        .watch(&dir, notify::RecursiveMode::Recursive)
        .expect("failed to watch directory");

    while running.load(Ordering::SeqCst) {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(events)) => {
                let spec_events: Vec<_> = events
                    .iter()
                    .filter(|e| {
                        if e.kind != DebouncedEventKind::Any {
                            return false;
                        }
                        let is_spec = e.path
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .is_some_and(|ext| ext == "spec");
                        if !is_spec {
                            return false;
                        }
                        // If watching a single file, filter to that file and its dependents
                        if let Some(ref sf) = single_file {
                            return e.path == *sf || is_dependent_of(&e.path, sf, &cache);
                        }
                        true
                    })
                    .collect();

                if !spec_events.is_empty() {
                    handle_watch_events(&spec_events, &dir, &mut cache);

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

/// Check if a changed path is a dependent of the single watched file.
fn is_dependent_of(changed: &Path, watched: &Path, cache: &GraphCache) -> bool {
    let watched_name = watched
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let changed_name = changed
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if watched_name.is_empty() || changed_name.is_empty() {
        return false;
    }
    if let Some(entry) = cache.specs.get(changed_name) {
        return entry.dependencies.contains(&watched_name.to_string());
    }
    false
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

    let spec_files = match discover::discover_spec_files(dir) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("error: {}", e);
            return cache;
        }
    };

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

    let entry = match parse_and_validate(path, &source, dir) {
        Some((spec, valid)) => CachedEntry {
            content_hash: hash,
            version: spec.version.clone(),
            behavior_count: spec.behaviors.len(),
            valid,
            dependencies: spec.dep_names(),
            path: path.display().to_string(),
        },
        None => CachedEntry {
            content_hash: hash,
            version: String::new(),
            behavior_count: 0,
            valid: false,
            dependencies: vec![],
            path: path.display().to_string(),
        },
    };
    cache.upsert(name.to_string(), entry);
}

/// Parse and validate a spec file. Returns (spec, valid) or None on parse failure.
fn parse_and_validate(path: &Path, source: &str, dir: &Path) -> Option<(Spec, bool)> {
    let spec = match parser::parse(source) {
        Ok(s) => s,
        Err(errors) => {
            println!("{}\u{2717}{} {}", RED, RESET, path.display());
            let _ = std::io::stdout().flush();
            for e in &errors {
                eprintln!("{}: {}", path.display(), e);
            }
            return None;
        }
    };

    let valid = semantic::validate(&spec).is_ok();
    if valid {
        print_success_with_deps(&spec, dir);
    } else {
        display::print_failure(&spec);
    }
    Some((spec, valid))
}

/// Print success with colored output and shallow 1-level dep tree (watch-specific).
fn print_success_with_deps(spec: &Spec, dir: &Path) {
    display::print_success(spec);

    if !spec.dependencies.is_empty() {
        let color = display::use_color();
        let siblings = discover::discover_specs(dir, None);
        let mut seen = HashSet::new();
        seen.insert(spec.name.clone());
        for (i, dep) in spec.dependencies.iter().enumerate() {
            let is_last = i == spec.dependencies.len() - 1;
            let connector = display::tree_connector(is_last);
            if let Some(dep_path) = siblings.get(&dep.spec_name) {
                if let Ok(dep_source) = fs::read_to_string(dep_path)
                    && let Ok(dep_spec) = parser::parse(&dep_source) {
                        let valid = semantic::validate(&dep_spec).is_ok();
                        let mark = if color {
                            if valid {
                                format!("{}\u{2713}{}", GREEN, RESET)
                            } else {
                                format!("{}\u{2717}{}", RED, RESET)
                            }
                        } else if valid {
                            "\u{2713}".to_string()
                        } else {
                            "\u{2717}".to_string()
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
                if color {
                    println!(
                        "{}{}\u{2717}{} {} (unresolved)",
                        connector, RED, RESET, dep.spec_name
                    );
                } else {
                    println!(
                        "{}\u{2717} {} (unresolved)",
                        connector, dep.spec_name
                    );
                }
            }
        }
    }
    let _ = std::io::stdout().flush();
}
