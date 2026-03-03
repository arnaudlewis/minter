use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::cli::display::{self, use_color};
use crate::core::graph::{self, CachedEntry, GraphState, discover_and_parse_nfrs};
use crate::core::{discover, io, parser};
use crate::model::{NfrRefKind, NfrSpec};

/// Display the dependency graph for spec files in a directory.
pub fn run_graph(dir: &Path, impacted: Option<&str>) -> i32 {
    if !dir.exists() {
        eprintln!("error: directory not found: {}", dir.display());
        return 1;
    }

    let spec_files = match discover::discover_spec_files(dir) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("error: {}", e);
            return 1;
        }
    };

    if spec_files.is_empty() {
        eprintln!("error: no spec files found in {}", dir.display());
        return 1;
    }

    // Parse all specs and collect names, versions, dependency edges
    let mut specs: Vec<SpecNode> = Vec::new();
    let mut graph_state = GraphState::load_or_build();
    for path in &spec_files {
        let source = match io::read_file_safe(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: cannot read {}: {}", path.display(), e);
                return 1;
            }
        };
        let spec = match parser::parse(&source) {
            Ok(s) => s,
            Err(errors) => {
                for e in &errors {
                    eprintln!("{}: {}", path.display(), e);
                }
                return 1;
            }
        };
        let nfr_cats = spec.all_nfr_categories();
        let nfr_refs_grouped = spec.all_nfr_refs_grouped();
        let dep_names = spec.dep_names();

        let hash = graph::content_hash(&source);
        if graph_state.cache.is_changed(&spec.name, &hash) {
            graph_state.cache.upsert(
                spec.name.clone(),
                CachedEntry {
                    content_hash: hash,
                    version: spec.version.clone(),
                    behavior_count: spec.behaviors.len(),
                    valid: true,
                    dependencies: dep_names.clone(),
                    path: path.display().to_string(),
                    nfr_categories: nfr_cats,
                },
            );
            graph_state.dirty = true;
        }

        specs.push(SpecNode {
            name: spec.name,
            version: spec.version,
            behavior_count: spec.behaviors.len(),
            deps: dep_names,
            nfr_refs: nfr_refs_grouped,
        });
    }

    // Discover, parse, and sync NFR files
    let nfr_root = nfr_search_root(dir);
    let nfr_discovery = discover_and_parse_nfrs(&nfr_root);
    graph_state.sync_nfrs(&nfr_discovery);
    let nfr_on_disk: HashSet<String> = nfr_discovery.hashes.keys().cloned().collect();
    graph_state.prune_stale_nfrs(&nfr_on_disk);
    let nfr_specs: HashMap<String, NfrSpec> = nfr_discovery.specs;

    // Prune specs no longer on disk and persist
    let on_disk: HashSet<String> = specs.iter().map(|s| s.name.clone()).collect();
    graph_state.prune_stale(&on_disk);
    graph_state.save_if_dirty();

    let by_name: HashMap<String, usize> = specs
        .iter()
        .enumerate()
        .map(|(i, s)| (s.name.clone(), i))
        .collect();

    match impacted {
        Some(target) => print_impacted(&specs, &by_name, &nfr_specs, target),
        None => print_tree(&specs, &by_name, &nfr_specs),
    }
}

struct SpecNode {
    name: String,
    version: String,
    behavior_count: usize,
    deps: Vec<String>,
    /// NFR references grouped by category → ref kind (whole-file or specific anchors).
    nfr_refs: BTreeMap<String, NfrRefKind>,
}

impl SpecNode {
    fn label(&self) -> String {
        format!(
            "{} v{} ({})",
            self.name,
            self.version,
            display::behavior_count_label(self.behavior_count),
        )
    }
}

struct GraphContext<'a> {
    specs: &'a [SpecNode],
    by_name: &'a HashMap<String, usize>,
    nfr_specs: &'a HashMap<String, NfrSpec>,
    printed: HashSet<String>,
    color: bool,
}

