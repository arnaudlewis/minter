mod common;

use std::fs;

use common::{minter, spec_two_behaviors};
use predicates::prelude::*;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

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

fn spec_with_dep(name: &str, version: &str, behavior: &str, dep_name: &str) -> String {
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

depends on {dep_name} >= 1.0.0
"
    )
}

fn spec_with_nfr(name: &str, version: &str, behavior: &str, nfr_category: &str) -> String {
    format!(
        "\
spec {name} v{version}
title \"{name}\"

description
  Test.

motivation
  Test.

nfr
  {nfr_category}#api-latency

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

/// Compute SHA-256 hex hash of content.
fn sha256_hex(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Build a lock JSON for a project with given specs, nfrs, and test files.
/// Each spec entry: path -> (content, behaviors, dependencies, nfr_refs)
/// Each nfr entry: path -> content
/// Each test entry: path -> (content, covers)
fn build_lock(
    specs: &[(&str, &str, &[&str], &[&str], &[&str])],
    nfrs: &[(&str, &str)],
    tests: &[(&str, &str, &[&str])],
) -> String {
    let mut spec_entries = Vec::new();
    for (path, content, behaviors, deps, nfr_refs) in specs {
        let hash = sha256_hex(content);
        let behaviors_json: Vec<String> = behaviors.iter().map(|b| format!("\"{}\"", b)).collect();
        let deps_json: Vec<String> = deps.iter().map(|d| format!("\"{}\"", d)).collect();
        let nfrs_json: Vec<String> = nfr_refs.iter().map(|n| format!("\"{}\"", n)).collect();

        // Build test_files for this spec
        let mut test_file_entries = Vec::new();
        for (test_path, test_content, covers) in tests {
            // Check if any cover matches a behavior in this spec
            let relevant_covers: Vec<&str> = covers
                .iter()
                .filter(|c| behaviors.contains(c))
                .copied()
                .collect();
            if !relevant_covers.is_empty() {
                let covers_json: Vec<String> = relevant_covers
                    .iter()
                    .map(|c| format!("\"{}\"", c))
                    .collect();
                let test_hash = sha256_hex(test_content);
                test_file_entries.push(format!(
                    "        \"{}\": {{ \"hash\": \"{}\", \"covers\": [{}] }}",
                    test_path,
                    test_hash,
                    covers_json.join(", ")
                ));
            }
        }

        spec_entries.push(format!(
            "    \"{}\": {{\n      \"hash\": \"{}\",\n      \"behaviors\": [{}],\n      \"dependencies\": [{}],\n      \"nfrs\": [{}],\n      \"test_files\": {{\n{}\n      }}\n    }}",
            path,
            hash,
            behaviors_json.join(", "),
            deps_json.join(", "),
            nfrs_json.join(", "),
            test_file_entries.join(",\n")
        ));
    }

    let mut nfr_entries = Vec::new();
    for (path, content) in nfrs {
        let hash = sha256_hex(content);
        nfr_entries.push(format!("    \"{}\": {{ \"hash\": \"{}\" }}", path, hash));
    }

    format!(
        "{{\n  \"version\": 1,\n  \"specs\": {{\n{}\n  }},\n  \"nfrs\": {{\n{}\n  }}\n}}",
        spec_entries.join(",\n"),
        nfr_entries.join(",\n")
    )
}

/// Set up a standard project directory with specs/, tests/, and minter.lock.
/// Returns the TempDir handle (must stay alive).
fn setup_project(
    spec_files: &[(&str, &str)],
    nfr_files: &[(&str, &str)],
    test_files: &[(&str, &str)],
    lock_content: &str,
) -> TempDir {
    let dir = TempDir::new().unwrap();

    // Create specs/ directory and write spec files
    let specs_dir = dir.path().join("specs");
    fs::create_dir_all(&specs_dir).unwrap();
    for (name, content) in spec_files {
        let path = specs_dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
    }

    // Create nfr files (under specs/nfr/)
    if !nfr_files.is_empty() {
        let nfr_dir = specs_dir.join("nfr");
        fs::create_dir_all(&nfr_dir).unwrap();
        for (name, content) in nfr_files {
            fs::write(nfr_dir.join(name), content).unwrap();
        }
    }

    // Create tests/ directory and write test files
    let tests_dir = dir.path().join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    for (name, content) in test_files {
        fs::write(tests_dir.join(name), content).unwrap();
    }

    // Write minter.lock
    fs::write(dir.path().join("minter.lock"), lock_content).unwrap();

    dir
}

// ═══════════════════════════════════════════════════════════════
// ci-command.spec behaviors
// ═══════════════════════════════════════════════════════════════

// ── Happy paths ─────────────────────────────────────────

/// ci-command: ci-all-checks-pass
// @minter:e2e ci-all-checks-pass
#[test]
fn ci_all_checks_pass() {
    let spec_content = spec_two_behaviors();
    let test_content_a = "// @minter:unit do-thing\n";
    let test_content_b = "// @minter:e2e do-other\n";

    let lock = build_lock(
        &[(
            "specs/a.spec",
            spec_content,
            &["do-thing", "do-other"],
            &[],
            &[],
        )],
        &[],
        &[
            ("tests/a_test.rs", test_content_a, &["do-thing"]),
            ("tests/b_test.rs", test_content_b, &["do-other"]),
        ],
    );

    let dir = setup_project(
        &[("a.spec", spec_content)],
        &[],
        &[("a_test.rs", test_content_a), ("b_test.rs", test_content_b)],
        &lock,
    );

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("pass spec integrity (1 specs"))
        .stdout(predicate::str::contains("pass nfr integrity (0 nfrs)"))
        .stdout(predicate::str::contains(
            "pass dependency structure (0 edges)",
        ))
        .stdout(predicate::str::contains(
            "pass test integrity (2 test files)",
        ))
        .stdout(predicate::str::contains(
            "pass coverage (2/2 behaviors, 100%)",
        ))
        .stdout(predicate::str::contains("pass orphan (0 orphaned tags)"));
}

/// ci-command: ci-summary-shows-stats
// @minter:e2e ci-summary-shows-stats
#[test]
fn ci_summary_shows_stats() {
    // Create a project with known quantities:
    // 2 specs, 3 behaviors, 1 nfr, 1 dependency edge, 2 test files
    let spec_a = spec_with_dep("a", "1.0.0", "do-thing", "b");
    let spec_b = spec_with_nfr("b", "1.0.0", "b-thing", "performance");
    let nfr_content = nfr_performance();
    let test_content_a = "// @minter:unit do-thing\n";
    let test_content_b = "// @minter:unit b-thing\n";

    let dir = TempDir::new().unwrap();

    // Create specs/
    let spec_dir = dir.path().join("specs");
    fs::create_dir_all(&spec_dir).unwrap();
    fs::write(spec_dir.join("a.spec"), &spec_a).unwrap();
    fs::write(spec_dir.join("b.spec"), &spec_b).unwrap();

    // Create NFR
    let nfr_dir = spec_dir.join("nfr");
    fs::create_dir_all(&nfr_dir).unwrap();
    fs::write(nfr_dir.join("performance.nfr"), nfr_content).unwrap();

    // Create tests/
    let test_dir = dir.path().join("tests");
    fs::create_dir_all(&test_dir).unwrap();
    fs::write(test_dir.join("a_test.rs"), test_content_a).unwrap();
    fs::write(test_dir.join("b_test.rs"), test_content_b).unwrap();

    // Generate lock, then run CI
    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "pass spec integrity (2 specs, 1 nfrs)",
        ))
        .stdout(predicate::str::contains("pass nfr integrity (1 nfrs)"))
        .stdout(predicate::str::contains(
            "pass dependency structure (1 edges)",
        ))
        .stdout(predicate::str::contains(
            "pass test integrity (2 test files)",
        ))
        .stdout(predicate::str::contains(
            "pass coverage (2/2 behaviors, 100%)",
        ))
        .stdout(predicate::str::contains("pass orphan (0 orphaned tags)"));
}

