use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use tempfile::TempDir;

/// Create a Command for the specval binary.
pub fn specval() -> Command {
    Command::cargo_bin("specval").expect("specval binary should exist")
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

/// A minimal valid spec that exercises the core structure.
pub const VALID_SPEC: &str = "\
spec test-spec v1.0.0
title \"Test Spec\"

description
  A test spec for validation.

motivation
  Testing specval.

behavior do-thing [happy_path]
  \"Do the thing\"

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
";