/// Merge NFR refs across all specs (whole-file wins, otherwise union anchors)
/// and count the total referenced constraints.
fn compute_nfr_summary<'a>(
    specs: &'a [SpecNode],
    nfr_specs: &HashMap<String, NfrSpec>,
) -> (BTreeMap<&'a str, NfrRefKind>, usize) {
    let mut merged: BTreeMap<&str, NfrRefKind> = BTreeMap::new();
    for s in specs {
        for (cat, ref_kind) in &s.nfr_refs {
            match merged.get(cat.as_str()) {
                None => {
                    merged.insert(cat.as_str(), ref_kind.clone());
                }
                Some(NfrRefKind::WholeFile) => {}
                Some(NfrRefKind::Anchors(existing)) => match ref_kind {
                    NfrRefKind::WholeFile => {
                        merged.insert(cat.as_str(), NfrRefKind::WholeFile);
                    }
                    NfrRefKind::Anchors(new) => {
                        let mut combined: Vec<String> = existing.clone();
                        for a in new {
                            if !combined.contains(a) {
                                combined.push(a.clone());
                            }
                        }
                        merged.insert(cat.as_str(), NfrRefKind::Anchors(combined));
                    }
                },
            }
        }
    }
    let mut total = 0;
    for (cat, ref_kind) in &merged {
        if let Some(nfr) = nfr_specs.get(*cat) {
            total += match ref_kind {
                NfrRefKind::WholeFile => nfr.constraints.len(),
                NfrRefKind::Anchors(anchors) => anchors.len(),
            };
        }
    }
    (merged, total)
}

fn print_tree(
    specs: &[SpecNode],
    by_name: &HashMap<String, usize>,
    nfr_specs: &HashMap<String, NfrSpec>,
) -> i32 {
    let total_behaviors: usize = specs.iter().map(|s| s.behavior_count).sum();
    let (merged_refs, total_nfrs) = compute_nfr_summary(specs, nfr_specs);

    let color = use_color();
    let summary = format!(
        "{} specs, {}, {}, {}",
        specs.len(),
        display::behavior_count_label(total_behaviors),
        display::nfr_category_count_label(merged_refs.len()),
        display::nfr_count_label(total_nfrs),
    );
    if color {
        println!("{}{}{}", display::BOLD, summary, display::RESET);
    } else {
        println!("{}", summary);
    }
    println!();

    // Find all names that appear as a dependency
    let mut depended_on: HashSet<&str> = HashSet::new();
    for s in specs {
        for d in &s.deps {
            depended_on.insert(d.as_str());
        }
    }

    // Roots = specs nothing depends on, sorted
    let mut roots: Vec<usize> = specs
        .iter()
        .enumerate()
        .filter(|(_, s)| !depended_on.contains(s.name.as_str()))
        .map(|(i, _)| i)
        .collect();
    roots.sort_by_key(|&i| &specs[i].name);

    let mut ctx = GraphContext {
        specs,
        by_name,
        nfr_specs,
        printed: HashSet::new(),
        color,
    };

    for (ri, &root_idx) in roots.iter().enumerate() {
        let root = &ctx.specs[root_idx];
        if ctx.color {
            println!("{}{}{}", display::CYAN, root.label(), display::RESET);
        } else {
            println!("{}", root.label());
        }

        if !root.deps.is_empty() || !root.nfr_refs.is_empty() {
            let deps = root.deps.clone();
            let nfr_refs = root.nfr_refs.clone();
            print_children(&deps, &nfr_refs, "", &mut ctx);
        }

        ctx.printed.insert(ctx.specs[root_idx].name.clone());

        if ri < roots.len() - 1 {
            println!();
        }
    }

    0
}

