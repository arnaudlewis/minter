mod common;

use common::{minter, temp_nfr, temp_spec, VALID_NFR};
use predicates::prelude::*;

// ═══════════════════════════════════════════════════════════════
// Happy paths (inspect-command.spec)
// ═══════════════════════════════════════════════════════════════

/// inspect-command.spec: inspect-behavior-count
#[test]
fn inspect_behavior_count() {
    let spec = "\
spec my-feature v1.0.0
title \"My Feature\"

description
  A feature.

motivation
  Testing.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"

behavior do-other [error_case]
  \"Handles error\"

  given
    Bad state

  when act

  then returns error
    assert status == \"fail\"

behavior edge [edge_case]
  \"Edge case\"

  given
    Edge state

  when act

  then returns result
    assert status == \"ok\"
";
    let (_dir, path) = temp_spec("multi", spec);
    minter()
        .arg("inspect")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("3 behaviors"));
}

/// inspect-command.spec: inspect-category-distribution
#[test]
fn inspect_category_distribution() {
    let spec = "\
spec category-test v1.0.0
title \"Category Test\"

description
  Testing categories.

motivation
  Category distribution.

behavior hp1 [happy_path]
  \"Happy 1\"

  given
    Ready

  when act

  then returns r
    assert x == \"1\"

behavior hp2 [happy_path]
  \"Happy 2\"

  given
    Ready

  when act

  then returns r
    assert x == \"2\"

behavior hp3 [happy_path]
  \"Happy 3\"

  given
    Ready

  when act

  then returns r
    assert x == \"3\"

behavior hp4 [happy_path]
  \"Happy 4\"

  given
    Ready

  when act

  then returns r
    assert x == \"4\"

behavior ec1 [error_case]
  \"Error 1\"

  given
    Ready

  when act

  then returns r
    assert x == \"5\"

behavior ec2 [error_case]
  \"Error 2\"

  given
    Ready

  when act

  then returns r
    assert x == \"6\"

behavior edge1 [edge_case]
  \"Edge 1\"

  given
    Ready

  when act

  then returns r
    assert x == \"7\"
";
    let (_dir, path) = temp_spec("categories", spec);
    let output = minter()
        .arg("inspect")
        .arg(&path)
        .output()
        .expect("run inspect");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("happy_path"));
    assert!(stdout.contains("4"));
    assert!(stdout.contains("error_case"));
    assert!(stdout.contains("2"));
    assert!(stdout.contains("edge_case"));
    assert!(stdout.contains("1"));
}

/// inspect-command.spec: inspect-dependencies
#[test]
fn inspect_dependencies() {
    let spec = "\
spec dep-test v1.0.0
title \"Dep Test\"

description
  Testing deps.

motivation
  Deps.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"

depends on user-auth >= 1.0.0
depends on billing >= 2.0.0
";
    let (_dir, path) = temp_spec("deps", spec);
    minter()
        .arg("inspect")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("user-auth"))
        .stdout(predicate::str::contains("1.0.0"))
        .stdout(predicate::str::contains("billing"))
        .stdout(predicate::str::contains("2.0.0"));
}

/// inspect-command.spec: inspect-assertion-types
#[test]
fn inspect_assertion_types() {
    let spec = "\
spec assert-test v1.0.0
title \"Assert Test\"

description
  Testing assertions.

motivation
  Assertions.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
    assert body contains \"hello\"
    assert field is_present
";
    let (_dir, path) = temp_spec("assertions", spec);
    minter()
        .arg("inspect")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("equals"))
        .stdout(predicate::str::contains("contains"))
        .stdout(predicate::str::contains("is_present"));
}

// ═══════════════════════════════════════════════════════════════
// Error cases (inspect-command.spec)
// ═══════════════════════════════════════════════════════════════

/// inspect-command.spec: inspect-invalid-spec
#[test]
fn inspect_invalid_spec() {
    let (_dir, path) = temp_spec("broken", "this is not a valid spec");
    minter()
        .arg("inspect")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

/// inspect-command.spec: inspect-nonexistent-file
#[test]
fn inspect_nonexistent_file() {
    minter()
        .arg("inspect")
        .arg("missing.spec")
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing.spec"));
}

/// inspect-command.spec: inspect-no-dependencies
#[test]
fn inspect_no_dependencies() {
    let spec = "\
spec no-deps v1.0.0
title \"No Deps\"

description
  No dependencies here.

motivation
  Testing.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
";
    let (_dir, path) = temp_spec("nodeps", spec);
    minter()
        .arg("inspect")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("no dependencies"));
}

// ═══════════════════════════════════════════════════════════════
// NFR inspection (inspect-command.spec)
// ═══════════════════════════════════════════════════════════════

/// inspect-command.spec: inspect-nfr-constraint-count
#[test]
fn inspect_nfr_constraint_count() {
    let (_dir, path) = temp_nfr("perf", VALID_NFR);
    minter()
        .arg("inspect")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 constraint"));
}

/// inspect-command.spec: inspect-nfr-type-distribution
#[test]
fn inspect_nfr_type_distribution() {
    let content = "\
nfr performance v1.0.0
title \"Perf\"

description
  D.

motivation
  M.


constraint metric-one [metric]
  \"First metric\"

  metric \"m1\"
  threshold < 1s

  verification
    environment all
    benchmark \"b1\"
    pass \"p1\"

  violation high
  overridable yes


constraint rule-one [rule]
  \"First rule\"

  rule
    Invariant.

  verification
    static \"S1\"

  violation low
  overridable no
";
    let (_dir, path) = temp_nfr("mixed", content);
    let output = minter()
        .arg("inspect")
        .arg(&path)
        .output()
        .expect("run inspect");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("metric: 1"));
    assert!(stdout.contains("rule: 1"));
}

/// inspect-command.spec: inspect-nfr-category
#[test]
fn inspect_nfr_category() {
    let (_dir, path) = temp_nfr("perf", VALID_NFR);
    minter()
        .arg("inspect")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("category: performance"));
}

/// inspect-command.spec: inspect-nfr-no-dependencies
#[test]
fn inspect_nfr_no_dependencies() {
    let (_dir, path) = temp_nfr("perf", VALID_NFR);
    minter()
        .arg("inspect")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("no dependencies"));
}
