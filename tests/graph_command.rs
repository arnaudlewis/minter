mod common;

use std::fs;

use common::{minter, temp_dir_with_specs};
use predicates::prelude::*;
use tempfile::TempDir;

// ═══════════════════════════════════════════════════════════════
// Happy paths (graph-command.spec)
// ═══════════════════════════════════════════════════════════════

fn spec_with_dep(name: &str, dep_name: &str) -> String {
    format!(
        "\
spec {name} v1.0.0
title \"{name}\"

description
  Spec {name}.

motivation
  Testing.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"

depends on {dep_name} >= 1.0.0
"
    )
}

fn spec_no_deps(name: &str) -> String {
    format!(
        "\
spec {name} v1.0.0
title \"{name}\"

description
  Spec {name}.

motivation
  Testing.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
"
    )
}

/// graph-command.spec: display-summary-header
#[test]
fn display_summary_header() {
    let spec_a = "\
spec spec-a v1.0.0
title \"spec-a\"

description
  Spec spec-a.

motivation
  Testing.

nfr
  performance#api-response-time

behavior first-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"

behavior second-thing [happy_path]
  \"Does another thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
";
    let spec_b = "\
spec spec-b v1.0.0
title \"spec-b\"

description
  Spec spec-b.

motivation
  Testing.

nfr
  reliability

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
";
    let perf_nfr = "\
nfr performance v1.0.0
title \"Performance\"

description
  Perf constraints.

motivation
  Speed.


constraint api-response-time [metric]
  \"API response time\"

  metric \"HTTP response time, p95\"
  threshold < 1s

  verification
    environment staging
    benchmark \"100 requests\"
    pass \"p95 < threshold\"

  violation high
  overridable yes


constraint throughput [metric]
  \"Throughput\"

  metric \"requests per second\"
  threshold >= 1000

  verification
    environment staging
    benchmark \"sustained load\"
    pass \"rps >= threshold\"

  violation high
  overridable yes
";
    let rel_nfr = "\
nfr reliability v1.0.0
title \"Reliability\"

description
  Reliability constraints.

motivation
  Uptime.


constraint uptime-target [metric]
  \"System uptime\"

  metric \"uptime percentage\"
  threshold >= 99.9%

  verification
    environment production
    benchmark \"30-day rolling window\"
    pass \"uptime >= threshold\"

  violation critical
  overridable no
";

    // Unreferenced NFR — should NOT appear in summary counts
    let sec_nfr = "\
nfr security v1.0.0
title \"Security\"

description
  Security constraints.

motivation
  Safety.


constraint input-validation [metric]
  \"Input validation\"

  metric \"rejected inputs\"
  threshold < 1%

  verification
    environment staging
    benchmark \"fuzz test\"
    pass \"rejection rate < threshold\"

  violation critical
  overridable no
";

    let dir = TempDir::new().expect("create temp dir");
    fs::write(dir.path().join("spec-a.spec"), spec_a).unwrap();
    fs::write(dir.path().join("spec-b.spec"), spec_b).unwrap();
    fs::write(dir.path().join("performance.nfr"), perf_nfr).unwrap();
    fs::write(dir.path().join("reliability.nfr"), rel_nfr).unwrap();
    fs::write(dir.path().join("security.nfr"), sec_nfr).unwrap();

    let output = minter()
        .env("NO_COLOR", "1")
        .arg("graph")
        .arg(dir.path())
        .output()
        .expect("run graph");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first_line = stdout.lines().next().unwrap();
    // spec-a refs performance#api-response-time (1 anchor = 1 nfr), spec-b refs reliability whole-file (1 constraint = 1 nfr)
    // security.nfr is unreferenced and excluded from counts
    assert_eq!(
        first_line, "2 specs, 3 behaviors, 2 nfr categories, 2 nfrs",
        "summary should only count referenced nfr categories and nfrs, got:\n{}",
        stdout,
    );
}