fn print_children(
    deps: &[String],
    nfr_refs: &BTreeMap<String, NfrRefKind>,
    prefix: &str,
    ctx: &mut GraphContext,
) {
    let nfr_cats: Vec<(&String, &NfrRefKind)> = nfr_refs.iter().collect();
    let total = deps.len() + nfr_cats.len();
    for (i, dep_name) in deps.iter().enumerate() {
        let is_last = i == total - 1;
        let connector = display::tree_connector(is_last);
        let child_prefix = display::tree_child_prefix(prefix, is_last);

        if let Some(&idx) = ctx.by_name.get(dep_name) {
            let node = &ctx.specs[idx];
            let already_seen = ctx.printed.contains(dep_name);

            if already_seen {
                if ctx.color {
                    println!(
                        "{}{}{}{}{}",
                        prefix,
                        connector,
                        display::DIM,
                        node.label(),
                        display::RESET
                    );
                } else {
                    println!("{}{}{}", prefix, connector, node.label());
                }
            } else {
                println!("{}{}{}", prefix, connector, node.label());
                ctx.printed.insert(dep_name.clone());
                let sub_deps = node.deps.clone();
                let sub_nfr_refs = node.nfr_refs.clone();
                if !sub_deps.is_empty() || !sub_nfr_refs.is_empty() {
                    print_children(&sub_deps, &sub_nfr_refs, &child_prefix, ctx);
                }
            }
        } else {
            println!("{}{}{} (unresolved)", prefix, connector, dep_name);
        }
    }

    // Print NFR refs grouped by category with anchor sub-items
    for (i, (cat, ref_kind)) in nfr_cats.iter().enumerate() {
        let is_last = deps.len() + i == total - 1;
        let connector = display::tree_connector(is_last);
        let child_prefix = display::tree_child_prefix(prefix, is_last);
        print_nfr_node(
            cat,
            ref_kind,
            ctx.nfr_specs,
            prefix,
            connector,
            &child_prefix,
            ctx.color,
        );
    }
}

fn nfr_label(cat: &str, ref_kind: &NfrRefKind, nfr_specs: &HashMap<String, NfrSpec>) -> String {
    if let Some(nfr) = nfr_specs.get(cat) {
        let count = match ref_kind {
            NfrRefKind::WholeFile => nfr.constraints.len(),
            NfrRefKind::Anchors(anchors) => anchors.len(),
        };
        format!(
            "[nfr] {} v{} ({})",
            cat,
            nfr.version,
            display::constraint_count_label(count),
        )
    } else {
        format!("[nfr] {}", cat)
    }
}

fn print_nfr_node(
    cat: &str,
    ref_kind: &NfrRefKind,
    nfr_specs: &HashMap<String, NfrSpec>,
    prefix: &str,
    connector: &str,
    child_prefix: &str,
    color: bool,
) {
    // Print category line
    let label = nfr_label(cat, ref_kind, nfr_specs);
    if color {
        let tag = format!("{}[nfr]{}", display::MAGENTA, display::RESET);
        let rest = if let Some(nfr) = nfr_specs.get(cat) {
            let count = match ref_kind {
                NfrRefKind::WholeFile => nfr.constraints.len(),
                NfrRefKind::Anchors(anchors) => anchors.len(),
            };
            format!(
                " {} v{} ({})",
                cat,
                nfr.version,
                display::constraint_count_label(count),
            )
        } else {
            format!(" {}", cat)
        };
        println!("{}{}{}{}", prefix, connector, tag, rest);
    } else {
        println!("{}{}{}", prefix, connector, label);
    }

    // Print anchor sub-items only for Anchors kind (not WholeFile)
    if let NfrRefKind::Anchors(anchors) = ref_kind {
        for (i, anchor) in anchors.iter().enumerate() {
            let is_last = i == anchors.len() - 1;
            let anchor_connector = display::tree_connector(is_last);
            if color {
                println!(
                    "{}{}{}#{}{}",
                    child_prefix,
                    anchor_connector,
                    display::MAGENTA,
                    display::RESET,
                    anchor
                );
            } else {
                println!("{}{}#{}", child_prefix, anchor_connector, anchor);
            }
        }
    }
}

/// BFS to find all specs transitively impacted by changes to a given spec.
fn find_spec_dependents(specs: &[SpecNode], target: &str) -> Vec<String> {
    let mut reverse_deps: HashMap<&str, Vec<&str>> = HashMap::new();
    for s in specs {
        for dep in &s.deps {
            reverse_deps
                .entry(dep.as_str())
                .or_default()
                .push(s.name.as_str());
        }
    }

    let mut impacted = Vec::new();
    let mut visited: HashSet<&str> = HashSet::new();
    let mut queue: Vec<&str> = vec![target];
    visited.insert(target);

    while let Some(current) = queue.pop() {
        if let Some(dependents) = reverse_deps.get(current) {
            for &dep in dependents {
                if visited.insert(dep) {
                    impacted.push(dep.to_string());
                    queue.push(dep);
                }
            }
        }
    }
    impacted
}

