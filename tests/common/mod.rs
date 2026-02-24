#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

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

/// Write multiple .spec files to the same temp directory.
/// Returns the dir handle and all file paths.
pub fn temp_specs(specs: &[(&str, &str)]) -> (TempDir, Vec<PathBuf>) {
    let dir = TempDir::new().expect("create temp dir");
    let mut paths = Vec::new();
    for (name, content) in specs {
        let path = dir.path().join(format!("{}.spec", name));
        fs::write(&path, content).expect("write spec file");
        paths.push(path);
    }
    (dir, paths)
}

/// Write a non-.spec file to a temp directory.
pub fn temp_file(name: &str, content: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("create temp dir");
    let path = dir.path().join(name);
    fs::write(&path, content).expect("write file");
    (dir, path)
}

/// Write multiple .spec files to a temp directory and return the directory path.
/// Useful for testing directory validation (where minter receives a dir, not files).
pub fn temp_dir_with_specs(specs: &[(&str, &str)]) -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("create temp dir");
    for (name, content) in specs {
        let path = dir.path().join(format!("{}.spec", name));
        fs::write(&path, content).expect("write spec file");
    }
    let dir_path = dir.path().to_path_buf();
    (dir, dir_path)
}

/// Read and parse .minter/graph.json from a directory.
pub fn read_graph_json(dir: &Path) -> serde_json::Value {
    let path = dir.join(".minter").join("graph.json");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e))
}

/// Write arbitrary content to .minter/graph.json in a directory.
pub fn write_graph_json(dir: &Path, content: &str) {
    let minter_dir = dir.join(".minter");
    fs::create_dir_all(&minter_dir).expect("create .minter dir");
    fs::write(minter_dir.join("graph.json"), content).expect("write graph.json");
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

/// A minimal valid spec that exercises the core structure.
pub const VALID_SPEC: &str = "\
spec test-spec v1.0.0
title \"Test Spec\"

description
  A test spec for validation.

motivation
  Testing minter.

behavior do-thing [happy_path]
  \"Do the thing\"

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
";