/// graph-command.spec: display-full-graph
#[test]
fn display_full_graph() {
    let spec_a = spec_with_dep("spec-a", "spec-b");
    let spec_b = spec_with_dep("spec-b", "spec-c");
    let spec_c = spec_no_deps("spec-c");

    let (_dir, dir_path) = temp_dir_with_specs(&[
        ("spec-a", &spec_a),
        ("spec-b", &spec_b),
        ("spec-c", &spec_c),
    ]);

    let output = minter()
        .env("NO_COLOR", "1")
        .arg("graph")
        .arg(&dir_path)
        .output()
        .expect("run graph");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // spec-a is the root (nothing depends on it)
    assert!(stdout.contains("spec-a v1.0.0"));
    // spec-b and spec-c are nested via tree connectors
    assert!(
        stdout.contains("\u{2514}\u{2500}\u{2500} spec-b")
            || stdout.contains("\u{251c}\u{2500}\u{2500} spec-b")
    );
    assert!(
        stdout.contains("\u{2514}\u{2500}\u{2500} spec-c")
            || stdout.contains("\u{251c}\u{2500}\u{2500} spec-c")
    );
}

/// graph-command.spec: display-impacted-specs
#[test]
fn display_impacted_specs() {
    let spec_a = spec_with_dep("spec-a", "spec-b");
    let spec_b = spec_no_deps("spec-b");
    let spec_c = spec_with_dep("spec-c", "spec-b");

    let (_dir, dir_path) = temp_dir_with_specs(&[
        ("spec-a", &spec_a),
        ("spec-b", &spec_b),
        ("spec-c", &spec_c),
    ]);

    let output = minter()
        .env("NO_COLOR", "1")
        .arg("graph")
        .arg(&dir_path)
        .arg("--impacted")
        .arg("spec-b")
        .output()
        .expect("run graph");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("spec-b"), "header should mention target");
    assert!(stdout.contains("spec-a"), "should list spec-a as impacted");
    assert!(stdout.contains("spec-c"), "should list spec-c as impacted");
    // Impacted specs shown with tree connectors
    assert!(
        stdout.contains("\u{2514}\u{2500}\u{2500}") || stdout.contains("\u{251c}\u{2500}\u{2500}")
    );
}

/// graph-command.spec: display-transitive-impacted
#[test]
fn display_transitive_impacted() {
    let spec_a = spec_with_dep("spec-a", "spec-b");
    let spec_b = spec_with_dep("spec-b", "spec-c");
    let spec_c = spec_no_deps("spec-c");

    let (_dir, dir_path) = temp_dir_with_specs(&[
        ("spec-a", &spec_a),
        ("spec-b", &spec_b),
        ("spec-c", &spec_c),
    ]);

    let output = minter()
        .env("NO_COLOR", "1")
        .arg("graph")
        .arg(&dir_path)
        .arg("--impacted")
        .arg("spec-c")
        .output()
        .expect("run graph");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("spec-c"), "header should mention target");
    assert!(stdout.contains("spec-b"), "direct dep should be impacted");
    assert!(
        stdout.contains("spec-a"),
        "transitive dep should be impacted"
    );
}

/// graph-command.spec: graph-persists-cache
#[test]
fn graph_persists_cache() {
    let spec_a = spec_with_dep("spec-a", "spec-b");
    let spec_b = spec_no_deps("spec-b");

    let (dir, dir_path) = temp_dir_with_specs(&[("spec-a", &spec_a), ("spec-b", &spec_b)]);

    // No .minter/graph.json should exist beforehand
    let graph_path = dir.path().join(".minter").join("graph.json");
    assert!(
        !graph_path.exists(),
        "graph.json should not exist before graph command"
    );

    minter()
        .current_dir(dir.path())
        .env("NO_COLOR", "1")
        .arg("graph")
        .arg(&dir_path)
        .assert()
        .success();

    // After running graph, .minter/graph.json should be created and contain spec entries
    assert!(
        graph_path.exists(),
        "graph.json should be created by graph command"
    );
    let content = std::fs::read_to_string(&graph_path).expect("read graph.json");
    assert!(
        content.contains("spec-a"),
        "graph.json should contain spec-a"
    );
    assert!(
        content.contains("spec-b"),
        "graph.json should contain spec-b"
    );
}

