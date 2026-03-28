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

/// Write a single .nfr file to a temp directory.
/// Returns the dir handle (must be kept alive) and the file path.
pub fn temp_nfr(name: &str, content: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("create temp dir");
    let path = dir.path().join(format!("{}.nfr", name));
    fs::write(&path, content).expect("write nfr file");
    (dir, path)
}

/// A minimal valid NFR spec that exercises the core structure.
pub const VALID_NFR: &str = "\
nfr performance v1.0.0
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
";

/// Write a .spec and one or more .nfr files to a temp directory.
/// Returns the dir handle and directory path.
pub fn temp_dir_with_spec_and_nfrs(
    spec_name: &str,
    spec_content: &str,
    nfrs: &[(&str, &str)],
) -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("create temp dir");
    let spec_path = dir.path().join(format!("{}.spec", spec_name));
    fs::write(&spec_path, spec_content).expect("write spec file");
    for (name, content) in nfrs {
        let nfr_path = dir.path().join(format!("{}.nfr", name));
        fs::write(&nfr_path, content).expect("write nfr file");
    }
    let dir_path = dir.path().to_path_buf();
    (dir, dir_path)
}

/// Get the path to the minter binary.
pub fn minter_bin() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin!("minter").to_path_buf()
}

/// Generate a spec with a single behavior.
pub fn spec_one_behavior(name: &str, version: &str, behavior: &str) -> String {
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

/// Generate a spec with three parameterized behaviors.
pub fn spec_three_behaviors(name: &str, version: &str, b1: &str, b2: &str, b3: &str) -> String {
    format!(
        "\
spec {name} v{version}
title \"{name}\"

description
  Test.

motivation
  Test.

behavior {b1} [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"


behavior {b2} [happy_path]
  \"Does another\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"


behavior {b3} [happy_path]
  \"Does a third\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
"
    )
}

/// A minimal NFR spec for performance testing.
pub fn nfr_performance() -> &'static str {
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

/// Helper: a valid spec with a given name, version, and optional dependency.
pub fn valid_spec(name: &str, version: &str, dep: Option<(&str, &str)>) -> String {
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
  Testing.

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

/// A minimal spec with two behaviors and a custom name, v1.0.0.
pub fn spec_two_behaviors_named(name: &str) -> String {
    format!(
        "\
spec {name} v1.0.0
title \"{name}\"

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
    )
}

/// A minimal spec with two behaviors, fixed as spec "a" v1.0.0.
pub fn spec_two_behaviors() -> &'static str {
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
