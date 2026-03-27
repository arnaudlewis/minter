mod common;

use std::fs;

use common::{minter, spec_two_behaviors};
use predicates::prelude::*;
use tempfile::TempDir;

fn spec_three_behaviors() -> &'static str {
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


behavior do-more [edge_case]
  \"Does more\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
"
}

fn spec_one_behavior(name: &str, version: &str, behavior: &str) -> String {
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

fn spec_with_nfr_ref(name: &str, nfr_ref: &str) -> String {
    format!(
        "\
spec {name} v1.0.0
title \"{name}\"

description
  Test.

motivation
  Test.

nfr
  {nfr_ref}

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

fn spec_no_nfr(name: &str) -> String {
    spec_one_behavior(name, "1.0.0", "do-thing")
}

fn nfr_performance() -> &'static str {
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

fn broken_spec() -> &'static str {
    "\
spec broken v1.0.0
title \"Broken\"

description
  Broken.

motivation

behavior do-thing [happy_path]
"
}

/// Read and parse minter.lock from a directory.
fn read_lock_json(dir: &std::path::Path) -> serde_json::Value {
    let path = dir.join("minter.lock");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e))
}

// ═══════════════════════════════════════════════════════════════
// Happy paths (lock-command.spec)
// ═══════════════════════════════════════════════════════════════

/// lock-command: generate-lock-file
// @minter:e2e generate-lock-file
#[test]
fn generate_lock_file() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_two_behaviors()).unwrap();

    let nfr_dir = spec_dir.join("nfr");
    fs::create_dir(&nfr_dir).unwrap();
    fs::write(nfr_dir.join("performance.nfr"), nfr_performance()).unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("a_test.rs"), "// @minter:unit do-thing\n").unwrap();
    fs::write(test_dir.join("b_test.rs"), "// @minter:e2e do-other\n").unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("lock"));

    let lock = read_lock_json(dir.path());

    // Must be valid JSON (already parsed above)
    assert!(lock.is_object(), "lock file must be a JSON object");
    assert!(lock.get("version").is_some(), "lock must contain version");

    // Spec entry
    let specs = lock.get("specs").expect("lock must contain specs");
    let spec_entry = specs
        .as_object()
        .expect("specs must be an object")
        .values()
        .find(|v| v.as_object().and_then(|o| o.get("hash")).is_some());
    assert!(
        spec_entry.is_some(),
        "lock must contain a spec entry with hash"
    );

    // NFR entry
    let nfrs = lock.get("nfrs").expect("lock must contain nfrs");
    assert!(
        !nfrs.as_object().expect("nfrs must be an object").is_empty(),
        "lock must contain nfr entries"
    );

    // Test files
    let has_test_files = specs
        .as_object()
        .unwrap()
        .values()
        .any(|v| v.get("test_files").is_some());
    assert!(has_test_files, "lock must contain test_files entries");
}

/// lock-command: lock-contains-spec-hashes
// @minter:e2e lock-contains-spec-hashes
#[test]
fn lock_contains_spec_hashes() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    let lock = read_lock_json(dir.path());
    let specs = lock.get("specs").expect("lock must contain specs");
    let spec_obj = specs.as_object().expect("specs must be an object");

    // Find the entry for a.spec (key may be relative path)
    let entry = spec_obj
        .iter()
        .find(|(k, _)| k.contains("a.spec"))
        .expect("lock must contain a.spec entry");

    let hash = entry
        .1
        .get("hash")
        .expect("spec entry must have hash field");
    let hash_str = hash.as_str().expect("hash must be a string");

    // SHA-256 hex string is 64 characters
    assert_eq!(
        hash_str.len(),
        64,
        "hash must be a SHA-256 hex string (64 chars), got: {}",
        hash_str,
    );
    assert!(
        hash_str.chars().all(|c| c.is_ascii_hexdigit()),
        "hash must contain only hex digits, got: {}",
        hash_str,
    );
}

