mod common;

use std::fs;

use common::minter;
use predicates::prelude::*;
use tempfile::TempDir;

// ── Spec fixtures ───────────────────────────────────────

fn spec_two_behaviors() -> &'static str {
    "\
spec a v1.0.0
title \"A\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"


behavior do-other [happy_path]
  \"Does another\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
"
}

fn spec_three_behaviors() -> &'static str {
    "\
spec a v1.0.0
title \"A\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"


behavior do-other [happy_path]
  \"Does another\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"


behavior do-missing [happy_path]
  \"Does missing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
"
}

fn spec_one_behavior(name: &str, version: &str, behavior: &str) -> String {
    format!(
        "\
spec {name} v{version}
title \"{name}\"

description
  Test.

motivation
  Test.

behavior {behavior} [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
"
    )
}

fn spec_with_nfr_ref() -> &'static str {
    "\
spec a v1.0.0
title \"A\"

description
  Test.

motivation
  Test.

nfr
  performance#api-latency

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
"
}

fn nfr_performance() -> &'static str {
    "\
nfr performance v1.0.0
title \"Perf\"

description
  Perf.

motivation
  Perf.


constraint api-latency [metric]
  \"API latency\"

  metric \"p95 response time\"
  threshold < 500ms

  verification
    environment staging
    benchmark \"load test\"
    pass \"p95 < 500ms\"

  violation high
  overridable yes
"
}

// ═══════════════════════════════════════════════════════════════
// Happy paths (coverage-command.spec)
// ═══════════════════════════════════════════════════════════════

/// coverage-command: report-full-coverage
#[test]
fn report_full_coverage() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_two_behaviors()).unwrap();

    fs::write(dir.path().join("a.test.ts"), "// @minter:unit do-thing\n").unwrap();
    fs::write(dir.path().join("b.test.ts"), "// @minter:e2e do-other\n").unwrap();

    minter()
        .arg("coverage")
        .arg(spec_dir)
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("do-thing"))
        .stdout(predicate::str::contains("do-other"))
        .stdout(predicate::str::contains("2/2"))
        .stdout(predicate::str::contains("100"));
}

/// coverage-command: report-partial-coverage
#[test]
fn report_partial_coverage() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_three_behaviors()).unwrap();

    fs::write(dir.path().join("a.test.ts"), "// @minter:unit do-thing\n").unwrap();
    fs::write(dir.path().join("b.test.ts"), "// @minter:e2e do-other\n").unwrap();

    minter()
        .arg("coverage")
        .arg(spec_dir)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("do-thing"))
        .stdout(predicate::str::contains("do-other"))
        .stdout(predicate::str::contains("do-missing"))
        .stdout(predicate::str::contains("uncovered"))
        .stdout(predicate::str::contains("2/3"));
}

/// coverage-command: group-by-spec
#[test]
fn group_by_spec() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();
    fs::write(
        spec_dir.join("b.spec"),
        spec_one_behavior("b", "2.0.0", "do-other"),
    )
    .unwrap();

    fs::write(
        dir.path().join("a.test.ts"),
        "// @minter:unit do-thing\n// @minter:unit do-other\n",
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(spec_dir)
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("a v1.0.0"))
        .stdout(predicate::str::contains("b v2.0.0"));
}

/// coverage-command: show-test-types
#[test]
fn show_test_types() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    fs::write(dir.path().join("a.test.ts"), "// @minter:unit do-thing\n").unwrap();
    fs::write(dir.path().join("b.test.ts"), "// @minter:e2e do-thing\n").unwrap();

    minter()
        .arg("coverage")
        .arg(spec_dir)
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("do-thing"))
        .stdout(predicate::str::contains("unit"))
        .stdout(predicate::str::contains("e2e"));
}

/// coverage-command: show-summary
#[test]
fn show_summary() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_three_behaviors()).unwrap();

    // Cover do-thing with unit, do-other with unit and e2e
    fs::write(
        dir.path().join("a.test.ts"),
        "// @minter:unit do-thing\n// @minter:unit do-other\n",
    )
    .unwrap();
    fs::write(dir.path().join("b.test.ts"), "// @minter:e2e do-other\n").unwrap();

    minter()
        .arg("coverage")
        .arg(spec_dir)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("2/3"))
        .stdout(predicate::str::contains("66"))
        .stdout(predicate::str::contains("unit"))
        .stdout(predicate::str::contains("e2e"));
}

