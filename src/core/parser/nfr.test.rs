use super::*;

// ── Helpers ─────────────────────────────────────────────

const MINIMAL_METRIC: &str = "\
nfr performance v1.0.0
title \"Perf\"

description
  Desc.

motivation
  Motiv.


constraint response-time [metric]
  \"Response time\"

  metric \"HTTP p95\"
  threshold < 1s

  verification
    environment staging
    benchmark \"load test\"
    pass \"p95 < 1s\"

  violation high
  overridable yes
";

const MINIMAL_RULE: &str = "\
nfr security v2.1.0
title \"Security\"

description
  Desc.

motivation
  Motiv.


constraint tls-required [rule]
  \"TLS required\"

  rule
    All connections must use TLS 1.2+.

  verification
    static \"code review\"

  violation critical
  overridable no
";

fn nfr_with_constraint(constraint_block: &str) -> String {
    format!(
        "\
nfr performance v1.0.0
title \"Test\"

description
  Desc.

motivation
  Motiv.


{constraint_block}
"
    )
}

// ═══════════════════════════════════════════════════════════════
// Header parsing (nfr-dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// nfr-dsl-format: parse-nfr-declaration
#[test]
fn parse_nfr_declaration() {
    let nfr = parse_nfr(MINIMAL_METRIC).unwrap();
    assert_eq!(nfr.category, "performance");
    assert_eq!(nfr.version, "1.0.0");
}

/// nfr-dsl-format: parse-nfr-title
#[test]
fn parse_nfr_title() {
    let nfr = parse_nfr(MINIMAL_METRIC).unwrap();
    assert_eq!(nfr.title, "Perf");
}

/// nfr-dsl-format: parse-nfr-description-block
#[test]
fn parse_nfr_description_block() {
    let nfr = parse_nfr(MINIMAL_METRIC).unwrap();
    assert_eq!(nfr.description, "Desc.");
}

/// nfr-dsl-format: parse-nfr-motivation-block
#[test]
fn parse_nfr_motivation_block() {
    let nfr = parse_nfr(MINIMAL_METRIC).unwrap();
    assert_eq!(nfr.motivation, "Motiv.");
}

/// nfr-dsl-format: parse-nfr-category-validated
#[test]
fn parse_nfr_category_validated() {
    // All seven valid categories should parse
    for cat in VALID_NFR_CATEGORIES {
        let input = MINIMAL_METRIC.replace("nfr performance", &format!("nfr {cat}"));
        assert!(
            parse_nfr(&input).is_ok(),
            "category '{}' should be valid",
            cat
        );
    }
}

// ═══════════════════════════════════════════════════════════════
// Metric constraint parsing (nfr-dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// nfr-dsl-format: parse-metric-constraint-declaration
#[test]
fn parse_metric_constraint_declaration() {
    let nfr = parse_nfr(MINIMAL_METRIC).unwrap();
    assert_eq!(nfr.constraints.len(), 1);
    assert_eq!(nfr.constraints[0].name, "response-time");
    assert_eq!(nfr.constraints[0].constraint_type, ConstraintType::Metric);
}

/// nfr-dsl-format: parse-metric-field
#[test]
fn parse_metric_field() {
    let nfr = parse_nfr(MINIMAL_METRIC).unwrap();
    match &nfr.constraints[0].body {
        ConstraintBody::Metric { metric, .. } => assert_eq!(metric, "HTTP p95"),
        other => panic!("Expected Metric body, got {other:?}"),
    }
}

/// nfr-dsl-format: parse-threshold-less-than
#[test]
fn parse_threshold_less_than() {
    let nfr = parse_nfr(MINIMAL_METRIC).unwrap();
    match &nfr.constraints[0].body {
        ConstraintBody::Metric {
            threshold_operator,
            threshold_value,
            ..
        } => {
            assert_eq!(threshold_operator, "<");
            assert_eq!(threshold_value, "1s");
        }
        other => panic!("Expected Metric body, got {other:?}"),
    }
}

/// nfr-dsl-format: parse-threshold-greater-than
#[test]
fn parse_threshold_greater_than() {
    let input = MINIMAL_METRIC.replace("threshold < 1s", "threshold > 100");
    let nfr = parse_nfr(&input).unwrap();
    match &nfr.constraints[0].body {
        ConstraintBody::Metric {
            threshold_operator,
            threshold_value,
            ..
        } => {
            assert_eq!(threshold_operator, ">");
            assert_eq!(threshold_value, "100");
        }
        other => panic!("Expected Metric body, got {other:?}"),
    }
}

