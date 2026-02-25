mod common;

use common::{minter, temp_dir_with_nested_specs, temp_dir_with_spec_and_nfrs, temp_dir_with_specs, temp_nfr, temp_spec, temp_specs, VALID_NFR, VALID_SPEC};
use predicates::prelude::*;

// ═══════════════════════════════════════════════════════════════
// Happy paths (validate-command.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-command.spec: validate-valid-spec
#[test]
fn validate_valid_spec() {
    let (_dir, path) = temp_spec("test-spec", VALID_SPEC);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("test-spec"));
}

/// validate-command.spec: validate-multiple-files-all-valid
#[test]
fn validate_multiple_files_all_valid() {
    let second_spec = "\
spec other-spec v2.0.0
title \"Other Spec\"

description
  Another valid spec.

motivation
  Testing multi-file validation.

behavior other-thing [happy_path]
  \"Do another thing\"

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, paths) = temp_specs(&[("test-spec", VALID_SPEC), ("other-spec", second_spec)]);
    minter()
        .arg("validate")
        .arg(&paths[0])
        .arg(&paths[1])
        .assert()
        .success();
}

/// validate-command.spec: validate-single-file-is-isolated
#[test]
fn validate_single_file_is_isolated() {
    let spec_a = "\
spec a v1.0.0
title \"A\"

description
  Spec A depends on b but b does not exist.

motivation
  Test isolated single-file validation.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

depends on b >= 1.0.0
";
    let (_dir, path) = temp_spec("a", spec_a);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("a"));
}

/// validate-command.spec: validate-deep-single-file
#[test]
fn validate_deep_single_file() {
    let spec_a = "\
spec a v1.0.0
title \"A\"

description
  Spec A depends on b.

motivation
  Test deep single-file validation.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

depends on b >= 1.0.0
";
    let spec_b = "\
spec b v1.0.0
title \"B\"

description
  Spec B.

motivation
  Test.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, paths) = temp_specs(&[("a", spec_a), ("b", spec_b)]);
    let output = minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("a") && stdout.contains("b"),
        "Expected both a and b in output, got: {stdout}"
    );
}

/// validate-command.spec: validate-directory-is-always-deep
#[test]
fn validate_directory_is_always_deep() {
    let spec_a = "\
spec a v1.0.0
title \"A\"

description
  Spec A depends on b.

motivation
  Test directory-always-deep.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

depends on b >= 1.0.0
";
    let spec_b = "\
spec b v1.0.0
title \"B\"

description
  Spec B.

motivation
  Test.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, dir_path) = temp_dir_with_specs(&[("a", spec_a), ("b", spec_b)]);
    let output = minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("a") && stdout.contains("b"),
        "Directory validation should always be deep; expected both a and b, got: {stdout}"
    );
}

/// validate-command.spec: discover-specs-in-directory
#[test]
fn discover_specs_in_directory() {
    let spec_a = "\
spec a v1.0.0
title \"A\"

description
  Spec A.

motivation
  Test.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let spec_b = "\
spec b v1.0.0
title \"B\"

description
  Spec B.

motivation
  Test.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, dir_path) = temp_dir_with_nested_specs(&[
        ("sub/a", spec_a),
        ("sub/deep/b", spec_b),
    ]);
    let output = minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("a") && stdout.contains("b"),
        "Expected both nested specs discovered, got: {stdout}"
    );
}

// ═══════════════════════════════════════════════════════════════
// Error cases — semantic validation (validate-command.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-command.spec: reject-duplicate-behavior-names
#[test]
fn reject_duplicate_behavior_names() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"First\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

behavior do-thing [happy_path]
  \"Second with same name\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("dup-names", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("do-thing"))
        .stderr(predicate::str::contains("Duplicate"))
        .stderr(predicate::str::contains(path.to_str().unwrap()));
}

/// validate-command.spec: reject-unresolved-alias
#[test]
fn reject_unresolved_alias() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"Uses unresolved alias\"

  given
    The system is ready

  when act
    user_id = @nonexistent.id

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("unresolved-alias", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"))
        .stderr(predicate::str::contains(path.to_str().unwrap()));
}

/// validate-command.spec: reject-duplicate-aliases
#[test]
fn reject_duplicate_aliases() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"Has duplicate aliases\"

  given
    @user = User { id: \"1\" }
    @user = User { id: \"2\" }

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("dup-aliases", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("user"))
        .stderr(predicate::str::contains("Duplicate"))
        .stderr(predicate::str::contains(path.to_str().unwrap()));
}

/// validate-command.spec: reject-invalid-identity-name
#[test]
fn reject_invalid_identity_name() {
    let spec = "\
spec InvalidName v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("bad-name", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("InvalidName").or(predicate::str::contains("kebab-case")))
        .stderr(predicate::str::contains(path.to_str().unwrap()));
}

/// validate-command.spec: reject-invalid-semver
#[test]
fn reject_invalid_semver() {
    let spec = "\
spec test-spec vNOPE
title \"Test\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("bad-version", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("NOPE").or(predicate::str::contains("semver")))
        .stderr(predicate::str::contains(path.to_str().unwrap()));
}

/// validate-command.spec: reject-no-happy-path
#[test]
fn reject_no_happy_path() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior fail-thing [error_case]
  \"Only error cases\"

  given
    Ready

  when act

  then emits stderr
    assert output contains \"error\"
";
    let (_dir, path) = temp_spec("no-happy", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("happy_path"))
        .stderr(predicate::str::contains(path.to_str().unwrap()));
}

