mod common;

use common::{minter, temp_dir_with_specs, temp_nfr, temp_spec, temp_specs, VALID_SPEC};
use predicates::prelude::*;

/// Helper: a valid spec with a given name, version, and number of behaviors.
fn spec_with_behaviors(name: &str, version: &str, count: usize) -> String {
    let mut s = format!(
        "\
spec {name} v{version}
title \"Test\"

description
  Test spec.

motivation
  Testing display.

"
    );
    for i in 0..count {
        let category = if i == 0 { "happy_path" } else { "edge_case" };
        s.push_str(&format!(
            "\
behavior do-thing-{i} [{category}]
  \"Behavior {i}\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

"
        ));
    }
    s
}

/// Helper: a spec that fails semantic validation (no happy_path).
fn failing_spec(name: &str, version: &str) -> String {
    format!(
        "\
spec {name} v{version}
title \"Failing\"

description
  A spec that fails.

motivation
  Testing failure display.

behavior fail-thing [error_case]
  \"Only error case\"

  given
    Ready

  when act

  then emits stderr
    assert output contains \"error\"
"
    )
}

/// Helper: a valid spec with dependencies.
fn spec_with_dep(name: &str, version: &str, deps: &[(&str, &str)]) -> String {
    let mut s = spec_with_behaviors(name, version, 1);
    for (dep_name, dep_ver) in deps {
        s.push_str(&format!("depends on {dep_name} >= {dep_ver}\n"));
    }
    s
}

// ═══════════════════════════════════════════════════════════════
// Success output (validate-display.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-display.spec: display-success-line
#[test]
fn display_success_line() {
    let spec = spec_with_behaviors("my-feature", "1.2.0", 12);
    let (_dir, path) = temp_spec("my-feature", &spec);
    minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ my-feature v1.2.0 (12 behaviors)"));
}

/// validate-display.spec: display-singular-behavior-count
#[test]
fn display_singular_behavior_count() {
    let spec = spec_with_behaviors("single-case", "1.0.0", 1);
    let (_dir, path) = temp_spec("single-case", &spec);
    minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ single-case v1.0.0 (1 behavior)"));
}

// ═══════════════════════════════════════════════════════════════
// Failure output (validate-display.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-display.spec: display-failure-line
#[test]
fn display_failure_line() {
    let spec = failing_spec("broken-feature", "2.0.0");
    let (_dir, path) = temp_spec("broken-feature", &spec);
    minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stdout(predicate::str::contains("✗ broken-feature v2.0.0"));
}

/// validate-display.spec: display-errors-on-stderr
#[test]
fn display_errors_on_stderr() {
    let spec = failing_spec("broken-feature", "2.0.0");
    let (_dir, path) = temp_spec("broken-feature", &spec);
    let output = minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Errors should be on stderr with the specific validation error
    assert!(
        stderr.contains("happy_path"),
        "stderr should contain 'happy_path' error detail, got: {stderr}"
    );

    // stdout should NOT contain error details (only the result line)
    assert!(
        !stdout.contains("happy_path") && !stdout.contains("error:"),
        "stdout should not contain error details, got: {stdout}"
    );
}

// ═══════════════════════════════════════════════════════════════
// Dependency tree (validate-display.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-display.spec: display-dependency-tree
#[test]
fn display_dependency_tree() {
    let a = spec_with_dep("a", "1.0.0", &[("b", "1.0.0")]);
    let b = spec_with_dep("b", "1.0.0", &[("c", "1.0.0")]);
    let c = spec_with_behaviors("c", "1.0.0", 1);

    let (_dir, paths) = temp_specs(&[("a", &a), ("b", &b), ("c", &c)]);
    let output = minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Should show tree connectors
    let has_connectors = stdout.contains("├──") || stdout.contains("└──") || stdout.contains("│");
    assert!(
        has_connectors,
        "Expected tree connectors (├──, └──, │) in stdout, got: {stdout}"
    );

    // All three specs should appear
    assert!(stdout.contains("a"), "Missing 'a' in tree output: {stdout}");
    assert!(stdout.contains("b"), "Missing 'b' in tree output: {stdout}");
    assert!(stdout.contains("c"), "Missing 'c' in tree output: {stdout}");
}