/// nfr-dsl-format: parse-threshold-less-or-equal
#[test]
fn parse_threshold_less_or_equal() {
    let input = MINIMAL_METRIC.replace("threshold < 1s", "threshold <= 500ms");
    let nfr = parse_nfr(&input).unwrap();
    match &nfr.constraints[0].body {
        ConstraintBody::Metric {
            threshold_operator, ..
        } => assert_eq!(threshold_operator, "<="),
        other => panic!("Expected Metric body, got {other:?}"),
    }
}

/// nfr-dsl-format: parse-threshold-greater-or-equal
#[test]
fn parse_threshold_greater_or_equal() {
    let input = MINIMAL_METRIC.replace("threshold < 1s", "threshold >= 99.9%");
    let nfr = parse_nfr(&input).unwrap();
    match &nfr.constraints[0].body {
        ConstraintBody::Metric {
            threshold_operator,
            threshold_value,
            ..
        } => {
            assert_eq!(threshold_operator, ">=");
            assert_eq!(threshold_value, "99.9%");
        }
        other => panic!("Expected Metric body, got {other:?}"),
    }
}

/// nfr-dsl-format: parse-threshold-equals
#[test]
fn parse_threshold_equals() {
    let input = MINIMAL_METRIC.replace("threshold < 1s", "threshold == 0");
    let nfr = parse_nfr(&input).unwrap();
    match &nfr.constraints[0].body {
        ConstraintBody::Metric {
            threshold_operator, ..
        } => assert_eq!(threshold_operator, "=="),
        other => panic!("Expected Metric body, got {other:?}"),
    }
}

/// nfr-dsl-format: parse-metric-verification-block
#[test]
fn parse_metric_verification_block() {
    let nfr = parse_nfr(MINIMAL_METRIC).unwrap();
    match &nfr.constraints[0].body {
        ConstraintBody::Metric { verification, .. } => {
            assert_eq!(verification.environments, vec!["staging"]);
            assert_eq!(verification.benchmarks, vec!["load test"]);
            assert_eq!(verification.passes, vec!["p95 < 1s"]);
            assert!(verification.datasets.is_empty());
        }
        other => panic!("Expected Metric body, got {other:?}"),
    }
}

/// nfr-dsl-format: parse-environment-all
#[test]
fn parse_environment_all() {
    let input = MINIMAL_METRIC.replace("environment staging", "environment staging, production");
    let nfr = parse_nfr(&input).unwrap();
    match &nfr.constraints[0].body {
        ConstraintBody::Metric { verification, .. } => {
            assert_eq!(verification.environments, vec!["staging", "production"]);
        }
        other => panic!("Expected Metric body, got {other:?}"),
    }
}

/// nfr-dsl-format: parse-metric-verification-without-dataset
#[test]
fn parse_metric_verification_without_dataset() {
    let nfr = parse_nfr(MINIMAL_METRIC).unwrap();
    match &nfr.constraints[0].body {
        ConstraintBody::Metric { verification, .. } => {
            assert!(verification.datasets.is_empty());
        }
        other => panic!("Expected Metric body, got {other:?}"),
    }
}

/// nfr-dsl-format: parse-multiple-verification-lines
#[test]
fn parse_multiple_verification_lines() {
    let input = MINIMAL_METRIC
        .replace(
            "    benchmark \"load test\"\n    pass \"p95 < 1s\"",
            "    benchmark \"load test\"\n    benchmark \"soak test\"\n    dataset \"fixtures/perf.csv\"\n    pass \"p95 < 1s\"\n    pass \"p99 < 2s\"",
        );
    let nfr = parse_nfr(&input).unwrap();
    match &nfr.constraints[0].body {
        ConstraintBody::Metric { verification, .. } => {
            assert_eq!(verification.benchmarks.len(), 2);
            assert_eq!(verification.datasets, vec!["fixtures/perf.csv"]);
            assert_eq!(verification.passes.len(), 2);
        }
        other => panic!("Expected Metric body, got {other:?}"),
    }
}

// ═══════════════════════════════════════════════════════════════
// Rule constraint parsing (nfr-dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// nfr-dsl-format: parse-rule-constraint-declaration
#[test]
fn parse_rule_constraint_declaration() {
    let nfr = parse_nfr(MINIMAL_RULE).unwrap();
    assert_eq!(nfr.constraints[0].name, "tls-required");
    assert_eq!(nfr.constraints[0].constraint_type, ConstraintType::Rule);
}