/// lock-command: lock-contains-behaviors
// @minter:e2e lock-contains-behaviors
#[test]
fn lock_contains_behaviors() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_three_behaviors()).unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    let lock = read_lock_json(dir.path());
    let specs = lock.get("specs").expect("lock must contain specs");
    let spec_obj = specs.as_object().expect("specs must be an object");

    let entry = spec_obj
        .iter()
        .find(|(k, _)| k.contains("a.spec"))
        .expect("lock must contain a.spec entry");

    let behaviors = entry
        .1
        .get("behaviors")
        .expect("spec entry must have behaviors array");
    let behaviors_arr = behaviors.as_array().expect("behaviors must be an array");
    let behavior_names: Vec<&str> = behaviors_arr
        .iter()
        .map(|v| v.as_str().expect("behavior must be a string"))
        .collect();

    assert!(
        behavior_names.contains(&"do-thing"),
        "behaviors must contain do-thing"
    );
    assert!(
        behavior_names.contains(&"do-other"),
        "behaviors must contain do-other"
    );
    assert!(
        behavior_names.contains(&"do-more"),
        "behaviors must contain do-more"
    );
}

/// lock-command: lock-contains-dependencies
// @minter:e2e lock-contains-dependencies
#[test]
fn lock_contains_dependencies() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_with_dep("a", "b")).unwrap();
    fs::write(
        spec_dir.join("b.spec"),
        spec_one_behavior("b", "1.0.0", "do-thing"),
    )
    .unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    let lock = read_lock_json(dir.path());
    let specs = lock.get("specs").expect("lock must contain specs");
    let spec_obj = specs.as_object().expect("specs must be an object");

    let entry = spec_obj
        .iter()
        .find(|(k, _)| k.contains("a.spec"))
        .expect("lock must contain a.spec entry");

    let deps = entry
        .1
        .get("dependencies")
        .expect("spec entry must have dependencies array");
    let deps_arr = deps.as_array().expect("dependencies must be an array");
    let dep_strs: Vec<&str> = deps_arr
        .iter()
        .map(|v| v.as_str().expect("dependency must be a string"))
        .collect();

    assert!(
        dep_strs.iter().any(|d| d.contains("b.spec")),
        "dependencies must reference b.spec, got: {:?}",
        dep_strs,
    );
}

/// lock-command: lock-contains-nfr-refs
// @minter:e2e lock-contains-nfr-refs
#[test]
fn lock_contains_nfr_refs() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_with_nfr_ref("a", "performance#api-latency"),
    )
    .unwrap();

    let nfr_dir = spec_dir.join("nfr");
    fs::create_dir(&nfr_dir).unwrap();
    fs::write(nfr_dir.join("performance.nfr"), nfr_performance()).unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    let lock = read_lock_json(dir.path());
    let specs = lock.get("specs").expect("lock must contain specs");
    let spec_obj = specs.as_object().expect("specs must be an object");

    let entry = spec_obj
        .iter()
        .find(|(k, _)| k.contains("a.spec"))
        .expect("lock must contain a.spec entry");

    let nfrs = entry
        .1
        .get("nfrs")
        .expect("spec entry must have nfrs array");
    let nfrs_arr = nfrs.as_array().expect("nfrs must be an array");
    let nfr_strs: Vec<&str> = nfrs_arr
        .iter()
        .map(|v| v.as_str().expect("nfr ref must be a string"))
        .collect();

    assert!(
        nfr_strs.contains(&"performance#api-latency"),
        "nfrs must contain performance#api-latency, got: {:?}",
        nfr_strs,
    );
}

