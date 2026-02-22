mod common;

use std::fs;

use common::{read_graph_json, specval, temp_dir_with_specs, write_graph_json};
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
  Testing graph cache.

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
// graph-cache.spec behaviors
// ═══════════════════════════════════════════════════════════════

/// graph-cache.spec: specval-directory-at-cwd
/// .specval/ is created at CWD (project root), not inside the specs directory.
#[test]
fn specval_directory_at_cwd() {
    // Create a project structure: project_root/specs/a.spec
    let project_root = tempfile::TempDir::new().unwrap();
    let specs_dir = project_root.path().join("specs");
    std::fs::create_dir(&specs_dir).unwrap();
    std::fs::write(
        specs_dir.join("a.spec"),
        valid_spec("a", "1.0.0", None),
    )
    .unwrap();

    // Run from project_root, pointing to specs/ subdirectory
    specval()
        .current_dir(project_root.path())
        .arg("validate")
        .arg("--deps")
        .arg("specs")
        .assert()
        .success();

    // .specval should be at project_root, NOT inside specs/
    assert!(
        project_root.path().join(".specval").join("graph.json").exists(),
        ".specval/graph.json should be at the project root (CWD)"
    );
    assert!(
        !specs_dir.join(".specval").exists(),
        ".specval should NOT be created inside the specs directory"
    );
}

/// graph-cache.spec: build-graph-cold-start
/// .specval/graph.json created on first --deps run, contains spec names + hashes + edges, exit 0.
#[test]
fn build_graph_cold_start() {
    let (dir, dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", Some(("b", "1.0.0")))),
        ("b", &valid_spec("b", "1.0.0", None)),
    ]);

    specval()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deps")
        .arg(&dir_path)
        .assert()
        .success();

    // .specval/graph.json should exist at CWD
    let graph = read_graph_json(dir.path());

    // Should contain spec entries
    let specs = graph.get("specs").expect("graph.json should have 'specs' key");
    assert!(specs.get("a").is_some(), "graph should contain spec 'a'");
    assert!(specs.get("b").is_some(), "graph should contain spec 'b'");

    // Each entry should have a content_hash
    let a_entry = &specs["a"];
    assert!(
        a_entry.get("content_hash").is_some(),
        "spec entry should have content_hash"
    );

    // Should have dependency edges
    assert!(
        a_entry.get("dependencies").is_some(),
        "spec entry should have dependencies"
    );
}

/// graph-cache.spec: empty-specval-directory
/// .specval/ dir created when absent.
#[test]
fn create_specval_directory() {
    let (dir, dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", None)),
    ]);

    // Ensure no .specval dir exists
    assert!(!dir.path().join(".specval").exists());

    specval()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deps")
        .arg(&dir_path)
        .assert()
        .success();

    assert!(
        dir.path().join(".specval").exists(),
        ".specval directory should be created"
    );
    assert!(
        dir.path().join(".specval").join("graph.json").exists(),
        ".specval/graph.json should be created"
    );
}

/// graph-cache.spec: load-cached-graph
/// Second run doesn't rewrite graph.json (check mtime), exit 0.
#[test]
fn load_cached_graph() {
    let (dir, dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", None)),
    ]);

    // First run — creates graph.json
    specval()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deps")
        .arg(&dir_path)
        .assert()
        .success();

    let graph_path = dir.path().join(".specval").join("graph.json");
    let mtime1 = fs::metadata(&graph_path)
        .expect("graph.json should exist")
        .modified()
        .expect("should have mtime");

    // Small delay to ensure mtime would differ if file were rewritten
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Second run — should not rewrite graph.json
    specval()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deps")
        .arg(&dir_path)
        .assert()
        .success();

    let mtime2 = fs::metadata(&graph_path)
        .expect("graph.json should still exist")
        .modified()
        .expect("should have mtime");

    assert_eq!(
        mtime1, mtime2,
        "graph.json should not be rewritten when nothing changed"
    );
}

/// graph-cache.spec: write-updated-graph
/// After modifying a spec, graph.json is updated with new hash.
#[test]
fn write_updated_graph() {
    let (dir, dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", None)),
    ]);

    // First run — creates graph.json
    specval()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deps")
        .arg(&dir_path)
        .assert()
        .success();

    let graph1 = read_graph_json(dir.path());
    let hash1 = graph1["specs"]["a"]["content_hash"]
        .as_str()
        .expect("should have content_hash")
        .to_string();

    // Modify the spec file
    let spec_path = dir.path().join("a.spec");
    fs::write(
        &spec_path,
        &valid_spec("a", "1.1.0", None), // changed version
    )
    .expect("write modified spec");

    // Second run — should update graph.json
    specval()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deps")
        .arg(&dir_path)
        .assert()
        .success();

    let graph2 = read_graph_json(dir.path());
    let hash2 = graph2["specs"]["a"]["content_hash"]
        .as_str()
        .expect("should have content_hash");

    assert_ne!(hash1, hash2, "content_hash should change when spec is modified");
}

/// graph-cache.spec: validate-without-deps-ignores-graph
/// `validate` without `--deps` doesn't create/modify graph.json.
#[test]
fn validate_without_deps_ignores_graph() {
    let (dir, dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", None)),
    ]);

    // Run without --deps
    specval()
        .current_dir(dir.path())
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .success();

    assert!(
        !dir.path().join(".specval").exists(),
        ".specval should not be created without --deps"
    );

    // Now create a graph.json manually, run without --deps, check it's untouched
    write_graph_json(dir.path(), r#"{"schema_version": 1, "specs": {}}"#);
    let mtime1 = fs::metadata(dir.path().join(".specval").join("graph.json"))
        .unwrap()
        .modified()
        .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(50));

    specval()
        .current_dir(dir.path())
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .success();

    let mtime2 = fs::metadata(dir.path().join(".specval").join("graph.json"))
        .unwrap()
        .modified()
        .unwrap();

    assert_eq!(
        mtime1, mtime2,
        "graph.json should not be modified when --deps is not used"
    );
}

/// graph-cache.spec: rebuild-on-corrupted-graph
/// Corrupted JSON → stderr warns + rebuilds, exit reflects validation.
#[test]
fn rebuild_on_corrupted_graph() {
    let (dir, dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", None)),
    ]);

    // Write corrupted graph.json
    write_graph_json(dir.path(), "this is not valid json {{{");

    specval()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deps")
        .arg(&dir_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("corrupt").or(predicate::str::contains("invalid")))
        .stderr(predicate::str::contains("rebuild").or(predicate::str::contains("rebuilding")));

    // graph.json should now be valid
    let graph = read_graph_json(dir.path());
    assert!(graph.get("specs").is_some(), "rebuilt graph should have specs");
}

/// graph-cache.spec: rebuild-on-schema-mismatch
/// Valid JSON wrong schema → stderr warns + rebuilds.
#[test]
fn rebuild_on_schema_mismatch() {
    let (dir, dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", None)),
    ]);

    // Write valid JSON but wrong schema
    write_graph_json(dir.path(), r#"{"wrong_field": true}"#);

    specval()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deps")
        .arg(&dir_path)
        .assert()
        .success()
        .stderr(
            predicate::str::contains("incompatible")
                .or(predicate::str::contains("schema"))
                .or(predicate::str::contains("format")),
        );

    // graph.json should now be valid with correct schema
    let graph = read_graph_json(dir.path());
    assert!(graph.get("specs").is_some(), "rebuilt graph should have specs");
    assert!(
        graph.get("schema_version").is_some(),
        "rebuilt graph should have schema_version"
    );
}