// ═══════════════════════════════════════════════════════════════
// Edge cases (validate-command.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-command.spec: reject-nonexistent-file
#[test]
fn handle_nonexistent_file() {
    minter()
        .arg("validate")
        .arg("/tmp/minter_nonexistent_file.spec")
        .assert()
        .failure()
        .stderr(predicate::str::contains("minter_nonexistent_file.spec"));
}

/// validate-command.spec: reject-unreadable-file
#[test]
fn handle_unreadable_file() {
    let (_dir, path) = temp_spec("unreadable", VALID_SPEC);

    // Remove read permission
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o000);
        std::fs::set_permissions(&path, perms).expect("set permissions");
    }

    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("permission").or(predicate::str::contains("Permission")));
}

/// validate-command.spec: reject-empty-file
#[test]
fn handle_empty_file() {
    let (_dir, path) = temp_spec("empty", "");
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

/// validate-command.spec: report-all-errors
#[test]
fn report_all_errors() {
    // Three independent semantic errors: bad name, bad version, no happy_path
    let spec = "\
spec InvalidName vNOPE
title \"Test\"

description
  Test.

motivation
  Test.

behavior fail-thing [error_case]
  \"Only error case\"

  given
    Ready

  when act

  then emits stderr
    assert output contains \"error\"
";
    let (_dir, path) = temp_spec("multi-error", spec);
    let output = minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure();

    // Should report all three errors, not just the first
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    assert!(
        stderr.contains("kebab-case"),
        "Expected kebab-case-name error, got: {stderr}"
    );
    assert!(
        stderr.contains("valid-semver"),
        "Expected valid-semver error, got: {stderr}"
    );
    assert!(
        stderr.contains("happy_path"),
        "Expected at-least-one-happy-path error, got: {stderr}"
    );
}

/// validate-command.spec: skip-semantic-when-parse-fails
#[test]
fn skip_semantic_when_parse_fails() {
    // Has a parse error (unknown keyword) AND would fail semantics
    // (InvalidName, no happy_path) — but only parse errors should appear
    let spec = "\
spec InvalidName v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

frobnicate something

behavior fail-thing [error_case]
  \"Only error case\"

  given
    Ready

  when act

  then emits stderr
    assert output contains \"error\"
";
    let (_dir, path) = temp_spec("parse-blocks-semantic", spec);
    let output = minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    // Should mention the parse error with the unrecognised keyword
    assert!(
        stderr.contains("frobnicate"),
        "Expected parse error mentioning 'frobnicate' in stderr, got: {stderr}"
    );
    // Should NOT mention semantic errors like kebab-case or happy_path
    assert!(
        !stderr.contains("kebab") && !stderr.contains("happy_path"),
        "Semantic errors should not appear when parse fails, got: {stderr}"
    );
}

/// validate-command.spec: exit-1-when-any-file-invalid
#[test]
fn exit_1_when_any_file_invalid() {
    let invalid_spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior fail-thing [error_case]
  \"No happy path\"

  given
    Ready

  when act

  then emits stderr
    assert output contains \"error\"
";
    let (_dir, paths) = temp_specs(&[("valid", VALID_SPEC), ("invalid", invalid_spec)]);
    minter()
        .arg("validate")
        .arg(&paths[0])
        .arg(&paths[1])
        .assert()
        .failure()
        .stderr(predicate::str::contains("happy_path"));
}

/// validate-command.spec: validate-all-files-independently
#[test]
fn validate_all_files_independently() {
    let invalid_spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior fail-thing [error_case]
  \"No happy path\"

  given
    Ready

  when act

  then emits stderr
    assert output contains \"error\"
";
    let (_dir, paths) = temp_specs(&[("invalid", invalid_spec), ("valid", VALID_SPEC)]);
    let output = minter()
        .arg("validate")
        .arg(&paths[0])
        .arg(&paths[1])
        .assert()
        .failure();

    // Both files should get result lines in stdout
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    // One should fail (✗) and one should pass (✓)
    assert!(
        stdout.contains("\u{2717}") && stdout.contains("\u{2713}"),
        "Expected both ✗ and ✓ in stdout (both files validated), got: {stdout}"
    );
    // Stderr should report the specific error for the invalid file
    assert!(
        stderr.contains("happy_path"),
        "Expected happy_path error in stderr, got: {stderr}"
    );
}

/// validate-command.spec: validate-directory-with-invalid
#[test]
fn validate_directory_with_invalid() {
    let invalid_spec = "\
spec invalid v1.0.0
title \"Invalid\"

description
  Invalid spec.

motivation
  Test.

behavior fail-thing [error_case]
  \"No happy path\"

  given
    Ready

  when act

  then emits stderr
    assert output contains \"error\"
";
    let (_dir, dir_path) = temp_dir_with_specs(&[("valid", VALID_SPEC), ("invalid", invalid_spec)]);
    let output = minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .failure();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);

    // Both specs should get result lines in stdout with their names
    assert!(
        stdout.contains("invalid v1.0.0"),
        "Expected 'invalid' spec in stdout, got: {stdout}"
    );
    assert!(
        stdout.contains("valid v1.0.0"),
        "Expected 'valid' spec in stdout, got: {stdout}"
    );
    // Stderr should report the specific error for the invalid file
    assert!(
        stderr.contains("happy_path"),
        "Expected happy_path error in stderr, got: {stderr}"
    );
}

