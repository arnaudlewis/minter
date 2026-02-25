mod common;

use common::minter;
use predicates::prelude::*;

// ═══════════════════════════════════════════════════════════════
// Core output (explain-command.spec)
// ═══════════════════════════════════════════════════════════════

/// explain-command.spec: explain-prints-methodology
#[test]
fn explain_prints_methodology() {
    minter()
        .arg("explain")
        .assert()
        .success()
        .stdout(predicate::str::contains("spec"))
        .stdout(predicate::str::contains("NFR"))
        .stdout(predicate::str::contains("behavior"))
        .stdout(predicate::str::contains("constraint"));
}

// ═══════════════════════════════════════════════════════════════
// Spec role (explain-command.spec)
// ═══════════════════════════════════════════════════════════════

/// explain-command.spec: explain-describes-spec-role
#[test]
fn explain_describes_spec_role() {
    minter()
        .arg("explain")
        .assert()
        .success()
        .stdout(predicate::str::contains("source of truth"))
        .stdout(predicate::str::contains("behavior"))
        .stdout(predicate::str::contains("1 behavior").or(predicate::str::contains("one behavior")))
        .stdout(predicate::str::contains("1 test").or(predicate::str::contains("one test")));
}

// ═══════════════════════════════════════════════════════════════
// NFR role (explain-command.spec)
// ═══════════════════════════════════════════════════════════════

/// explain-command.spec: explain-describes-nfr-constraints
#[test]
fn explain_describes_nfr_constraints() {
    minter()
        .arg("explain")
        .assert()
        .success()
        .stdout(predicate::str::contains("non-functional"))
        .stdout(predicate::str::contains("constraint"))
        .stdout(predicate::str::contains("metric"))
        .stdout(predicate::str::contains("rule"));
}

/// explain-command.spec: explain-lists-nfr-categories
#[test]
fn explain_lists_nfr_categories() {
    minter()
        .arg("explain")
        .assert()
        .success()
        .stdout(predicate::str::contains("performance"))
        .stdout(predicate::str::contains("reliability"))
        .stdout(predicate::str::contains("security"))
        .stdout(predicate::str::contains("observability"))
        .stdout(predicate::str::contains("scalability"))
        .stdout(predicate::str::contains("cost"))
        .stdout(predicate::str::contains("operability"));
}

// ═══════════════════════════════════════════════════════════════
// Cross-reference binding (explain-command.spec)
// ═══════════════════════════════════════════════════════════════

/// explain-command.spec: explain-describes-spec-level-binding
#[test]
fn explain_describes_spec_level_binding() {
    minter()
        .arg("explain")
        .assert()
        .success()
        .stdout(predicate::str::contains("spec-level"))
        .stdout(predicate::str::contains("all behaviors"));
}

/// explain-command.spec: explain-describes-behavior-level-binding
#[test]
fn explain_describes_behavior_level_binding() {
    minter()
        .arg("explain")
        .assert()
        .success()
        .stdout(predicate::str::contains("behavior-level"))
        .stdout(predicate::str::contains("anchor"));
}

/// explain-command.spec: explain-describes-whole-file-vs-anchor
#[test]
fn explain_describes_whole_file_vs_anchor() {
    minter()
        .arg("explain")
        .assert()
        .success()
        .stdout(predicate::str::contains("whole-file"))
        .stdout(predicate::str::contains("anchor"))
        .stdout(predicate::str::contains("#"));
}

// ═══════════════════════════════════════════════════════════════
// Validation rules (explain-command.spec)
// ═══════════════════════════════════════════════════════════════

/// explain-command.spec: explain-describes-containment-rule
#[test]
fn explain_describes_containment_rule() {
    minter()
        .arg("explain")
        .assert()
        .success()
        .stdout(predicate::str::contains("containment"))
        .stdout(predicate::str::contains("spec-level"));
}

/// explain-command.spec: explain-describes-override-rules
#[test]
fn explain_describes_override_rules() {
    minter()
        .arg("explain")
        .assert()
        .success()
        .stdout(predicate::str::contains("override"))
        .stdout(predicate::str::contains("stricter"))
        .stdout(predicate::str::contains("overridable"))
        .stdout(predicate::str::contains("metric"));
}

// ═══════════════════════════════════════════════════════════════
// Test generation (explain-command.spec)
// ═══════════════════════════════════════════════════════════════

/// explain-command.spec: explain-describes-test-emission
#[test]
fn explain_describes_test_emission() {
    minter()
        .arg("explain")
        .assert()
        .success()
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("emit").or(predicate::str::contains("generate")));
}

// ═══════════════════════════════════════════════════════════════
// Reference syntax (explain-command.spec)
// ═══════════════════════════════════════════════════════════════

/// explain-command.spec: explain-shows-reference-syntax
#[test]
fn explain_shows_reference_syntax() {
    minter()
        .arg("explain")
        .assert()
        .success()
        .stdout(predicate::str::contains("category"))
        .stdout(predicate::str::contains("category#constraint"))
        .stdout(predicate::str::contains("category#constraint operator value"));
}