/// validate-display.spec: display-first-occurrence-expanded
#[test]
fn display_first_occurrence_expanded() {
    let a = spec_with_dep("a", "1.0.0", &[("b", "1.0.0")]);
    let b = spec_with_behaviors("b", "2.0.0", 3);

    let (_dir, paths) = temp_specs(&[("a", &a), ("b", &b)]);
    let output = minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // First occurrence of b should show version and behavior count
    assert!(
        stdout.contains("b v2.0.0") && stdout.contains("3 behaviors"),
        "First occurrence should show version and behavior count, got: {stdout}"
    );
}

/// validate-display.spec: display-repeated-dep-dimmed
/// b is expanded at depth 1 (shallowest), dimmed at depth 2 under c.
/// Dep order is c before b to ensure shallowest wins over traversal order.
#[test]
fn display_repeated_dep_dimmed() {
    // a depends on c then b; c also depends on b
    // Depth-first would visit c first, then b under c (depth 2), then b under a (depth 1).
    // Correct behavior: b at depth 1 is expanded, b at depth 2 is dimmed.
    let a = spec_with_dep("a", "1.0.0", &[("c", "1.0.0"), ("b", "1.0.0")]);
    let b = spec_with_behaviors("b", "1.0.0", 1);
    let c = spec_with_dep("c", "1.0.0", &[("b", "1.0.0")]);

    let (_dir, paths) = temp_specs(&[("a", &a), ("b", &b), ("c", &c)]);
    let output = minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // b should be expanded at its shallowest depth (depth 1, direct child of a)
    // Look for the expanded b line at root level (not indented under c)
    let lines: Vec<&str> = stdout.lines().collect();
    let b_expanded_at_depth1 = lines.iter().any(|l| {
        // Direct child of a: starts with tree connector, not nested further
        (l.starts_with("├── ") || l.starts_with("└── "))
            && l.contains("b v1.0.0 (1 behavior)")
    });
    assert!(
        b_expanded_at_depth1,
        "b should be expanded at depth 1 (direct child of a), got:\n{stdout}"
    );

    // b should be dimmed at deeper occurrence (depth 2, under c)
    assert!(
        stdout.contains("\x1b[2m"),
        "b should be dimmed at deeper occurrence, got:\n{stdout}"
    );

    // b appears fully expanded exactly once
    let expanded_count = stdout.matches("b v1.0.0 (1 behavior)").count();
    assert_eq!(
        expanded_count, 1,
        "b should be fully expanded exactly once, got {expanded_count} in:\n{stdout}"
    );
}

/// validate-display.spec: display-repeated-dep-preserves-status
#[test]
fn display_repeated_dep_preserves_status() {
    // a depends on b and c; c also depends on b; b fails validation
    let a = spec_with_dep("a", "1.0.0", &[("b", "1.0.0"), ("c", "1.0.0")]);
    let b = failing_spec("b", "1.0.0");
    let c = spec_with_dep("c", "1.0.0", &[("b", "1.0.0")]);

    let (_dir, paths) = temp_specs(&[("a", &a), ("b", &b), ("c", &c)]);
    let output = minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .failure();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // b should show ✗ on first occurrence
    assert!(
        stdout.contains("✗"),
        "Expected ✗ for failed dep b, got: {stdout}"
    );

    // b should also show ✗ on the dimmed repeated occurrence
    let cross_count = stdout.matches("✗").count();
    assert!(
        cross_count >= 2,
        "Expected ✗ on both first and dimmed occurrence of b, got {cross_count} in: {stdout}"
    );
}

