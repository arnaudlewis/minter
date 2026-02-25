mod common;

use common::{minter, temp_dir_with_spec_and_nfrs, temp_spec};
use predicates::prelude::PredicateBooleanExt;

// ── Spec-level NFR content helpers ──────────────────────

/// A .spec with a spec-level nfr section referencing a whole-file category.
fn spec_with_nfr_whole_file(category: &str) -> String {
    format!(
        "\
spec test-spec v1.0.0
title \"Test Spec\"

description
  A test spec.

motivation
  Testing.

nfr
  {category}

behavior do-thing [happy_path]
  \"Do the thing\"

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
"
    )
}

/// A .spec with a spec-level nfr section referencing an anchor.
fn spec_with_nfr_anchor(category: &str, anchor: &str) -> String {
    format!(
        "\
spec test-spec v1.0.0
title \"Test Spec\"

description
  A test spec.

motivation
  Testing.

nfr
  {category}#{anchor}

behavior do-thing [happy_path]
  \"Do the thing\"

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
"
    )
}

/// A .spec with spec-level nfr section with 3 mixed refs.
fn spec_with_nfr_mixed() -> String {
    "\
spec test-spec v1.0.0
title \"Test Spec\"

description
  A test spec.

motivation
  Testing.

nfr
  security
  performance
  reliability#completeness

behavior do-thing [happy_path]
  \"Do the thing\"

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
"
    .to_string()
}

/// A .spec with a spec-level override (should be rejected).
fn spec_with_nfr_spec_level_override() -> String {
    "\
spec test-spec v1.0.0
title \"Test Spec\"

description
  A test spec.

motivation
  Testing.

nfr
  performance#api-response-time < 500ms

behavior do-thing [happy_path]
  \"Do the thing\"

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
"
    .to_string()
}

/// A .spec with a behavior-level nfr anchor reference.
fn spec_with_behavior_nfr_anchor(category: &str, anchor: &str) -> String {
    format!(
        "\
spec test-spec v1.0.0
title \"Test Spec\"

description
  A test spec.

motivation
  Testing.

nfr
  {category}

behavior do-thing [happy_path]
  \"Do the thing\"

  nfr
    {category}#{anchor}

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
"
    )
}

/// A .spec with a behavior-level nfr override.
fn spec_with_behavior_nfr_override(
    category: &str,
    anchor: &str,
    operator: &str,
    value: &str,
) -> String {
    format!(
        "\
spec test-spec v1.0.0
title \"Test Spec\"

description
  A test spec.

motivation
  Testing.

nfr
  {category}

behavior do-thing [happy_path]
  \"Do the thing\"

  nfr
    {category}#{anchor} {operator} {value}

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
"
    )
}

/// A .spec with behavior-level nfr with multiple refs.
fn spec_with_behavior_nfr_multiple() -> String {
    "\
spec test-spec v1.0.0
title \"Test Spec\"

description
  A test spec.

motivation
  Testing.

nfr
  performance
  reliability

behavior do-thing [happy_path]
  \"Do the thing\"

  nfr
    performance#api-response-time < 500ms
    reliability#completeness >= 100%

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
"
    .to_string()
}

/// A .spec with behavior-level whole-file ref (should be rejected).
fn spec_with_behavior_nfr_whole_file() -> String {
    "\
spec test-spec v1.0.0
title \"Test Spec\"

description
  A test spec.

motivation
  Testing.

nfr
  performance

behavior do-thing [happy_path]
  \"Do the thing\"

  nfr
    performance

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
"
    .to_string()
}

/// A .spec with behavior-level nfr referencing a category NOT in spec-level nfr.
fn spec_with_containment_violation() -> String {
    "\
spec test-spec v1.0.0
title \"Test Spec\"

description
  A test spec.

motivation
  Testing.

nfr
  performance

behavior do-thing [happy_path]
  \"Do the thing\"

  nfr
    reliability#completeness

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
"
    .to_string()
}

// ── NFR file helpers ────────────────────────────────────

fn nfr_with_metric_constraint(
    category: &str,
    constraint_name: &str,
    operator: &str,
    threshold: &str,
    overridable: &str,
) -> String {
    format!(
        "\
nfr {category} v1.0.0
title \"{category} Requirements\"

description
  Defines {category} constraints.

motivation
  {category} matters.


constraint {constraint_name} [metric]
  \"Constraint description\"

  metric \"Measurement metric\"
  threshold {operator} {threshold}

  verification
    environment staging
    benchmark \"standard benchmark\"
    pass \"p95 {operator} threshold\"

  violation high
  overridable {overridable}
"
    )
}