/// ci-command: ci-reads-config
// @minter:e2e ci-reads-config
#[test]
fn ci_reads_config() {
    let dir = TempDir::new().unwrap();

    // Create non-default directories per config
    let spec_dir = dir.path().join("specifications");
    fs::create_dir_all(&spec_dir).unwrap();
    let test_dir = dir.path().join("src").join("tests");
    fs::create_dir_all(&test_dir).unwrap();

    let spec_content = spec_one_behavior("a", "1.0.0", "do-thing");
    fs::write(spec_dir.join("a.spec"), &spec_content).unwrap();

    let test_content = "// @minter:unit do-thing\n";
    fs::write(test_dir.join("a_test.rs"), test_content).unwrap();

    // Write minter.config.json
    fs::write(
        dir.path().join("minter.config.json"),
        r#"{ "specs": "specifications/", "tests": ["src/tests/"] }"#,
    )
    .unwrap();

    let lock = build_lock(
        &[(
            "specifications/a.spec",
            &spec_content,
            &["do-thing"],
            &[],
            &[],
        )],
        &[],
        &[("src/tests/a_test.rs", test_content, &["do-thing"])],
    );
    fs::write(dir.path().join("minter.lock"), &lock).unwrap();

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .success();
}

/// ci-command: ci-uses-default-conventions
// @minter:e2e ci-uses-default-conventions
#[test]
fn ci_uses_default_conventions() {
    let spec_content = spec_one_behavior("a", "1.0.0", "do-thing");
    let test_content = "// @minter:unit do-thing\n";

    let lock = build_lock(
        &[("specs/a.spec", &spec_content, &["do-thing"], &[], &[])],
        &[],
        &[("tests/a_test.rs", test_content, &["do-thing"])],
    );

    let dir = setup_project(
        &[("a.spec", &spec_content)],
        &[],
        &[("a_test.rs", test_content)],
        &lock,
    );

    // Ensure no config exists
    assert!(!dir.path().join("minter.config.json").exists());

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .success();
}

