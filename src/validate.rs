use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::deps::{self, ResolutionContext};
use crate::display::{self, TreeContext};
use crate::graph::{self, CachedEntry, GraphState};
use crate::{discover, parser, semantic};

pub struct ValidationContext<'a> {
    pub check_deps: bool,
    pub seen: &'a mut HashSet<String>,
    pub graph_state: Option<&'a mut GraphState>,
}

/// Validate files/directories and return exit code (0 = success, 1 = failure).
pub fn run_validate(files: &[std::path::PathBuf], check_deps: bool) -> i32 {
    let mut any_failed = false;
    let mut seen = HashSet::new();

    for file in files {
        if file.is_dir() {
            if !validate_directory(file, check_deps, &mut seen) {
                any_failed = true;
            }
        } else {
            let mut graph_state: Option<GraphState> = None;
            if check_deps {
                graph_state = Some(GraphState::load_or_build());
            }

            let tree_root = file.parent().unwrap_or(Path::new(".")).to_path_buf();
            let mut ctx = ValidationContext {
                check_deps,
                seen: &mut seen,
                graph_state: graph_state.as_mut(),
            };
            if !validate_one(file, &tree_root, &mut ctx) {
                any_failed = true;
            }

            if let Some(state) = graph_state {
                state.save_if_dirty();
            }
        }
    }

    if any_failed { 1 } else { 0 }
}

/// Validate all .spec files in a directory (recursively).
fn validate_directory(dir: &Path, check_deps: bool, seen: &mut HashSet<String>) -> bool {
    if !dir.exists() {
        eprintln!("error: directory not found: {}", dir.display());
        return false;
    }

    let spec_files = match discover::discover_spec_files(dir) {
        Ok(files) => files,
        Err(dup_err) => {
            eprintln!("error: {}", dup_err);
            return false;
        }
    };

    if spec_files.is_empty() {
        eprintln!("error: no .spec files found in {}", dir.display());
        return false;
    }

    let mut graph_state: Option<GraphState> = None;
    if check_deps {
        graph_state = Some(GraphState::load_or_build());
    }

    let tree_root = dir.to_path_buf();
    let mut any_failed = false;
    for file in &spec_files {
        let mut ctx = ValidationContext {
            check_deps,
            seen,
            graph_state: graph_state.as_mut(),
        };
        if !validate_one(file, &tree_root, &mut ctx) {
            any_failed = true;
        }
    }

    if let Some(ref mut state) = graph_state {
        let on_disk: HashSet<String> = spec_files
            .iter()
            .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(String::from))
            .collect();
        state.prune_stale(&on_disk);
    }

    if let Some(state) = graph_state {
        state.save_if_dirty();
    }

    !any_failed
}

/// Validate a single file. Returns true if valid, false if invalid.
fn validate_one(path: &Path, spec_tree_root: &Path, ctx: &mut ValidationContext) -> bool {
    let (source, spec) = match read_and_parse(path) {
        Ok(result) => result,
        Err(_) => return false,
    };

    if let Err(errors) = semantic::validate(&spec) {
        display::print_failure(&spec);
        let filename = path.display();
        for e in &errors {
            eprintln!("{}: {}", filename, e);
        }
        return false;
    }

    if ctx.check_deps {
        return validate_with_deps(path, &source, &spec, spec_tree_root, ctx);
    }

    display::print_success(&spec);
    true
}

fn read_and_parse(path: &Path) -> Result<(String, crate::model::Spec), ()> {
    let filename = path.display();

    if !path.exists() {
        eprintln!("error: cannot read {}: No such file or directory", filename);
        return Err(());
    }

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "spec" {
        eprintln!("error: {} does not have a .spec extension", filename);
        return Err(());
    }

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("ermission") {
                eprintln!("error: permission denied reading {}", filename);
            } else {
                eprintln!("error: cannot read {}: {}", filename, e);
            }
            return Err(());
        }
    };

    let spec = match parser::parse(&source) {
        Ok(spec) => spec,
        Err(errors) => {
            for e in &errors {
                eprintln!("{}: {}", filename, e);
            }
            return Err(());
        }
    };

    Ok((source, spec))
}

fn validate_with_deps(
    path: &Path,
    source: &str,
    spec: &crate::model::Spec,
    spec_tree_root: &Path,
    ctx: &mut ValidationContext,
) -> bool {
    let all_specs = discover::discover_specs(spec_tree_root, Some(path));
    let mut res_ctx = ResolutionContext {
        siblings: all_specs.clone(),
        resolved: HashMap::new(),
        stack: vec![spec.name.clone()],
        errors: Vec::new(),
    };

    deps::resolve_and_collect(&spec.dependencies, &mut res_ctx);

    update_graph_cache(path, spec, source, &all_specs, &res_ctx, ctx);

    if !ctx.seen.contains(&spec.name) {
        display::print_success(spec);
        ctx.seen.insert(spec.name.clone());
        let shallowest = display::compute_shallowest_depths(&spec.dependencies, &res_ctx.resolved);
        let mut tree_ctx = TreeContext {
            resolved: &res_ctx.resolved,
            seen: ctx.seen,
            shallowest: &shallowest,
        };
        display::print_dep_tree(&spec.dependencies, &mut tree_ctx, "", 1);
    }

    if !res_ctx.errors.is_empty() {
        for e in &res_ctx.errors {
            eprintln!("{}", e);
        }
        return false;
    }

    true
}

fn update_graph_cache(
    path: &Path,
    spec: &crate::model::Spec,
    source: &str,
    all_specs: &HashMap<String, std::path::PathBuf>,
    res_ctx: &ResolutionContext,
    ctx: &mut ValidationContext,
) {
    let state = match ctx.graph_state.as_mut() {
        Some(s) => s,
        None => return,
    };

    let hash = graph::content_hash(source);
    let dep_names: Vec<String> = spec
        .dependencies
        .iter()
        .map(|d| d.spec_name.clone())
        .collect();

    if state.cache.is_changed(&spec.name, &hash) {
        state.cache.upsert(
            spec.name.clone(),
            CachedEntry {
                content_hash: hash,
                version: spec.version.clone(),
                behavior_count: spec.behaviors.len(),
                valid: res_ctx.errors.is_empty(),
                dependencies: dep_names,
                path: path.display().to_string(),
            },
        );
        state.dirty = true;
    }

    for (dep_name, rd) in &res_ctx.resolved {
        if let Some(dep_path) = all_specs.get(dep_name)
            && let Ok(dep_source) = fs::read_to_string(dep_path) {
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
                        CachedEntry {
                            content_hash: dep_hash,
                            version: rd.spec.version.clone(),
                            behavior_count: rd.spec.behaviors.len(),
                            valid: rd.valid,
                            dependencies: dep_dep_names,
                            path: dep_path.display().to_string(),
                        },
                    );
                    state.dirty = true;
                }
            }
    }
}