/// lock-command: lock-contains-nfr-hashes
// @minter:e2e lock-contains-nfr-hashes
#[test]
fn lock_contains_nfr_hashes() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_with_nfr_ref("a", "performance#api-latency"),
    )
    .unwrap();

    let nfr_dir = spec_dir.join("nfr");
    fs::create_dir(&nfr_dir).unwrap();
    fs::write(nfr_dir.join("performance.nfr"), nfr_performance()).unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    let lock = read_lock_json(dir.path());
    let nfrs = lock.get("nfrs").expect("lock must contain nfrs section");
    let nfrs_obj = nfrs.as_object().expect("nfrs must be an object");

    let entry = nfrs_obj
        .iter()
        .find(|(k, _)| k.contains("performance"))
        .expect("nfrs must contain performance entry");

    let hash = entry.1.get("hash").expect("nfr entry must have hash field");
    let hash_str = hash.as_str().expect("hash must be a string");
    assert_eq!(
        hash_str.len(),
        64,
        "nfr hash must be a SHA-256 hex string (64 chars)"
    );
}

/// lock-command: lock-contains-test-file-hashes
// @minter:e2e lock-contains-test-file-hashes
#[test]
fn lock_contains_test_file_hashes() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("a_test.rs"), "// @minter:unit do-thing\n").unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    let lock = read_lock_json(dir.path());
    let specs = lock.get("specs").expect("lock must contain specs");
    let spec_obj = specs.as_object().expect("specs must be an object");

    let entry = spec_obj
        .iter()
        .find(|(k, _)| k.contains("a.spec"))
        .expect("lock must contain a.spec entry");

    let test_files = entry
        .1
        .get("test_files")
        .expect("spec entry must have test_files");
    let test_files_obj = test_files
        .as_object()
        .expect("test_files must be an object");

    let test_entry = test_files_obj
        .iter()
        .find(|(k, _)| k.contains("a_test.rs"))
        .expect("test_files must contain a_test.rs");

    let hash = test_entry
        .1
        .get("hash")
        .expect("test file entry must have hash field");
    let hash_str = hash.as_str().expect("hash must be a string");

    assert_eq!(
        hash_str.len(),
        64,
        "test file hash must be SHA-256 hex (64 chars), got: {}",
        hash_str,
    );
    assert!(
        hash_str.chars().all(|c| c.is_ascii_hexdigit()),
        "hash must contain only hex digits"
    );
}

/// lock-command: lock-contains-covers-mapping
// @minter:e2e lock-contains-covers-mapping
#[test]
fn lock_contains_covers_mapping() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_two_behaviors()).unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();
    fs::write(
        test_dir.join("a_test.rs"),
        "// @minter:unit do-thing\n// @minter:e2e do-other\n",
    )
    .unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    let lock = read_lock_json(dir.path());
    let specs = lock.get("specs").expect("lock must contain specs");
    let spec_obj = specs.as_object().expect("specs must be an object");

    let entry = spec_obj
        .iter()
        .find(|(k, _)| k.contains("a.spec"))
        .expect("lock must contain a.spec entry");

    let test_files = entry
        .1
        .get("test_files")
        .expect("spec entry must have test_files");
    let test_files_obj = test_files
        .as_object()
        .expect("test_files must be an object");

    let test_entry = test_files_obj
        .iter()
        .find(|(k, _)| k.contains("a_test.rs"))
        .expect("test_files must contain a_test.rs");

    let covers = test_entry
        .1
        .get("covers")
        .expect("test file entry must have covers array");
    let covers_arr = covers.as_array().expect("covers must be an array");
    let cover_strs: Vec<&str> = covers_arr
        .iter()
        .map(|v| v.as_str().expect("cover must be a string"))
        .collect();

    assert!(
        cover_strs.contains(&"do-thing"),
        "covers must contain do-thing, got: {:?}",
        cover_strs,
    );
    assert!(
        cover_strs.contains(&"do-other"),
        "covers must contain do-other, got: {:?}",
        cover_strs,
    );
}

