mod common;

use std::fs;

use common::{minter, read_graph_json, temp_dir_with_specs, write_graph_json};
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

// @minter:e2e cache-directory-location
#[test]
fn minter_directory_at_cwd() {
    // Create a project structure: project_root/specs/a.spec
    let project_root = tempfile::TempDir::new().unwrap();
    let specs_dir = project_root.path().join("specs");
    std::fs::create_dir(&specs_dir).unwrap();
    std::fs::write(specs_dir.join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

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
        project_root
            .path()
            .join(".minter")
            .join("graph.json")
            .exists(),
        ".minter/graph.json should be at the project root (CWD)"
    );
    assert!(
        !specs_dir.join(".minter").exists(),
        ".minter should NOT be created inside the specs directory"
    );
}

// @minter:e2e cache-cold-start-creates-directory
#[test]
fn create_minter_directory() {
    let (dir, dir_path) = temp_dir_with_specs(&[("a", &valid_spec("a", "1.0.0", None))]);

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

// @minter:e2e cache-produces-correct-results
#[test]
fn load_cached_graph() {
    let (dir, dir_path) = temp_dir_with_specs(&[("a", &valid_spec("a", "1.0.0", None))]);

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

// @minter:e2e cache-revalidates-modified-and-dependents
#[test]
fn write_updated_graph() {
    let (dir, dir_path) = temp_dir_with_specs(&[("a", &valid_spec("a", "1.0.0", None))]);

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
        valid_spec("a", "1.1.0", None), // changed version
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

    assert_ne!(
        hash1, hash2,
        "content_hash should change when spec is modified"
    );
}

// @minter:e2e validate-without-deep-ignores-graph
#[test]
fn validate_without_deps_ignores_graph() {
    let (dir, _dir_path) = temp_dir_with_specs(&[("a", &valid_spec("a", "1.0.0", None))]);

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

// @minter:e2e rebuild-on-corrupted-graph
#[test]
fn rebuild_on_corrupted_graph() {
    let (dir, dir_path) = temp_dir_with_specs(&[("a", &valid_spec("a", "1.0.0", None))]);

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
    assert!(
        graph.get("specs").is_some(),
        "rebuilt graph should have specs"
    );
}

// @minter:e2e rebuild-on-schema-mismatch
#[test]
fn rebuild_on_schema_mismatch() {
    let (dir, dir_path) = temp_dir_with_specs(&[("a", &valid_spec("a", "1.0.0", None))]);

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
    assert!(
        graph.get("specs").is_some(),
        "rebuilt graph should have specs"
    );
    assert!(
        graph.get("schema_version").is_some(),
        "rebuilt graph should have schema_version"
    );
}

// ═══════════════════════════════════════════════════════════════
// dependency-resolution.spec: Graph cache — NFR tracking
// ═══════════════════════════════════════════════════════════════

fn valid_nfr(category: &str, version: &str) -> String {
    format!(
        "\
nfr {category} v{version}
title \"{category} Requirements\"

description
  Defines {category} constraints.

motivation
  {category} matters.


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
"
    )
}

fn spec_with_nfr_ref(name: &str, version: &str, nfr_category: &str) -> String {
    format!(
        "\
spec {name} v{version}
title \"{name}\"

description
  A spec for testing.

motivation
  Testing graph cache.

nfr
  {nfr_category}#api-response-time

behavior do-thing [happy_path]
  \"Do the thing\"

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
"
    )
}

// @minter:e2e cache-tracks-nfr-files
#[test]
fn cache_tracks_nfr_files() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(
        dir.path().join("a.spec"),
        spec_with_nfr_ref("a", "1.0.0", "performance"),
    )
    .unwrap();
    fs::write(
        dir.path().join("performance.nfr"),
        valid_nfr("performance", "1.0.0"),
    )
    .unwrap();

    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deep")
        .arg(dir.path())
        .assert()
        .success();

    let graph = read_graph_json(dir.path());
    assert!(
        graph["nfrs"].is_object(),
        "graph should have an nfrs section"
    );
    assert!(
        graph["nfrs"]["performance"].is_object(),
        "nfrs section should contain performance entry"
    );
    assert!(
        graph["nfrs"]["performance"]["content_hash"].is_string(),
        "NFR entry should have content_hash"
    );
    assert_eq!(
        graph["nfrs"]["performance"]["version"].as_str().unwrap(),
        "1.0.0",
        "NFR entry should have correct version"
    );
    assert!(
        graph["nfrs"]["performance"]["constraint_count"]
            .as_u64()
            .unwrap()
            > 0,
        "NFR entry should have constraint_count"
    );
    // Also verify the spec entry tracks nfr_categories
    assert!(
        graph["specs"]["a"]["nfr_categories"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str() == Some("performance")),
        "spec entry should track nfr_categories"
    );
}

// @minter:e2e cache-revalidates-on-nfr-change
#[test]
fn cache_revalidates_on_nfr_change() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(
        dir.path().join("a.spec"),
        spec_with_nfr_ref("a", "1.0.0", "performance"),
    )
    .unwrap();
    fs::write(
        dir.path().join("performance.nfr"),
        valid_nfr("performance", "1.0.0"),
    )
    .unwrap();

    // First run — build cache
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deep")
        .arg(dir.path())
        .assert()
        .success();

    let graph1 = read_graph_json(dir.path());
    let nfr_hash1 = graph1["nfrs"]["performance"]["content_hash"]
        .as_str()
        .unwrap()
        .to_string();

    // Modify the NFR file (bump version)
    fs::write(
        dir.path().join("performance.nfr"),
        valid_nfr("performance", "2.0.0"),
    )
    .unwrap();

    // Second run — should revalidate spec a (not skip it)
    let output = minter()
        .current_dir(dir.path())
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg("--deep")
        .arg(dir.path())
        .output()
        .expect("run validate");
    assert!(output.status.success());

    let graph2 = read_graph_json(dir.path());
    let nfr_hash2 = graph2["nfrs"]["performance"]["content_hash"]
        .as_str()
        .unwrap();
    assert_ne!(
        nfr_hash1, nfr_hash2,
        "NFR content_hash should change when .nfr file is modified"
    );
    assert_eq!(
        graph2["nfrs"]["performance"]["version"].as_str().unwrap(),
        "2.0.0",
        "NFR version should update to 2.0.0"
    );
}

// @minter:e2e cache-integrates-new-nfr-files
#[test]
fn cache_integrates_new_nfr_files() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    // First run — no NFR files
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deep")
        .arg(dir.path())
        .assert()
        .success();

    let graph1 = read_graph_json(dir.path());
    assert!(
        graph1["nfrs"]["security"].is_null(),
        "security should not be in initial graph"
    );

    // Add a new NFR file
    fs::write(
        dir.path().join("security.nfr"),
        valid_nfr("security", "1.0.0"),
    )
    .unwrap();

    // Second run
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deep")
        .arg(dir.path())
        .assert()
        .success();

    let graph2 = read_graph_json(dir.path());
    assert!(
        graph2["nfrs"]["security"].is_object(),
        "security should be added to graph after creation"
    );
}