/// validate-display.spec: display-tree-error-on-stderr
#[test]
fn display_tree_error_on_stderr() {
    let a = spec_with_dep("a", "1.0.0", &[("b", "1.0.0")]);
    let b = failing_spec("b", "1.0.0");

    let (_dir, paths) = temp_specs(&[("a", &a), ("b", &b)]);
    let output = minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);

    // Error details for b should be on stderr
    assert!(
        stderr.contains("b") && stderr.contains("validation errors"),
        "stderr should mention dep 'b' has validation errors, got: {stderr}"
    );
}

// ═══════════════════════════════════════════════════════════════
// Channel separation (validate-display.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-display.spec: separate-result-and-errors
#[test]
fn separate_result_and_errors() {
    let (_dir, path) = temp_spec("test-spec", VALID_SPEC);
    let output = minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);

    // stdout should have the result line
    assert!(
        stdout.contains("test-spec"),
        "stdout should contain result line, got: {stdout}"
    );

    // stderr should be empty for a valid spec
    assert!(
        stderr.is_empty(),
        "stderr should be empty for valid spec, got: {stderr}"
    );
}

// ═══════════════════════════════════════════════════════════════
// Directory tree output (validate-display.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-display.spec: skip-already-shown-root
#[test]
fn skip_already_shown_root() {
    // a depends on b; b has no deps. Both are files in the directory.
    let a = spec_with_dep("a", "1.0.0", &[("b", "1.0.0")]);
    let b = spec_with_behaviors("b", "1.0.0", 1);

    let (_dir, dir_path) = temp_dir_with_specs(&[("a", &a), ("b", &b)]);
    let output = minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg("--deep")
        .arg(&dir_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // a should appear as the root with its tree including b
    assert!(
        stdout.contains("✓ a v1.0.0"),
        "a should appear as root, got:\n{stdout}"
    );
    assert!(
        stdout.contains("b v1.0.0 (1 behavior)"),
        "b should appear in a's tree, got:\n{stdout}"
    );

    // Count how many root-level result lines there are (lines without tree prefix,
    // excluding the dependency count summary line).
    // b was already shown in a's tree, so it should NOT appear as a separate root line.
    let root_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| !l.starts_with("├") && !l.starts_with("│") && !l.starts_with("└") && !l.starts_with(" "))
        .filter(|l| !l.contains("resolved"))
        .collect();

    assert_eq!(
        root_lines.len(), 1,
        "Only a should appear at root level (b already shown in tree), got root lines: {root_lines:?}\nfull output:\n{stdout}"
    );
}

// ═══════════════════════════════════════════════════════════════
// ANSI color output (validate-display.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-display.spec: color-success-checkmark-green
#[test]
fn color_success_checkmark_green() {
    let spec = spec_with_behaviors("valid", "1.0.0", 1);
    let (_dir, path) = temp_spec("valid", &spec);
    let output = minter()
        .env_remove("NO_COLOR")
        .arg("validate")
        .arg(&path)
        .assert()
        .success();

    let stdout = output.get_output().stdout.clone();
    let stdout_str = String::from_utf8_lossy(&stdout);

    // The ✓ should be wrapped in ANSI green: \x1b[32m ✓ \x1b[0m
    assert!(
        stdout_str.contains("\x1b[32m\u{2713}\x1b[0m"),
        "Expected green ANSI escape around ✓, got: {:?}",
        stdout_str
    );
}

/// validate-display.spec: color-failure-cross-red
#[test]
fn color_failure_cross_red() {
    let spec = failing_spec("broken", "1.0.0");
    let (_dir, path) = temp_spec("broken", &spec);
    let output = minter()
        .env_remove("NO_COLOR")
        .arg("validate")
        .arg(&path)
        .assert()
        .failure();

    let stdout = output.get_output().stdout.clone();
    let stdout_str = String::from_utf8_lossy(&stdout);

    // The ✗ should be wrapped in ANSI red: \x1b[31m ✗ \x1b[0m
    assert!(
        stdout_str.contains("\x1b[31m\u{2717}\x1b[0m"),
        "Expected red ANSI escape around ✗, got: {:?}",
        stdout_str
    );
}

