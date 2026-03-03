mod common;

use common::{VALID_NFR, minter, temp_nfr};
use predicates::prelude::*;

// ═══════════════════════════════════════════════════════════════
// Helper: build a full valid NFR around a single metric constraint
// ═══════════════════════════════════════════════════════════════

fn valid_nfr_with_metric_constraint(constraint_body: &str) -> String {
    format!(
        "\
nfr performance v1.0.0
title \"Performance Requirements\"

description
  Defines performance constraints.

motivation
  Performance matters.


constraint api-response-time [metric]
  \"API endpoints must respond within acceptable latency bounds\"

{constraint_body}

  violation high
  overridable yes
"
    )
}

fn valid_nfr_with_rule_constraint(constraint_body: &str) -> String {
    format!(
        "\
nfr performance v1.0.0
title \"Performance Requirements\"

description
  Defines performance constraints.

motivation
  Performance matters.


constraint no-n-plus-one [rule]
  \"No endpoint may issue unbounded database calls\"

{constraint_body}

  violation high
  overridable no
"
    )
}

fn minimal_metric_body() -> String {
    "\
  metric \"HTTP response time, p95\"
  threshold < 1s

  verification
    environment staging, production
    benchmark \"100 concurrent requests per endpoint\"
    pass \"p95 < threshold\""
        .to_string()
}

fn minimal_rule_body() -> String {
    "\
  rule
    No endpoint may issue more than a fixed number of calls.

  verification
    static \"Query count per request path does not scale with input size\""
        .to_string()
}

// ═══════════════════════════════════════════════════════════════
// Header section (nfr-grammar.spec)
// ═══════════════════════════════════════════════════════════════

// @minter:e2e parse-nfr-declaration
#[test]
fn parse_nfr_declaration() {
    let (_dir, path) = temp_nfr("perf", VALID_NFR);
    minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("performance"))
        .stdout(predicate::str::contains("1.0.0"));
}

// @minter:e2e parse-nfr-category-validated
#[test]
fn parse_nfr_category_validated() {
    let categories = [
        "performance",
        "reliability",
        "security",
        "observability",
        "scalability",
        "cost",
        "operability",
    ];
    for cat in &categories {
        let content = format!(
            "\
nfr {cat} v1.0.0
title \"{cat} Requirements\"

description
  Defines {cat} constraints.

motivation
  {cat} matters.


constraint test-constraint [rule]
  \"A test constraint\"

  rule
    Some invariant.

  verification
    static \"Check it\"

  violation medium
  overridable no
"
        );
        let (_dir, path) = temp_nfr(cat, &content);
        minter().arg("validate").arg(&path).assert().success();
    }
}

// @minter:e2e reject-invalid-nfr-category
#[test]
fn reject_invalid_nfr_category() {
    let content = "\
nfr banana v1.0.0
title \"Banana\"

description
  Bad.

motivation
  Bad.


constraint c [rule]
  \"C\"

  rule
    R.

  verification
    static \"S\"

  violation low
  overridable no
";
    let (_dir, path) = temp_nfr("banana", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::contains("banana"))
        .stderr(predicate::str::contains("category"));
}

// @minter:e2e parse-nfr-title
#[test]
fn parse_nfr_title() {
    let (_dir, path) = temp_nfr("perf", VALID_NFR);
    minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("performance"));
}

