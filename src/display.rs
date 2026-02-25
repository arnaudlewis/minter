use std::collections::{HashMap, HashSet};

use crate::model::{Dependency, NfrSpec, Spec};

// ANSI color codes
pub const GREEN: &str = "\x1b[32m";
pub const RED: &str = "\x1b[31m";
pub const YELLOW: &str = "\x1b[33m";
pub const CYAN: &str = "\x1b[36m";
pub const DIM: &str = "\x1b[2m";
pub const RESET: &str = "\x1b[0m";

pub fn tree_connector(is_last: bool) -> &'static str {
    if is_last { "\u{2514}\u{2500}\u{2500} " } else { "\u{251c}\u{2500}\u{2500} " }
}

pub fn tree_child_prefix(prefix: &str, is_last: bool) -> String {
    if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}\u{2502}   ", prefix)
    }
}

pub fn behavior_count_label(count: usize) -> String {
    if count == 1 {
        "1 behavior".to_string()
    } else {
        format!("{} behaviors", count)
    }
}

pub fn use_color() -> bool {
    std::env::var("NO_COLOR").is_err()
}

pub fn print_success(spec: &Spec) {
    if use_color() {
        println!(
            "{GREEN}\u{2713}{RESET} {} v{} ({})",
            spec.name,
            spec.version,
            behavior_count_label(spec.behaviors.len()),
        );
    } else {
        println!(
            "\u{2713} {} v{} ({})",
            spec.name,
            spec.version,
            behavior_count_label(spec.behaviors.len()),
        );
    }
}

pub fn print_failure(spec: &Spec) {
    if use_color() {
        println!("{RED}\u{2717}{RESET} {} v{}", spec.name, spec.version);
    } else {
        println!("\u{2717} {} v{}", spec.name, spec.version);
    }
}

pub fn constraint_count_label(count: usize) -> String {
    if count == 1 {
        "1 constraint".to_string()
    } else {
        format!("{} constraints", count)
    }
}

pub fn print_nfr_success(nfr: &NfrSpec) {
    if use_color() {
        println!(
            "{GREEN}\u{2713}{RESET} {} v{} ({})",
            nfr.category,
            nfr.version,
            constraint_count_label(nfr.constraints.len()),
        );
    } else {
        println!(
            "\u{2713} {} v{} ({})",
            nfr.category,
            nfr.version,
            constraint_count_label(nfr.constraints.len()),
        );
    }
}

pub fn print_nfr_failure(nfr: &NfrSpec) {
    if use_color() {
        println!("{RED}\u{2717}{RESET} {} v{}", nfr.category, nfr.version);
    } else {
        println!("\u{2717} {} v{}", nfr.category, nfr.version);
    }
}

pub struct TreeContext<'a> {
    pub resolved: &'a HashMap<String, crate::deps::ResolvedDep>,
    pub seen: &'a mut HashSet<String>,
    pub shallowest: &'a HashMap<String, usize>,
    pub depth: usize,
}

struct DepthAccumulator<'a> {
    resolved: &'a HashMap<String, crate::deps::ResolvedDep>,
    depths: HashMap<String, usize>,
    visited: HashSet<String>,
}

pub fn compute_shallowest_depths(
    deps: &[Dependency],
    resolved: &HashMap<String, crate::deps::ResolvedDep>,
) -> HashMap<String, usize> {
    let mut acc = DepthAccumulator {
        resolved,
        depths: HashMap::new(),
        visited: HashSet::new(),
    };
    compute_depths_recursive(deps, 1, &mut acc);
    acc.depths
}

fn compute_depths_recursive(deps: &[Dependency], depth: usize, acc: &mut DepthAccumulator) {
    for dep in deps {
        let entry = acc.depths.entry(dep.spec_name.clone()).or_insert(depth);
        if depth < *entry {
            *entry = depth;
        }
        if acc.visited.contains(&dep.spec_name) {
            continue;
        }
        acc.visited.insert(dep.spec_name.clone());
        let sub_deps: Option<Vec<Dependency>> = acc
            .resolved
            .get(&dep.spec_name)
            .filter(|rd| !rd.spec.dependencies.is_empty())
            .map(|rd| rd.spec.dependencies.clone());
        if let Some(sub) = sub_deps {
            compute_depths_recursive(&sub, depth + 1, acc);
        }
    }
}

pub fn print_dep_tree(deps: &[Dependency], ctx: &mut TreeContext, prefix: &str) {
    let color = use_color();
    let depth = ctx.depth;
    for (i, dep) in deps.iter().enumerate() {
        let is_last = i == deps.len() - 1;
        let connector = tree_connector(is_last);
        let child_prefix = tree_child_prefix(prefix, is_last);

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
                    ctx.depth = depth + 1;
                    print_dep_tree(&rd.spec.dependencies, ctx, &child_prefix);
                    ctx.depth = depth;
                }
            } else if color {
                println!(
                    "{}{}{} {}{} v{}{}",
                    prefix, connector, mark, DIM, rd.spec.name, rd.spec.version, RESET
                );
            } else {
                println!(
                    "{}{}{} {} v{}",
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