// ── Spec integrity ──────────────────────────────────────

/// ci-command: detect-spec-hash-mismatch
// @minter:e2e detect-spec-hash-mismatch
#[test]
fn detect_spec_hash_mismatch() {
    let original_content = spec_one_behavior("a", "1.0.0", "do-thing");
    let test_content = "// @minter:unit do-thing\n";

    // Lock is built with the original content
    let lock = build_lock(
        &[("specs/a.spec", &original_content, &["do-thing"], &[], &[])],
        &[],
        &[("tests/a_test.rs", test_content, &["do-thing"])],
    );

    let dir = setup_project(
        &[("a.spec", &original_content)],
        &[],
        &[("a_test.rs", test_content)],
        &lock,
    );

    // Modify the spec after lock was created
    let modified_content = spec_one_behavior("a", "2.0.0", "do-thing");
    fs::write(dir.path().join("specs").join("a.spec"), &modified_content).unwrap();

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("a.spec").and(
                predicate::str::contains("hash mismatch")
                    .or(predicate::str::contains("modified since last lock")),
            ),
        );
}

/// ci-command: detect-new-spec-not-in-lock
// @minter:e2e detect-new-spec-not-in-lock
#[test]
fn detect_new_spec_not_in_lock() {
    let spec_content = spec_one_behavior("a", "1.0.0", "do-thing");
    let test_content = "// @minter:unit do-thing\n";

    // Lock only contains specs/a.spec
    let lock = build_lock(
        &[("specs/a.spec", &spec_content, &["do-thing"], &[], &[])],
        &[],
        &[("tests/a_test.rs", test_content, &["do-thing"])],
    );

    let dir = setup_project(
        &[("a.spec", &spec_content)],
        &[],
        &[("a_test.rs", test_content)],
        &lock,
    );

    // Add a new spec not in the lock
    let new_spec = spec_one_behavior("new", "1.0.0", "new-thing");
    fs::write(dir.path().join("specs").join("new.spec"), &new_spec).unwrap();

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("new.spec").and(
                predicate::str::contains("not in lock").or(predicate::str::contains("unlocked")),
            ),
        );
}

