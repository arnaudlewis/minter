mod common;

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
    assert!(stdout.contains("\u{2514}\u{2500}\u{2500} spec-b") || stdout.contains("\u{251c}\u{2500}\u{2500} spec-b"));
    assert!(stdout.contains("\u{2514}\u{2500}\u{2500} spec-c") || stdout.contains("\u{251c}\u{2500}\u{2500} spec-c"));
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
    assert!(stdout.contains("\u{2514}\u{2500}\u{2500}") || stdout.contains("\u{251c}\u{2500}\u{2500}"));
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
    assert!(stdout.contains("spec-a"), "transitive dep should be impacted");
}

/// graph-command.spec: graph-persists-cache
#[test]
fn graph_persists_cache() {
    let spec_a = spec_with_dep("spec-a", "spec-b");
    let spec_b = spec_no_deps("spec-b");

    let (dir, dir_path) = temp_dir_with_specs(&[
        ("spec-a", &spec_a),
        ("spec-b", &spec_b),
    ]);

    // No .minter/graph.json should exist beforehand
    let graph_path = dir.path().join(".minter").join("graph.json");
    assert!(!graph_path.exists(), "graph.json should not exist before graph command");

    minter()
        .current_dir(dir.path())
        .env("NO_COLOR", "1")
        .arg("graph")
        .arg(&dir_path)
        .assert()
        .success();

    // After running graph, .minter/graph.json should be created and contain spec entries
    assert!(graph_path.exists(), "graph.json should be created by graph command");
    let content = std::fs::read_to_string(&graph_path).expect("read graph.json");
    assert!(content.contains("spec-a"), "graph.json should contain spec-a");
    assert!(content.contains("spec-b"), "graph.json should contain spec-b");
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

    let (_dir, dir_path) = temp_dir_with_specs(&[
        ("spec-a", &spec_a),
        ("spec-b", &spec_b),
    ]);

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