// ═══════════════════════════════════════════════════════════════
// Error cases (graph-command.spec)
// ═══════════════════════════════════════════════════════════════

/// graph-command.spec: impacted-unknown-spec
#[test]
fn impacted_unknown_spec() {
    let spec_a = spec_no_deps("spec-a");
    let (_dir, dir_path) = temp_dir_with_specs(&[("spec-a", &spec_a)]);

    minter()
        .arg("graph")
        .arg(&dir_path)
        .arg("--impacted")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"))
        .stderr(predicate::str::contains("not found"));
}

/// graph-command.spec: graph-no-specs
#[test]
fn graph_no_specs() {
    let dir = TempDir::new().expect("create temp dir");
    minter()
        .arg("graph")
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("no spec files"));
}

/// graph-command.spec: graph-no-dependencies
#[test]
fn graph_no_dependencies() {
    let spec_a = spec_no_deps("spec-a");
    let spec_b = spec_no_deps("spec-b");

    let (_dir, dir_path) = temp_dir_with_specs(&[("spec-a", &spec_a), ("spec-b", &spec_b)]);

    let output = minter()
        .env("NO_COLOR", "1")
        .arg("graph")
        .arg(&dir_path)
        .output()
        .expect("run graph");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("spec-a"));
    assert!(stdout.contains("spec-b"));
    // No tree connectors when there are no dependencies
    assert!(!stdout.contains("\u{2514}\u{2500}\u{2500}"));
    assert!(!stdout.contains("\u{251c}\u{2500}\u{2500}"));
}

// ═══════════════════════════════════════════════════════════════
// NFR graph display (graph-command.spec: NFR references)
// ═══════════════════════════════════════════════════════════════

fn spec_with_nfr(name: &str, categories: &[&str]) -> String {
    let nfr_lines: String = categories
        .iter()
        .map(|c| format!("  {}", c))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "\
spec {name} v1.0.0
title \"{name}\"

description
  Spec {name}.

motivation
  Testing.

nfr
{nfr_lines}

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
"
    )
}

fn spec_with_dep_and_nfr(name: &str, dep_name: &str, categories: &[&str]) -> String {
    let nfr_lines: String = categories
        .iter()
        .map(|c| format!("  {}", c))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "\
spec {name} v1.0.0
title \"{name}\"

description
  Spec {name}.

motivation
  Testing.

nfr
{nfr_lines}

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"

depends on {dep_name} >= 1.0.0
"
    )
}

fn nfr_file(category: &str, version: &str) -> String {
    format!(
        "\
nfr {category} v{version}
title \"{category} Requirements\"

description
  Defines {category} constraints.

motivation
  {category} matters.


constraint api-response-time [metric]
  \"API endpoints must respond within acceptable latency bounds\"

  metric \"HTTP response time, p95\"
  threshold < 1s

  verification
    environment staging, production
    benchmark \"100 concurrent requests per endpoint\"
    pass \"p95 < threshold\"

  violation high
  overridable yes
"
    )
}