/// ci-command: detect-deleted-spec-in-lock
// @minter:e2e detect-deleted-spec-in-lock
#[test]
fn detect_deleted_spec_in_lock() {
    let spec_a = spec_one_behavior("a", "1.0.0", "do-thing");
    let spec_removed = spec_one_behavior("removed", "1.0.0", "removed-thing");
    let test_content_a = "// @minter:unit do-thing\n";
    let test_content_r = "// @minter:unit removed-thing\n";

    // Lock contains both specs
    let lock = build_lock(
        &[
            ("specs/a.spec", &spec_a, &["do-thing"], &[], &[]),
            (
                "specs/removed.spec",
                &spec_removed,
                &["removed-thing"],
                &[],
                &[],
            ),
        ],
        &[],
        &[
            ("tests/a_test.rs", test_content_a, &["do-thing"]),
            ("tests/r_test.rs", test_content_r, &["removed-thing"]),
        ],
    );

    let dir = setup_project(
        &[("a.spec", &spec_a)],
        &[],
        &[("a_test.rs", test_content_a), ("r_test.rs", test_content_r)],
        &lock,
    );

    // specs/removed.spec does NOT exist on disk (we didn't create it)

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("removed.spec")
                .and(predicate::str::contains("missing").or(predicate::str::contains("deleted"))),
        );
}

// ── NFR integrity ───────────────────────────────────────

/// ci-command: detect-nfr-hash-mismatch
// @minter:e2e detect-nfr-hash-mismatch
#[test]
fn detect_nfr_hash_mismatch() {
    let nfr_content = nfr_performance();
    let spec_content = spec_with_nfr("a", "1.0.0", "do-thing", "performance");
    let test_content = "// @minter:unit do-thing\n";

    let lock = build_lock(
        &[(
            "specs/a.spec",
            &spec_content,
            &["do-thing"],
            &[],
            &["performance#api-latency"],
        )],
        &[("specs/nfr/performance.nfr", nfr_content)],
        &[("tests/a_test.rs", test_content, &["do-thing"])],
    );

    let dir = setup_project(
        &[("a.spec", &spec_content)],
        &[("performance.nfr", nfr_content)],
        &[("a_test.rs", test_content)],
        &lock,
    );

    // Modify the NFR file after lock
    let modified_nfr = nfr_content.replace("500ms", "200ms");
    fs::write(
        dir.path().join("specs").join("nfr").join("performance.nfr"),
        &modified_nfr,
    )
    .unwrap();

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("performance.nfr").and(
                predicate::str::contains("hash mismatch")
                    .or(predicate::str::contains("modified since last lock")),
            ),
        );
}

/// ci-command: detect-new-nfr-not-in-lock
// @minter:e2e detect-new-nfr-not-in-lock
#[test]
fn detect_new_nfr_not_in_lock() {
    let spec_content = spec_one_behavior("a", "1.0.0", "do-thing");
    let test_content = "// @minter:unit do-thing\n";

    // Lock has no NFR entries
    let lock = build_lock(
        &[("specs/a.spec", &spec_content, &["do-thing"], &[], &[])],
        &[],
        &[("tests/a_test.rs", test_content, &["do-thing"])],
    );

    let dir = setup_project(
        &[("a.spec", &spec_content)],
        &[],
        &[("a_test.rs", test_content)],
        &lock,
    );

    // Add an NFR file not in the lock
    let nfr_dir = dir.path().join("specs").join("nfr");
    fs::create_dir_all(&nfr_dir).unwrap();
    fs::write(
        nfr_dir.join("security.nfr"),
        "\
nfr security v1.0.0
title \"Security\"

description
  Security.

motivation
  Security.


constraint auth-required [rule]
  \"All endpoints require authentication\"

  rule \"Every HTTP endpoint must validate a bearer token\"

  verification
    environment staging, production
    benchmark \"auth test suite\"
    pass \"no unauthenticated access\"

  violation critical
  overridable no
",
    )
    .unwrap();

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("security.nfr").and(
                predicate::str::contains("not in lock").or(predicate::str::contains("unlocked")),
            ),
        );
}

// ── Dependency structure ────────────────────────────────