/// nfr-dsl-format: parse-rule-text-block
#[test]
fn parse_rule_text_block() {
    let nfr = parse_nfr(MINIMAL_RULE).unwrap();
    match &nfr.constraints[0].body {
        ConstraintBody::Rule { rule_text, .. } => {
            assert_eq!(rule_text, "All connections must use TLS 1.2+.");
        }
        other => panic!("Expected Rule body, got {other:?}"),
    }
}

/// nfr-dsl-format: parse-rule-verification-static-only
#[test]
fn parse_rule_verification_static_only() {
    let nfr = parse_nfr(MINIMAL_RULE).unwrap();
    match &nfr.constraints[0].body {
        ConstraintBody::Rule { verification, .. } => {
            assert_eq!(verification.statics, vec!["code review"]);
            assert!(verification.runtimes.is_empty());
        }
        other => panic!("Expected Rule body, got {other:?}"),
    }
}

/// nfr-dsl-format: parse-rule-verification-runtime-only
#[test]
fn parse_rule_verification_runtime_only() {
    let input = MINIMAL_RULE.replace("    static \"code review\"", "    runtime \"TLS scanner\"");
    let nfr = parse_nfr(&input).unwrap();
    match &nfr.constraints[0].body {
        ConstraintBody::Rule { verification, .. } => {
            assert!(verification.statics.is_empty());
            assert_eq!(verification.runtimes, vec!["TLS scanner"]);
        }
        other => panic!("Expected Rule body, got {other:?}"),
    }
}

/// nfr-dsl-format: parse-rule-verification-both
#[test]
fn parse_rule_verification_both() {
    let input = MINIMAL_RULE.replace(
        "    static \"code review\"",
        "    static \"code review\"\n    runtime \"TLS scanner\"",
    );
    let nfr = parse_nfr(&input).unwrap();
    match &nfr.constraints[0].body {
        ConstraintBody::Rule { verification, .. } => {
            assert_eq!(verification.statics, vec!["code review"]);
            assert_eq!(verification.runtimes, vec!["TLS scanner"]);
        }
        other => panic!("Expected Rule body, got {other:?}"),
    }
}

// ═══════════════════════════════════════════════════════════════
// Violation & overridable (nfr-dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// nfr-dsl-format: parse-violation-severity
#[test]
fn parse_violation_severity() {
    let nfr = parse_nfr(MINIMAL_METRIC).unwrap();
    assert_eq!(nfr.constraints[0].violation, "high");

    let nfr = parse_nfr(MINIMAL_RULE).unwrap();
    assert_eq!(nfr.constraints[0].violation, "critical");

    for sev in &["critical", "high", "medium", "low"] {
        let input = MINIMAL_METRIC.replace("violation high", &format!("violation {sev}"));
        let nfr = parse_nfr(&input).unwrap();
        assert_eq!(nfr.constraints[0].violation, *sev);
    }
}

/// nfr-dsl-format: parse-overridable-values
#[test]
fn parse_overridable_values() {
    let nfr = parse_nfr(MINIMAL_METRIC).unwrap();
    assert!(nfr.constraints[0].overridable);

    let nfr = parse_nfr(MINIMAL_RULE).unwrap();
    assert!(!nfr.constraints[0].overridable);
}

// ═══════════════════════════════════════════════════════════════
// Multiple constraints (nfr-dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// nfr-dsl-format: parse-multiple-constraints
#[test]
fn parse_multiple_constraints() {
    let input = "\
nfr performance v1.0.0
title \"Perf\"

description
  Desc.

motivation
  Motiv.


constraint response-time [metric]
  \"RT\"

  metric \"p95\"
  threshold < 1s

  verification
    environment staging
    benchmark \"load\"
    pass \"ok\"

  violation high
  overridable yes


constraint no-blocking [rule]
  \"No blocking IO\"

  rule
    Must be async.

  verification
    static \"code review\"

  violation medium
  overridable no
";
    let nfr = parse_nfr(input).unwrap();
    assert_eq!(nfr.constraints.len(), 2);
    assert_eq!(nfr.constraints[0].name, "response-time");
    assert_eq!(nfr.constraints[0].constraint_type, ConstraintType::Metric);
    assert_eq!(nfr.constraints[1].name, "no-blocking");
    assert_eq!(nfr.constraints[1].constraint_type, ConstraintType::Rule);
}