/// graph-command.spec: display-nfr-refs-in-tree
#[test]
fn display_nfr_refs_in_tree() {
    // spec-a references two anchors in performance + whole-file reliability
    let spec_a = spec_with_nfr(
        "spec-a",
        &[
            "performance#api-response-time",
            "performance#cache-hit-ratio",
            "reliability",
        ],
    );
    let perf_nfr = "\
nfr performance v1.1.0
title \"Performance Requirements\"

description
  Defines performance constraints.

motivation
  Performance matters.


constraint api-response-time [metric]
  \"API endpoints must respond within acceptable latency bounds\"

  metric \"HTTP response time, p95\"
  threshold < 1s

  verification
    environment staging, production
    benchmark \"100 concurrent requests per endpoint\"
    pass \"p95 < threshold\"

  violation high
  overridable yes


constraint cache-hit-ratio [metric]
  \"Cache must maintain acceptable hit ratio\"

  metric \"cache hit ratio\"
  threshold >= 90%

  verification
    environment staging, production
    benchmark \"1000 lookups per test run\"
    pass \"hit ratio >= threshold\"

  violation medium
  overridable yes
";
    let rel_nfr = "\
nfr reliability v1.0.0
title \"Reliability Requirements\"

description
  Defines reliability constraints.

motivation
  Reliability matters.


constraint graceful-recovery [rule]
  \"System must recover gracefully from failures\"

  rule \"On crash, restart within SLA bounds\"

  verification
    runtime \"Process supervisor heartbeat check\"

  violation critical
  overridable no
";

    let dir = TempDir::new().expect("create temp dir");
    fs::write(dir.path().join("spec-a.spec"), &spec_a).unwrap();
    fs::write(dir.path().join("performance.nfr"), perf_nfr).unwrap();
    fs::write(dir.path().join("reliability.nfr"), rel_nfr).unwrap();

    let output = minter()
        .env("NO_COLOR", "1")
        .arg("graph")
        .arg(dir.path())
        .output()
        .expect("run graph");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Performance should be a parent node with anchor sub-items
    assert!(
        stdout.contains("[nfr] performance"),
        "should show [nfr] performance tag"
    );
    assert!(
        stdout.contains("#api-response-time"),
        "should show anchor #api-response-time"
    );
    assert!(
        stdout.contains("#cache-hit-ratio"),
        "should show anchor #cache-hit-ratio"
    );

    // Reliability is a whole-file ref — leaf node, no anchor sub-items
    assert!(
        stdout.contains("[nfr] reliability"),
        "should show [nfr] reliability tag"
    );

    // NFR categories sorted: performance before reliability
    let perf_pos = stdout.find("[nfr] performance").unwrap();
    let rel_pos = stdout.find("[nfr] reliability").unwrap();
    assert!(
        perf_pos < rel_pos,
        "performance should appear before reliability (sorted)"
    );

    // Anchors should use tree connectors (sub-items of the [nfr] node)
    // Find the line with #api-response-time — it should have a tree connector
    for line in stdout.lines() {
        if line.contains("#api-response-time") || line.contains("#cache-hit-ratio") {
            assert!(
                line.contains("\u{2514}\u{2500}\u{2500}")
                    || line.contains("\u{251c}\u{2500}\u{2500}"),
                "anchor sub-items should use tree connectors, got: {}",
                line,
            );
        }
    }

    // With color enabled: anchors must NOT be dimmed, and "#" must be magenta
    let color_output = minter()
        .env_remove("NO_COLOR")
        .arg("graph")
        .arg(dir.path())
        .output()
        .expect("run graph with color");
    assert!(color_output.status.success());
    let color_stdout = String::from_utf8(color_output.stdout).unwrap();
    for line in color_stdout.lines() {
        if line.contains("#api-response-time") || line.contains("#cache-hit-ratio") {
            assert!(
                !line.contains("\x1b[2m"),
                "anchor sub-items must not be dimmed, got: {:?}",
                line,
            );
            assert!(
                line.contains("\x1b[35m#\x1b[0m"),
                "only '#' must be magenta (\\x1b[35m#\\x1b[0m), got: {:?}",
                line,
            );
        }
    }
}

/// graph-command.spec: display-nfr-refs-hidden-when-spec-dimmed
#[test]
fn display_nfr_refs_hidden_when_spec_dimmed() {
    let spec_a = spec_with_dep_and_nfr("spec-a", "spec-b", &[]);
    let spec_c = spec_with_dep("spec-c", "spec-b");
    let spec_b = spec_with_nfr("spec-b", &["performance#api-response-time"]);
    let perf_nfr = nfr_file("performance", "1.0.0");

    let dir = TempDir::new().expect("create temp dir");
    fs::write(dir.path().join("spec-a.spec"), &spec_a).unwrap();
    fs::write(dir.path().join("spec-b.spec"), &spec_b).unwrap();
    fs::write(dir.path().join("spec-c.spec"), &spec_c).unwrap();
    fs::write(dir.path().join("performance.nfr"), &perf_nfr).unwrap();

    let output = minter()
        .env("NO_COLOR", "1")
        .arg("graph")
        .arg(dir.path())
        .output()
        .expect("run graph");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // [nfr] performance should appear exactly once (under expanded spec-b)
    let nfr_count = stdout.matches("[nfr] performance").count();
    assert_eq!(
        nfr_count, 1,
        "NFR ref should appear once (only under expanded spec-b), found {} times in:\n{}",
        nfr_count, stdout
    );

    // #api-response-time anchor should also appear exactly once (not under dimmed spec-b)
    let anchor_count = stdout.matches("#api-response-time").count();
    assert_eq!(
        anchor_count, 1,
        "anchor should appear once (only under expanded spec-b), found {} times in:\n{}",
        anchor_count, stdout
    );
}