/// validate-command.spec: reject-empty-directory
#[test]
fn handle_empty_directory() {
    let dir = tempfile::TempDir::new().expect("create temp dir");
    let dir_path = dir.path().to_path_buf();
    minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("no .spec")
            .or(predicate::str::contains("no spec files")));
}

/// validate-command.spec: reject-nonexistent-directory
#[test]
fn handle_nonexistent_directory() {
    let nonexistent = "/tmp/minter_nonexistent_dir_test";
    minter()
        .arg("validate")
        .arg(nonexistent)
        .assert()
        .failure()
        .stderr(predicate::str::contains(nonexistent));
}

// ═══════════════════════════════════════════════════════════════
// NFR validation (validate-command.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-command.spec: validate-valid-nfr
#[test]
fn validate_valid_nfr() {
    let (_dir, path) = temp_nfr("perf", VALID_NFR);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// validate-command.spec: validate-nfr-single-file-is-isolated
#[test]
fn validate_nfr_single_file_is_isolated() {
    // NFR files have no dependency graph — always isolated
    let (_dir, path) = temp_nfr("perf", VALID_NFR);
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&path)
        .assert()
        .success();
}

/// validate-command.spec: reject-invalid-nfr
#[test]
fn reject_invalid_nfr() {
    let content = "\
nfr banana v1.0.0
title \"Bad\"

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
    let (_dir, path) = temp_nfr("bad", content);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid category"));
}

/// validate-command.spec: discover-nfr-in-directory
#[test]
fn discover_nfr_in_directory() {
    let dir = tempfile::TempDir::new().expect("create temp dir");
    let nfr_path = dir.path().join("perf.nfr");
    std::fs::write(&nfr_path, VALID_NFR).expect("write nfr file");

    minter()
        .arg("validate")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("performance"));
}

/// validate-command.spec: validate-cross-references-in-directory
#[test]
fn validate_cross_references_in_directory() {
    let spec_content = "\
spec crossref-test v1.0.0
title \"Crossref Test\"

description
  A spec with nfr cross-references.

motivation
  Testing cross-ref resolution.

nfr
  performance#api-response-time

behavior do-thing [happy_path]
  \"Do the thing\"

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, dir_path) =
        temp_dir_with_spec_and_nfrs("crossref-test", spec_content, &[("performance", VALID_NFR)]);

    let output = minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("crossref-test"),
        "Should validate .spec file, got: {stdout}"
    );
    assert!(
        stdout.contains("performance"),
        "Should validate .nfr file, got: {stdout}"
    );
}

/// validate-command.spec: reject-broken-cross-reference
#[test]
fn reject_broken_cross_reference() {
    let spec_content = "\
spec broken-ref v1.0.0
title \"Broken Ref\"

description
  A spec referencing a nonexistent NFR category.

motivation
  Testing broken cross-refs.

nfr
  reliability#completeness

behavior do-thing [happy_path]
  \"Do the thing\"

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let dir = tempfile::TempDir::new().expect("create temp dir");
    let spec_path = dir.path().join("broken-ref.spec");
    std::fs::write(&spec_path, spec_content).expect("write spec file");

    minter()
        .arg("validate")
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("reliability"));
}

/// validate-command.spec: validate-mixed-spec-and-nfr-directory
#[test]
fn validate_mixed_spec_and_nfr_directory() {
    let dir = tempfile::TempDir::new().expect("create temp dir");
    let spec_path = dir.path().join("test-spec.spec");
    std::fs::write(&spec_path, VALID_SPEC).expect("write spec file");
    let nfr_path = dir.path().join("perf.nfr");
    std::fs::write(&nfr_path, VALID_NFR).expect("write nfr file");

    let output = minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("test-spec"),
        "Should validate .spec file, got: {stdout}"
    );
    assert!(
        stdout.contains("performance"),
        "Should validate .nfr file, got: {stdout}"
    );
}

