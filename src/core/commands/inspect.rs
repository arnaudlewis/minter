use std::collections::HashMap;
use std::path::Path;

use crate::core::io;
use crate::core::parser;
use crate::core::parser::nfr as nfr_parser;
use crate::model::{Assertion, BehaviorCategory, ConstraintType, NfrSpec, Spec};

/// Count behaviors by category, returning sorted (label, count) pairs.
fn count_categories(spec: &Spec) -> Vec<(&'static str, usize)> {
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
    cat_list
}

/// Collect assertion types with counts, returning sorted (label, count) pairs.
fn collect_assertion_types(spec: &Spec) -> Vec<(&'static str, usize)> {
    let mut assertion_types: HashMap<&str, usize> = HashMap::new();
    for b in &spec.behaviors {
        for post in &b.postconditions {
            for assertion in &post.assertions {
                let label = match assertion {
                    Assertion::Equals { .. } | Assertion::EqualsRef { .. } => "equals",
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
    let mut at_list: Vec<_> = assertion_types.into_iter().collect();
    at_list.sort_by_key(|(name, _)| *name);
    at_list
}

/// Count constraint types in an NFR spec, returning (metric_count, rule_count).
fn count_constraint_types(nfr: &NfrSpec) -> (usize, usize) {
    let mut metric_count = 0;
    let mut rule_count = 0;
    for c in &nfr.constraints {
        match c.constraint_type {
            ConstraintType::Metric => metric_count += 1,
            ConstraintType::Rule => rule_count += 1,
        }
    }
    (metric_count, rule_count)
}

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

    let source = match io::read_file_safe(file) {
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
    let cat_list = count_categories(&spec);
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
    let at_list = collect_assertion_types(&spec);
    if !at_list.is_empty() {
        println!("assertion types:");
        for (at, n) in &at_list {
            println!("  {}: {}", at, n);
        }
    }

    0
}

/// Display structured metadata for an NFR file.
fn inspect_nfr(file: &Path) -> i32 {
    let source = match io::read_file_safe(file) {
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
    let (metric_count, rule_count) = count_constraint_types(&nfr);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        Action, Behavior, ConstraintBody, MetricVerification, NfrConstraint, Postcondition,
        PostconditionKind, RuleVerification,
    };

    fn make_behavior(category: BehaviorCategory, assertions: Vec<Assertion>) -> Behavior {
        Behavior {
            name: "test-behavior".to_string(),
            category,
            description: "test".to_string(),
            nfr_refs: vec![],
            preconditions: vec![],
            action: Action {
                name: "test-action".to_string(),
                inputs: vec![],
            },
            postconditions: if assertions.is_empty() {
                vec![]
            } else {
                vec![Postcondition {
                    kind: PostconditionKind::SideEffect,
                    assertions,
                }]
            },
        }
    }

    fn make_spec(behaviors: Vec<Behavior>) -> Spec {
        Spec {
            name: "test-spec".to_string(),
            version: "1.0.0".to_string(),
            title: "Test Spec".to_string(),
            description: "test".to_string(),
            motivation: "test".to_string(),
            nfr_refs: vec![],
            behaviors,
            dependencies: vec![],
        }
    }

    fn make_nfr_constraint(ct: ConstraintType) -> NfrConstraint {
        NfrConstraint {
            name: "test-constraint".to_string(),
            constraint_type: ct,
            description: "test".to_string(),
            body: match ct {
                ConstraintType::Metric => ConstraintBody::Metric {
                    metric: "latency".to_string(),
                    threshold_operator: "<".to_string(),
                    threshold_value: "100ms".to_string(),
                    verification: MetricVerification {
                        environments: vec![],
                        benchmarks: vec![],
                        datasets: vec![],
                        passes: vec![],
                    },
                },
                ConstraintType::Rule => ConstraintBody::Rule {
                    rule_text: "test rule".to_string(),
                    verification: RuleVerification {
                        statics: vec![],
                        runtimes: vec![],
                    },
                },
            },
            violation: "test violation".to_string(),
            overridable: false,
        }
    }

    fn make_nfr(constraints: Vec<NfrConstraint>) -> NfrSpec {
        NfrSpec {
            category: "performance".to_string(),
            version: "1.0.0".to_string(),
            title: "Performance NFR".to_string(),
            description: "test".to_string(),
            motivation: "test".to_string(),
            constraints,
        }
    }

    // ── count_categories ──────────────────────────────

    #[test]
    /// inspect: category_counts_mixed
    fn category_counts_mixed() {
        let spec = make_spec(vec![
            make_behavior(BehaviorCategory::HappyPath, vec![]),
            make_behavior(BehaviorCategory::HappyPath, vec![]),
            make_behavior(BehaviorCategory::ErrorCase, vec![]),
            make_behavior(BehaviorCategory::EdgeCase, vec![]),
        ]);
        let counts = count_categories(&spec);
        assert_eq!(
            counts,
            vec![("edge_case", 1), ("error_case", 1), ("happy_path", 2)]
        );
    }

    #[test]
    /// inspect: category_counts_single
    fn category_counts_single() {
        let spec = make_spec(vec![make_behavior(BehaviorCategory::HappyPath, vec![])]);
        let counts = count_categories(&spec);
        assert_eq!(counts, vec![("happy_path", 1)]);
    }

    // ── collect_assertion_types ────────────────────────

    #[test]
    /// inspect: assertion_type_collection
    fn assertion_type_collection() {
        let spec = make_spec(vec![make_behavior(
            BehaviorCategory::HappyPath,
            vec![
                Assertion::Equals {
                    field: "f".to_string(),
                    value: "v".to_string(),
                },
                Assertion::Contains {
                    field: "f".to_string(),
                    value: "v".to_string(),
                },
                Assertion::Prose("test".to_string()),
            ],
        )]);
        let types = collect_assertion_types(&spec);
        assert_eq!(types, vec![("contains", 1), ("equals", 1), ("prose", 1)]);
    }

    #[test]
    /// inspect: assertion_types_empty
    fn assertion_types_empty() {
        let spec = make_spec(vec![make_behavior(BehaviorCategory::HappyPath, vec![])]);
        let types = collect_assertion_types(&spec);
        assert!(types.is_empty());
    }

    // ── count_constraint_types ────────────────────────

    #[test]
    /// inspect: nfr_constraint_type_counts
    fn nfr_constraint_type_counts() {
        let nfr = make_nfr(vec![
            make_nfr_constraint(ConstraintType::Metric),
            make_nfr_constraint(ConstraintType::Metric),
            make_nfr_constraint(ConstraintType::Metric),
            make_nfr_constraint(ConstraintType::Rule),
        ]);
        let (metric, rule) = count_constraint_types(&nfr);
        assert_eq!(metric, 3);
        assert_eq!(rule, 1);
    }

    #[test]
    /// inspect: nfr_all_rules
    fn nfr_all_rules() {
        let nfr = make_nfr(vec![
            make_nfr_constraint(ConstraintType::Rule),
            make_nfr_constraint(ConstraintType::Rule),
        ]);
        let (metric, rule) = count_constraint_types(&nfr);
        assert_eq!(metric, 0);
        assert_eq!(rule, 2);
    }
}