/// ci-command: detect-dependency-change
// @minter:e2e detect-dependency-change
#[test]
fn detect_dependency_change() {
    let spec_b = spec_one_behavior("b", "1.0.0", "b-thing");
    let spec_c = spec_one_behavior("c", "1.0.0", "c-thing");
    // Initially a depends on b
    let spec_a_original = spec_with_dep("a", "1.0.0", "do-thing", "b");
    let test_content_a = "// @minter:unit do-thing\n";
    let test_content_b = "// @minter:unit b-thing\n";
    let test_content_c = "// @minter:unit c-thing\n";

    let lock = build_lock(
        &[
            (
                "specs/a.spec",
                &spec_a_original,
                &["do-thing"],
                &["specs/b.spec"],
                &[],
            ),
            ("specs/b.spec", &spec_b, &["b-thing"], &[], &[]),
            ("specs/c.spec", &spec_c, &["c-thing"], &[], &[]),
        ],
        &[],
        &[
            ("tests/a_test.rs", test_content_a, &["do-thing"]),
            ("tests/b_test.rs", test_content_b, &["b-thing"]),
            ("tests/c_test.rs", test_content_c, &["c-thing"]),
        ],
    );

    let dir = setup_project(
        &[
            ("a.spec", &spec_a_original),
            ("b.spec", &spec_b),
            ("c.spec", &spec_c),
        ],
        &[],
        &[
            ("a_test.rs", test_content_a),
            ("b_test.rs", test_content_b),
            ("c_test.rs", test_content_c),
        ],
        &lock,
    );

    // Change a to depend on c instead of b
    let spec_a_changed = spec_with_dep("a", "1.0.0", "do-thing", "c");
    fs::write(dir.path().join("specs").join("a.spec"), &spec_a_changed).unwrap();

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("a.spec").and(
                predicate::str::contains("dependency").or(predicate::str::contains("structure")),
            ),
        );
}

// ── Test integrity ──────────────────────────────────────

/// ci-command: detect-test-hash-mismatch
// @minter:e2e detect-test-hash-mismatch
#[test]
fn detect_test_hash_mismatch() {
    let spec_content = spec_one_behavior("a", "1.0.0", "do-thing");
    let test_content = "// @minter:unit do-thing\nfn test_original() {}\n";

    let lock = build_lock(
        &[("specs/a.spec", &spec_content, &["do-thing"], &[], &[])],
        &[],
        &[("tests/a_test.rs", test_content, &["do-thing"])],
    );

    let dir = setup_project(
        &[("a.spec", &spec_content)],
        &[],
        &[("a_test.rs", test_content)],
        &lock,
    );

    // Modify the test file after lock
    let modified_test = "// @minter:unit do-thing\nfn test_modified() { assert!(true); }\n";
    fs::write(dir.path().join("tests").join("a_test.rs"), modified_test).unwrap();

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("a_test.rs").and(
                predicate::str::contains("hash mismatch")
                    .or(predicate::str::contains("modified since last lock")),
            ),
        );
}

/// ci-command: detect-new-test-not-in-lock
// @minter:e2e detect-new-test-not-in-lock
#[test]
fn detect_new_test_not_in_lock() {
    let spec_content = spec_one_behavior("a", "1.0.0", "do-thing");
    let test_content = "// @minter:unit do-thing\n";

    let lock = build_lock(
        &[("specs/a.spec", &spec_content, &["do-thing"], &[], &[])],
        &[],
        &[("tests/a_test.rs", test_content, &["do-thing"])],
    );

    let dir = setup_project(
        &[("a.spec", &spec_content)],
        &[],
        &[("a_test.rs", test_content)],
        &lock,
    );

    // Add a new test file not in the lock
    fs::write(
        dir.path().join("tests").join("new_test.rs"),
        "// @minter:unit do-thing\n",
    )
    .unwrap();

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("new_test.rs").and(
                predicate::str::contains("not in lock").or(predicate::str::contains("unlocked")),
            ),
        );
}