/// coverage-command: multiple-ids-in-one-tag
#[test]
fn multiple_ids_in_one_tag() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_two_behaviors()).unwrap();

    fs::write(
        dir.path().join("a.test.ts"),
        "// @minter:e2e do-thing do-other\n",
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(spec_dir)
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("do-thing"))
        .stdout(predicate::str::contains("do-other"))
        .stdout(predicate::str::contains("2/2"));
}

/// coverage-command: scan-double-slash-comments
#[test]
fn scan_double_slash_comments() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    fs::write(dir.path().join("a.test.ts"), "// @minter:unit do-thing\n").unwrap();

    minter()
        .arg("coverage")
        .arg(spec_dir)
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("do-thing"))
        .stdout(predicate::str::contains("unit"));
}

/// coverage-command: scan-hash-comments
#[test]
fn scan_hash_comments() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    fs::write(dir.path().join("a.test.py"), "# @minter:unit do-thing\n").unwrap();

    minter()
        .arg("coverage")
        .arg(spec_dir)
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("do-thing"))
        .stdout(predicate::str::contains("unit"));
}

// ═══════════════════════════════════════════════════════════════
// Scan scoping (coverage-command.spec)
// ═══════════════════════════════════════════════════════════════

/// coverage-command: scope-scan-with-flag
#[test]
fn scope_scan_with_flag() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    let unit_dir = dir.path().join("tests").join("unit");
    fs::create_dir_all(&unit_dir).unwrap();
    fs::write(unit_dir.join("a.test.ts"), "// @minter:unit do-thing\n").unwrap();

    let e2e_dir = dir.path().join("tests").join("e2e");
    fs::create_dir_all(&e2e_dir).unwrap();
    fs::write(e2e_dir.join("a.spec.ts"), "// @minter:e2e do-thing\n").unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .arg("--scan")
        .arg(&unit_dir)
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("unit"))
        .stdout(predicate::str::contains("e2e").not());
}

/// coverage-command: multiple-scan-flags
#[test]
fn multiple_scan_flags() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_with_nfr_ref()).unwrap();
    fs::write(spec_dir.join("performance.nfr"), nfr_performance()).unwrap();

    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("a.rs"), "// @minter:unit do-thing\n").unwrap();

    let bench_dir = dir.path().join("benches");
    fs::create_dir(&bench_dir).unwrap();
    fs::write(
        bench_dir.join("a.rs"),
        "// @minter:benchmark #performance#api-latency\n",
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .arg("--scan")
        .arg(&src_dir)
        .arg("--scan")
        .arg(&bench_dir)
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("unit"))
        .stdout(predicate::str::contains("benchmark"));
}

/// coverage-command: single-spec-file
#[test]
fn single_spec_file() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_two_behaviors()).unwrap();
    fs::write(
        spec_dir.join("b.spec"),
        spec_one_behavior("b", "1.0.0", "unrelated"),
    )
    .unwrap();

    fs::write(dir.path().join("a.test.ts"), "// @minter:unit do-thing\n").unwrap();

    minter()
        .arg("coverage")
        .arg(spec_dir.join("a.spec"))
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("do-thing"))
        .stdout(predicate::str::contains("do-other"))
        .stdout(predicate::str::contains("unrelated").not());
}

/// coverage-command: skip-gitignored-paths
#[test]
fn skip_gitignored_paths() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    // Create .gitignore
    fs::write(dir.path().join(".gitignore"), "node_modules/\n").unwrap();

    // Create node_modules with a tag (should be ignored)
    let nm_dir = dir.path().join("node_modules").join("dep");
    fs::create_dir_all(&nm_dir).unwrap();
    fs::write(nm_dir.join("test.js"), "// @minter:unit do-thing\n").unwrap();

    // Create tests with a tag (should be found)
    fs::write(dir.path().join("a.test.ts"), "// @minter:e2e do-thing\n").unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("e2e"))
        .stdout(predicate::str::contains("[unit]").not());
}

// ═══════════════════════════════════════════════════════════════
// NFR derived coverage (coverage-command.spec)
// ═══════════════════════════════════════════════════════════════

/// coverage-command: derive-nfr-from-covered-behavior
#[test]
fn derive_nfr_from_covered_behavior() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_with_nfr_ref()).unwrap();
    fs::write(spec_dir.join("performance.nfr"), nfr_performance()).unwrap();

    fs::write(dir.path().join("a.test.ts"), "// @minter:e2e do-thing\n").unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("performance#api-latency"))
        .stdout(predicate::str::contains("do-thing"));
}

