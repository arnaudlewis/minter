mod common;

use common::{specval, temp_spec, temp_specs};
use predicates::prelude::*;

/// Helper: a valid spec that depends on another spec.
fn consumer_spec(dep_name: &str, dep_version: &str) -> String {
    format!(
        "\
spec consumer v1.0.0
title \"Consumer\"

description
  A spec that depends on another.

motivation
  Testing dependency resolution.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

depends on {dep_name} >= {dep_version}
"
    )
}

/// Helper: a valid spec with a given name and version.
fn provider_spec(name: &str, version: &str) -> String {
    format!(
        "\
spec {name} v{version}
title \"Provider\"

description
  A dependency spec.

motivation
  Exists to be depended upon.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
"
    )
}

// ═══════════════════════════════════════════════════════════════
// Happy paths (validate-dependencies.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-dependencies.spec: validate-spec-then-resolve-deps
#[test]
fn validate_spec_then_resolve_deps() {
    let (_dir, paths) = temp_specs(&[
        ("consumer", &consumer_spec("provider", "1.0.0")),
        ("provider", &provider_spec("provider", "1.0.0")),
    ]);
    specval()
        .arg("validate")
        .arg("--deps")
        .arg(&paths[0])
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ consumer"))
        .stdout(predicate::str::contains("provider"));
}

/// validate-dependencies.spec: resolve-by-sibling-name
#[test]
fn resolve_by_sibling_name() {
    let (_dir, paths) = temp_specs(&[
        ("consumer", &consumer_spec("provider", "1.0.0")),
        ("provider", &provider_spec("provider", "1.2.0")),
    ]);
    specval()
        .arg("validate")
        .arg("--deps")
        .arg(&paths[0])
        .assert()
        .success();
}

/// validate-dependencies.spec: resolve-transitive-dependencies
#[test]
fn resolve_transitive_dependencies() {
    let a_spec = "\
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

depends on b >= 1.0.0
";
    let b_spec = "\
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

depends on c >= 1.0.0
";
    let c_spec = "\
spec c v1.0.0
title \"C\"

description
  Spec C.

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
    let (_dir, paths) = temp_specs(&[("a", a_spec), ("b", b_spec), ("c", c_spec)]);
    specval()
        .arg("validate")
        .arg("--deps")
        .arg(&paths[0])
        .assert()
        .success();
}

// ═══════════════════════════════════════════════════════════════
// Error cases (validate-dependencies.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-dependencies.spec: reject-missing-dependency
#[test]
fn reject_missing_dependency() {
    let (_dir, path) = temp_spec("consumer", &consumer_spec("nonexistent", "1.0.0"));
    specval()
        .arg("validate")
        .arg("--deps")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

/// validate-dependencies.spec: reject-incompatible-version
#[test]
fn reject_incompatible_version() {
    let (_dir, paths) = temp_specs(&[
        ("consumer", &consumer_spec("provider", "2.0.0")),
        ("provider", &provider_spec("provider", "1.5.0")),
    ]);
    specval()
        .arg("validate")
        .arg("--deps")
        .arg(&paths[0])
        .assert()
        .failure()
        .stderr(predicate::str::contains("provider"))
        .stderr(predicate::str::contains("2.0.0").or(predicate::str::contains("1.5.0")));
}

/// validate-dependencies.spec: reject-cyclic-dependencies
#[test]
fn reject_cyclic_dependencies() {
    let a_spec = "\
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

depends on b >= 1.0.0
";
    let b_spec = "\
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

depends on a >= 1.0.0
";
    let (_dir, paths) = temp_specs(&[("a", a_spec), ("b", b_spec)]);
    specval()
        .arg("validate")
        .arg("--deps")
        .arg(&paths[0])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cycle").or(predicate::str::contains("cycl")));
}

/// validate-dependencies.spec: reject-invalid-dependency-spec
#[test]
fn reject_invalid_dependency_spec() {
    // Provider exists but has no happy_path (semantic error)
    let broken = "\
spec broken v1.0.0
title \"Broken\"

description
  Broken spec.

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
    let (_dir, paths) = temp_specs(&[
        ("consumer", &consumer_spec("broken", "1.0.0")),
        ("broken", broken),
    ]);
    specval()
        .arg("validate")
        .arg("--deps")
        .arg(&paths[0])
        .assert()
        .failure()
        .stderr(predicate::str::contains("broken"));
}

// ═══════════════════════════════════════════════════════════════
// Edge cases (validate-dependencies.spec)
// ═══════════════════════════════════════════════════════════════

/// validate-dependencies.spec: skip-deps-when-spec-invalid
#[test]
fn skip_deps_when_spec_invalid() {
    // Spec has a parse error — deps should not be checked
    let bad_spec = "\
spec InvalidName v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

frobnicate something
";
    let (_dir, paths) = temp_specs(&[
        ("consumer", bad_spec),
        ("provider", &provider_spec("provider", "1.0.0")),
    ]);
    let output = specval()
        .arg("validate")
        .arg("--deps")
        .arg(&paths[0])
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Should contain parse/validation errors in stderr
    assert!(
        stderr.contains("frobnicate") || stderr.contains("parse") || stderr.contains("keyword"),
        "Expected parse error in stderr, got: {stderr}"
    );
    // Should NOT contain dependency resolution output
    assert!(
        !stdout.contains("dependencies resolved") && !stdout.contains("resolved"),
        "Dependency resolution should be skipped when spec is invalid, got stdout: {stdout}"
    );
}

/// validate-dependencies.spec: handle-no-dependencies
#[test]
fn handle_no_dependencies() {
    let spec = "\
spec test-spec v1.0.0
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
    let (_dir, path) = temp_spec("no-deps", spec);
    specval()
        .arg("validate")
        .arg("--deps")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ test-spec v1.0.0 (1 behavior)"));
}

/// validate-dependencies.spec: report-all-resolution-errors
#[test]
fn report_all_resolution_errors() {
    let spec = "\
spec consumer v1.0.0
title \"Consumer\"

description
  Consumer with multiple missing deps.

motivation
  Test.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

depends on missing-one >= 1.0.0
depends on missing-two >= 1.0.0
depends on missing-three >= 1.0.0
";
    let (_dir, path) = temp_spec("consumer", spec);
    let output = specval()
        .arg("validate")
        .arg("--deps")
        .arg(&path)
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    // All three missing deps should be reported
    assert!(
        stderr.contains("missing-one") && stderr.contains("missing-two") && stderr.contains("missing-three"),
        "All unresolved deps should be reported, got: {stderr}"
    );
}
