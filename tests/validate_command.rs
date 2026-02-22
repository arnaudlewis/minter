mod common;

use common::{specval, temp_spec, temp_specs, VALID_SPEC};
use predicates::prelude::*;

// ═══════════════════════════════════════════════════════════════
// Happy paths (validate-command.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-command.spec: validate-valid-spec
#[test]
fn validate_valid_spec() {
    let (_dir, path) = temp_spec("test-spec", VALID_SPEC);
    specval()
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("test-spec"))
        .stdout(predicate::str::contains("valid"));
}

/// validate-command.spec: validate-prints-summary
#[test]
fn validate_prints_summary() {
    let (_dir, path) = temp_spec("test-spec", VALID_SPEC);
    specval()
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("test-spec"))
        .stdout(predicate::str::contains("1.0.0"))
        .stdout(predicate::str::contains("1")); // behavior count
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
    specval()
        .arg("validate")
        .arg(&paths[0])
        .arg(&paths[1])
        .assert()
        .success();
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
    specval()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("do-thing"));
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
    specval()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
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
    specval()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("user"));
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
    specval()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("InvalidName").or(predicate::str::contains("kebab-case")));
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
    specval()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("NOPE").or(predicate::str::contains("semver")));
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
    specval()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("happy_path"));
}

// ═══════════════════════════════════════════════════════════════
// Edge cases (validate-command.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-command.spec: handle-nonexistent-file
#[test]
fn handle_nonexistent_file() {
    specval()
        .arg("validate")
        .arg("/tmp/specval_nonexistent_file.spec")
        .assert()
        .failure()
        .stderr(predicate::str::contains("specval_nonexistent_file.spec"));
}

/// validate-command.spec: handle-unreadable-file
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

    specval()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("permission").or(predicate::str::contains("Permission")));
}

/// validate-command.spec: handle-empty-file
#[test]
fn handle_empty_file() {
    let (_dir, path) = temp_spec("empty", "");
    specval()
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
    let output = specval()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure();

    // Should report multiple errors, not just the first
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    // Count error occurrences — expect at least 2 distinct errors reported
    let error_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();
    assert!(
        error_lines.len() >= 2,
        "Expected at least 2 error lines, got {}: {:?}",
        error_lines.len(),
        error_lines
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
    let output = specval()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    // Should mention the parse error
    assert!(
        stderr.contains("frobnicate") || stderr.contains("parse") || stderr.contains("keyword"),
        "Expected parse error in stderr, got: {stderr}"
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
    specval()
        .arg("validate")
        .arg(&paths[0])
        .arg(&paths[1])
        .assert()
        .failure();
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
    let output = specval()
        .arg("validate")
        .arg(&paths[0])
        .arg(&paths[1])
        .assert()
        .failure();

    // Both files should be mentioned in output (stdout or stderr combined)
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("invalid") && combined.contains("valid"),
        "Both files should be reported, got stdout: {stdout}\nstderr: {stderr}"
    );
}
