use std::collections::HashMap;

use crate::model::{Assertion, BehaviorCategory, ConstraintType, NfrSpec, Spec};

/// Metadata computed from a parsed spec.
pub struct SpecInspectResult {
    pub name: String,
    pub version: String,
    pub title: String,
    pub behavior_count: usize,
    /// Sorted (category_label, count) pairs.
    pub categories: Vec<(String, usize)>,
    /// Sorted (assertion_type_label, count) pairs.
    pub assertion_types: Vec<(String, usize)>,
    /// (dep_name, version_constraint) pairs.
    pub dependencies: Vec<(String, String)>,
    pub has_error_case: bool,
}

/// Metadata computed from a parsed NFR spec.
pub struct NfrInspectResult {
    pub category: String,
    pub version: String,
    pub title: String,
    pub constraint_count: usize,
    pub metric_count: usize,
    pub rule_count: usize,
}

/// Compute inspect metadata from a parsed spec.
pub fn inspect_spec(spec: &Spec) -> SpecInspectResult {
    let categories = count_categories(spec);
    let assertion_types = collect_assertion_types(spec);
    let has_error_case = spec
        .behaviors
        .iter()
        .any(|b| b.category == BehaviorCategory::ErrorCase);
    let dependencies = spec
        .dependencies
        .iter()
        .map(|d| (d.spec_name.clone(), d.version_constraint.clone()))
        .collect();

    SpecInspectResult {
        name: spec.name.clone(),
        version: spec.version.clone(),
        title: spec.title.clone(),
        behavior_count: spec.behaviors.len(),
        categories,
        assertion_types,
        dependencies,
        has_error_case,
    }
}

/// Compute inspect metadata from a parsed NFR spec.
pub fn inspect_nfr(nfr: &NfrSpec) -> NfrInspectResult {
    let (metric_count, rule_count) = count_constraint_types(nfr);
    NfrInspectResult {
        category: nfr.category.clone(),
        version: nfr.version.clone(),
        title: nfr.title.clone(),
        constraint_count: nfr.constraints.len(),
        metric_count,
        rule_count,
    }
}

/// Count behaviors by category, returning sorted (label, count) pairs.
fn count_categories(spec: &Spec) -> Vec<(String, usize)> {
    let mut categories: HashMap<&str, usize> = HashMap::new();
    for b in &spec.behaviors {
        let label = match b.category {
            BehaviorCategory::HappyPath => "happy_path",
            BehaviorCategory::ErrorCase => "error_case",
            BehaviorCategory::EdgeCase => "edge_case",
        };
        *categories.entry(label).or_insert(0) += 1;
    }
    let mut cat_list: Vec<_> = categories
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    cat_list.sort_by(|a, b| a.0.cmp(&b.0));
    cat_list
}

/// Collect assertion types with counts, returning sorted (label, count) pairs.
fn collect_assertion_types(spec: &Spec) -> Vec<(String, usize)> {
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
    let mut at_list: Vec<_> = assertion_types
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    at_list.sort_by(|a, b| a.0.cmp(&b.0));
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

    // ── inspect_spec ──────────────────────────────────

    #[test]
    /// inspect: category_counts_mixed
    fn category_counts_mixed() {
        let spec = make_spec(vec![
            make_behavior(BehaviorCategory::HappyPath, vec![]),
            make_behavior(BehaviorCategory::HappyPath, vec![]),
            make_behavior(BehaviorCategory::ErrorCase, vec![]),
            make_behavior(BehaviorCategory::EdgeCase, vec![]),
        ]);
        let result = inspect_spec(&spec);
        assert_eq!(
            result.categories,
            vec![
                ("edge_case".to_string(), 1),
                ("error_case".to_string(), 1),
                ("happy_path".to_string(), 2),
            ]
        );
        assert!(result.has_error_case);
    }

    #[test]
    /// inspect: category_counts_single
    fn category_counts_single() {
        let spec = make_spec(vec![make_behavior(BehaviorCategory::HappyPath, vec![])]);
        let result = inspect_spec(&spec);
        assert_eq!(result.categories, vec![("happy_path".to_string(), 1)]);
        assert!(!result.has_error_case);
    }

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
        let result = inspect_spec(&spec);
        assert_eq!(
            result.assertion_types,
            vec![
                ("contains".to_string(), 1),
                ("equals".to_string(), 1),
                ("prose".to_string(), 1),
            ]
        );
    }

    #[test]
    /// inspect: assertion_types_empty
    fn assertion_types_empty() {
        let spec = make_spec(vec![make_behavior(BehaviorCategory::HappyPath, vec![])]);
        let result = inspect_spec(&spec);
        assert!(result.assertion_types.is_empty());
    }

    // ── inspect_nfr ───────────────────────────────────

    #[test]
    /// inspect: nfr_constraint_type_counts
    fn nfr_constraint_type_counts() {
        let nfr = make_nfr(vec![
            make_nfr_constraint(ConstraintType::Metric),
            make_nfr_constraint(ConstraintType::Metric),
            make_nfr_constraint(ConstraintType::Metric),
            make_nfr_constraint(ConstraintType::Rule),
        ]);
        let result = inspect_nfr(&nfr);
        assert_eq!(result.metric_count, 3);
        assert_eq!(result.rule_count, 1);
        assert_eq!(result.constraint_count, 4);
    }

    #[test]
    /// inspect: nfr_all_rules
    fn nfr_all_rules() {
        let nfr = make_nfr(vec![
            make_nfr_constraint(ConstraintType::Rule),
            make_nfr_constraint(ConstraintType::Rule),
        ]);
        let result = inspect_nfr(&nfr);
        assert_eq!(result.metric_count, 0);
        assert_eq!(result.rule_count, 2);
    }
}
