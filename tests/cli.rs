mod common;

use common::{minter, temp_spec, VALID_SPEC};
use predicates::prelude::*;

// ═══════════════════════════════════════════════════════════════
// Happy paths (cli.spec)
// ═══════════════════════════════════════════════════════════════

/// cli.spec: show-help — --help flag prints usage
#[test]
fn show_help_flag() {
    minter()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("minter"))
        .stdout(predicate::str::contains("validate"));
}

/// cli.spec: show-help — no arguments prints usage
#[test]
fn show_help_no_args() {
    minter()
        .assert()
        .success()
        .stdout(predicate::str::contains("minter"))
        .stdout(predicate::str::contains("validate"));
}

/// cli.spec: show-version
#[test]
fn show_version() {
    minter()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

/// cli.spec: validate-command — routes to validate with file args
#[test]
fn validate_command_routing() {
    let (_dir, path) = temp_spec("valid", VALID_SPEC);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// cli.spec: validate-deps-flag — routes to validate with --deps
#[test]
fn validate_deps_flag_routing() {
    let (_dir, path) = temp_spec("valid", VALID_SPEC);
    minter()
        .arg("validate")
        .arg("--deps")
        .arg(&path)
        .assert()
        .success();
}

// ═══════════════════════════════════════════════════════════════
// Error cases (cli.spec)
// ═══════════════════════════════════════════════════════════════

/// cli.spec: reject-unknown-command
#[test]
fn reject_unknown_command() {
    minter()
        .arg("frobnicate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("frobnicate"));
}

/// cli.spec: reject-no-files
#[test]
fn reject_no_files() {
    minter()
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

/// cli.spec: reject-non-spec-extension
#[test]
fn reject_non_spec_extension() {
    let (_dir, path) = common::temp_file("readme.md", "not a spec");
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(".spec"));
}

/// cli.spec: reject-unknown-flag
#[test]
fn reject_unknown_flag() {
    minter()
        .arg("validate")
        .arg("--frobnicate")
        .arg("file.spec")
        .assert()
        .failure()
        .stderr(predicate::str::contains("frobnicate"));
}

// ═══════════════════════════════════════════════════════════════
// Edge cases (cli.spec)
// ═══════════════════════════════════════════════════════════════

/// cli.spec: handle-mixed-valid-invalid-files
#[test]
fn handle_mixed_valid_invalid_files() {
    let (_dir, path) = temp_spec("valid", VALID_SPEC);
    minter()
        .arg("validate")
        .arg(&path)
        .arg("nonexistent.spec")
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent.spec"));
}
