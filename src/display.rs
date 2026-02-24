use std::collections::{HashMap, HashSet};

use crate::model::{Dependency, Spec};

// ANSI color codes
pub const GREEN: &str = "\x1b[32m";
pub const RED: &str = "\x1b[31m";
pub const YELLOW: &str = "\x1b[33m";
pub const CYAN: &str = "\x1b[36m";
pub const RESET: &str = "\x1b[0m";

pub fn behavior_count_label(count: usize) -> String {
    if count == 1 {
        "1 behavior".to_string()
    } else {
        format!("{} behaviors", count)
    }
}

pub fn print_success(spec: &Spec) {
    println!(
        "\u{2713} {} v{} ({})",
        spec.name,
        spec.version,
        behavior_count_label(spec.behaviors.len()),
    );
}

pub fn print_failure(spec: &Spec) {
    println!("\u{2717} {} v{}", spec.name, spec.version);
}

pub fn print_success_colored(spec: &Spec) {
    println!(
        "{}\u{2713}{} {} v{} ({})",
        GREEN,
        RESET,
        spec.name,
        spec.version,
        behavior_count_label(spec.behaviors.len()),
    );
}

pub fn print_failure_colored(spec: &Spec) {
    println!("{}\u{2717}{} {} v{}", RED, RESET, spec.name, spec.version);
}

pub struct TreeContext<'a> {
    pub resolved: &'a HashMap<String, crate::deps::ResolvedDep>,
    pub seen: &'a mut HashSet<String>,
    pub shallowest: &'a HashMap<String, usize>,
}

pub fn compute_shallowest_depths(
    deps: &[Dependency],
    resolved: &HashMap<String, crate::deps::ResolvedDep>,
) -> HashMap<String, usize> {
    let mut depths: HashMap<String, usize> = HashMap::new();
    let mut visited = HashSet::new();
    compute_depths_recursive(deps, resolved, 1, &mut depths, &mut visited);
    depths
}

fn compute_depths_recursive(
    deps: &[Dependency],
    resolved: &HashMap<String, crate::deps::ResolvedDep>,
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
        if let Some(rd) = resolved.get(&dep.spec_name)
            && !rd.spec.dependencies.is_empty() {
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

pub fn print_dep_tree(deps: &[Dependency], ctx: &mut TreeContext, prefix: &str, depth: usize) {
    for (i, dep) in deps.iter().enumerate() {
        let is_last = i == deps.len() - 1;
        let connector = if is_last { "\u{2514}\u{2500}\u{2500} " } else { "\u{251c}\u{2500}\u{2500} " };
        let child_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}\u{2502}   ", prefix)
        };

        if let Some(rd) = ctx.resolved.get(&dep.spec_name) {
            let mark = if rd.valid { "\u{2713}" } else { "\u{2717}" };
            let is_shallowest =
                ctx.shallowest.get(&dep.spec_name).copied().unwrap_or(depth) == depth;

            if is_shallowest && !ctx.seen.contains(&dep.spec_name) {
                ctx.seen.insert(dep.spec_name.clone());
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
                    print_dep_tree(&rd.spec.dependencies, ctx, &child_prefix, depth + 1);
                }
            } else {
                println!(
                    "{}{}{} \x1b[2m{} v{}\x1b[0m",
                    prefix, connector, mark, rd.spec.name, rd.spec.version
                );
            }
        } else {
            println!(
                "{}{}\u{2717} {} (unresolved)",
                prefix, connector, dep.spec_name
            );
        }
    }
}