/// ci-command: detect-deleted-test-in-lock
// @minter:e2e detect-deleted-test-in-lock
#[test]
fn detect_deleted_test_in_lock() {
    let spec_content = spec_two_behaviors();
    let test_content_a = "// @minter:unit do-thing\n";
    let test_content_removed = "// @minter:unit do-other\n";

    // Lock contains both test files
    let lock = build_lock(
        &[(
            "specs/a.spec",
            spec_content,
            &["do-thing", "do-other"],
            &[],
            &[],
        )],
        &[],
        &[
            ("tests/a_test.rs", test_content_a, &["do-thing"]),
            ("tests/removed_test.rs", test_content_removed, &["do-other"]),
        ],
    );

    let dir = setup_project(
        &[("a.spec", spec_content)],
        &[],
        &[("a_test.rs", test_content_a)],
        // removed_test.rs is NOT created on disk
        &lock,
    );

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("removed_test.rs")
                .and(predicate::str::contains("missing").or(predicate::str::contains("deleted"))),
        );
}

// ── Coverage ────────────────────────────────────────────

/// ci-command: detect-uncovered-behavior
// @minter:e2e detect-uncovered-behavior
#[test]
fn detect_uncovered_behavior() {
    let spec_content = spec_two_behaviors();
    let test_content = "// @minter:unit do-thing\n";

    // Lock reflects that only do-thing is covered
    let lock = build_lock(
        &[(
            "specs/a.spec",
            spec_content,
            &["do-thing", "do-other"],
            &[],
            &[],
        )],
        &[],
        &[("tests/a_test.rs", test_content, &["do-thing"])],
    );

    let dir = setup_project(
        &[("a.spec", spec_content)],
        &[],
        &[("a_test.rs", test_content)],
        &lock,
    );

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("do-other").and(
            predicate::str::contains("uncovered").or(predicate::str::contains("no test coverage")),
        ));
}

// ── Orphan detection ────────────────────────────────────

/// ci-command: detect-orphaned-tag
// @minter:e2e detect-orphaned-tag
#[test]
fn detect_orphaned_tag() {
    let spec_content = spec_one_behavior("a", "1.0.0", "do-thing");
    let test_content_valid = "// @minter:unit do-thing\n";
    let test_content_orphan = "// @minter:unit nonexistent-behavior\n";

    let lock = build_lock(
        &[("specs/a.spec", &spec_content, &["do-thing"], &[], &[])],
        &[],
        &[
            ("tests/a_test.rs", test_content_valid, &["do-thing"]),
            (
                "tests/orphan_test.rs",
                test_content_orphan,
                &["nonexistent-behavior"],
            ),
        ],
    );

    let dir = setup_project(
        &[("a.spec", &spec_content)],
        &[],
        &[
            ("a_test.rs", test_content_valid),
            ("orphan_test.rs", test_content_orphan),
        ],
        &lock,
    );

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("nonexistent-behavior")
                .and(predicate::str::contains("orphan").or(predicate::str::contains("unknown"))),
        );
}

// ── Missing / corrupted lock ────────────────────────────

/// ci-command: reject-missing-lock
// @minter:e2e reject-missing-lock
#[test]
fn reject_missing_lock() {
    let dir = TempDir::new().unwrap();
    let specs_dir = dir.path().join("specs");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::write(
        specs_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    // No minter.lock file

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("minter.lock")
                .and(predicate::str::contains("not found").or(predicate::str::contains("missing")))
                .and(predicate::str::contains("minter lock")),
        );
}

/// ci-command: reject-corrupted-lock
// @minter:e2e reject-corrupted-lock
#[test]
fn reject_corrupted_lock() {
    let dir = TempDir::new().unwrap();
    let specs_dir = dir.path().join("specs");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::write(
        specs_dir.join("a.spec"),
        spec_one_behavior("a", "1.0.0", "do-thing"),
    )
    .unwrap();

    // Write invalid JSON as minter.lock
    fs::write(dir.path().join("minter.lock"), "this is not valid json {{{").unwrap();

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("minter.lock")
                .and(predicate::str::contains("invalid").or(predicate::str::contains("corrupted"))),
        );
}

// ── Report format ───────────────────────────────────────