/// Print a sorted impacted-by list with tree connectors.
fn print_impacted_list(
    label: &str,
    empty_msg: &str,
    impacted: &mut [String],
    specs: &[SpecNode],
    by_name: &HashMap<String, usize>,
) {
    if impacted.is_empty() {
        println!("{}", empty_msg);
        return;
    }
    let color = use_color();
    if color {
        println!("{}impacted by {}{}", display::YELLOW, label, display::RESET);
    } else {
        println!("impacted by {}", label);
    }
    impacted.sort();
    for (i, name) in impacted.iter().enumerate() {
        let is_last = i == impacted.len() - 1;
        let connector = display::tree_connector(is_last);
        if let Some(&idx) = by_name.get(name.as_str()) {
            println!("{}{}", connector, specs[idx].label());
        } else {
            println!("{}{}", connector, name);
        }
    }
}

fn print_impacted(
    specs: &[SpecNode],
    by_name: &HashMap<String, usize>,
    nfr_specs: &HashMap<String, NfrSpec>,
    target: &str,
) -> i32 {
    if by_name.contains_key(target) {
        let mut impacted = find_spec_dependents(specs, target);
        let target_node = &specs[by_name[target]];
        let label = target_node.label();
        print_impacted_list(
            &label,
            &format!("no specs depend on {}", label),
            &mut impacted,
            specs,
            by_name,
        );
        0
    } else if nfr_specs.contains_key(target) {
        let mut impacted: Vec<String> = specs
            .iter()
            .filter(|s| s.nfr_refs.contains_key(target))
            .map(|s| s.name.clone())
            .collect();
        let label = nfr_label(target, &NfrRefKind::WholeFile, nfr_specs);
        print_impacted_list(
            &label,
            &format!("no specs reference {}", label),
            &mut impacted,
            specs,
            by_name,
        );
        0
    } else {
        eprintln!(
            "error: spec or NFR category '{}' not found in directory",
            target
        );
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_spec_node(name: &str, deps: Vec<&str>) -> SpecNode {
        SpecNode {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            behavior_count: 1,
            deps: deps.into_iter().map(String::from).collect(),
            nfr_refs: BTreeMap::new(),
        }
    }

    #[test]
    /// graph: bfs_no_dependents
    fn bfs_no_dependents() {
        let specs = vec![make_spec_node("a", vec![]), make_spec_node("b", vec![])];
        let result = find_spec_dependents(&specs, "a");
        assert!(result.is_empty());
    }

    #[test]
    /// graph: bfs_direct_dependents
    fn bfs_direct_dependents() {
        let specs = vec![
            make_spec_node("target", vec![]),
            make_spec_node("a", vec!["target"]),
            make_spec_node("b", vec!["target"]),
        ];
        let mut result = find_spec_dependents(&specs, "target");
        result.sort();
        assert_eq!(result, vec!["a", "b"]);
    }

    #[test]
    /// graph: bfs_transitive_chain
    fn bfs_transitive_chain() {
        let specs = vec![
            make_spec_node("target", vec![]),
            make_spec_node("b", vec!["target"]),
            make_spec_node("a", vec!["b"]),
        ];
        let mut result = find_spec_dependents(&specs, "target");
        result.sort();
        assert_eq!(result, vec!["a", "b"]);
    }

    #[test]
    /// graph: bfs_diamond
    fn bfs_diamond() {
        let specs = vec![
            make_spec_node("target", vec![]),
            make_spec_node("a", vec!["target"]),
            make_spec_node("b", vec!["target"]),
            make_spec_node("c", vec!["a", "b"]),
        ];
        let mut result = find_spec_dependents(&specs, "target");
        result.sort();
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    /// graph: bfs_empty_graph
    fn bfs_empty_graph() {
        let specs: Vec<SpecNode> = vec![];
        let result = find_spec_dependents(&specs, "nonexistent");
        assert!(result.is_empty());
    }
}

/// Find the best root directory for NFR file discovery.
/// When given a file, walks up parent directories to find the nearest
/// ancestor that contains .nfr files. Falls back to the file's parent.
fn nfr_search_root(path: &Path) -> PathBuf {
    if path.is_dir() {
        return path.to_path_buf();
    }
    let fallback = path.parent().unwrap_or(Path::new("."));
    let mut current = fallback;
    for _ in 0..10 {
        if !discover::discover_nfr_files(current).is_empty() {
            return current.to_path_buf();
        }
        match current.parent() {
            Some(parent) if parent != current => current = parent,
            _ => break,
        }
    }
    fallback.to_path_buf()
}
