mod common;

use std::fs;

use common::{minter, read_graph_json, temp_dir_with_specs};

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
// dependency-resolution.spec: incremental validation behaviors
// ═══════════════════════════════════════════════════════════════

// @minter:e2e cache-integrates-new-files
#[test]
fn integrate_new_spec_file() {
    let (dir, dir_path) = temp_dir_with_specs(&[("a", &valid_spec("a", "1.0.0", None))]);

    // First run — build initial graph
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deep")
        .arg(&dir_path)
        .assert()
        .success();

    let graph1 = read_graph_json(dir.path());
    assert!(
        graph1["specs"]["d"].is_null(),
        "d should not be in initial graph"
    );

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
        .arg("--deep")
        .arg(&dir_path)
        .assert()
        .success();

    let graph2 = read_graph_json(dir.path());
    assert!(
        graph2["specs"]["d"].is_object(),
        "d should be added to graph after creation"
    );
}