/// lock-command: lock-reads-config
// @minter:e2e lock-reads-config
#[test]
fn lock_reads_config() {
    let dir = TempDir::new().unwrap();

    // Write config pointing to non-default paths
    fs::write(
        dir.path().join("minter.config.json"),
        r#"{ "specs": "specifications/", "tests": ["src/tests/"] }"#,
    )
    .unwrap();

    let spec_dir = dir.path().join("specifications");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    let test_dir = dir.path().join("src").join("tests");
    fs::create_dir_all(&test_dir).unwrap();
    fs::write(test_dir.join("a_test.rs"), "// @minter:unit do-thing\n").unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    let lock = read_lock_json(dir.path());

    // Lock must reference the config paths
    let lock_str = serde_json::to_string(&lock).unwrap();
    assert!(
        lock_str.contains("a.spec"),
        "lock must contain specifications/a.spec"
    );
    assert!(
        lock_str.contains("a_test.rs"),
        "lock must contain src/tests/a_test.rs"
    );
}

/// lock-command: lock-uses-default-conventions
// @minter:e2e lock-uses-default-conventions
#[test]
fn lock_uses_default_conventions() {
    let dir = TempDir::new().unwrap();

    // No minter.config.json
    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("a_test.rs"), "// @minter:unit do-thing\n").unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    let lock = read_lock_json(dir.path());
    let lock_str = serde_json::to_string(&lock).unwrap();
    assert!(
        lock_str.contains("a.spec"),
        "lock must contain specs/a.spec"
    );
    assert!(
        lock_str.contains("a_test.rs"),
        "lock must contain tests/a_test.rs"
    );
}

/// lock-command: lock-deterministic
// @minter:e2e lock-deterministic
#[test]
fn lock_deterministic() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("a_test.rs"), "// @minter:unit do-thing\n").unwrap();

    // First run
    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();
    let first = fs::read(dir.path().join("minter.lock")).expect("read first lock");

    // Second run
    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();
    let second = fs::read(dir.path().join("minter.lock")).expect("read second lock");

    assert_eq!(
        first, second,
        "running lock twice on unchanged files must produce identical output"
    );
}

/// lock-command: lock-updates-existing
// @minter:e2e lock-updates-existing
#[test]
fn lock_updates_existing() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();

    // First lock
    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();
    let first = fs::read(dir.path().join("minter.lock")).expect("read first lock");

    // Modify spec
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "2.0.0", "do-thing-differently"),
    )
    .unwrap();

    // Second lock
    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();
    let second = fs::read(dir.path().join("minter.lock")).expect("read second lock");

    assert_ne!(
        first, second,
        "lock must reflect updated spec content — hashes must differ"
    );
}

/// lock-command: lock-multiple-specs
// @minter:e2e lock-multiple-specs
#[test]
fn lock_multiple_specs() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_two_behaviors()).unwrap();

    let sub_dir = spec_dir.join("sub");
    fs::create_dir(&sub_dir).unwrap();
    fs::write(sub_dir.join("b.spec"), spec_three_behaviors()).unwrap();

    let deep_dir = sub_dir.join("deep");
    fs::create_dir(&deep_dir).unwrap();
    fs::write(
        deep_dir.join("c.spec"),
        spec_one_behavior("c", "1.0.0", "do-deep"),
    )
    .unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    let lock = read_lock_json(dir.path());
    let lock_str = serde_json::to_string(&lock).unwrap();

    assert!(
        lock_str.contains("a.spec"),
        "lock must contain specs/a.spec"
    );
    assert!(
        lock_str.contains("b.spec"),
        "lock must contain specs/sub/b.spec"
    );
    assert!(
        lock_str.contains("c.spec"),
        "lock must contain specs/sub/deep/c.spec"
    );
}

