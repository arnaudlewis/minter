use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use tempfile::TempDir;

/// Create a Command for the minter binary.
pub fn minter() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("minter")
}

/// Write a single .spec file to a temp directory.
/// Returns the dir handle (must be kept alive) and the file path.
pub fn temp_spec(name: &str, content: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("create temp dir");
    let path = dir.path().join(format!("{}.spec", name));
    fs::write(&path, content).expect("write spec file");
    (dir, path)
}

/// Write multiple .spec files to a temp directory and return the directory path.
pub fn temp_dir_with_specs(specs: &[(&str, &str)]) -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("create temp dir");
    for (name, content) in specs {
        let path = dir.path().join(format!("{}.spec", name));
        fs::write(&path, content).expect("write spec file");
    }
    let dir_path = dir.path().to_path_buf();
    (dir, dir_path)
}

/// Create a temp directory with specs in subdirectories.
/// Accepts ("subdir/name", content) pairs where the subdir path can be nested.
pub fn temp_dir_with_nested_specs(specs: &[(&str, &str)]) -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("create temp dir");
    for (relative_path, content) in specs {
        let path = dir.path().join(format!("{}.spec", relative_path));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create subdirectory");
        }
        fs::write(&path, content).expect("write spec file");
    }
    let dir_path = dir.path().to_path_buf();
    (dir, dir_path)
}

/// Generate a spec with N behaviors for benchmarking.
pub fn spec_with_n_behaviors(name: &str, n: usize) -> String {
    let mut s = format!(
        "\
spec {name} v1.0.0
title \"{name}\"

description
  Benchmark spec with {n} behaviors.

motivation
  Performance testing.

"
    );

    for i in 0..n {
        let category = if i == 0 { "happy_path" } else { "edge_case" };
        s.push_str(&format!(
            "\
behavior do-thing-{i} [{category}]
  \"Does thing {i}\"

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"

"
        ));
    }
    s
}

/// Generate a spec with dependencies and N behaviors for benchmarking.
pub fn spec_with_deps_and_behaviors(name: &str, deps: &[&str], n: usize) -> String {
    let mut s = spec_with_n_behaviors(name, n);
    for dep in deps {
        s.push_str(&format!("\ndepends on {dep} >= 1.0.0\n"));
    }
    s
}