/// coverage-command: derive-nfr-uncovered-from-uncovered-behavior
#[test]
fn derive_nfr_uncovered_from_uncovered_behavior() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_with_nfr_ref()).unwrap();
    fs::write(spec_dir.join("performance.nfr"), nfr_performance()).unwrap();

    // No test files referencing do-thing

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("performance#api-latency"))
        .stdout(predicate::str::contains("uncovered"));
}

// ═══════════════════════════════════════════════════════════════
// Benchmark NFR coverage (coverage-command.spec)
// ═══════════════════════════════════════════════════════════════

/// coverage-command: report-benchmark-nfr
#[test]
fn report_benchmark_nfr() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();
    fs::write(spec_dir.join("performance.nfr"), nfr_performance()).unwrap();

    fs::write(dir.path().join("a.test.ts"), "// @minter:unit do-thing\n").unwrap();
    fs::write(
        dir.path().join("bench.rs"),
        "// @minter:benchmark #performance#api-latency\n",
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("performance#api-latency"))
        .stdout(predicate::str::contains("benchmark"));
}

// ═══════════════════════════════════════════════════════════════
// JSON output (coverage-command.spec)
// ═══════════════════════════════════════════════════════════════

/// coverage-command: json-output
#[test]
fn json_output() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_two_behaviors()).unwrap();

    fs::write(dir.path().join("a.test.ts"), "// @minter:unit do-thing\n").unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .arg("--format")
        .arg("json")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("total_behaviors"))
        .stdout(predicate::str::contains("covered_behaviors"))
        .stdout(predicate::str::contains("coverage_percentage"))
        .stdout(predicate::str::contains("do-thing"))
        .stdout(predicate::str::contains("do-other"))
        .stdout(predicate::str::contains("uncovered"));
}

// ═══════════════════════════════════════════════════════════════
// Tag validation — error cases (coverage-command.spec)
// ═══════════════════════════════════════════════════════════════

/// coverage-command: reject-unknown-behavior-id
#[test]
fn reject_unknown_behavior_id() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    fs::write(
        dir.path().join("a.test.ts"),
        "// @minter:unit nonexistent-behavior\n",
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent-behavior"))
        .stderr(predicate::str::contains("unknown"));
}

/// coverage-command: reject-unknown-nfr-constraint
#[test]
fn reject_unknown_nfr_constraint() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();
    fs::write(spec_dir.join("performance.nfr"), nfr_performance()).unwrap();

    fs::write(
        dir.path().join("a.test.ts"),
        "// @minter:benchmark #performance#nonexistent\n",
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"))
        .stderr(predicate::str::contains("unknown"));
}

/// coverage-command: reject-missing-type
#[test]
fn reject_missing_type() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    fs::write(dir.path().join("a.test.ts"), "// @minter do-thing\n").unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("@minter"))
        .stderr(predicate::str::contains("type"));
}

/// coverage-command: reject-invalid-type
#[test]
fn reject_invalid_type() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    fs::write(
        dir.path().join("a.test.ts"),
        "// @minter:acceptance do-thing\n",
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("acceptance"))
        .stderr(predicate::str::contains("invalid"));
}

/// coverage-command: reject-behavior-in-benchmark
#[test]
fn reject_behavior_in_benchmark() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    fs::write(
        dir.path().join("a.test.ts"),
        "// @minter:benchmark do-thing\n",
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("do-thing"))
        .stderr(predicate::str::contains("benchmark"));
}

/// coverage-command: reject-nfr-in-behavioral-tag
#[test]
fn reject_nfr_in_behavioral_tag() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    fs::write(
        dir.path().join("a.test.ts"),
        "// @minter:unit #performance#api-latency\n",
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("performance"))
        .stderr(predicate::str::contains("benchmark"));
}

/// coverage-command: reject-nonexistent-spec-path
#[test]
fn reject_nonexistent_spec_path() {
    let dir = TempDir::new().unwrap();

    minter()
        .arg("coverage")
        .arg("nonexistent/")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

/// coverage-command: reject-no-specs-in-path
#[test]
fn reject_no_specs_in_path() {
    let dir = TempDir::new().unwrap();

    let empty_dir = dir.path().join("empty-dir");
    fs::create_dir(&empty_dir).unwrap();

    minter()
        .arg("coverage")
        .arg(&empty_dir)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("no spec files found"));
}

/// coverage-command: reject-nonexistent-scan-path
#[test]
fn reject_nonexistent_scan_path() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .arg("--scan")
        .arg("nonexistent/")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