// @minter:e2e cache-prunes-deleted-nfr-files
#[test]
fn cache_prunes_deleted_nfr_files() {
    let dir = tempfile::TempDir::new().unwrap();
    // Use a spec without NFR refs so deletion doesn't cause cross-ref failure
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();
    fs::write(
        dir.path().join("performance.nfr"),
        valid_nfr("performance", "1.0.0"),
    )
    .unwrap();

    // First run — builds cache with performance NFR
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deep")
        .arg(dir.path())
        .assert()
        .success();

    let graph1 = read_graph_json(dir.path());
    assert!(
        graph1["nfrs"]["performance"].is_object(),
        "performance should be in initial graph"
    );

    // Delete the NFR file
    fs::remove_file(dir.path().join("performance.nfr")).unwrap();

    // Second run — should prune the deleted NFR from cache
    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deep")
        .arg(dir.path())
        .assert()
        .success();

    let graph2 = read_graph_json(dir.path());
    assert!(
        graph2["nfrs"]["performance"].is_null(),
        "performance should be pruned from graph after deletion"
    );
}

// @minter:e2e rebuild-on-schema-mismatch
#[test]
fn rebuild_v2_cache_to_v3() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    // Write a v2 schema cache (without nfrs field)
    write_graph_json(dir.path(), r#"{"schema_version": 2, "specs": {}}"#);

    minter()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--deep")
        .arg(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("incompatible").or(predicate::str::contains("format")));

    // graph.json should now be v3 with nfrs field
    let graph = read_graph_json(dir.path());
    assert_eq!(
        graph["schema_version"].as_u64().unwrap(),
        3,
        "rebuilt graph should have schema_version 3"
    );
}
