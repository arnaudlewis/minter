use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::cli::display::{self, TreeContext};
use crate::core::deps::{self, ResolutionContext};
use crate::core::graph::{self, CachedEntry, GraphCache, GraphState, discover_and_parse_nfrs};
use crate::core::io;
use crate::core::parser::nfr as nfr_parser;
use crate::core::validation::{crossref as nfr_crossref, nfr_semantic, semantic};
use crate::core::{discover, parser};
use crate::model::{NfrSpec, Spec};

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
            if !validate_directory(file, true, &mut seen) {
                // directories are always deep
                any_failed = true;
            }
        } else {
            let mut graph_state: Option<GraphState> = None;
            if check_deep {
                graph_state = Some(GraphState::load_or_build());
            }

            let tree_root = file.parent().unwrap_or(Path::new(".")).to_path_buf();
            let nfr_discovery = if check_deep {
                Some(discover_and_parse_nfrs(&tree_root))
            } else {
                None
            };
            let mut ctx = ValidationContext {
                check_deep,
                seen: &mut seen,
                graph_state: graph_state.as_mut(),
                nfr_specs: nfr_discovery.as_ref().map(|d| &d.specs),
            };
            if !validate_one(file, &tree_root, &mut ctx) {
                any_failed = true;
            }

            if let Some(ref mut state) = graph_state
                && let Some(ref discovery) = nfr_discovery
            {
                state.sync_nfrs(discovery);
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

    let nfr_discovery = if check_deep {
        Some(discover_and_parse_nfrs(dir))
    } else {
        None
    };

    let changed_nfr_categories: HashSet<String> = match (&mut graph_state, &nfr_discovery) {
        (Some(state), Some(discovery)) => state.sync_nfrs(discovery),
        _ => HashSet::new(),
    };

    let tree_root = dir.to_path_buf();
    let skippable = match graph_state.as_ref() {
        Some(state) => compute_skippable(&spec_files, &state.cache, &changed_nfr_categories),
        None => HashSet::new(),
    };

    let mut any_failed = false;
    for file in &spec_files {
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext == "spec"
            && let Some(stem) = file.file_stem().and_then(|s| s.to_str())
            && skippable.contains(stem)
        {
            if let Some(entry) = graph_state.as_ref().and_then(|s| s.cache.specs.get(stem))
                && !seen.contains(stem)
            {
                display::print_cached_success(stem, &entry.version, entry.behavior_count);
                seen.insert(stem.to_string());
            }
            continue;
        }

        let mut ctx = ValidationContext {
            check_deep,
            seen,
            graph_state: graph_state.as_mut(),
            nfr_specs: nfr_discovery.as_ref().map(|d| &d.specs),
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

        if let Some(ref discovery) = nfr_discovery {
            let nfr_on_disk: HashSet<String> = discovery.hashes.keys().cloned().collect();
            state.prune_stale_nfrs(&nfr_on_disk);
        }
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

    let source = match io::read_file_safe(path) {
        Ok(s) => s,
        Err(io::ReadError::PermissionDenied) => {
            eprintln!("error: permission denied reading {}", filename);
            return false;
        }
        Err(io::ReadError::TooLarge) => {
            eprintln!("error: {} exceeds maximum file size of 10MB", filename);
            return false;
        }
        Err(e) => {
            eprintln!("error: cannot read {}: {}", filename, e);
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

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "spec" && ext != "nfr" {
        eprintln!(
            "error: {} does not have a .spec or .nfr extension",
            filename
        );
        return Err(());
    }

    let source = match io::read_file_safe(path) {
        Ok(s) => s,
        Err(io::ReadError::PermissionDenied) => {
            eprintln!("error: permission denied reading {}", filename);
            return Err(());
        }
        Err(io::ReadError::TooLarge) => {
            eprintln!("error: {} exceeds maximum file size of 10MB", filename);
            return Err(());
        }
        Err(e) => {
            eprintln!("error: cannot read {}: {}", filename, e);
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

    deps::resolve_and_collect(&parsed.spec.dependencies, &mut res_ctx, 0);

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
                if dep_count == 1 {
                    "dependency"
                } else {
                    "dependencies"
                }
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
        if has_nfr_refs && let Err(errors) = nfr_crossref::cross_validate(&parsed.spec, nfr_specs) {
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
                nfr_categories: parsed.spec.all_nfr_categories(),
            },
        );
        state.dirty = true;
    }

    for (dep_name, rd) in &res_ctx.resolved {
        if let Some(dep_path) = res_ctx.siblings.get(dep_name)
            && let Ok(dep_source) = io::read_file_safe(dep_path)
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
                        nfr_categories: rd.spec.all_nfr_categories(),
                    },
                );
                state.dirty = true;
            }
        }
    }
}

/// Compute the set of spec names that can be safely skipped (cache hit).
/// A spec is skippable if: hash matches cache, valid == true, all transitive
/// dependencies are also skippable, and no referenced NFR categories have changed.
fn compute_skippable(
    files: &[PathBuf],
    cache: &GraphCache,
    changed_nfr_categories: &HashSet<String>,
) -> HashSet<String> {
    // First pass: find specs whose content hash matches and are valid in cache
    let mut hash_ok: HashMap<String, bool> = HashMap::new();
    for file in files {
        if file.extension().and_then(|e| e.to_str()) != Some("spec") {
            continue;
        }
        if let Some(stem) = file.file_stem().and_then(|s| s.to_str()) {
            if let Ok(source) = io::read_file_safe(file) {
                let hash = graph::content_hash(&source);
                if let Some(entry) = cache.specs.get(stem) {
                    hash_ok.insert(stem.to_string(), entry.content_hash == hash && entry.valid);
                    continue;
                }
            }
            hash_ok.insert(stem.to_string(), false);
        }
    }

    // Second pass: transitive dependency check — a spec is skippable only if
    // it and all its transitive deps are hash-ok and no referenced NFRs changed
    let mut result: HashMap<String, bool> = HashMap::new();
    for name in hash_ok.keys() {
        is_transitively_skippable(name, &hash_ok, cache, &mut result, changed_nfr_categories);
    }

    result
        .into_iter()
        .filter_map(|(k, v)| if v { Some(k) } else { None })
        .collect()
}

/// Recursively check if a spec and all its deps are skippable.
pub(crate) fn is_transitively_skippable(
    name: &str,
    hash_ok: &HashMap<String, bool>,
    cache: &GraphCache,
    memo: &mut HashMap<String, bool>,
    changed_nfr_categories: &HashSet<String>,
) -> bool {
    if let Some(&cached) = memo.get(name) {
        return cached;
    }

    // Guard against cycles: mark as not-skippable while visiting
    memo.insert(name.to_string(), false);

    if !hash_ok.get(name).copied().unwrap_or(false) {
        return false;
    }

    if let Some(entry) = cache.specs.get(name) {
        // Check if any NFR categories referenced by this spec have changed
        for cat in &entry.nfr_categories {
            if changed_nfr_categories.contains(cat) {
                return false;
            }
        }
        for dep in &entry.dependencies {
            if !is_transitively_skippable(dep, hash_ok, cache, memo, changed_nfr_categories) {
                return false;
            }
        }
    }

    memo.insert(name.to_string(), true);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::graph::CachedEntry;

    fn make_cached_entry(
        hash: &str,
        valid: bool,
        deps: Vec<&str>,
        nfr_cats: Vec<&str>,
    ) -> CachedEntry {
        CachedEntry {
            content_hash: hash.to_string(),
            version: "1.0.0".to_string(),
            behavior_count: 1,
            valid,
            dependencies: deps.into_iter().map(String::from).collect(),
            path: "test.spec".to_string(),
            nfr_categories: nfr_cats.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    /// validate: skippable_when_hash_ok_no_deps
    fn skippable_when_hash_ok_no_deps() {
        let mut cache = GraphCache::new();
        cache.upsert(
            "auth".to_string(),
            make_cached_entry("abc", true, vec![], vec![]),
        );
        let hash_ok: HashMap<String, bool> = [("auth".to_string(), true)].into();
        let changed_nfrs = HashSet::new();
        let mut memo = HashMap::new();
        assert!(is_transitively_skippable(
            "auth",
            &hash_ok,
            &cache,
            &mut memo,
            &changed_nfrs
        ));
    }

    #[test]
    /// validate: not_skippable_when_hash_mismatch
    fn not_skippable_when_hash_mismatch() {
        let mut cache = GraphCache::new();
        cache.upsert(
            "auth".to_string(),
            make_cached_entry("abc", true, vec![], vec![]),
        );
        let hash_ok: HashMap<String, bool> = [("auth".to_string(), false)].into();
        let changed_nfrs = HashSet::new();
        let mut memo = HashMap::new();
        assert!(!is_transitively_skippable(
            "auth",
            &hash_ok,
            &cache,
            &mut memo,
            &changed_nfrs
        ));
    }

    #[test]
    /// validate: not_skippable_when_dep_changed
    fn not_skippable_when_dep_changed() {
        let mut cache = GraphCache::new();
        cache.upsert(
            "auth".to_string(),
            make_cached_entry("abc", true, vec!["user"], vec![]),
        );
        cache.upsert(
            "user".to_string(),
            make_cached_entry("def", true, vec![], vec![]),
        );
        let hash_ok: HashMap<String, bool> = [
            ("auth".to_string(), true),
            ("user".to_string(), false), // dep hash changed
        ]
        .into();
        let changed_nfrs = HashSet::new();
        let mut memo = HashMap::new();
        assert!(!is_transitively_skippable(
            "auth",
            &hash_ok,
            &cache,
            &mut memo,
            &changed_nfrs
        ));
    }

    #[test]
    /// validate: not_skippable_when_nfr_changed
    fn not_skippable_when_nfr_changed() {
        let mut cache = GraphCache::new();
        cache.upsert(
            "auth".to_string(),
            make_cached_entry("abc", true, vec![], vec!["security"]),
        );
        let hash_ok: HashMap<String, bool> = [("auth".to_string(), true)].into();
        let changed_nfrs: HashSet<String> = ["security".to_string()].into();
        let mut memo = HashMap::new();
        assert!(!is_transitively_skippable(
            "auth",
            &hash_ok,
            &cache,
            &mut memo,
            &changed_nfrs
        ));
    }

    #[test]
    /// validate: skippable_transitive_chain
    fn skippable_transitive_chain() {
        let mut cache = GraphCache::new();
        cache.upsert(
            "a".to_string(),
            make_cached_entry("h1", true, vec!["b"], vec![]),
        );
        cache.upsert(
            "b".to_string(),
            make_cached_entry("h2", true, vec!["c"], vec![]),
        );
        cache.upsert(
            "c".to_string(),
            make_cached_entry("h3", true, vec![], vec![]),
        );
        let hash_ok: HashMap<String, bool> = [
            ("a".to_string(), true),
            ("b".to_string(), true),
            ("c".to_string(), true),
        ]
        .into();
        let changed_nfrs = HashSet::new();
        let mut memo = HashMap::new();
        assert!(is_transitively_skippable(
            "a",
            &hash_ok,
            &cache,
            &mut memo,
            &changed_nfrs
        ));
        assert_eq!(memo.get("a"), Some(&true));
        assert_eq!(memo.get("b"), Some(&true));
        assert_eq!(memo.get("c"), Some(&true));
    }
}