/// graph-command.spec: display-nfr-anchors-show-referenced-count
#[test]
fn display_nfr_anchors_show_referenced_count() {
    // NFR file with 4 constraints
    let perf_nfr = "\
nfr performance v1.0.0
title \"Performance Requirements\"

description
  Defines performance constraints.

motivation
  Performance matters.


constraint api-response-time [metric]
  \"API response time\"

  metric \"HTTP response time, p95\"
  threshold < 1s

  verification
    environment staging
    benchmark \"100 requests\"
    pass \"p95 < threshold\"

  violation high
  overridable yes


constraint cache-hit-ratio [metric]
  \"Cache hit ratio\"

  metric \"cache hit ratio\"
  threshold >= 90%

  verification
    environment staging
    benchmark \"1000 lookups\"
    pass \"hit ratio >= threshold\"

  violation medium
  overridable yes


constraint throughput [metric]
  \"System throughput\"

  metric \"requests per second\"
  threshold >= 1000

  verification
    environment staging
    benchmark \"sustained load test\"
    pass \"rps >= threshold\"

  violation high
  overridable yes


constraint error-rate [metric]
  \"Error rate\"

  metric \"error percentage\"
  threshold < 1%

  verification
    environment staging
    benchmark \"10000 requests\"
    pass \"error rate < threshold\"

  violation critical
  overridable yes
";
    // Spec references only 2 of the 4 anchors
    let spec_a = spec_with_nfr(
        "spec-a",
        &[
            "performance#api-response-time",
            "performance#cache-hit-ratio",
        ],
    );

    let dir = TempDir::new().expect("create temp dir");
    fs::write(dir.path().join("spec-a.spec"), &spec_a).unwrap();
    fs::write(dir.path().join("performance.nfr"), perf_nfr).unwrap();

    let output = minter()
        .env("NO_COLOR", "1")
        .arg("graph")
        .arg(dir.path())
        .output()
        .expect("run graph");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should show (2 constraints) — only the referenced anchors, not all 4
    assert!(
        stdout.contains("(2 constraints)"),
        "should show 2 constraints (referenced anchors only), got:\n{}",
        stdout,
    );
    assert!(
        !stdout.contains("(4 constraints)"),
        "should NOT show 4 constraints (total file count), got:\n{}",
        stdout,
    );
    assert!(
        stdout.contains("#api-response-time"),
        "should show anchor sub-item"
    );
    assert!(
        stdout.contains("#cache-hit-ratio"),
        "should show anchor sub-item"
    );
}