// @minter:e2e parse-nfr-description-block
#[test]
fn parse_nfr_description_block() {
    let (_dir, path) = temp_nfr("perf", VALID_NFR);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e parse-nfr-motivation-block
#[test]
fn parse_nfr_motivation_block() {
    let (_dir, path) = temp_nfr("perf", VALID_NFR);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e reject-nfr-missing-version
#[test]
fn reject_nfr_missing_version() {
    let content = "\
nfr performance
title \"Perf\"

description
  D.

motivation
  M.


constraint c [rule]
  \"C\"

  rule
    R.

  verification
    static \"S\"

  violation low
  overridable no
";
    let (_dir, path) = temp_nfr("nover", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::contains("version"));
}

// ═══════════════════════════════════════════════════════════════
// Constraint declaration (nfr-grammar.spec)
// ═══════════════════════════════════════════════════════════════

// @minter:e2e parse-metric-constraint-declaration
#[test]
fn parse_metric_constraint_declaration() {
    let (_dir, path) = temp_nfr("perf", VALID_NFR);
    minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 constraint"));
}

// @minter:e2e parse-rule-constraint-declaration
#[test]
fn parse_rule_constraint_declaration() {
    let content = valid_nfr_with_rule_constraint(&minimal_rule_body());
    let (_dir, path) = temp_nfr("perf-rule", &content);
    minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 constraint"));
}

// @minter:e2e reject-unknown-constraint-type
#[test]
fn reject_unknown_constraint_type() {
    let content = "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.


constraint my-constraint [banana]
  \"Some description\"

  metric \"something\"
  threshold < 1s

  verification
    environment all
    benchmark \"b\"
    pass \"p\"

  violation low
  overridable no
";
    let (_dir, path) = temp_nfr("bad-type", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::contains("banana"));
}

// @minter:e2e reject-constraint-without-description
#[test]
fn reject_constraint_without_description() {
    let content = "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.


constraint my-constraint [metric]
  metric \"Something\"
  threshold < 1s

  verification
    environment all
    benchmark \"b\"
    pass \"p\"

  violation low
  overridable no
";
    let (_dir, path) = temp_nfr("no-desc", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::is_empty().not());
}

// @minter:e2e reject-non-kebab-constraint-name
#[test]
fn reject_non_kebab_constraint_name() {
    let content = "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.


constraint MyConstraint [rule]
  \"Some description\"

  rule
    R.

  verification
    static \"S\"

  violation low
  overridable no
";
    let (_dir, path) = temp_nfr("bad-name", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::contains("MyConstraint"))
        .stderr(predicate::str::contains("kebab"));
}

// ═══════════════════════════════════════════════════════════════
// Metric fields (nfr-grammar.spec)
// ═══════════════════════════════════════════════════════════════

// @minter:e2e parse-metric-field
#[test]
fn parse_metric_field() {
    let body = minimal_metric_body();
    let content = valid_nfr_with_metric_constraint(&body);
    let (_dir, path) = temp_nfr("metric", &content);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e parse-threshold-less-than
#[test]
fn parse_threshold_less_than() {
    let body = "\
  metric \"m\"
  threshold < 500ms

  verification
    environment all
    benchmark \"b\"
    pass \"p\"";
    let content = valid_nfr_with_metric_constraint(body);
    let (_dir, path) = temp_nfr("lt", &content);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e parse-threshold-greater-than
#[test]
fn parse_threshold_greater_than() {
    let body = "\
  metric \"m\"
  threshold > 100

  verification
    environment all
    benchmark \"b\"
    pass \"p\"";
    let content = valid_nfr_with_metric_constraint(body);
    let (_dir, path) = temp_nfr("gt", &content);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e parse-threshold-less-or-equal
#[test]
fn parse_threshold_less_or_equal() {
    let body = "\
  metric \"m\"
  threshold <= 99.9%

  verification
    environment all
    benchmark \"b\"
    pass \"p\"";
    let content = valid_nfr_with_metric_constraint(body);
    let (_dir, path) = temp_nfr("le", &content);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e parse-threshold-greater-or-equal
#[test]
fn parse_threshold_greater_or_equal() {
    let body = "\
  metric \"m\"
  threshold >= 99.9%

  verification
    environment all
    benchmark \"b\"
    pass \"p\"";
    let content = valid_nfr_with_metric_constraint(body);
    let (_dir, path) = temp_nfr("ge", &content);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e parse-threshold-equals
#[test]
fn parse_threshold_equals() {
    let body = "\
  metric \"m\"
  threshold == 100%

  verification
    environment all
    benchmark \"b\"
    pass \"p\"";
    let content = valid_nfr_with_metric_constraint(body);
    let (_dir, path) = temp_nfr("eq", &content);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e reject-threshold-invalid-operator
#[test]
fn reject_threshold_invalid_operator() {
    let body = "\
  metric \"m\"
  threshold != 500ms

  verification
    environment all
    benchmark \"b\"
    pass \"p\"";
    let content = valid_nfr_with_metric_constraint(body);
    let (_dir, path) = temp_nfr("bad-op", &content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::contains("!="))
        .stderr(predicate::str::contains("operator"));
}

// @minter:e2e parse-metric-verification-block
#[test]
fn parse_metric_verification_block() {
    let body = "\
  metric \"m\"
  threshold < 1s

  verification
    environment staging, production
    benchmark \"100 concurrent requests per endpoint\"
    dataset \"Production-representative volume\"
    pass \"p95 < threshold\"";
    let content = valid_nfr_with_metric_constraint(body);
    let (_dir, path) = temp_nfr("verif", &content);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e parse-metric-verification-without-dataset
#[test]
fn parse_metric_verification_without_dataset() {
    let body = "\
  metric \"m\"
  threshold < 1s

  verification
    environment all
    benchmark \"Assert response Content-Length on representative queries\"
    pass \"No response exceeds threshold\"";
    let content = valid_nfr_with_metric_constraint(body);
    let (_dir, path) = temp_nfr("no-dataset", &content);
    minter().arg("validate").arg(&path).assert().success();
}

// ═══════════════════════════════════════════════════════════════
// Rule fields (nfr-grammar.spec)
// ═══════════════════════════════════════════════════════════════

// @minter:e2e parse-rule-text-block
#[test]
fn parse_rule_text_block() {
    let content = valid_nfr_with_rule_constraint(&minimal_rule_body());
    let (_dir, path) = temp_nfr("rule-text", &content);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e parse-rule-verification-static-only
#[test]
fn parse_rule_verification_static_only() {
    let body = "\
  rule
    Some invariant.

  verification
    static \"Query count per request path does not scale with input size\"";
    let content = valid_nfr_with_rule_constraint(body);
    let (_dir, path) = temp_nfr("static-only", &content);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e parse-rule-verification-runtime-only
#[test]
fn parse_rule_verification_runtime_only() {
    let body = "\
  rule
    Some invariant.

  verification
    runtime \"Call every endpoint without auth header, assert 401\"";
    let content = valid_nfr_with_rule_constraint(body);
    let (_dir, path) = temp_nfr("runtime-only", &content);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e parse-rule-verification-both
#[test]
fn parse_rule_verification_both() {
    let body = "\
  rule
    Some invariant.

  verification
    static \"Every database query includes tenant filter\"
    runtime \"Authenticate as tenant A, attempt to access tenant B data\"";
    let content = valid_nfr_with_rule_constraint(body);
    let (_dir, path) = temp_nfr("both-verif", &content);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e reject-rule-verification-empty
#[test]
fn reject_rule_verification_empty() {
    let content = "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.


constraint no-n-plus-one [rule]
  \"No endpoint may issue unbounded database calls\"

  rule
    Some invariant.

  verification

  violation high
  overridable no
";
    let (_dir, path) = temp_nfr("empty-verif", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::is_empty().not());
}

// ═══════════════════════════════════════════════════════════════
// Shared fields (nfr-grammar.spec)
// ═══════════════════════════════════════════════════════════════

// @minter:e2e parse-violation-severity
#[test]
fn parse_violation_severity() {
    for severity in &["critical", "high", "medium", "low"] {
        let content = format!(
            "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.


constraint test-constraint [rule]
  \"A test constraint\"

  rule
    Some invariant.

  verification
    static \"Check it\"

  violation {severity}
  overridable no
"
        );
        let (_dir, path) = temp_nfr(&format!("sev-{severity}"), &content);
        minter().arg("validate").arg(&path).assert().success();
    }
}

// @minter:e2e reject-invalid-violation-severity
#[test]
fn reject_invalid_violation_severity() {
    let content = "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.


constraint test-constraint [rule]
  \"A test constraint\"

  rule
    Some invariant.

  verification
    static \"Check it\"

  violation banana
  overridable no
";
    let (_dir, path) = temp_nfr("bad-sev", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::contains("banana"))
        .stderr(predicate::str::contains("violation"));
}

// @minter:e2e parse-overridable-values
#[test]
fn parse_overridable_values() {
    for val in &["yes", "no"] {
        let content = format!(
            "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.


constraint test-constraint [rule]
  \"A test constraint\"

  rule
    Some invariant.

  verification
    static \"Check it\"

  violation medium
  overridable {val}
"
        );
        let (_dir, path) = temp_nfr(&format!("over-{val}"), &content);
        minter().arg("validate").arg(&path).assert().success();
    }
}

// @minter:e2e reject-invalid-overridable-value
#[test]
fn reject_invalid_overridable_value() {
    let content = "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.


constraint test-constraint [rule]
  \"A test constraint\"

  rule
    Some invariant.

  verification
    static \"Check it\"

  violation medium
  overridable maybe
";
    let (_dir, path) = temp_nfr("bad-over", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::contains("maybe"))
        .stderr(predicate::str::contains("overridable"));
}

// ═══════════════════════════════════════════════════════════════
// Structural errors (nfr-grammar.spec)
// ═══════════════════════════════════════════════════════════════

// @minter:e2e reject-missing-nfr-declaration
#[test]
fn reject_missing_nfr_declaration() {
    let content = "\
title \"My NFR\"

description
  D.

motivation
  M.


constraint c [rule]
  \"C\"

  rule
    R.

  verification
    static \"S\"

  violation low
  overridable no
";
    let (_dir, path) = temp_nfr("no-decl", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::is_empty().not());
}

// @minter:e2e reject-nfr-missing-title
#[test]
fn reject_nfr_missing_title() {
    let content = "\
nfr performance v1.0.0

description
  D.

motivation
  M.


constraint c [rule]
  \"C\"

  rule
    R.

  verification
    static \"S\"

  violation low
  overridable no
";
    let (_dir, path) = temp_nfr("no-title", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::contains("title"));
}

// @minter:e2e reject-nfr-missing-description
#[test]
fn reject_nfr_missing_description() {
    let content = "\
nfr performance v1.0.0
title \"Perf\"

motivation
  M.


constraint c [rule]
  \"C\"

  rule
    R.

  verification
    static \"S\"

  violation low
  overridable no
";
    let (_dir, path) = temp_nfr("no-desc", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::contains("description"));
}

// @minter:e2e reject-nfr-missing-motivation
#[test]
fn reject_nfr_missing_motivation() {
    let content = "\
nfr performance v1.0.0
title \"Perf\"

description
  D.


constraint c [rule]
  \"C\"

  rule
    R.

  verification
    static \"S\"

  violation low
  overridable no
";
    let (_dir, path) = temp_nfr("no-mot", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::contains("motivation"));
}

// @minter:e2e reject-nfr-no-constraints
#[test]
fn reject_nfr_no_constraints() {
    let content = "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.
";
    let (_dir, path) = temp_nfr("no-constraints", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::contains("constraint"));
}

// @minter:e2e reject-duplicate-constraint-names
#[test]
fn reject_duplicate_constraint_names() {
    let content = "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.


constraint api-response-time [rule]
  \"First\"

  rule
    R1.

  verification
    static \"S1\"

  violation high
  overridable no


constraint api-response-time [rule]
  \"Second\"

  rule
    R2.

  verification
    static \"S2\"

  violation high
  overridable no
";
    let (_dir, path) = temp_nfr("dupes", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::contains("api-response-time"))
        .stderr(predicate::str::contains("Duplicate"));
}

// @minter:e2e reject-metric-missing-required-fields
#[test]
fn reject_metric_missing_required_fields() {
    let content = "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.


constraint api-response-time [metric]
  \"API endpoints must respond within acceptable latency bounds\"

  metric \"HTTP response time, p95\"

  violation high
  overridable yes
";
    let (_dir, path) = temp_nfr("missing-metric-fields", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::is_empty().not());
}

// @minter:e2e reject-metric-verification-missing-required
#[test]
fn reject_metric_verification_missing_required() {
    let content = "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.


constraint api-response-time [metric]
  \"API endpoints must respond within acceptable latency bounds\"

  metric \"HTTP response time, p95\"
  threshold < 1s

  verification
    environment all

  violation high
  overridable yes
";
    let (_dir, path) = temp_nfr("missing-verif-fields", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::is_empty().not());
}

// @minter:e2e reject-rule-missing-required-fields
#[test]
fn reject_rule_missing_required_fields() {
    let content = "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.


constraint no-n-plus-one [rule]
  \"No endpoint may issue unbounded database calls\"

  rule
    Some invariant.

  violation high
  overridable no
";
    let (_dir, path) = temp_nfr("missing-rule-verif", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::is_empty().not());
}

// @minter:e2e reject-nfr-tab-indentation
#[test]
fn reject_nfr_tab_indentation() {
    let content = "nfr performance v1.0.0\ntitle \"Perf\"\n\ndescription\n\tD.\n\nmotivation\n  M.\n\nconstraint c [rule]\n  \"C\"\n\n  rule\n    R.\n\n  verification\n    static \"S\"\n\n  violation low\n  overridable no\n";
    let (_dir, path) = temp_nfr("tabs", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(path.to_str().unwrap()))
        .stderr(predicate::str::is_empty().not());
}

// ═══════════════════════════════════════════════════════════════
// Edge cases (nfr-grammar.spec)
// ═══════════════════════════════════════════════════════════════

// @minter:e2e parse-multiple-constraints
#[test]
fn parse_multiple_constraints() {
    let content = "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.


constraint metric-one [metric]
  \"First metric\"

  metric \"m1\"
  threshold < 1s

  verification
    environment all
    benchmark \"b1\"
    pass \"p1\"

  violation high
  overridable yes


constraint metric-two [metric]
  \"Second metric\"

  metric \"m2\"
  threshold > 100

  verification
    environment staging
    benchmark \"b2\"
    pass \"p2\"

  violation medium
  overridable yes


constraint rule-one [rule]
  \"First rule\"

  rule
    Invariant.

  verification
    static \"S1\"

  violation low
  overridable no
";
    let (_dir, path) = temp_nfr("multi", content);
    minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("3 constraints"));
}

// @minter:e2e parse-multiple-verification-lines
#[test]
fn parse_multiple_verification_lines() {
    let body = "\
  metric \"m\"
  threshold < 1s

  verification
    environment staging, production
    benchmark \"Load test at 100 RPS\"
    benchmark \"Spike test at 500 RPS\"
    dataset \"Production-representative volume\"
    pass \"p95 < threshold at 100 RPS\"
    pass \"No errors above 1% at 500 RPS\"";
    let content = valid_nfr_with_metric_constraint(body);
    let (_dir, path) = temp_nfr("multi-verif", &content);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e parse-environment-all
#[test]
fn parse_environment_all() {
    let body = "\
  metric \"m\"
  threshold < 1s

  verification
    environment all
    benchmark \"b\"
    pass \"p\"";
    let content = valid_nfr_with_metric_constraint(body);
    let (_dir, path) = temp_nfr("env-all", &content);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e ignore-nfr-comments
#[test]
fn ignore_nfr_comments() {
    let content = "\
# This is a comment
nfr performance v1.0.0
# Another comment
title \"Performance Requirements\"

description
  Defines performance constraints.

motivation
  Performance matters.

# Comment before constraint

constraint test-constraint [rule]
  \"A test constraint\"

  rule
    Some invariant.

  verification
    static \"Check it\"

  violation medium
  overridable no
";
    let (_dir, path) = temp_nfr("comments", content);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e ignore-nfr-blank-lines
#[test]
fn ignore_nfr_blank_lines() {
    let content = "\
nfr performance v1.0.0



title \"Performance Requirements\"



description
  Defines performance constraints.



motivation
  Performance matters.



constraint test-constraint [rule]
  \"A test constraint\"

  rule
    Some invariant.

  verification
    static \"Check it\"

  violation medium
  overridable no
";
    let (_dir, path) = temp_nfr("blanks", content);
    minter().arg("validate").arg(&path).assert().success();
}
