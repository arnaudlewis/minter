mod common;

use common::minter;
use predicates::prelude::*;

// ═══════════════════════════════════════════════════════════════
// Happy paths (format-command.spec)
// ═══════════════════════════════════════════════════════════════

// @minter:e2e display-spec-grammar
#[test]
fn display_spec_grammar() {
    minter()
        .arg("format")
        .arg("spec")
        .assert()
        .success()
        .stdout(predicate::str::contains("spec"))
        .stdout(predicate::str::contains("title"))
        .stdout(predicate::str::contains("description"))
        .stdout(predicate::str::contains("motivation"))
        .stdout(predicate::str::contains("behavior"))
        .stdout(predicate::str::contains("given"))
        .stdout(predicate::str::contains("when"))
        .stdout(predicate::str::contains("then"))
        .stdout(predicate::str::contains("assert"))
        .stdout(predicate::str::contains("depends on"))
        .stdout(predicate::str::contains("whole-file reference"))
        .stdout(predicate::str::contains("anchor reference"))
        .stdout(predicate::str::contains("override"));
}

// @minter:e2e display-nfr-grammar
#[test]
fn display_nfr_grammar() {
    minter()
        .arg("format")
        .arg("nfr")
        .assert()
        .success()
        .stdout(predicate::str::contains("nfr"))
        .stdout(predicate::str::contains("constraint"))
        .stdout(predicate::str::contains("metric"))
        .stdout(predicate::str::contains("threshold"))
        .stdout(predicate::str::contains("rule"))
        .stdout(predicate::str::contains("verification"))
        .stdout(predicate::str::contains("violation"))
        .stdout(predicate::str::contains("overridable"))
        .stdout(predicate::str::contains("environment"))
        .stdout(predicate::str::contains("benchmark"))
        .stdout(predicate::str::contains("pass"))
        .stdout(predicate::str::contains("static"))
        .stdout(predicate::str::contains("runtime"));
}

// ═══════════════════════════════════════════════════════════════
// Error cases (format-command.spec)
// ═══════════════════════════════════════════════════════════════

// @minter:e2e reject-unknown-format-type
#[test]
fn reject_unknown_format_type() {
    minter()
        .arg("format")
        .arg("banana")
        .assert()
        .failure()
        .stderr(predicate::str::contains("banana"))
        .stderr(predicate::str::contains("spec"))
        .stderr(predicate::str::contains("nfr"));
}

// @minter:e2e reject-missing-format-type
#[test]
fn reject_missing_format_type() {
    minter()
        .arg("format")
        .assert()
        .failure()
        .stderr(predicate::str::contains("spec"))
        .stderr(predicate::str::contains("nfr"));
}
