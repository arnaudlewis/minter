mod common;

use std::fs;

use common::{read_graph_json, minter, temp_dir_with_specs};
use predicates::prelude::*;

/// Helper: a valid spec with a given name, version, and optional dependency.
fn valid_spec(name: &str, version: &str, dep: Option<(&str, &str)>) -> String {
    let dep_line = match dep {
        Some((dep_name, dep_ver)) => format!("\ndepends on {} >= {}\n", dep_name, dep_ver),
        None => String::new(),
    };
    format!(
        "\
spec {name} v{version}
title \"{name}\"

description
  A spec for testing.

motivation
  Testing incremental validation.

behavior do-thing [happy_path]
  \"Do the thing\"

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
{dep_line}"
    )
}

// ═══════════════════════════════════════════════════════════════
// incremental-validation.spec behaviors
// ═══════════════════════════════════════════════════════════════

/// incremental-validation.spec: detect-changed-file
/// Modify b.spec → only b and its dependents revalidated, graph updated.
#[test]
fn detect_changed_file() {
    let (dir, dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", Some(("b", "1.0.0")))),
        ("b", &valid_spec("b", "1.0.0", None)),
        ("c", &valid_spec("c", "1.0.0", None)),
    ]);

    // First run — build the graph
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deps")
        .arg(&dir_path)
        .assert()
        .success();

    let graph1 = read_graph_json(dir.path());
    let hash_b_1 = graph1["specs"]["b"]["content_hash"]
        .as_str()
        .unwrap()
        .to_string();
    let hash_c_1 = graph1["specs"]["c"]["content_hash"]
        .as_str()
        .unwrap()
        .to_string();

    // Modify b.spec
    fs::write(
        dir.path().join("b.spec"),
        valid_spec("b", "1.1.0", None), // changed version
    )
    .unwrap();

    // Second run — should detect change and update graph
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deps")
        .arg(&dir_path)
        .assert()
        .success();

    let graph2 = read_graph_json(dir.path());
    let hash_b_2 = graph2["specs"]["b"]["content_hash"].as_str().unwrap();
    let hash_c_2 = graph2["specs"]["c"]["content_hash"].as_str().unwrap();

    // b's hash should change
    assert_ne!(hash_b_1, hash_b_2, "b's hash should change after modification");
    // c's hash should not change (not affected)
    assert_eq!(hash_c_1, hash_c_2, "c's hash should not change (unaffected)");
}

/// incremental-validation.spec: integrate-new-spec-file
/// Add d.spec + dep → d added to graph, exit 0.
#[test]
fn integrate_new_spec_file() {
    let (dir, dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", None)),
    ]);

    // First run — build initial graph
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deps")
        .arg(&dir_path)
        .assert()
        .success();

    let graph1 = read_graph_json(dir.path());
    assert!(graph1["specs"]["d"].is_null(), "d should not be in initial graph");

    // Add d.spec and update a to depend on it
    fs::write(dir.path().join("d.spec"), valid_spec("d", "1.0.0", None)).unwrap();
    fs::write(
        dir.path().join("a.spec"),
        valid_spec("a", "1.0.0", Some(("d", "1.0.0"))),
    )
    .unwrap();

    // Second run
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deps")
        .arg(&dir_path)
        .assert()
        .success();

    let graph2 = read_graph_json(dir.path());
    assert!(
        graph2["specs"]["d"].is_object(),
        "d should be added to graph after creation"
    );
}

/// incremental-validation.spec: reject-broken-deps-after-deletion
/// Delete b.spec → stderr mentions missing, exit 1, b removed from graph.
#[test]
fn reject_broken_deps_after_deletion() {
    let (dir, dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", Some(("b", "1.0.0")))),
        ("b", &valid_spec("b", "1.0.0", None)),
    ]);

    // First run — build graph
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deps")
        .arg(&dir_path)
        .assert()
        .success();

    let graph1 = read_graph_json(dir.path());
    assert!(graph1["specs"]["b"].is_object(), "b should be in initial graph");

    // Delete b.spec
    fs::remove_file(dir.path().join("b.spec")).unwrap();

    // Second run — should fail and report missing dep
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deps")
        .arg(&dir_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("b"));

    // b should be removed from graph
    let graph2 = read_graph_json(dir.path());
    assert!(
        graph2["specs"]["b"].is_null(),
        "b should be removed from graph after deletion"
    );
}

/// incremental-validation.spec: rebuild-when-files-moved
/// Stale paths → graph rebuilt from current dir.
#[test]
fn rebuild_when_files_moved() {
    let (dir, dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", None)),
    ]);

    // First run — build graph
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deps")
        .arg(&dir_path)
        .assert()
        .success();

    // Manually inject a stale entry into the graph
    let graph_path = dir.path().join(".minter").join("graph.json");
    let mut graph: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&graph_path).unwrap()).unwrap();
    // Add a fake stale entry that references a file that doesn't exist
    graph["specs"]["stale-spec"] = serde_json::json!({
        "content_hash": "deadbeef",
        "version": "1.0.0",
        "behavior_count": 1,
        "valid": true,
        "dependencies": []
    });
    fs::write(&graph_path, serde_json::to_string_pretty(&graph).unwrap()).unwrap();

    // Run again — should rebuild and remove stale entry
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deps")
        .arg(&dir_path)
        .assert()
        .success();

    let graph2 = read_graph_json(dir.path());
    assert!(
        graph2["specs"]["stale-spec"].is_null(),
        "stale entries should be removed from graph"
    );
    assert!(
        graph2["specs"]["a"].is_object(),
        "current specs should still be in graph"
    );
}