/// coverage-command: reject-invalid-format
#[test]
fn reject_invalid_format() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .arg("--format")
        .arg("xml")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("xml"))
        .stderr(predicate::str::contains("invalid"));
}

/// coverage-command: report-tag-errors-with-location
#[test]
fn report_tag_errors_with_location() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    let tests_dir = dir.path().join("tests");
    fs::create_dir(&tests_dir).unwrap();
    fs::write(
        tests_dir.join("a.test.ts"),
        "// line 1\n// line 2\n// line 3\n// line 4\n// @minter:unit nonexistent-behavior\n",
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("a.test.ts"))
        .stderr(predicate::str::contains("5"));
}

// ═══════════════════════════════════════════════════════════════
// Edge cases (coverage-command.spec)
// ═══════════════════════════════════════════════════════════════

/// coverage-command: warn-empty-tag
#[test]
fn warn_empty_tag() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    fs::write(dir.path().join("a.test.ts"), "// @minter:unit do-thing\n").unwrap();
    fs::write(dir.path().join("b.test.ts"), "// @minter:e2e\n").unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("empty"))
        .stdout(predicate::str::contains("do-thing"))
        .stdout(predicate::str::contains("1/1"));
}

/// coverage-command: info-duplicate-coverage
#[test]
fn info_duplicate_coverage() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    let tests_dir = dir.path().join("tests");
    fs::create_dir(&tests_dir).unwrap();
    fs::write(tests_dir.join("a.test.ts"), "// @minter:unit do-thing\n").unwrap();
    fs::write(tests_dir.join("b.test.ts"), "// @minter:unit do-thing\n").unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("do-thing"))
        .stdout(predicate::str::contains("unit"))
        .stdout(predicate::str::contains("duplicate"));
}

/// coverage-command: no-tags-found
#[test]
fn no_tags_found() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_two_behaviors()).unwrap();

    // No test files at all (or files without tags)
    fs::write(dir.path().join("readme.txt"), "no tags here\n").unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("do-thing"))
        .stdout(predicate::str::contains("do-other"))
        .stdout(predicate::str::contains("uncovered"))
        .stdout(predicate::str::contains("0/2"));
}

/// coverage-command: disambiguate-with-qualified-name
#[test]
fn disambiguate_with_qualified_name() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "handle-error"),
    )
    .unwrap();
    fs::write(
        spec_dir.join("b.spec"),
        spec_one_behavior("b", "1.0.0", "handle-error"),
    )
    .unwrap();

    fs::write(
        dir.path().join("a.test.ts"),
        "// @minter:unit a/handle-error\n",
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("a"))
        .stdout(predicate::str::contains("handle-error"))
        .stdout(predicate::str::contains("unit"));
}

/// coverage-command: reject-ambiguous-unqualified-name
#[test]
fn reject_ambiguous_unqualified_name() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "handle-error"),
    )
    .unwrap();
    fs::write(
        spec_dir.join("b.spec"),
        spec_one_behavior("b", "1.0.0", "handle-error"),
    )
    .unwrap();

    fs::write(
        dir.path().join("a.test.ts"),
        "// @minter:unit handle-error\n",
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("handle-error"))
        .stderr(predicate::str::contains("ambiguous"));
}

/// coverage-command: report-all-tag-errors
#[test]
fn report_all_tag_errors() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    fs::write(
        dir.path().join("a.test.ts"),
        "// line 1\n// line 2\n// @minter:unit nonexistent-one\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("b.test.ts"),
        "// line 1\n// line 2\n// line 3\n// line 4\n// line 5\n// line 6\n// @minter:unit nonexistent-two\n",
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent-one"))
        .stderr(predicate::str::contains("nonexistent-two"))
        .stderr(predicate::str::contains("a.test.ts"))
        .stderr(predicate::str::contains("b.test.ts"));
}

/// coverage-command: json-errors
#[test]
fn json_errors() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    fs::write(
        dir.path().join("a.test.ts"),
        "// @minter:unit nonexistent-behavior\n",
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .arg("--format")
        .arg("json")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("errors"))
        .stdout(predicate::str::contains("nonexistent-behavior"));
}

/// coverage-command: mixed-valid-and-invalid-tags
#[test]
fn mixed_valid_and_invalid_tags() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_two_behaviors()).unwrap();

    fs::write(dir.path().join("a.test.ts"), "// @minter:unit do-thing\n").unwrap();
    fs::write(
        dir.path().join("b.test.ts"),
        "// @minter:unit nonexistent-behavior\n",
    )
    .unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent-behavior"));
}