/// lock-command: lock-test-covers-multiple-specs
// @minter:e2e lock-test-covers-multiple-specs
#[test]
fn lock_test_covers_multiple_specs() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();
    fs::write(
        spec_dir.join("b.spec"),
        spec_one_behavior("b", "1.0.0", "do-other"),
    )
    .unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();
    fs::write(
        test_dir.join("shared_test.rs"),
        "// @minter:e2e do-thing\n// @minter:e2e do-other\n",
    )
    .unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    let lock = read_lock_json(dir.path());
    let specs = lock.get("specs").expect("lock must contain specs");
    let spec_obj = specs.as_object().expect("specs must be an object");

    // spec a should have shared_test.rs covering do-thing
    let entry_a = spec_obj
        .iter()
        .find(|(k, _)| k.contains("a.spec"))
        .expect("lock must contain a.spec");
    let test_files_a = entry_a
        .1
        .get("test_files")
        .expect("a.spec must have test_files");
    let test_files_a_obj = test_files_a.as_object().expect("test_files must be object");
    let shared_a = test_files_a_obj
        .iter()
        .find(|(k, _)| k.contains("shared_test.rs"))
        .expect("a.spec test_files must contain shared_test.rs");
    let covers_a: Vec<&str> = shared_a
        .1
        .get("covers")
        .expect("must have covers")
        .as_array()
        .expect("covers must be array")
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(
        covers_a.contains(&"do-thing"),
        "a.spec shared_test.rs covers must contain do-thing"
    );

    // spec b should have shared_test.rs covering do-other
    let entry_b = spec_obj
        .iter()
        .find(|(k, _)| k.contains("b.spec"))
        .expect("lock must contain b.spec");
    let test_files_b = entry_b
        .1
        .get("test_files")
        .expect("b.spec must have test_files");
    let test_files_b_obj = test_files_b.as_object().expect("test_files must be object");
    let shared_b = test_files_b_obj
        .iter()
        .find(|(k, _)| k.contains("shared_test.rs"))
        .expect("b.spec test_files must contain shared_test.rs");
    let covers_b: Vec<&str> = shared_b
        .1
        .get("covers")
        .expect("must have covers")
        .as_array()
        .expect("covers must be array")
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(
        covers_b.contains(&"do-other"),
        "b.spec shared_test.rs covers must contain do-other"
    );
}

// ═══════════════════════════════════════════════════════════════
// Error cases (lock-command.spec)
// ═══════════════════════════════════════════════════════════════

/// lock-command: reject-invalid-specs
// @minter:e2e reject-invalid-specs
#[test]
fn reject_invalid_specs() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("broken.spec"), broken_spec()).unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("broken"));
}

/// lock-command: reject-tag-errors
// @minter:e2e reject-tag-errors
#[test]
fn reject_tag_errors() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();
    fs::write(
        test_dir.join("a_test.rs"),
        "// @minter:unit nonexistent-behavior\n",
    )
    .unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent-behavior"));
}

/// lock-command: reject-no-specs-found
// @minter:e2e reject-no-specs-found
#[test]
fn reject_no_specs_found() {
    let dir = TempDir::new().unwrap();

    // Create empty specs directory
    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("no spec files found"));
}

// ═══════════════════════════════════════════════════════════════
// Edge cases (lock-command.spec)
// ═══════════════════════════════════════════════════════════════

/// lock-command: lock-no-tests
// @minter:e2e lock-no-tests
#[test]
fn lock_no_tests() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    // Create tests dir but no files with @minter tags
    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("a_test.rs"), "fn test() {}\n").unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    let lock = read_lock_json(dir.path());
    let specs = lock.get("specs").expect("lock must contain specs");
    let spec_obj = specs.as_object().expect("specs must be an object");

    let entry = spec_obj
        .iter()
        .find(|(k, _)| k.contains("a.spec"))
        .expect("lock must contain a.spec");

    let test_files = entry
        .1
        .get("test_files")
        .expect("spec entry must have test_files");

    // test_files should be empty (no @minter tags found)
    let test_files_obj = test_files
        .as_object()
        .expect("test_files must be an object");
    assert!(
        test_files_obj.is_empty(),
        "test_files must be empty when no tests have @minter tags, got: {:?}",
        test_files_obj,
    );
}