/// graph-command.spec: display-nfr-whole-file-absorbs-anchors
#[test]
fn display_nfr_whole_file_absorbs_anchors() {
    // NFR file with 3 constraints
    let perf_nfr = "\
nfr performance v1.0.0
title \"Performance Requirements\"

description
  Defines performance constraints.

motivation
  Performance matters.


constraint api-response-time [metric]
  \"API response time\"

  metric \"HTTP response time, p95\"
  threshold < 1s

  verification
    environment staging
    benchmark \"100 requests\"
    pass \"p95 < threshold\"

  violation high
  overridable yes


constraint cache-hit-ratio [metric]
  \"Cache hit ratio\"

  metric \"cache hit ratio\"
  threshold >= 90%

  verification
    environment staging
    benchmark \"1000 lookups\"
    pass \"hit ratio >= threshold\"

  violation medium
  overridable yes


constraint throughput [metric]
  \"System throughput\"

  metric \"requests per second\"
  threshold >= 1000

  verification
    environment staging
    benchmark \"sustained load test\"
    pass \"rps >= threshold\"

  violation high
  overridable yes
";
    // Spec has whole-file ref at spec level + behavior-level anchor
    let spec_a = "\
spec spec-a v1.0.0
title \"spec-a\"

description
  Spec spec-a.

motivation
  Testing.

nfr
  performance

behavior do-thing [happy_path]
  \"Does a thing\"

  nfr
    performance#api-response-time

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
";

    let dir = TempDir::new().expect("create temp dir");
    fs::write(dir.path().join("spec-a.spec"), spec_a).unwrap();
    fs::write(dir.path().join("performance.nfr"), perf_nfr).unwrap();

    let output = minter()
        .env("NO_COLOR", "1")
        .arg("graph")
        .arg(dir.path())
        .output()
        .expect("run graph");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Whole-file absorbs anchors: show full count (3 constraints)
    assert!(
        stdout.contains("(3 constraints)"),
        "should show 3 constraints (whole-file = full count), got:\n{}",
        stdout,
    );
    // Anchor sub-items should NOT appear (absorbed by whole-file)
    assert!(
        !stdout.contains("#api-response-time"),
        "should NOT show anchor sub-items when whole-file absorbs, got:\n{}",
        stdout,
    );
}

/// graph-command.spec: impacted-by-nfr
#[test]
fn impacted_by_nfr() {
    let spec_a = spec_with_nfr("spec-a", &["performance#api-response-time"]);
    let spec_b = spec_with_nfr("spec-b", &["performance#api-response-time"]);
    let spec_c = spec_no_deps("spec-c");
    let perf_nfr = nfr_file("performance", "1.0.0");

    let dir = TempDir::new().expect("create temp dir");
    fs::write(dir.path().join("spec-a.spec"), &spec_a).unwrap();
    fs::write(dir.path().join("spec-b.spec"), &spec_b).unwrap();
    fs::write(dir.path().join("spec-c.spec"), &spec_c).unwrap();
    fs::write(dir.path().join("performance.nfr"), &perf_nfr).unwrap();

    let output = minter()
        .env("NO_COLOR", "1")
        .arg("graph")
        .arg(dir.path())
        .arg("--impacted")
        .arg("performance")
        .output()
        .expect("run graph");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("spec-a"), "spec-a references performance");
    assert!(stdout.contains("spec-b"), "spec-b references performance");
    assert!(
        !stdout.contains("spec-c"),
        "spec-c does not reference performance"
    );
    assert!(stdout.contains("[nfr]"), "header should show [nfr] tag");
}

/// graph-command: single-file-resolves-nfr-from-sibling-dirs
#[test]
fn single_file_resolves_nfr_from_sibling_dir() {
    let spec_a = spec_with_nfr("spec-a", &["performance#api-response-time"]);
    let perf_nfr = nfr_file("performance", "1.0.0");

    let dir = TempDir::new().expect("create temp dir");
    // Place spec and nfr in separate subdirectories (sibling dirs)
    let spec_dir = dir.path().join("features");
    let nfr_dir = dir.path().join("nfr");
    fs::create_dir_all(&spec_dir).unwrap();
    fs::create_dir_all(&nfr_dir).unwrap();
    fs::write(spec_dir.join("spec-a.spec"), &spec_a).unwrap();
    fs::write(nfr_dir.join("performance.nfr"), &perf_nfr).unwrap();

    // Run graph on the single spec file (not the directory)
    let output = minter()
        .env("NO_COLOR", "1")
        .arg("graph")
        .arg(spec_dir.join("spec-a.spec"))
        .output()
        .expect("run graph on single file");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // NFR should be resolved: version and constraint count visible
    assert!(
        stdout.contains("1 nfr"),
        "summary should count 1 nfr, got:\n{}",
        stdout,
    );
    assert!(
        stdout.contains("[nfr] performance v1.0.0"),
        "should resolve NFR with version, got:\n{}",
        stdout,
    );
    assert!(
        stdout.contains("#api-response-time"),
        "should show anchor sub-item, got:\n{}",
        stdout,
    );
}