// ═══════════════════════════════════════════════════════════════
// No-color mode (validate-display.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-display.spec: no-color-mode
#[test]
fn no_color_mode() {
    let spec = spec_with_behaviors("plain", "1.0.0", 1);
    let (_dir, path) = temp_spec("plain", &spec);
    let output = minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg(&path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Should contain the checkmark
    assert!(
        stdout.contains("\u{2713}"),
        "Expected ✓ in output, got: {stdout}"
    );

    // Should NOT contain any ANSI escape sequences
    assert!(
        !stdout.contains("\x1b["),
        "Expected no ANSI escapes with NO_COLOR=1, got: {stdout:?}"
    );
}

// ═══════════════════════════════════════════════════════════════
// Dependency count (validate-display.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-display.spec: display-dependency-count
#[test]
fn display_dependency_count() {
    let a = spec_with_dep("a", "1.0.0", &[("b", "1.0.0"), ("c", "1.0.0")]);
    let b = spec_with_behaviors("b", "1.0.0", 1);
    let c = spec_with_behaviors("c", "1.0.0", 1);

    let (_dir, paths) = temp_specs(&[("a", &a), ("b", &b), ("c", &c)]);
    let output = minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Should show "2 dependencies resolved"
    assert!(
        stdout.contains("2 dependencies resolved"),
        "Expected dependency count '2 dependencies resolved' in output, got: {stdout}"
    );
}

// ═══════════════════════════════════════════════════════════════
// Long spec name (validate-display.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-display.spec: display-long-spec-name
#[test]
fn display_long_spec_name() {
    let long_name = "my-very-long-feature-name-that-exceeds-typical-width";
    let spec = spec_with_behaviors(long_name, "1.0.0", 5);
    let (_dir, path) = temp_spec(long_name, &spec);
    minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "✓ my-very-long-feature-name-that-exceeds-typical-width v1.0.0 (5 behaviors)",
        ));
}

// ═══════════════════════════════════════════════════════════════
// NFR display output (validate-display.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-display.spec: display-nfr-success-line
#[test]
fn display_nfr_success_line() {
    let content = "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.


constraint c1 [metric]
  \"M1\"

  metric \"m\"
  threshold < 1s

  verification
    environment all
    benchmark \"b\"
    pass \"p\"

  violation high
  overridable yes


constraint c2 [metric]
  \"M2\"

  metric \"m\"
  threshold > 100

  verification
    environment all
    benchmark \"b\"
    pass \"p\"

  violation medium
  overridable yes


constraint c3 [rule]
  \"R1\"

  rule
    Invariant.

  verification
    static \"S\"

  violation low
  overridable no


constraint c4 [rule]
  \"R2\"

  rule
    Another invariant.

  verification
    runtime \"R\"

  violation medium
  overridable no
";
    let (_dir, path) = temp_nfr("perf", content);
    minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "✓ performance v1.0.0 (4 constraints)",
        ));
}

/// validate-display.spec: display-nfr-singular-constraint-count
#[test]
fn display_nfr_singular_constraint_count() {
    let content = "\
nfr security v1.0.0
title \"Security\"

description
  D.

motivation
  M.


constraint one-rule [rule]
  \"Single rule\"

  rule
    Invariant.

  verification
    static \"S\"

  violation critical
  overridable no
";
    let (_dir, path) = temp_nfr("sec", content);
    minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "✓ security v1.0.0 (1 constraint)",
        ));
}

/// validate-display.spec: display-nfr-failure-line
#[test]
fn display_nfr_failure_line() {
    // NFR with non-kebab constraint name triggers semantic error
    let content = "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.


constraint BadName [rule]
  \"Bad\"

  rule
    R.

  verification
    static \"S\"

  violation low
  overridable no
";
    let (_dir, path) = temp_nfr("bad", content);
    minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stdout(predicate::str::contains("✗ performance v1.0.0"));
}
