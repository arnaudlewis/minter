mod common;

use common::{minter, temp_spec};
use predicates::prelude::*;

// ═══════════════════════════════════════════════════════════════
// Happy paths (scaffold-command.spec)
// ═══════════════════════════════════════════════════════════════

/// scaffold-command.spec: scaffold-fr
#[test]
fn scaffold_fr() {
    minter()
        .arg("scaffold")
        .arg("fr")
        .assert()
        .success()
        .stdout(predicate::str::contains("spec"))
        .stdout(predicate::str::contains("title"))
        .stdout(predicate::str::contains("description"))
        .stdout(predicate::str::contains("motivation"))
        .stdout(predicate::str::contains("behavior"))
        .stdout(predicate::str::contains("given"))
        .stdout(predicate::str::contains("when"))
        .stdout(predicate::str::contains("then"));
}

/// scaffold-command.spec: scaffold-nfr-with-category
#[test]
fn scaffold_nfr_with_category() {
    minter()
        .arg("scaffold")
        .arg("nfr")
        .arg("performance")
        .assert()
        .success()
        .stdout(predicate::str::contains("performance"));
}

/// scaffold-command.spec: scaffold-nfr-all-categories
#[test]
fn scaffold_nfr_all_categories() {
    minter()
        .arg("scaffold")
        .arg("nfr")
        .arg("security")
        .assert()
        .success()
        .stdout(predicate::str::contains("security"));
}

// ═══════════════════════════════════════════════════════════════
// Error cases (scaffold-command.spec)
// ═══════════════════════════════════════════════════════════════

/// scaffold-command.spec: reject-unknown-nfr-category
#[test]
fn reject_unknown_nfr_category() {
    minter()
        .arg("scaffold")
        .arg("nfr")
        .arg("banana")
        .assert()
        .failure()
        .stderr(predicate::str::contains("banana"))
        .stderr(predicate::str::contains("performance"))
        .stderr(predicate::str::contains("security"))
        .stderr(predicate::str::contains("reliability"));
}

/// scaffold-command.spec: reject-nfr-missing-category
#[test]
fn reject_nfr_missing_category() {
    minter()
        .arg("scaffold")
        .arg("nfr")
        .assert()
        .failure()
        .stderr(predicate::str::contains("category"));
}

/// scaffold-command.spec: reject-unknown-scaffold-type
#[test]
fn reject_unknown_scaffold_type() {
    minter()
        .arg("scaffold")
        .arg("banana")
        .assert()
        .failure()
        .stderr(predicate::str::contains("banana"))
        .stderr(predicate::str::contains("fr"))
        .stderr(predicate::str::contains("nfr"));
}

/// scaffold-command.spec: scaffold-output-is-parseable
#[test]
fn scaffold_output_is_parseable() {
    // Run scaffold fr and capture the output
    let output = minter()
        .arg("scaffold")
        .arg("fr")
        .output()
        .expect("failed to run scaffold fr");
    assert!(output.status.success());

    let scaffold_content = String::from_utf8(output.stdout).expect("invalid utf8");

    // Write it to a temp file and validate it
    let (_dir, path) = temp_spec("scaffolded", &scaffold_content);

    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}