fn nfr_with_rule_constraint(
    category: &str,
    constraint_name: &str,
    overridable: &str,
) -> String {
    format!(
        "\
nfr {category} v1.0.0
title \"{category} Requirements\"

description
  Defines {category} constraints.

motivation
  {category} matters.


constraint {constraint_name} [rule]
  \"Rule description\"

  rule
    No violations allowed

  verification
    static \"lint check\"

  violation high
  overridable {overridable}
"
    )
}

// ═══════════════════════════════════════════════════════════════
// Spec-level nfr section parsing (behaviors 1-6)
// ═══════════════════════════════════════════════════════════════

/// nfr-cross-reference: parse-spec-level-whole-file-ref
#[test]
fn parse_spec_level_whole_file_ref() {
    let content = spec_with_nfr_whole_file("security");
    let (_dir, path) = temp_spec("test-spec", &content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicates::str::contains("test-spec"));
}

/// nfr-cross-reference: parse-spec-level-anchor-ref
#[test]
fn parse_spec_level_anchor_ref() {
    let content = spec_with_nfr_anchor("reliability", "completeness");
    let (_dir, path) = temp_spec("test-spec", &content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicates::str::contains("test-spec"));
}

/// nfr-cross-reference: parse-spec-level-mixed-refs
#[test]
fn parse_spec_level_mixed_refs() {
    let content = spec_with_nfr_mixed();
    let (_dir, path) = temp_spec("test-spec", &content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicates::str::contains("test-spec"));
}

