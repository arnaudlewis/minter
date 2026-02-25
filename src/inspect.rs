use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::model::{Assertion, BehaviorCategory, ConstraintType};
use crate::{nfr_parser, parser};

/// Display structured metadata for a spec file.
pub fn run_inspect(file: &Path) -> i32 {
    if !file.exists() {
        eprintln!("error: file not found: {}", file.display());
        return 1;
    }

    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext == "nfr" {
        return inspect_nfr(file);
    }

    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {}: {}", file.display(), e);
            return 1;
        }
    };

    let spec = match parser::parse(&source) {
        Ok(s) => s,
        Err(errors) => {
            for e in &errors {
                eprintln!("{}: {}", file.display(), e);
            }
            return 1;
        }
    };

    // Header
    println!("{} v{}", spec.name, spec.version);
    println!("title: {}", spec.title);
    println!();

    // Behavior count
    let count = spec.behaviors.len();
    println!(
        "{} {}",
        count,
        if count == 1 { "behavior" } else { "behaviors" }
    );

    // Category distribution
    let mut categories: HashMap<&str, usize> = HashMap::new();
    for b in &spec.behaviors {
        let label = match b.category {
            BehaviorCategory::HappyPath => "happy_path",
            BehaviorCategory::ErrorCase => "error_case",
            BehaviorCategory::EdgeCase => "edge_case",
        };
        *categories.entry(label).or_insert(0) += 1;
    }
    let mut cat_list: Vec<_> = categories.into_iter().collect();
    cat_list.sort_by_key(|(name, _)| *name);
    for (cat, n) in &cat_list {
        println!("  {}: {}", cat, n);
    }
    println!();

    // Dependencies
    if spec.dependencies.is_empty() {
        println!("no dependencies");
    } else {
        println!("dependencies:");
        for dep in &spec.dependencies {
            println!("  {} >= {}", dep.spec_name, dep.version_constraint);
        }
    }
    println!();

    // Assertion types
    let mut assertion_types: HashMap<&str, usize> = HashMap::new();
    for b in &spec.behaviors {
        for post in &b.postconditions {
            for assertion in &post.assertions {
                let label = match assertion {
                    Assertion::Equals { .. } => "equals",
                    Assertion::EqualsRef { .. } => "equals",
                    Assertion::IsPresent { .. } => "is_present",
                    Assertion::Contains { .. } => "contains",
                    Assertion::InRange { .. } => "in_range",
                    Assertion::MatchesPattern { .. } => "matches_pattern",
                    Assertion::GreaterOrEqual { .. } => "greater_or_equal",
                    Assertion::Prose(_) => "prose",
                };
                *assertion_types.entry(label).or_insert(0) += 1;
            }
        }
    }
    if !assertion_types.is_empty() {
        println!("assertion types:");
        let mut at_list: Vec<_> = assertion_types.into_iter().collect();
        at_list.sort_by_key(|(name, _)| *name);
        for (at, n) in &at_list {
            println!("  {}: {}", at, n);
        }
    }

    0
}

/// Display structured metadata for an NFR file.
fn inspect_nfr(file: &Path) -> i32 {
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {}: {}", file.display(), e);
            return 1;
        }
    };

    let nfr = match nfr_parser::parse_nfr(&source) {
        Ok(n) => n,
        Err(errors) => {
            for e in &errors {
                eprintln!("{}: {}", file.display(), e);
            }
            return 1;
        }
    };

    // Header
    println!("{} v{}", nfr.category, nfr.version);
    println!("title: {}", nfr.title);
    println!();

    // Constraint count
    let count = nfr.constraints.len();
    println!(
        "{} {}",
        count,
        if count == 1 {
            "constraint"
        } else {
            "constraints"
        }
    );

    // Type distribution
    let mut metric_count = 0;
    let mut rule_count = 0;
    for c in &nfr.constraints {
        match c.constraint_type {
            ConstraintType::Metric => metric_count += 1,
            ConstraintType::Rule => rule_count += 1,
        }
    }
    println!("  metric: {}", metric_count);
    println!("  rule: {}", rule_count);
    println!();

    // Category
    println!("category: {}", nfr.category);
    println!();

    // No dependencies for NFR files
    println!("no dependencies");

    0
}