/// ci-command: report-all-failures
// @minter:e2e report-all-failures
#[test]
fn report_all_failures() {
    let spec_content = spec_two_behaviors();
    let test_content = "// @minter:unit do-thing\n";

    // Lock with original content
    let lock = build_lock(
        &[(
            "specs/a.spec",
            spec_content,
            &["do-thing", "do-other"],
            &[],
            &[],
        )],
        &[],
        &[("tests/a_test.rs", test_content, &["do-thing"])],
    );

    let dir = setup_project(
        &[("a.spec", spec_content)],
        &[],
        &[("a_test.rs", test_content)],
        &lock,
    );

    // Modify the spec (hash mismatch)
    let modified_spec = spec_two_behaviors().replace("v1.0.0", "v2.0.0");
    fs::write(dir.path().join("specs").join("a.spec"), &modified_spec).unwrap();

    // Modify the test file (hash mismatch)
    let modified_test = "// @minter:unit do-thing\n// modified\n";
    fs::write(dir.path().join("tests").join("a_test.rs"), modified_test).unwrap();

    // do-other has no coverage (coverage failure)

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("a.spec")
                .and(predicate::str::contains("a_test.rs"))
                .and(predicate::str::contains("do-other")),
        );
}

/// ci-command: report-check-summary
// @minter:e2e report-check-summary
#[test]
fn report_check_summary() {
    let spec_content = spec_one_behavior("a", "1.0.0", "do-thing");
    let test_content = "// @minter:unit do-thing\n";

    // Lock with correct spec hash but wrong test hash
    let lock = build_lock(
        &[("specs/a.spec", &spec_content, &["do-thing"], &[], &[])],
        &[],
        &[("tests/a_test.rs", test_content, &["do-thing"])],
    );

    let dir = setup_project(
        &[("a.spec", &spec_content)],
        &[],
        &[("a_test.rs", test_content)],
        &lock,
    );

    // Modify only the test file so spec integrity passes but test integrity fails
    let modified_test = "// @minter:unit do-thing\n// changed\n";
    fs::write(dir.path().join("tests").join("a_test.rs"), modified_test).unwrap();

    // The output should show each check with pass/fail status and stats for passing checks
    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("pass spec integrity (1 specs)"))
        .stdout(predicate::str::contains("FAIL test integrity"));
}

// ── Edge cases ──────────────────────────────────────────

/// ci-command: ignore-untagged-test-files
// @minter:e2e ignore-untagged-test-files
#[test]
fn ignore_untagged_test_files() {
    let spec_content = spec_one_behavior("a", "1.0.0", "do-thing");
    let test_content = "// @minter:unit do-thing\n";

    let lock = build_lock(
        &[("specs/a.spec", &spec_content, &["do-thing"], &[], &[])],
        &[],
        &[("tests/a_test.rs", test_content, &["do-thing"])],
    );

    let dir = setup_project(
        &[("a.spec", &spec_content)],
        &[],
        &[("a_test.rs", test_content)],
        &lock,
    );

    // Add an untagged helper file and modify it — should not cause failure
    fs::write(
        dir.path().join("tests").join("helper.rs"),
        "fn helper() { /* no @minter tags */ }\n",
    )
    .unwrap();

    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .success();
}

/// ci-command: ci-multi-test-dirs
// @minter:e2e ci-multi-test-dirs
#[test]
fn ci_multi_test_dirs() {
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
    let spec_content = spec_one_behavior("a", "1.0.0", "do-thing");
    fs::write(spec_dir.join("a.spec"), &spec_content).unwrap();

    // Create NFR for benchmark references
    let nfr_dir = spec_dir.join("nfr");
    fs::create_dir(&nfr_dir).unwrap();
    fs::write(nfr_dir.join("performance.nfr"), nfr_performance()).unwrap();

    // Create tests/ with a unit test
    let test_dir = dir.path().join("tests");
    fs::create_dir(&test_dir).unwrap();
    let test_content = "// @minter:unit do-thing\n";
    fs::write(test_dir.join("a_test.rs"), test_content).unwrap();

    // Create benches/ with a benchmark test
    let bench_dir = dir.path().join("benches");
    fs::create_dir(&bench_dir).unwrap();
    let bench_content = "// @minter:benchmark #performance#api-latency\n";
    fs::write(bench_dir.join("perf_test.rs"), bench_content).unwrap();

    // Generate the lock file first
    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    // CI should pass — all files from all configured dirs are in the lock
    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("pass test integrity"));
}
