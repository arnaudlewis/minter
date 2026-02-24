mod common;

use std::fs;

use common::{read_graph_json, minter, temp_dir_with_specs, write_graph_json};
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

/// dependency-resolution.spec: cache-directory-location
#[test]
fn minter_directory_at_cwd() {
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
    minter()
        .current_dir(project_root.path())
        .arg("validate")
        .arg("--deep")
        .arg("specs")
        .assert()
        .success();

    // .minter should be at project_root, NOT inside specs/
    assert!(
        project_root.path().join(".minter").join("graph.json").exists(),
        ".minter/graph.json should be at the project root (CWD)"
    );
    assert!(
        !specs_dir.join(".minter").exists(),
        ".minter should NOT be created inside the specs directory"
    );
}

/// dependency-resolution.spec: cache-cold-start-creates-directory
#[test]
fn create_minter_directory() {
    let (dir, dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", None)),
    ]);

    // Ensure no .minter dir exists
    assert!(!dir.path().join(".minter").exists());

    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deep")
        .arg(&dir_path)
        .assert()
        .success();

    assert!(
        dir.path().join(".minter").exists(),
        ".minter directory should be created"
    );
    assert!(
        dir.path().join(".minter").join("graph.json").exists(),
        ".minter/graph.json should be created"
    );
}

/// dependency-resolution.spec: cache-produces-correct-results
#[test]
fn load_cached_graph() {
    let (dir, dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", None)),
    ]);

    // First run — creates graph.json
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deep")
        .arg(&dir_path)
        .assert()
        .success();

    let graph_path = dir.path().join(".minter").join("graph.json");
    let mtime1 = fs::metadata(&graph_path)
        .expect("graph.json should exist")
        .modified()
        .expect("should have mtime");

    // Small delay to ensure mtime would differ if file were rewritten
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Second run — should not rewrite graph.json
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deep")
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

/// dependency-resolution.spec: cache-revalidates-modified-and-dependents
#[test]
fn write_updated_graph() {
    let (dir, dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", None)),
    ]);

    // First run — creates graph.json
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deep")
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
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deep")
        .arg(&dir_path)
        .assert()
        .success();

    let graph2 = read_graph_json(dir.path());
    let hash2 = graph2["specs"]["a"]["content_hash"]
        .as_str()
        .expect("should have content_hash");

    assert_ne!(hash1, hash2, "content_hash should change when spec is modified");
}

/// dependency-resolution.spec: validate-without-deep-ignores-graph
#[test]
fn validate_without_deps_ignores_graph() {
    let (dir, _dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", None)),
    ]);

    let spec_file = dir.path().join("a.spec");

    // Run single file without --deep
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg(&spec_file)
        .assert()
        .success();

    assert!(
        !dir.path().join(".minter").exists(),
        ".minter should not be created without --deep for single file validation"
    );

    // Now create a graph.json manually, run single file without --deep, check it's untouched
    write_graph_json(dir.path(), r#"{"schema_version": 1, "specs": {}}"#);
    let mtime1 = fs::metadata(dir.path().join(".minter").join("graph.json"))
        .unwrap()
        .modified()
        .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(50));

    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg(&spec_file)
        .assert()
        .success();

    let mtime2 = fs::metadata(dir.path().join(".minter").join("graph.json"))
        .unwrap()
        .modified()
        .unwrap();

    assert_eq!(
        mtime1, mtime2,
        "graph.json should not be modified when --deep is not used for single file"
    );
}

/// dependency-resolution.spec: rebuild-on-corrupted-graph
#[test]
fn rebuild_on_corrupted_graph() {
    let (dir, dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", None)),
    ]);

    // Write corrupted graph.json
    write_graph_json(dir.path(), "this is not valid json {{{");

    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deep")
        .arg(&dir_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("corrupt").or(predicate::str::contains("invalid")))
        .stderr(predicate::str::contains("rebuild").or(predicate::str::contains("rebuilding")));

    // graph.json should now be valid
    let graph = read_graph_json(dir.path());
    assert!(graph.get("specs").is_some(), "rebuilt graph should have specs");
}

/// dependency-resolution.spec: rebuild-on-schema-mismatch
#[test]
fn rebuild_on_schema_mismatch() {
    let (dir, dir_path) = temp_dir_with_specs(&[
        ("a", &valid_spec("a", "1.0.0", None)),
    ]);

    // Write valid JSON but wrong schema
    write_graph_json(dir.path(), r#"{"wrong_field": true}"#);

    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deep")
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