/// nfr-cross-reference: parse-spec-level-nfr-optional
#[test]
fn parse_spec_level_nfr_optional() {
    let (_dir, path) = temp_spec("test-spec", common::VALID_SPEC);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// nfr-cross-reference: parse-spec-level-nfr-position
#[test]
fn parse_spec_level_nfr_position() {
    // The nfr section appears between motivation and the first behavior
    let content = spec_with_nfr_whole_file("performance");
    let (_dir, path) = temp_spec("test-spec", &content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicates::str::contains("test-spec"));
}

/// nfr-cross-reference: reject-spec-level-override
#[test]
fn reject_spec_level_override() {
    let content = spec_with_nfr_spec_level_override();
    let (_dir, path) = temp_spec("test-spec", &content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicates::str::contains("override").and(predicates::str::contains("spec-level")))
        .stderr(predicates::str::contains(path.to_str().unwrap()));
}

// ═══════════════════════════════════════════════════════════════
// Behavior-level nfr section parsing (behaviors 7-11, 26-27)
// ═══════════════════════════════════════════════════════════════

/// nfr-cross-reference: parse-behavior-level-anchor-ref
#[test]
fn parse_behavior_level_anchor_ref() {
    let content = spec_with_behavior_nfr_anchor("performance", "data-freshness");
    let (_dir, path) = temp_spec("test-spec", &content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicates::str::contains("test-spec"));
}

/// nfr-cross-reference: parse-behavior-level-override
#[test]
fn parse_behavior_level_override() {
    let content = spec_with_behavior_nfr_override("performance", "api-response-time", "<", "500ms");
    let (_dir, path) = temp_spec("test-spec", &content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicates::str::contains("test-spec"));
}

/// nfr-cross-reference: parse-behavior-level-multiple-refs
#[test]
fn parse_behavior_level_multiple_refs() {
    let content = spec_with_behavior_nfr_multiple();
    let (_dir, path) = temp_spec("test-spec", &content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicates::str::contains("test-spec"));
}

/// nfr-cross-reference: parse-behavior-level-nfr-position
#[test]
fn parse_behavior_level_nfr_position() {
    // nfr section appears between description and given
    let content = spec_with_behavior_nfr_anchor("performance", "api-response-time");
    let (_dir, path) = temp_spec("test-spec", &content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicates::str::contains("test-spec"));
}

/// nfr-cross-reference: parse-behavior-level-nfr-optional
#[test]
fn parse_behavior_level_nfr_optional() {
    let (_dir, path) = temp_spec("test-spec", common::VALID_SPEC);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// nfr-cross-reference: reject-behavior-level-whole-file-ref
#[test]
fn reject_behavior_level_whole_file_ref() {
    let content = spec_with_behavior_nfr_whole_file();
    let (_dir, path) = temp_spec("test-spec", &content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(
            predicates::str::contains("whole-file")
                .and(predicates::str::contains("behavior")),
        )
        .stderr(predicates::str::contains(path.to_str().unwrap()));
}

/// nfr-cross-reference: accept-behavior-level-anchor-only
#[test]
fn accept_behavior_level_anchor_only() {
    let content = spec_with_behavior_nfr_anchor("performance", "api-response-time");
    let (_dir, path) = temp_spec("test-spec", &content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

// ═══════════════════════════════════════════════════════════════
// Cross-validation rule 1: Category exists (behaviors 12-13)
// ═══════════════════════════════════════════════════════════════

/// nfr-cross-reference: resolve-category-to-nfr-file
#[test]
fn resolve_category_to_nfr_file() {
    let spec_content = spec_with_nfr_whole_file("performance");
    let nfr_content = nfr_with_metric_constraint("performance", "api-response-time", "<", "1s", "yes");
    let (_dir, dir_path) = temp_dir_with_spec_and_nfrs(
        "test-spec",
        &spec_content,
        &[("performance", &nfr_content)],
    );
    minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .success();
}

/// nfr-cross-reference: reject-missing-nfr-category-file
#[test]
fn reject_missing_nfr_category_file() {
    let spec_content = spec_with_nfr_whole_file("performance");
    // No .nfr file for performance
    let (_dir, dir_path) = temp_dir_with_spec_and_nfrs(
        "test-spec",
        &spec_content,
        &[],
    );
    minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .failure()
        .stderr(predicates::str::contains("performance"))
        .stderr(predicates::str::contains("test-spec"));
}

// ═══════════════════════════════════════════════════════════════
// Cross-validation rule 2: Anchor exists (behaviors 14-15)
// ═══════════════════════════════════════════════════════════════

/// nfr-cross-reference: resolve-anchor-to-constraint
#[test]
fn resolve_anchor_to_constraint() {
    let spec_content = spec_with_behavior_nfr_anchor("performance", "api-response-time");
    let nfr_content = nfr_with_metric_constraint("performance", "api-response-time", "<", "1s", "yes");
    let (_dir, dir_path) = temp_dir_with_spec_and_nfrs(
        "test-spec",
        &spec_content,
        &[("performance", &nfr_content)],
    );
    minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .success();
}

/// nfr-cross-reference: reject-missing-anchor
#[test]
fn reject_missing_anchor() {
    let spec_content = spec_with_behavior_nfr_anchor("performance", "nonexistent");
    let nfr_content = nfr_with_metric_constraint("performance", "api-response-time", "<", "1s", "yes");
    let (_dir, dir_path) = temp_dir_with_spec_and_nfrs(
        "test-spec",
        &spec_content,
        &[("performance", &nfr_content)],
    );
    minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .failure()
        .stderr(
            predicates::str::contains("nonexistent")
                .and(predicates::str::contains("performance")),
        )
        .stderr(predicates::str::contains("test-spec"));
}

// ═══════════════════════════════════════════════════════════════
// Cross-validation rule 3: Containment (behaviors 16-17)
// ═══════════════════════════════════════════════════════════════

/// nfr-cross-reference: containment-satisfied
#[test]
fn containment_satisfied() {
    let spec_content = spec_with_behavior_nfr_anchor("performance", "api-response-time");
    let nfr_content = nfr_with_metric_constraint("performance", "api-response-time", "<", "1s", "yes");
    let (_dir, dir_path) = temp_dir_with_spec_and_nfrs(
        "test-spec",
        &spec_content,
        &[("performance", &nfr_content)],
    );
    minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .success();
}

/// nfr-cross-reference: reject-containment-violation
#[test]
fn reject_containment_violation() {
    let spec_content = spec_with_containment_violation();
    let perf_nfr = nfr_with_metric_constraint("performance", "api-response-time", "<", "1s", "yes");
    let rel_nfr = nfr_with_metric_constraint("reliability", "completeness", ">=", "99.9%", "yes");
    let (_dir, dir_path) = temp_dir_with_spec_and_nfrs(
        "test-spec",
        &spec_content,
        &[("performance", &perf_nfr), ("reliability", &rel_nfr)],
    );
    minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .failure()
        .stderr(predicates::str::contains("reliability"))
        .stderr(predicates::str::contains("test-spec"));
}

// ═══════════════════════════════════════════════════════════════
// Cross-validation rule 4: Overridable check (behaviors 18-19)
// ═══════════════════════════════════════════════════════════════

/// nfr-cross-reference: override-allowed-on-overridable-yes
#[test]
fn override_allowed_on_overridable_yes() {
    let spec_content =
        spec_with_behavior_nfr_override("performance", "api-response-time", "<", "500ms");
    let nfr_content = nfr_with_metric_constraint("performance", "api-response-time", "<", "1s", "yes");
    let (_dir, dir_path) = temp_dir_with_spec_and_nfrs(
        "test-spec",
        &spec_content,
        &[("performance", &nfr_content)],
    );
    minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .success();
}

/// nfr-cross-reference: reject-override-on-overridable-no
#[test]
fn reject_override_on_overridable_no() {
    let spec_content =
        spec_with_behavior_nfr_override("security", "tenant-isolation", "<", "500ms");
    let nfr_content = nfr_with_metric_constraint("security", "tenant-isolation", "<", "1s", "no");
    let (_dir, dir_path) = temp_dir_with_spec_and_nfrs(
        "test-spec",
        &spec_content,
        &[("security", &nfr_content)],
    );
    minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .failure()
        .stderr(predicates::str::contains("tenant-isolation"))
        .stderr(predicates::str::contains("test-spec"));
}

// ═══════════════════════════════════════════════════════════════
// Cross-validation rule 5: Metric only (behaviors 20-21)
// ═══════════════════════════════════════════════════════════════

/// nfr-cross-reference: override-allowed-on-metric
#[test]
fn override_allowed_on_metric() {
    let spec_content =
        spec_with_behavior_nfr_override("performance", "api-response-time", "<", "500ms");
    let nfr_content = nfr_with_metric_constraint("performance", "api-response-time", "<", "1s", "yes");
    let (_dir, dir_path) = temp_dir_with_spec_and_nfrs(
        "test-spec",
        &spec_content,
        &[("performance", &nfr_content)],
    );
    minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .success();
}

/// nfr-cross-reference: reject-override-on-rule
#[test]
fn reject_override_on_rule() {
    let spec_content =
        spec_with_behavior_nfr_override("performance", "no-n-plus-one", "<", "5");
    let nfr_content = nfr_with_rule_constraint("performance", "no-n-plus-one", "yes");
    let (_dir, dir_path) = temp_dir_with_spec_and_nfrs(
        "test-spec",
        &spec_content,
        &[("performance", &nfr_content)],
    );
    minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .failure()
        .stderr(predicates::str::contains("no-n-plus-one"))
        .stderr(predicates::str::contains("test-spec"));
}

// ═══════════════════════════════════════════════════════════════
// Cross-validation rule 6: Same operator (behaviors 22-23)
// ═══════════════════════════════════════════════════════════════

/// nfr-cross-reference: override-same-operator
#[test]
fn override_same_operator() {
    let spec_content =
        spec_with_behavior_nfr_override("performance", "api-response-time", "<", "500ms");
    let nfr_content = nfr_with_metric_constraint("performance", "api-response-time", "<", "1s", "yes");
    let (_dir, dir_path) = temp_dir_with_spec_and_nfrs(
        "test-spec",
        &spec_content,
        &[("performance", &nfr_content)],
    );
    minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .success();
}

/// nfr-cross-reference: reject-override-mismatched-operator
#[test]
fn reject_override_mismatched_operator() {
    let spec_content =
        spec_with_behavior_nfr_override("performance", "api-response-time", ">", "500ms");
    let nfr_content = nfr_with_metric_constraint("performance", "api-response-time", "<", "1s", "yes");
    let (_dir, dir_path) = temp_dir_with_spec_and_nfrs(
        "test-spec",
        &spec_content,
        &[("performance", &nfr_content)],
    );
    minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .failure()
        .stderr(
            predicates::str::contains("operator")
                .and(predicates::str::contains("api-response-time")),
        )
        .stderr(predicates::str::contains("test-spec"));
}

// ═══════════════════════════════════════════════════════════════
// Cross-validation rule 7: Stricter value (behaviors 24-25)
// ═══════════════════════════════════════════════════════════════

/// nfr-cross-reference: override-stricter-value
#[test]
fn override_stricter_value() {
    let spec_content =
        spec_with_behavior_nfr_override("performance", "api-response-time", "<", "500ms");
    let nfr_content = nfr_with_metric_constraint("performance", "api-response-time", "<", "1s", "yes");
    let (_dir, dir_path) = temp_dir_with_spec_and_nfrs(
        "test-spec",
        &spec_content,
        &[("performance", &nfr_content)],
    );
    minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .success();
}

/// nfr-cross-reference: reject-override-relaxed-value
#[test]
fn reject_override_relaxed_value() {
    let spec_content =
        spec_with_behavior_nfr_override("performance", "api-response-time", "<", "2s");
    let nfr_content = nfr_with_metric_constraint("performance", "api-response-time", "<", "1s", "yes");
    let (_dir, dir_path) = temp_dir_with_spec_and_nfrs(
        "test-spec",
        &spec_content,
        &[("performance", &nfr_content)],
    );
    minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .failure()
        .stderr(predicates::str::contains("api-response-time"))
        .stderr(predicates::str::contains("test-spec"));
}
