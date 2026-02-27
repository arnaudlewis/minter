use std::collections::HashMap;

use crate::model::{BehaviorNfrRef, ConstraintBody, ConstraintType, NfrSpec, Spec};

#[derive(Debug)]
pub struct CrossRefError {
    pub message: String,
}

impl std::fmt::Display for CrossRefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Cross-validate a spec's NFR references against the resolved NFR specs.
/// `nfr_specs` maps category name -> parsed NfrSpec.
pub fn cross_validate(
    spec: &Spec,
    nfr_specs: &HashMap<String, NfrSpec>,
) -> Result<(), Vec<CrossRefError>> {
    let mut errors = Vec::new();

    // Collect all NFR refs: spec-level + behavior-level
    for nfr_ref in &spec.nfr_refs {
        check_category_exists(&nfr_ref.category, nfr_specs, &mut errors);
        if let Some(anchor) = &nfr_ref.anchor {
            check_anchor_exists(&nfr_ref.category, anchor, nfr_specs, &mut errors);
        }
    }

    for behavior in &spec.behaviors {
        for nfr_ref in &behavior.nfr_refs {
            check_category_exists(&nfr_ref.category, nfr_specs, &mut errors);
            check_anchor_exists(&nfr_ref.category, &nfr_ref.anchor, nfr_specs, &mut errors);

            if nfr_ref.override_operator.is_some() {
                check_override(nfr_ref, nfr_specs, &mut errors);
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_category_exists(
    category: &str,
    nfr_specs: &HashMap<String, NfrSpec>,
    errors: &mut Vec<CrossRefError>,
) {
    if !nfr_specs.contains_key(category) {
        errors.push(CrossRefError {
            message: format!(
                "NFR category '{}' not found — no .nfr file declares this category",
                category
            ),
        });
    }
}

fn check_anchor_exists(
    category: &str,
    anchor: &str,
    nfr_specs: &HashMap<String, NfrSpec>,
    errors: &mut Vec<CrossRefError>,
) {
    if let Some(nfr) = nfr_specs.get(category) {
        let constraint_exists = nfr.constraints.iter().any(|c| c.name == anchor);
        if !constraint_exists {
            errors.push(CrossRefError {
                message: format!(
                    "Constraint '{}' not found in NFR category '{}'",
                    anchor, category
                ),
            });
        }
    }
    // If category doesn't exist, that error is already reported by check_category_exists
}

fn check_override(
    nfr_ref: &BehaviorNfrRef,
    nfr_specs: &HashMap<String, NfrSpec>,
    errors: &mut Vec<CrossRefError>,
) {
    let nfr = match nfr_specs.get(&nfr_ref.category) {
        Some(n) => n,
        None => return, // category missing error already reported
    };

    let constraint = match nfr.constraints.iter().find(|c| c.name == nfr_ref.anchor) {
        Some(c) => c,
        None => return, // anchor missing error already reported
    };

    let override_op = match &nfr_ref.override_operator {
        Some(op) => op.as_str(),
        None => return,
    };
    let override_val = match &nfr_ref.override_value {
        Some(v) => v.as_str(),
        None => return,
    };

    // Rule 4: overridable must be yes
    if !constraint.overridable {
        errors.push(CrossRefError {
            message: format!(
                "Constraint '{}' in '{}' is not overridable",
                nfr_ref.anchor, nfr_ref.category
            ),
        });
        return;
    }

    // Rule 5: must be a metric constraint (not rule)
    if constraint.constraint_type == ConstraintType::Rule {
        errors.push(CrossRefError {
            message: format!(
                "Cannot override rule constraint '{}' in '{}' — only metric constraints support overrides",
                nfr_ref.anchor, nfr_ref.category
            ),
        });
        return;
    }

    // Rule 6: operator must match the original threshold
    let (original_op, original_val) = match &constraint.body {
        ConstraintBody::Metric {
            threshold_operator,
            threshold_value,
            ..
        } => (threshold_operator.as_str(), threshold_value.as_str()),
        ConstraintBody::Rule { .. } => return,
    };

    if override_op != original_op {
        errors.push(CrossRefError {
            message: format!(
                "Override operator '{}' for '{}' does not match original threshold operator '{}'",
                override_op, nfr_ref.anchor, original_op
            ),
        });
        return;
    }

    // Rule 7: value must be stricter than the default
    let original_norm = normalize_value(original_val);
    let override_norm = normalize_value(override_val);

    if let (Some(orig), Some(ovr)) = (original_norm, override_norm) {
        let is_stricter = match override_op {
            "<" | "<=" => ovr < orig,
            ">" | ">=" => ovr > orig,
            "==" => (ovr - orig).abs() < f64::EPSILON,
            _ => true,
        };

        if !is_stricter {
            errors.push(CrossRefError {
                message: format!(
                    "Override value '{}' for '{}' is not stricter than the default '{}'",
                    override_val, nfr_ref.anchor, original_val
                ),
            });
        }
    }
}

/// Normalize a threshold value with optional unit to a canonical f64.
/// Supports: ms, s, %, KB, MB, GB, or bare numbers.
fn normalize_value(val: &str) -> Option<f64> {
    let val = val.trim();

    if let Some(num) = val.strip_suffix("ms") {
        num.trim().parse::<f64>().ok()
    } else if let Some(num) = val.strip_suffix('s') {
        num.trim().parse::<f64>().ok().map(|v| v * 1000.0) // convert to ms
    } else if let Some(num) = val.strip_suffix('%') {
        num.trim().parse::<f64>().ok()
    } else if let Some(num) = val.strip_suffix("GB") {
        num.trim().parse::<f64>().ok().map(|v| v * 1_000_000.0) // convert to KB
    } else if let Some(num) = val.strip_suffix("MB") {
        num.trim().parse::<f64>().ok().map(|v| v * 1_000.0) // convert to KB
    } else if let Some(num) = val.strip_suffix("KB") {
        num.trim().parse::<f64>().ok()
    } else {
        val.parse::<f64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use std::collections::HashMap;

    // ── Test helpers ──────────────────────────────────────

    fn make_metric_constraint(
        name: &str,
        operator: &str,
        value: &str,
        overridable: bool,
    ) -> NfrConstraint {
        NfrConstraint {
            name: name.to_string(),
            constraint_type: ConstraintType::Metric,
            description: "test constraint".to_string(),
            body: ConstraintBody::Metric {
                metric: "test-metric".to_string(),
                threshold_operator: operator.to_string(),
                threshold_value: value.to_string(),
                verification: MetricVerification {
                    environments: vec![],
                    benchmarks: vec![],
                    datasets: vec![],
                    passes: vec![],
                },
            },
            violation: "test violation".to_string(),
            overridable,
        }
    }

    fn make_rule_constraint(name: &str) -> NfrConstraint {
        NfrConstraint {
            name: name.to_string(),
            constraint_type: ConstraintType::Rule,
            description: "test rule".to_string(),
            body: ConstraintBody::Rule {
                rule_text: "some rule".to_string(),
                verification: RuleVerification {
                    statics: vec![],
                    runtimes: vec![],
                },
            },
            violation: "test violation".to_string(),
            overridable: true,
        }
    }

    fn make_nfr_spec(category: &str, constraints: Vec<NfrConstraint>) -> NfrSpec {
        NfrSpec {
            category: category.to_string(),
            version: "1.0.0".to_string(),
            title: "Test NFR".to_string(),
            description: "Test description".to_string(),
            motivation: "Test motivation".to_string(),
            constraints,
        }
    }

    fn make_behavior_nfr_ref(
        category: &str,
        anchor: &str,
        override_op: Option<&str>,
        override_val: Option<&str>,
    ) -> BehaviorNfrRef {
        BehaviorNfrRef {
            category: category.to_string(),
            anchor: anchor.to_string(),
            override_operator: override_op.map(|s| s.to_string()),
            override_value: override_val.map(|s| s.to_string()),
        }
    }

    fn make_minimal_spec(nfr_refs: Vec<NfrRef>, behaviors: Vec<Behavior>) -> Spec {
        Spec {
            name: "test-spec".to_string(),
            version: "1.0.0".to_string(),
            title: "Test Spec".to_string(),
            description: "Test description".to_string(),
            motivation: "Test motivation".to_string(),
            nfr_refs,
            behaviors,
            dependencies: vec![],
        }
    }

    fn make_behavior_with_nfr_refs(nfr_refs: Vec<BehaviorNfrRef>) -> Behavior {
        Behavior {
            name: "test-behavior".to_string(),
            category: BehaviorCategory::HappyPath,
            description: "Test behavior".to_string(),
            nfr_refs,
            preconditions: vec![],
            action: Action {
                name: "do-something".to_string(),
                inputs: vec![],
            },
            postconditions: vec![],
        }
    }

    // ── normalize_value tests ────────────────────────────

    #[test]
    /// crossref: normalize_ms
    fn normalize_ms() {
        assert_eq!(normalize_value("200ms"), Some(200.0));
    }

    #[test]
    /// crossref: normalize_s_to_ms
    fn normalize_s_to_ms() {
        assert_eq!(normalize_value("2s"), Some(2000.0));
    }

    #[test]
    /// crossref: normalize_percent
    fn normalize_percent() {
        assert_eq!(normalize_value("99.9%"), Some(99.9));
    }

    #[test]
    /// crossref: normalize_kb
    fn normalize_kb() {
        assert_eq!(normalize_value("512KB"), Some(512.0));
    }

    #[test]
    /// crossref: normalize_mb_to_kb
    fn normalize_mb_to_kb() {
        assert_eq!(normalize_value("1MB"), Some(1000.0));
    }

    #[test]
    /// crossref: normalize_gb_to_kb
    fn normalize_gb_to_kb() {
        assert_eq!(normalize_value("2GB"), Some(2_000_000.0));
    }

    #[test]
    /// crossref: normalize_bare_number
    fn normalize_bare_number() {
        assert_eq!(normalize_value("42"), Some(42.0));
    }

    #[test]
    /// crossref: normalize_invalid
    fn normalize_invalid() {
        assert_eq!(normalize_value("abc"), None);
    }

    #[test]
    /// crossref: normalize_bare_float
    fn normalize_bare_float() {
        #[allow(clippy::approx_constant)]
        let expected = 3.14;
        assert_eq!(normalize_value("3.14"), Some(expected));
    }

    // ── check_override tests ─────────────────────────────

    #[test]
    /// crossref: override_not_overridable
    fn override_not_overridable() {
        let nfr = make_nfr_spec(
            "performance",
            vec![make_metric_constraint(
                "api-response-time",
                "<",
                "1s",
                false,
            )],
        );
        let mut nfr_specs = HashMap::new();
        nfr_specs.insert("performance".to_string(), nfr);

        let nfr_ref =
            make_behavior_nfr_ref("performance", "api-response-time", Some("<"), Some("500ms"));
        let mut errors = Vec::new();
        check_override(&nfr_ref, &nfr_specs, &mut errors);

        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("not overridable"));
    }

    #[test]
    /// crossref: override_rule_constraint
    fn override_rule_constraint() {
        let nfr = make_nfr_spec("security", vec![make_rule_constraint("tls-required")]);
        let mut nfr_specs = HashMap::new();
        nfr_specs.insert("security".to_string(), nfr);

        let nfr_ref = make_behavior_nfr_ref("security", "tls-required", Some("<"), Some("500ms"));
        let mut errors = Vec::new();
        check_override(&nfr_ref, &nfr_specs, &mut errors);

        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("only metric"));
    }

    #[test]
    /// crossref: override_operator_mismatch
    fn override_operator_mismatch() {
        let nfr = make_nfr_spec(
            "performance",
            vec![make_metric_constraint("api-response-time", "<", "1s", true)],
        );
        let mut nfr_specs = HashMap::new();
        nfr_specs.insert("performance".to_string(), nfr);

        let nfr_ref =
            make_behavior_nfr_ref("performance", "api-response-time", Some(">"), Some("500ms"));
        let mut errors = Vec::new();
        check_override(&nfr_ref, &nfr_specs, &mut errors);

        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("does not match"));
    }

    #[test]
    /// crossref: override_stricter_less_than
    fn override_stricter_less_than() {
        let nfr = make_nfr_spec(
            "performance",
            vec![make_metric_constraint("api-response-time", "<", "1s", true)],
        );
        let mut nfr_specs = HashMap::new();
        nfr_specs.insert("performance".to_string(), nfr);

        let nfr_ref =
            make_behavior_nfr_ref("performance", "api-response-time", Some("<"), Some("500ms"));
        let mut errors = Vec::new();
        check_override(&nfr_ref, &nfr_specs, &mut errors);

        assert!(
            errors.is_empty(),
            "Expected no errors but got: {:?}",
            errors
        );
    }

    #[test]
    /// crossref: override_not_stricter_less_than
    fn override_not_stricter_less_than() {
        let nfr = make_nfr_spec(
            "performance",
            vec![make_metric_constraint(
                "api-response-time",
                "<",
                "500ms",
                true,
            )],
        );
        let mut nfr_specs = HashMap::new();
        nfr_specs.insert("performance".to_string(), nfr);

        let nfr_ref =
            make_behavior_nfr_ref("performance", "api-response-time", Some("<"), Some("1s"));
        let mut errors = Vec::new();
        check_override(&nfr_ref, &nfr_specs, &mut errors);

        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("not stricter"));
    }

    #[test]
    /// crossref: override_stricter_greater_than
    fn override_stricter_greater_than() {
        let nfr = make_nfr_spec(
            "reliability",
            vec![make_metric_constraint("uptime", ">", "90%", true)],
        );
        let mut nfr_specs = HashMap::new();
        nfr_specs.insert("reliability".to_string(), nfr);

        let nfr_ref = make_behavior_nfr_ref("reliability", "uptime", Some(">"), Some("95%"));
        let mut errors = Vec::new();
        check_override(&nfr_ref, &nfr_specs, &mut errors);

        assert!(
            errors.is_empty(),
            "Expected no errors but got: {:?}",
            errors
        );
    }

    #[test]
    /// crossref: override_not_stricter_greater_than
    fn override_not_stricter_greater_than() {
        let nfr = make_nfr_spec(
            "reliability",
            vec![make_metric_constraint("uptime", ">", "95%", true)],
        );
        let mut nfr_specs = HashMap::new();
        nfr_specs.insert("reliability".to_string(), nfr);

        let nfr_ref = make_behavior_nfr_ref("reliability", "uptime", Some(">"), Some("90%"));
        let mut errors = Vec::new();
        check_override(&nfr_ref, &nfr_specs, &mut errors);

        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("not stricter"));
    }

    #[test]
    /// crossref: override_equal_operator
    fn override_equal_operator() {
        let nfr = make_nfr_spec(
            "performance",
            vec![make_metric_constraint("fixed-latency", "==", "100ms", true)],
        );
        let mut nfr_specs = HashMap::new();
        nfr_specs.insert("performance".to_string(), nfr);

        let nfr_ref =
            make_behavior_nfr_ref("performance", "fixed-latency", Some("=="), Some("100ms"));
        let mut errors = Vec::new();
        check_override(&nfr_ref, &nfr_specs, &mut errors);

        assert!(
            errors.is_empty(),
            "Expected no errors but got: {:?}",
            errors
        );
    }

    // ── check_category_exists / check_anchor_exists tests ──

    #[test]
    /// crossref: category_exists
    fn category_exists() {
        let nfr = make_nfr_spec("performance", vec![]);
        let mut nfr_specs = HashMap::new();
        nfr_specs.insert("performance".to_string(), nfr);

        let mut errors = Vec::new();
        check_category_exists("performance", &nfr_specs, &mut errors);

        assert!(errors.is_empty());
    }

    #[test]
    /// crossref: category_missing
    fn category_missing() {
        let nfr_specs: HashMap<String, NfrSpec> = HashMap::new();

        let mut errors = Vec::new();
        check_category_exists("performance", &nfr_specs, &mut errors);

        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("not found"));
    }

    #[test]
    /// crossref: anchor_exists
    fn anchor_exists() {
        let nfr = make_nfr_spec(
            "performance",
            vec![make_metric_constraint("api-response-time", "<", "1s", true)],
        );
        let mut nfr_specs = HashMap::new();
        nfr_specs.insert("performance".to_string(), nfr);

        let mut errors = Vec::new();
        check_anchor_exists("performance", "api-response-time", &nfr_specs, &mut errors);

        assert!(errors.is_empty());
    }

    #[test]
    /// crossref: anchor_missing
    fn anchor_missing() {
        let nfr = make_nfr_spec("performance", vec![]);
        let mut nfr_specs = HashMap::new();
        nfr_specs.insert("performance".to_string(), nfr);

        let mut errors = Vec::new();
        check_anchor_exists("performance", "api-response-time", &nfr_specs, &mut errors);

        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("not found"));
    }

    // ── cross_validate integration tests ─────────────────

    #[test]
    /// crossref: valid_spec_level_ref
    fn valid_spec_level_ref() {
        let nfr = make_nfr_spec(
            "performance",
            vec![make_metric_constraint("api-response-time", "<", "1s", true)],
        );
        let mut nfr_specs = HashMap::new();
        nfr_specs.insert("performance".to_string(), nfr);

        let spec = make_minimal_spec(
            vec![NfrRef {
                category: "performance".to_string(),
                anchor: None,
            }],
            vec![],
        );

        let result = cross_validate(&spec, &nfr_specs);
        assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
    }

    #[test]
    /// crossref: valid_behavior_level_ref
    fn valid_behavior_level_ref() {
        let nfr = make_nfr_spec(
            "performance",
            vec![make_metric_constraint("api-response-time", "<", "1s", true)],
        );
        let mut nfr_specs = HashMap::new();
        nfr_specs.insert("performance".to_string(), nfr);

        let behavior = make_behavior_with_nfr_refs(vec![make_behavior_nfr_ref(
            "performance",
            "api-response-time",
            None,
            None,
        )]);
        let spec = make_minimal_spec(vec![], vec![behavior]);

        let result = cross_validate(&spec, &nfr_specs);
        assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
    }

    #[test]
    /// crossref: missing_category
    fn missing_category() {
        let nfr_specs: HashMap<String, NfrSpec> = HashMap::new();

        let spec = make_minimal_spec(
            vec![NfrRef {
                category: "performance".to_string(),
                anchor: None,
            }],
            vec![],
        );

        let result = cross_validate(&spec, &nfr_specs);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("not found")));
    }

    #[test]
    /// crossref: missing_anchor
    fn missing_anchor() {
        let nfr = make_nfr_spec("performance", vec![]);
        let mut nfr_specs = HashMap::new();
        nfr_specs.insert("performance".to_string(), nfr);

        let behavior = make_behavior_with_nfr_refs(vec![make_behavior_nfr_ref(
            "performance",
            "nonexistent-anchor",
            None,
            None,
        )]);
        let spec = make_minimal_spec(vec![], vec![behavior]);

        let result = cross_validate(&spec, &nfr_specs);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("not found")));
    }
}
