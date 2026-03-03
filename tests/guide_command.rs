mod common;

use common::minter;
use predicates::prelude::*;

// ═══════════════════════════════════════════════════════════════
// Methodology topic (guide-command.spec)
// ═══════════════════════════════════════════════════════════════

/// guide-command.spec: guide-methodology-prints-methodology
#[test]
fn guide_methodology_prints_methodology() {
    minter()
        .args(&["guide", "methodology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("spec"))
        .stdout(predicate::str::contains("NFR"))
        .stdout(predicate::str::contains("behavior"))
        .stdout(predicate::str::contains("constraint"));
}

/// guide-command.spec: guide-methodology-describes-spec-role
#[test]
fn guide_methodology_describes_spec_role() {
    minter()
        .args(&["guide", "methodology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("source of truth"))
        .stdout(predicate::str::contains("behavior"))
        .stdout(predicate::str::contains("1 behavior").or(predicate::str::contains("one behavior")))
        .stdout(predicate::str::contains("1 test").or(predicate::str::contains("one test")));
}

/// guide-command.spec: guide-methodology-describes-nfr-constraints
#[test]
fn guide_methodology_describes_nfr_constraints() {
    minter()
        .args(&["guide", "methodology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("non-functional"))
        .stdout(predicate::str::contains("constraint"))
        .stdout(predicate::str::contains("metric"))
        .stdout(predicate::str::contains("rule"));
}

/// guide-command.spec: guide-methodology-lists-nfr-categories
#[test]
fn guide_methodology_lists_nfr_categories() {
    minter()
        .args(&["guide", "methodology"])
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
// Cross-reference binding (guide-command.spec)
// ═══════════════════════════════════════════════════════════════

/// guide-command.spec: guide-methodology-describes-spec-level-binding
#[test]
fn guide_methodology_describes_spec_level_binding() {
    minter()
        .args(&["guide", "methodology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("spec-level"))
        .stdout(predicate::str::contains("all behaviors"));
}

/// guide-command.spec: guide-methodology-describes-behavior-level-binding
#[test]
fn guide_methodology_describes_behavior_level_binding() {
    minter()
        .args(&["guide", "methodology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("behavior-level"))
        .stdout(predicate::str::contains("anchor"));
}

/// guide-command.spec: guide-methodology-describes-whole-file-vs-anchor
#[test]
fn guide_methodology_describes_whole_file_vs_anchor() {
    minter()
        .args(&["guide", "methodology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("whole-file"))
        .stdout(predicate::str::contains("anchor"))
        .stdout(predicate::str::contains("#"));
}

// ═══════════════════════════════════════════════════════════════
// Validation rules (guide-command.spec)
// ═══════════════════════════════════════════════════════════════

/// guide-command.spec: guide-methodology-describes-containment-rule
#[test]
fn guide_methodology_describes_containment_rule() {
    minter()
        .args(&["guide", "methodology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("containment"))
        .stdout(predicate::str::contains("spec-level"));
}

/// guide-command.spec: guide-methodology-describes-override-rules
#[test]
fn guide_methodology_describes_override_rules() {
    minter()
        .args(&["guide", "methodology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("override"))
        .stdout(predicate::str::contains("stricter"))
        .stdout(predicate::str::contains("overridable"))
        .stdout(predicate::str::contains("metric"));
}

// ═══════════════════════════════════════════════════════════════
// Test generation (guide-command.spec)
// ═══════════════════════════════════════════════════════════════

/// guide-command.spec: guide-methodology-describes-test-emission
#[test]
fn guide_methodology_describes_test_emission() {
    minter()
        .args(&["guide", "methodology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("emit").or(predicate::str::contains("generate")));
}

// ═══════════════════════════════════════════════════════════════
// Reference syntax (guide-command.spec)
// ═══════════════════════════════════════════════════════════════

/// guide-command.spec: guide-methodology-shows-reference-syntax
#[test]
fn guide_methodology_shows_reference_syntax() {
    minter()
        .args(&["guide", "methodology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("category"))
        .stdout(predicate::str::contains("category#constraint"))
        .stdout(predicate::str::contains(
            "category#constraint operator value",
        ));
}

// ═══════════════════════════════════════════════════════════════
// Workflow (guide-command.spec)
// ═══════════════════════════════════════════════════════════════

/// guide-command.spec: guide-methodology-describes-workflow-phases
#[test]
fn guide_methodology_describes_workflow_phases() {
    minter()
        .args(&["guide", "methodology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Phase 1"))
        .stdout(predicate::str::contains("Phase 2"))
        .stdout(predicate::str::contains("Phase 3"))
        .stdout(predicate::str::contains("Phase 4"))
        .stdout(predicate::str::contains("Phase 5"));
}

/// guide-command.spec: guide-methodology-specs-before-code
#[test]
fn guide_methodology_specs_before_code() {
    minter()
        .args(&["guide", "methodology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("before"))
        .stdout(predicate::str::contains("implementation").or(predicate::str::contains("code")));
}

/// guide-command.spec: guide-methodology-red-tests
#[test]
fn guide_methodology_red_tests() {
    minter()
        .args(&["guide", "methodology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("fail"))
        .stdout(
            predicate::str::contains("1 behavior = 1 test")
                .or(predicate::str::contains("one behavior = one test")),
        );
}

// ═══════════════════════════════════════════════════════════════
// Topic-specific guides (guide-command.spec)
// ═══════════════════════════════════════════════════════════════

/// guide-command.spec: guide-workflow-topic
#[test]
fn guide_workflow_topic() {
    minter()
        .args(&["guide", "workflow"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Phase 1"))
        .stdout(predicate::str::contains("Phase 5"));
}

/// guide-command.spec: guide-authoring-topic
#[test]
fn guide_authoring_topic() {
    minter()
        .args(&["guide", "authoring"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Right Granularity"));
}

/// guide-command.spec: guide-smells-topic
#[test]
fn guide_smells_topic() {
    minter()
        .args(&["guide", "smells"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Ambiguity"));
}

/// guide-command.spec: guide-nfr-topic
#[test]
fn guide_nfr_topic() {
    minter()
        .args(&["guide", "nfr"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Seven Fixed Categories"));
}

/// guide-command.spec: guide-context-topic
#[test]
fn guide_context_topic() {
    minter()
        .args(&["guide", "context"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Lazy Loading Sequence"));
}

/// guide-command.spec: guide-coverage-topic
#[test]
fn guide_coverage_topic() {
    minter()
        .args(&["guide", "coverage"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Coverage Tagging"))
        .stdout(predicate::str::contains("@minter"))
        .stdout(predicate::str::contains("unit"))
        .stdout(predicate::str::contains("e2e"))
        .stdout(predicate::str::contains("benchmark"))
        .stdout(predicate::str::contains("Qualified Names"))
        .stdout(predicate::str::contains("Common Mistakes"));
}

// ═══════════════════════════════════════════════════════════════
// Error cases (guide-command.spec)
// ═══════════════════════════════════════════════════════════════

/// guide-command.spec: guide-unknown-topic
#[test]
fn guide_unknown_topic() {
    minter()
        .args(&["guide", "banana"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("banana"));
}

/// guide-command.spec: guide-missing-topic
#[test]
fn guide_missing_topic() {
    minter()
        .arg("guide")
        .assert()
        .failure()
        .stderr(predicate::str::contains("guide"));
}
