mod common;

use common::{minter, temp_nfr, temp_spec};
use predicates::prelude::*;

// ═══════════════════════════════════════════════════════════════
// Happy paths (scaffold-command.spec)
// ═══════════════════════════════════════════════════════════════

// @minter:e2e scaffold-spec
#[test]
fn scaffold_spec() {
    minter()
        .arg("scaffold")
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
        .stdout(predicate::str::contains("nfr"));
}

// @minter:e2e scaffold-nfr-with-category
#[test]
fn scaffold_nfr_with_category() {
    minter()
        .arg("scaffold")
        .arg("nfr")
        .arg("performance")
        .assert()
        .success()
        .stdout(predicate::str::contains("nfr"))
        .stdout(predicate::str::contains("performance"))
        .stdout(predicate::str::contains("constraint"))
        .stdout(predicate::str::contains("verification"))
        .stdout(predicate::str::contains("violation"))
        .stdout(predicate::str::contains("overridable"));
}

// @minter:e2e scaffold-nfr-all-categories
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

// @minter:e2e reject-unknown-nfr-category
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
        .stderr(predicate::str::contains("reliability"))
        .stderr(predicate::str::contains("cost"))
        .stderr(predicate::str::contains("operability"));
}

// @minter:e2e reject-nfr-missing-category
#[test]
fn reject_nfr_missing_category() {
    minter()
        .arg("scaffold")
        .arg("nfr")
        .assert()
        .failure()
        .stderr(predicate::str::contains("category"));
}

// @minter:e2e reject-unknown-scaffold-type
#[test]
fn reject_unknown_scaffold_type() {
    minter()
        .arg("scaffold")
        .arg("banana")
        .assert()
        .failure()
        .stderr(predicate::str::contains("banana"))
        .stderr(predicate::str::contains("spec"))
        .stderr(predicate::str::contains("nfr"));
}

// @minter:e2e scaffold-nfr-output-is-parseable
#[test]
fn scaffold_nfr_output_is_parseable() {
    let output = minter()
        .arg("scaffold")
        .arg("nfr")
        .arg("performance")
        .output()
        .expect("failed to run scaffold nfr");
    assert!(output.status.success());

    let scaffold_content = String::from_utf8(output.stdout).expect("invalid utf8");

    let (_dir, path) = temp_nfr("scaffolded", &scaffold_content);

    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e scaffold-output-is-parseable
#[test]
fn scaffold_output_is_parseable() {
    // Run scaffold spec and capture the output
    let output = minter()
        .arg("scaffold")
        .arg("spec")
        .output()
        .expect("failed to run scaffold spec");
    assert!(output.status.success());

    let scaffold_content = String::from_utf8(output.stdout).expect("invalid utf8");

    // Write it to a temp file and validate it
    let (_dir, path) = temp_spec("scaffolded", &scaffold_content);

    minter().arg("validate").arg(&path).assert().success();
}