/// lock-command: lock-no-nfrs
// @minter:e2e lock-no-nfrs
#[test]
fn lock_no_nfrs() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), spec_no_nfr("a")).unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    let lock = read_lock_json(dir.path());
    let nfrs = lock.get("nfrs").expect("lock must contain nfrs section");
    let nfrs_obj = nfrs.as_object().expect("nfrs must be an object");

    assert!(
        nfrs_obj.is_empty(),
        "nfrs section must be empty when no NFR files exist, got: {:?}",
        nfrs_obj,
    );
}

/// lock-command: lock-atomic-write
// @minter:e2e lock-atomic-write
#[test]
fn lock_atomic_write() {
    let dir = TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    // After successful write, minter.lock must exist and be valid JSON
    let lock_path = dir.path().join("minter.lock");
    assert!(
        lock_path.exists(),
        "minter.lock must exist after lock command"
    );

    let content = fs::read_to_string(&lock_path).expect("read minter.lock");
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
    assert!(
        parsed.is_ok(),
        "minter.lock must be valid JSON (atomic write should prevent corruption)"
    );

    // No temp file should be left behind
    let entries: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|s| s.contains("minter.lock") && s.contains(".tmp"))
                .unwrap_or(false)
        })
        .collect();
    assert!(
        entries.is_empty(),
        "no temp files should remain after atomic write"
    );
}

/// lock-command: lock-scans-all-test-dirs
// @minter:e2e lock-scans-all-test-dirs
#[test]
fn lock_scans_all_test_dirs() {
    let dir = TempDir::new().unwrap();

    // Write config with multiple test directories
    fs::write(
        dir.path().join("minter.config.json"),
        r#"{ "specs": "specs/", "tests": ["tests/", "benches/"] }"#,
    )
    .unwrap();

    // Create specs/
    let spec_dir = dir.path().join("specs");
    fs::create_dir(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    // Create NFR for benchmark references
    let nfr_dir = spec_dir.join("nfr");
    fs::create_dir(&nfr_dir).unwrap();
    fs::write(nfr_dir.join("performance.nfr"), nfr_performance()).unwrap();

    // Create tests/ with a unit test
    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("a_test.rs"), "// @minter:unit do-thing\n").unwrap();

    // Create benches/ with a benchmark test
    let bench_dir = dir.path().join("benches");
    fs::create_dir(&bench_dir).unwrap();
    fs::write(
        bench_dir.join("perf_test.rs"),
        "// @minter:benchmark #performance#api-latency\n",
    )
    .unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    let lock = read_lock_json(dir.path());
    let lock_str = serde_json::to_string(&lock).unwrap();

    // Lock must contain test files from tests/
    assert!(
        lock_str.contains("a_test.rs"),
        "lock must contain tests/a_test.rs"
    );

    // Lock must also contain benchmark files from benches/
    assert!(
        lock_str.contains("perf_test.rs"),
        "lock must contain benches/perf_test.rs — all configured test dirs must be scanned"
    );

    // Verify benchmark_files section exists
    let benchmarks = lock
        .get("benchmark_files")
        .expect("lock must contain benchmark_files section");
    let benchmarks_obj = benchmarks
        .as_object()
        .expect("benchmark_files must be an object");
    let bench_entry = benchmarks_obj
        .iter()
        .find(|(k, _)| k.contains("perf_test.rs"))
        .expect("benchmark_files must contain perf_test.rs");
    let hash = bench_entry
        .1
        .get("hash")
        .expect("benchmark file entry must have hash field");
    let hash_str = hash.as_str().expect("hash must be a string");
    assert_eq!(
        hash_str.len(),
        64,
        "benchmark file hash must be SHA-256 hex (64 chars)"
    );
}
