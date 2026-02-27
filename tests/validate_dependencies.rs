mod common;

use common::{minter, temp_dir_with_nested_specs, temp_dir_with_specs, temp_spec, temp_specs};
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
// Happy paths (dependency-resolution.spec)
// ═══════════════════════════════════════════════════════════════

/// dependency-resolution.spec: resolve-direct-dependencies
#[test]
fn validate_spec_then_resolve_deps() {
    let (_dir, paths) = temp_specs(&[
        ("consumer", &consumer_spec("provider", "1.0.0")),
        ("provider", &provider_spec("provider", "1.0.0")),
    ]);
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .success()
        .stdout(predicate::str::contains("consumer"))
        .stdout(predicate::str::contains("provider"));
}

/// dependency-resolution.spec: resolve-by-name-in-tree
#[test]
fn resolve_by_name_in_tree() {
    let (_dir, paths) = temp_specs(&[
        ("consumer", &consumer_spec("provider", "1.0.0")),
        ("provider", &provider_spec("provider", "1.2.0")),
    ]);
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .success();
}

/// dependency-resolution.spec: resolve-transitive-dependencies
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
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .success();
}

/// dependency-resolution.spec: version-constraint-satisfied
#[test]
fn version_constraint_satisfied() {
    let (_dir, paths) = temp_specs(&[
        ("consumer", &consumer_spec("provider", "1.0.0")),
        ("provider", &provider_spec("provider", "2.3.0")),
    ]);
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .success();
}

/// dependency-resolution.spec: version-constraint-exact-match
#[test]
fn version_constraint_exact_match() {
    let (_dir, paths) = temp_specs(&[
        ("consumer", &consumer_spec("provider", "1.0.0")),
        ("provider", &provider_spec("provider", "1.0.0")),
    ]);
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .success();
}

/// dependency-resolution.spec: version-constraint-patch-higher
#[test]
fn version_constraint_patch_higher() {
    let (_dir, paths) = temp_specs(&[
        ("consumer", &consumer_spec("provider", "1.0.0")),
        ("provider", &provider_spec("provider", "1.0.1")),
    ]);
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .success();
}

/// dependency-resolution.spec: reject-version-below-constraint
#[test]
fn reject_version_below_constraint() {
    let (_dir, paths) = temp_specs(&[
        ("consumer", &consumer_spec("provider", "1.0.0")),
        ("provider", &provider_spec("provider", "0.9.0")),
    ]);
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .failure()
        .stderr(predicate::str::contains("provider"))
        .stderr(predicate::str::contains(">= 1.0.0"))
        .stderr(predicate::str::contains("0.9.0"));
}

/// dependency-resolution.spec: reject-transitive-cycle
#[test]
fn reject_transitive_cycle() {
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

depends on a >= 1.0.0
";
    let (_dir, paths) = temp_specs(&[("a", a_spec), ("b", b_spec), ("c", c_spec)]);
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cycle").or(predicate::str::contains("cycl")))
        .stderr(predicate::str::contains("a"))
        .stderr(predicate::str::contains("b"))
        .stderr(predicate::str::contains("c"));
}

/// dependency-resolution.spec: folder-automatically-resolves-dependencies
#[test]
fn folder_automatically_resolves_dependencies() {
    let a_spec = "\
spec a v1.0.0
title \"A\"

description
  Spec A depends on b.

motivation
  Test folder auto-resolves deps.

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
";
    let (_dir, dir_path) = temp_dir_with_specs(&[("a", a_spec), ("b", b_spec)]);
    let output = minter().arg("validate").arg(&dir_path).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("a") && stdout.contains("b"),
        "Folder validation should auto-resolve deps; expected both a and b, got: {stdout}"
    );
}

// ═══════════════════════════════════════════════════════════════
// Error cases (dependency-resolution.spec)
// ═══════════════════════════════════════════════════════════════

/// dependency-resolution.spec: reject-missing-dependency
#[test]
fn reject_missing_dependency() {
    let (_dir, path) = temp_spec("consumer", &consumer_spec("nonexistent", "1.0.0"));
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

/// dependency-resolution.spec: reject-incompatible-version
#[test]
fn reject_incompatible_version() {
    let (_dir, paths) = temp_specs(&[
        ("consumer", &consumer_spec("provider", "2.0.0")),
        ("provider", &provider_spec("provider", "1.5.0")),
    ]);
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .failure()
        .stderr(predicate::str::contains("provider"))
        .stderr(predicate::str::contains("2.0.0").or(predicate::str::contains("1.5.0")));
}

/// dependency-resolution.spec: reject-cyclic-dependencies
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
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cycle").or(predicate::str::contains("cycl")));
}

/// dependency-resolution.spec: reject-invalid-dependency-spec
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
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .failure()
        .stderr(predicate::str::contains("broken"));
}

// ═══════════════════════════════════════════════════════════════
// Edge cases (dependency-resolution.spec)
// ═══════════════════════════════════════════════════════════════

/// dependency-resolution.spec: skip-deps-when-spec-invalid
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
    let output = minter()
        .arg("validate")
        .arg("--deep")
        .arg(&paths[0])
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Should contain the parse error with the unrecognised keyword
    assert!(
        stderr.contains("frobnicate"),
        "Expected parse error mentioning 'frobnicate' in stderr, got: {stderr}"
    );
    // Should NOT contain dependency resolution output
    assert!(
        !stdout.contains("dependencies resolved") && !stdout.contains("resolved"),
        "Dependency resolution should be skipped when spec is invalid, got stdout: {stdout}"
    );
}

/// dependency-resolution.spec: handle-no-dependencies
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
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("test-spec v1.0.0 (1 behavior)"));
}

/// dependency-resolution.spec: report-all-resolution-errors
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
    let output = minter()
        .arg("validate")
        .arg("--deep")
        .arg(&path)
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    // All three missing deps should be reported
    assert!(
        stderr.contains("missing-one")
            && stderr.contains("missing-two")
            && stderr.contains("missing-three"),
        "All unresolved deps should be reported, got: {stderr}"
    );
}

// ═══════════════════════════════════════════════════════════════
// Cross-directory and duplicate name tests (dependency-resolution.spec)
// ═══════════════════════════════════════════════════════════════

/// dependency-resolution.spec: resolve-cross-directory-dependency
#[test]
fn resolve_cross_directory_dependency() {
    let a_spec = "\
spec a v1.0.0
title \"A\"

description
  Spec A in validation subdir.

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
  Spec B in caching subdir.

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
    let (_dir, dir_path) =
        temp_dir_with_nested_specs(&[("validation/a", a_spec), ("caching/b", b_spec)]);
    // Validate the whole directory tree — cross-directory deps resolve when
    // the tree root encompasses both subdirectories
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&dir_path)
        .assert()
        .success();
}

/// dependency-resolution.spec: reject-duplicate-spec-names
#[test]
fn reject_duplicate_spec_names() {
    let spec_content = "\
spec my-feature v1.0.0
title \"My Feature\"

description
  A feature spec.

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
        ("sub1/my-feature", spec_content),
        ("sub2/my-feature", spec_content),
    ]);
    let output = minter()
        .arg("validate")
        .arg("--deep")
        .arg(&dir_path)
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    // Should mention the duplicate name and both paths
    assert!(
        stderr.contains("my-feature"),
        "stderr should mention the duplicate spec name, got: {stderr}"
    );
    assert!(
        stderr.contains("sub1") && stderr.contains("sub2"),
        "stderr should mention both file paths, got: {stderr}"
    );
}
