use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::deps::{self, ResolutionContext};
use crate::display::{self, TreeContext};
use crate::graph::{self, CachedEntry, GraphState};
use crate::model::{NfrSpec, Spec};
use crate::{discover, nfr_crossref, nfr_parser, nfr_semantic, parser, semantic};

pub struct ValidationContext<'a> {
    pub check_deep: bool,
    pub seen: &'a mut HashSet<String>,
    pub graph_state: Option<&'a mut GraphState>,
    pub nfr_specs: Option<&'a HashMap<String, NfrSpec>>,
}

struct ParsedFile {
    path: std::path::PathBuf,
    source: String,
    spec: Spec,
}

/// Validate files/directories and return exit code (0 = success, 1 = failure).
pub fn run_validate(files: &[std::path::PathBuf], check_deep: bool) -> i32 {
    let mut any_failed = false;
    let mut seen = HashSet::new();

    for file in files {
        if file.is_dir() {
            if !validate_directory(file, true, &mut seen) {  // directories are always deep
                any_failed = true;
            }
        } else {
            let mut graph_state: Option<GraphState> = None;
            if check_deep {
                graph_state = Some(GraphState::load_or_build());
            }

            let tree_root = file.parent().unwrap_or(Path::new(".")).to_path_buf();
            let nfr_map = if check_deep {
                Some(discover_and_parse_nfrs(&tree_root))
            } else {
                None
            };
            let mut ctx = ValidationContext {
                check_deep,
                seen: &mut seen,
                graph_state: graph_state.as_mut(),
                nfr_specs: nfr_map.as_ref(),
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

/// Validate all .spec and .nfr files in a directory (recursively).
fn validate_directory(dir: &Path, check_deep: bool, seen: &mut HashSet<String>) -> bool {
    if !dir.exists() {
        eprintln!("error: directory not found: {}", dir.display());
        return false;
    }

    let spec_files = match discover::discover_all_files(dir) {
        Ok(files) => files,
        Err(dup_err) => {
            eprintln!("error: {}", dup_err);
            return false;
        }
    };

    if spec_files.is_empty() {
        eprintln!("error: no .spec or .nfr files found in {}", dir.display());
        return false;
    }

    let mut graph_state: Option<GraphState> = None;
    if check_deep {
        graph_state = Some(GraphState::load_or_build());
    }

    let nfr_map = if check_deep {
        Some(discover_and_parse_nfrs(dir))
    } else {
        None
    };

    let tree_root = dir.to_path_buf();
    let mut any_failed = false;
    for file in &spec_files {
        let mut ctx = ValidationContext {
            check_deep,
            seen,
            graph_state: graph_state.as_mut(),
            nfr_specs: nfr_map.as_ref(),
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
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext == "nfr" {
        return validate_nfr_file(path);
    }

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

    if ctx.check_deep {
        let parsed = ParsedFile {
            path: path.to_path_buf(),
            source,
            spec,
        };
        return validate_with_deps(&parsed, spec_tree_root, ctx);
    }

    display::print_success(&spec);
    true
}

/// Validate a single .nfr file. Returns true if valid, false if invalid.
fn validate_nfr_file(path: &Path) -> bool {
    let filename = path.display();

    if !path.exists() {
        eprintln!("error: cannot read {}: No such file or directory", filename);
        return false;
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
            return false;
        }
    };

    let nfr = match nfr_parser::parse_nfr(&source) {
        Ok(nfr) => nfr,
        Err(errors) => {
            for e in &errors {
                eprintln!("{}: {}", filename, e);
            }
            return false;
        }
    };

    if let Err(errors) = nfr_semantic::validate(&nfr) {
        display::print_nfr_failure(&nfr);
        let filename = path.display();
        for e in &errors {
            eprintln!("{}: {}", filename, e);
        }
        return false;
    }

    display::print_nfr_success(&nfr);
    true
}

fn read_and_parse(path: &Path) -> Result<(String, Spec), ()> {
    let filename = path.display();

    if !path.exists() {
        eprintln!("error: cannot read {}: No such file or directory", filename);
        return Err(());
    }

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "spec" && ext != "nfr" {
        eprintln!("error: {} does not have a .spec or .nfr extension", filename);
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
    parsed: &ParsedFile,
    spec_tree_root: &Path,
    ctx: &mut ValidationContext,
) -> bool {
    let mut res_ctx = ResolutionContext {
        siblings: discover::discover_specs(spec_tree_root, Some(&parsed.path)),
        resolved: HashMap::new(),
        stack: vec![parsed.spec.name.clone()],
        errors: Vec::new(),
    };

    deps::resolve_and_collect(&parsed.spec.dependencies, &mut res_ctx);

    update_graph_cache(parsed, &res_ctx, ctx);

    if !ctx.seen.contains(&parsed.spec.name) {
        display::print_success(&parsed.spec);
        ctx.seen.insert(parsed.spec.name.clone());
        let shallowest =
            display::compute_shallowest_depths(&parsed.spec.dependencies, &res_ctx.resolved);
        let mut tree_ctx = TreeContext {
            resolved: &res_ctx.resolved,
            seen: ctx.seen,
            shallowest: &shallowest,
            depth: 1,
        };
        display::print_dep_tree(&parsed.spec.dependencies, &mut tree_ctx, "");
        let dep_count = res_ctx.resolved.len();
        if dep_count > 0 {
            println!(
                "{} {} resolved",
                dep_count,
                if dep_count == 1 { "dependency" } else { "dependencies" }
            );
        }
    }

    if !res_ctx.errors.is_empty() {
        for e in &res_ctx.errors {
            eprintln!("{}", e);
        }
        return false;
    }

    // Cross-validate NFR references if nfr_specs are available
    if let Some(nfr_specs) = ctx.nfr_specs {
        let has_nfr_refs = !parsed.spec.nfr_refs.is_empty()
            || parsed.spec.behaviors.iter().any(|b| !b.nfr_refs.is_empty());
        if has_nfr_refs
            && let Err(errors) = nfr_crossref::cross_validate(&parsed.spec, nfr_specs)
        {
            for e in &errors {
                eprintln!("{}: {}", parsed.path.display(), e);
            }
            return false;
        }
    }

    true
}

fn update_graph_cache(
    parsed: &ParsedFile,
    res_ctx: &ResolutionContext,
    ctx: &mut ValidationContext,
) {
    let state = match ctx.graph_state.as_mut() {
        Some(s) => s,
        None => return,
    };

    let hash = graph::content_hash(&parsed.source);
    if state.cache.is_changed(&parsed.spec.name, &hash) {
        state.cache.upsert(
            parsed.spec.name.clone(),
            CachedEntry {
                content_hash: hash,
                version: parsed.spec.version.clone(),
                behavior_count: parsed.spec.behaviors.len(),
                valid: res_ctx.errors.is_empty(),
                dependencies: parsed.spec.dep_names(),
                path: parsed.path.display().to_string(),
            },
        );
        state.dirty = true;
    }

    for (dep_name, rd) in &res_ctx.resolved {
        if let Some(dep_path) = res_ctx.siblings.get(dep_name)
            && let Ok(dep_source) = fs::read_to_string(dep_path)
        {
            let dep_hash = graph::content_hash(&dep_source);
            if state.cache.is_changed(dep_name, &dep_hash) {
                state.cache.upsert(
                    dep_name.clone(),
                    CachedEntry {
                        content_hash: dep_hash,
                        version: rd.spec.version.clone(),
                        behavior_count: rd.spec.behaviors.len(),
                        valid: rd.valid,
                        dependencies: rd.spec.dep_names(),
                        path: dep_path.display().to_string(),
                    },
                );
                state.dirty = true;
            }
        }
    }
}

/// Discover all .nfr files in a directory, parse them, and return a map of category -> NfrSpec.
fn discover_and_parse_nfrs(dir: &Path) -> HashMap<String, NfrSpec> {
    let mut map = HashMap::new();
    let nfr_files = discover::discover_nfr_files(dir);
    for path in &nfr_files {
        if let Ok(source) = fs::read_to_string(path)
            && let Ok(nfr) = nfr_parser::parse_nfr(&source)
        {
            map.insert(nfr.category.clone(), nfr);
        }
    }
    map
}