// ═══════════════════════════════════════════════════════════════
// Comments and blank lines (nfr-dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// nfr-dsl-format: ignore-nfr-comments
#[test]
fn ignore_nfr_comments() {
    let input = "\
nfr performance v1.0.0
# a comment
title \"Perf\"

description
  Desc.

motivation
  Motiv.

# comment before constraint

constraint response-time [metric]
  \"RT\"

  metric \"p95\"
  threshold < 1s

  verification
    environment staging
    benchmark \"load\"
    pass \"ok\"

  violation high
  overridable yes
";
    let nfr = parse_nfr(input).unwrap();
    assert_eq!(nfr.category, "performance");
    assert_eq!(nfr.constraints.len(), 1);
}

/// nfr-dsl-format: ignore-nfr-blank-lines
#[test]
fn ignore_nfr_blank_lines() {
    // Extra blank lines between sections
    let input =
        MINIMAL_METRIC.replace("motivation\n  Motiv.\n\n\n", "motivation\n  Motiv.\n\n\n\n");
    assert!(parse_nfr(&input).is_ok());
}

// ═══════════════════════════════════════════════════════════════
// Error cases — header (nfr-dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// nfr-dsl-format: reject-missing-nfr-declaration
#[test]
fn reject_missing_nfr_declaration() {
    let input = "title \"Test\"\n\ndescription\n  D.\n\nmotivation\n  M.\n";
    let errors = parse_nfr(input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("nfr"));
}

/// nfr-dsl-format: reject-nfr-missing-version
#[test]
fn reject_nfr_missing_version() {
    let input = MINIMAL_METRIC.replace("nfr performance v1.0.0", "nfr performance");
    let errors = parse_nfr(&input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("version"));
}

/// nfr-dsl-format: reject-invalid-nfr-category
#[test]
fn reject_invalid_nfr_category() {
    let input = MINIMAL_METRIC.replace("nfr performance", "nfr banana");
    let errors = parse_nfr(&input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("banana"));
    assert!(errors[0].message.contains("performance"));
}

/// nfr-dsl-format: reject-nfr-missing-title
#[test]
fn reject_nfr_missing_title() {
    let input = "\
nfr performance v1.0.0

description
  D.

motivation
  M.


constraint c [metric]
  \"C\"

  metric \"m\"
  threshold < 1s

  verification
    environment all
    benchmark \"b\"
    pass \"p\"

  violation high
  overridable yes
";
    let errors = parse_nfr(input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("title"));
}

/// nfr-dsl-format: reject-nfr-missing-description
#[test]
fn reject_nfr_missing_description() {
    let input = "\
nfr performance v1.0.0
title \"P\"

motivation
  M.


constraint c [metric]
  \"C\"

  metric \"m\"
  threshold < 1s

  verification
    environment all
    benchmark \"b\"
    pass \"p\"

  violation high
  overridable yes
";
    let errors = parse_nfr(input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("description"));
}

/// nfr-dsl-format: reject-nfr-missing-motivation
#[test]
fn reject_nfr_missing_motivation() {
    let input = "\
nfr performance v1.0.0
title \"P\"

description
  D.


constraint c [metric]
  \"C\"

  metric \"m\"
  threshold < 1s

  verification
    environment all
    benchmark \"b\"
    pass \"p\"

  violation high
  overridable yes
";
    let errors = parse_nfr(input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("motivation"));
}

// ═══════════════════════════════════════════════════════════════
// Error cases — constraints (nfr-dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// nfr-dsl-format: reject-nfr-no-constraints
#[test]
fn reject_nfr_no_constraints() {
    let input = "\
nfr performance v1.0.0
title \"P\"

description
  D.

motivation
  M.
";
    let errors = parse_nfr(input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("constraint"));
}

/// nfr-dsl-format: reject-unknown-constraint-type
#[test]
fn reject_unknown_constraint_type() {
    let input = nfr_with_constraint(
        "\
constraint c [banana]
  \"C\"

  metric \"m\"
  threshold < 1s

  verification
    environment all
    benchmark \"b\"
    pass \"p\"

  violation high
  overridable yes",
    );
    let errors = parse_nfr(&input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("banana"));
}

/// nfr-dsl-format: reject-constraint-without-description
#[test]
fn reject_constraint_without_description() {
    let input = nfr_with_constraint(
        "\
constraint c [metric]

  metric \"m\"
  threshold < 1s

  verification
    environment all
    benchmark \"b\"
    pass \"p\"

  violation high
  overridable yes",
    );
    let errors = parse_nfr(&input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("description"));
}

