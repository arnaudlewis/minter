use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::display::{self, use_color};
use crate::graph::{self, CachedEntry, GraphState};
use crate::{discover, parser};

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
        let source = match fs::read_to_string(path) {
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
                },
            );
            graph_state.dirty = true;
        }

        specs.push(SpecNode {
            name: spec.name,
            version: spec.version,
            behavior_count: spec.behaviors.len(),
            deps: dep_names,
        });
    }

    // Prune specs no longer on disk and persist
    let on_disk: HashSet<String> = specs.iter().map(|s| s.name.clone()).collect();
    graph_state.prune_stale(&on_disk);
    graph_state.save_if_dirty();

    let by_name: HashMap<String, usize> = specs.iter().enumerate().map(|(i, s)| (s.name.clone(), i)).collect();

    match impacted {
        Some(target) => print_impacted(&specs, &by_name, target),
        None => print_tree(&specs, &by_name),
    }
}

struct SpecNode {
    name: String,
    version: String,
    behavior_count: usize,
    deps: Vec<String>,
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

fn print_tree(specs: &[SpecNode], by_name: &HashMap<String, usize>) -> i32 {
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

    let color = use_color();
    let mut printed: HashSet<String> = HashSet::new();

    for (ri, &root_idx) in roots.iter().enumerate() {
        let root = &specs[root_idx];
        if color {
            println!("{}{}{}", display::CYAN, root.label(), display::RESET);
        } else {
            println!("{}", root.label());
        }

        if !root.deps.is_empty() {
            print_children(&root.deps, specs, by_name, "", &mut printed, color);
        }

        printed.insert(root.name.clone());

        if ri < roots.len() - 1 {
            println!();
        }
    }

    0
}

fn print_children(
    deps: &[String],
    specs: &[SpecNode],
    by_name: &HashMap<String, usize>,
    prefix: &str,
    printed: &mut HashSet<String>,
    color: bool,
) {
    for (i, dep_name) in deps.iter().enumerate() {
        let is_last = i == deps.len() - 1;
        let connector = display::tree_connector(is_last);
        let child_prefix = display::tree_child_prefix(prefix, is_last);

        if let Some(&idx) = by_name.get(dep_name) {
            let node = &specs[idx];
            let already_seen = printed.contains(dep_name);

            if already_seen {
                if color {
                    println!("{}{}{}{}{}", prefix, connector, display::DIM, node.label(), display::RESET);
                } else {
                    println!("{}{}{}", prefix, connector, node.label());
                }
            } else {
                println!("{}{}{}", prefix, connector, node.label());
                printed.insert(dep_name.clone());
                if !node.deps.is_empty() {
                    print_children(&node.deps, specs, by_name, &child_prefix, printed, color);
                }
            }
        } else {
            println!("{}{}{} (unresolved)", prefix, connector, dep_name);
        }
    }
}

fn print_impacted(specs: &[SpecNode], by_name: &HashMap<String, usize>, target: &str) -> i32 {
    if !by_name.contains_key(target) {
        eprintln!("error: spec '{}' not found in directory", target);
        return 1;
    }

    // Build reverse dependency map
    let mut reverse_deps: HashMap<&str, Vec<&str>> = HashMap::new();
    for s in specs {
        for dep in &s.deps {
            reverse_deps.entry(dep.as_str()).or_default().push(s.name.as_str());
        }
    }

    // BFS to find all transitive reverse dependencies
    let mut impacted: Vec<String> = Vec::new();
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

    let color = use_color();
    let target_idx = by_name[target];
    let target_node = &specs[target_idx];

    if impacted.is_empty() {
        println!("no specs depend on {}", target_node.label());
    } else {
        if color {
            println!("{}impacted by {}{}", display::YELLOW, target_node.label(), display::RESET);
        } else {
            println!("impacted by {}", target_node.label());
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
    0
}