/// nfr-dsl-format: reject-threshold-invalid-operator
#[test]
fn reject_threshold_invalid_operator() {
    let input = MINIMAL_METRIC.replace("threshold < 1s", "threshold ~= 1s");
    let errors = parse_nfr(&input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("~="));
}

/// nfr-dsl-format: reject-metric-missing-required-fields
#[test]
fn reject_metric_missing_metric_field() {
    let input = nfr_with_constraint(
        "\
constraint c [metric]
  \"C\"

  threshold < 1s

  verification
    environment all
    benchmark \"b\"
    pass \"p\"

  violation high
  overridable yes",
    );
    let errors = parse_nfr(&input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("metric"));
}

/// nfr-dsl-format: reject-metric-verification-missing-required
#[test]
fn reject_metric_verification_missing_environment() {
    let input = nfr_with_constraint(
        "\
constraint c [metric]
  \"C\"

  metric \"m\"
  threshold < 1s

  verification
    benchmark \"b\"
    pass \"p\"

  violation high
  overridable yes",
    );
    let errors = parse_nfr(&input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("environment"));
}

/// nfr-dsl-format: reject-metric-verification-missing-required
#[test]
fn reject_metric_verification_missing_benchmark() {
    let input = nfr_with_constraint(
        "\
constraint c [metric]
  \"C\"

  metric \"m\"
  threshold < 1s

  verification
    environment all
    pass \"p\"

  violation high
  overridable yes",
    );
    let errors = parse_nfr(&input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("benchmark"));
}

/// nfr-dsl-format: reject-metric-verification-missing-required
#[test]
fn reject_metric_verification_missing_pass() {
    let input = nfr_with_constraint(
        "\
constraint c [metric]
  \"C\"

  metric \"m\"
  threshold < 1s

  verification
    environment all
    benchmark \"b\"

  violation high
  overridable yes",
    );
    let errors = parse_nfr(&input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("pass"));
}

/// nfr-dsl-format: reject-rule-missing-required-fields
#[test]
fn reject_rule_missing_rule_block() {
    let input = nfr_with_constraint(
        "\
constraint c [rule]
  \"C\"

  verification
    static \"s\"

  violation high
  overridable yes",
    );
    let errors = parse_nfr(&input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("rule"));
}

/// nfr-dsl-format: reject-rule-verification-empty
#[test]
fn reject_rule_verification_empty() {
    let input = nfr_with_constraint(
        "\
constraint c [rule]
  \"C\"

  rule
    Invariant.

  verification

  violation high
  overridable yes",
    );
    let errors = parse_nfr(&input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("static") || errors[0].message.contains("runtime"));
}

/// nfr-dsl-format: reject-invalid-violation-severity
#[test]
fn reject_invalid_violation_severity() {
    let input = MINIMAL_METRIC.replace("violation high", "violation banana");
    let errors = parse_nfr(&input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("banana"));
}

/// nfr-dsl-format: reject-invalid-overridable-value
#[test]
fn reject_invalid_overridable_value() {
    let input = MINIMAL_METRIC.replace("overridable yes", "overridable maybe");
    let errors = parse_nfr(&input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("maybe"));
}

/// nfr-dsl-format: reject-nfr-tab-indentation
#[test]
fn reject_nfr_tab_indentation() {
    let input = MINIMAL_METRIC.replace("  Desc.", "\tDesc.");
    let errors = parse_nfr(&input).unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("tab"));
}

// ═══════════════════════════════════════════════════════════════
// Constraint description field
// ═══════════════════════════════════════════════════════════════

#[test]
fn parse_constraint_description() {
    let nfr = parse_nfr(MINIMAL_METRIC).unwrap();
    assert_eq!(nfr.constraints[0].description, "Response time");
}

// ═══════════════════════════════════════════════════════════════
// Full integration
// ═══════════════════════════════════════════════════════════════

#[test]
fn parse_full_nfr() {
    let nfr = parse_nfr(MINIMAL_RULE).unwrap();
    assert_eq!(nfr.category, "security");
    assert_eq!(nfr.version, "2.1.0");
    assert_eq!(nfr.title, "Security");
    assert_eq!(nfr.constraints.len(), 1);
    assert_eq!(nfr.constraints[0].name, "tls-required");
    assert_eq!(nfr.constraints[0].constraint_type, ConstraintType::Rule);
    assert_eq!(nfr.constraints[0].violation, "critical");
    assert!(!nfr.constraints[0].overridable);
}

#[test]
fn reject_empty_input() {
    let errors = parse_nfr("").unwrap_err();
    assert!(!errors.is_empty());
}
