mod common;

use std::fs;

use common::minter;
use predicates::prelude::*;
use tempfile::TempDir;

// ── Spec fixtures ───────────────────────────────────────

fn valid_spec(name: &str) -> String {
    format!(
        "\
spec {name} v1.0.0
title \"{name}\"

description
  A test spec.

motivation
  Testing.

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

fn valid_spec_two_behaviors(name: &str) -> String {
    format!(
        "\
spec {name} v1.0.0
title \"{name}\"

description
  A test spec.

motivation
  Testing.

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

/// Set up a project directory with a specs/ dir containing .spec files
/// and a tests/ dir containing files with @minter tags.
fn project_with_defaults(specs: &[(&str, &str)], tests: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().unwrap();

    let specs_dir = dir.path().join("specs");
    fs::create_dir(&specs_dir).unwrap();
    for (name, content) in specs {
        fs::write(specs_dir.join(format!("{}.spec", name)), content).unwrap();
    }

    let tests_dir = dir.path().join("tests");
    fs::create_dir(&tests_dir).unwrap();
    for (name, content) in tests {
        fs::write(tests_dir.join(name), content).unwrap();
    }

    dir
}

/// Set up a project directory with custom spec/test dirs and a minter.config.json.
fn project_with_config(config_json: &str, dirs_and_files: &[(&str, &[(&str, &str)])]) -> TempDir {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("minter.config.json"), config_json).unwrap();

    for (dir_name, files) in dirs_and_files {
        let sub_dir = dir.path().join(dir_name);
        fs::create_dir_all(&sub_dir).unwrap();
        for (file_name, content) in *files {
            fs::write(sub_dir.join(file_name), content).unwrap();
        }
    }

    dir
}

// ═══════════════════════════════════════════════════════════════
// Config loading — happy path
// ═══════════════════════════════════════════════════════════════

/// config: load-default-conventions
// @minter:e2e load-default-conventions
#[test]
fn load_default_conventions_validate() {
    let spec = valid_spec("my-feature");
    let dir = project_with_defaults(&[("my-feature", &spec)], &[]);

    // Invoke validate WITHOUT explicit path — should discover specs/ by convention
    minter()
        .arg("validate")
        .current_dir(dir.path())
        .assert()
        .success();
}

/// config: load-default-conventions
// @minter:e2e load-default-conventions
#[test]
fn load_default_conventions_coverage() {
    let spec = valid_spec_two_behaviors("my-feature");
    let dir = project_with_defaults(
        &[("my-feature", &spec)],
        &[(
            "a.test.ts",
            "// @minter:e2e do-thing\n// @minter:e2e do-other\n",
        )],
    );

    // Invoke coverage WITHOUT explicit paths — should discover specs/ and tests/
    minter()
        .arg("coverage")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("2/2"));
}

/// config: load-config-file
// @minter:e2e load-config-file
#[test]
fn load_config_file_validate() {
    let spec = valid_spec("custom-spec");
    let config = r#"{ "specs": "specifications/", "tests": ["tests/e2e/", "tests/unit/"] }"#;

    let dir = project_with_config(
        config,
        &[
            ("specifications", &[("custom-spec.spec", spec.as_str())]),
            ("tests/e2e", &[("a.test.ts", "// @minter:e2e do-thing\n")]),
            ("tests/unit", &[]),
        ],
    );

    // Invoke validate without explicit path — should use specifications/ from config
    minter()
        .arg("validate")
        .current_dir(dir.path())
        .assert()
        .success();
}

/// config: load-config-file
// @minter:e2e load-config-file
#[test]
fn load_config_file_coverage() {
    let spec = valid_spec("custom-spec");
    let config = r#"{ "specs": "specifications/", "tests": ["tests/e2e/", "tests/unit/"] }"#;

    let dir = project_with_config(
        config,
        &[
            ("specifications", &[("custom-spec.spec", spec.as_str())]),
            ("tests/e2e", &[("a.test.ts", "// @minter:e2e do-thing\n")]),
            ("tests/unit", &[]),
        ],
    );

    // Invoke coverage without explicit paths — should use config paths
    minter()
        .arg("coverage")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1/1"));
}

/// config: config-specs-string
// @minter:e2e config-specs-string
#[test]
fn config_specs_string() {
    let spec = valid_spec("feat");
    let config = r#"{ "specs": "my-specs/" }"#;

    let dir = project_with_config(config, &[("my-specs", &[("feat.spec", spec.as_str())])]);

    // validate should discover specs from my-specs/ via config
    minter()
        .arg("validate")
        .current_dir(dir.path())
        .assert()
        .success();
}

/// config: config-tests-array
// @minter:e2e config-tests-array
#[test]
fn config_tests_array() {
    let spec = valid_spec_two_behaviors("feat");
    let config = r#"{ "specs": "specs/", "tests": ["tests/unit/", "tests/e2e/", "benches/"] }"#;

    let dir = project_with_config(
        config,
        &[
            ("specs", &[("feat.spec", spec.as_str())]),
            ("tests/unit", &[("a.test.ts", "// @minter:unit do-thing\n")]),
            ("tests/e2e", &[("b.test.ts", "// @minter:e2e do-other\n")]),
            ("benches", &[]),
        ],
    );

    // coverage should scan all three test directories
    minter()
        .arg("coverage")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("2/2"));
}

/// config: config-tests-single-string
// @minter:e2e config-tests-single-string
#[test]
fn config_tests_single_string() {
    let spec = valid_spec("feat");
    let config = r#"{ "specs": "specs/", "tests": "tests/" }"#;

    let dir = project_with_config(
        config,
        &[
            ("specs", &[("feat.spec", spec.as_str())]),
            ("tests", &[("a.test.ts", "// @minter:e2e do-thing\n")]),
        ],
    );

    // coverage should scan tests/ as a single string
    minter()
        .arg("coverage")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1/1"));
}

/// config: config-partial-override
// @minter:e2e config-partial-override
#[test]
fn config_partial_override_specs_default() {
    let spec = valid_spec("feat");
    let config = r#"{ "tests": ["src/"] }"#;

    let dir = project_with_config(
        config,
        &[
            // specs/ is default — not mentioned in config
            ("specs", &[("feat.spec", spec.as_str())]),
            ("src", &[("lib.rs", "// @minter:e2e do-thing\n")]),
        ],
    );

    // No specs field in config — should fall back to default specs/
    minter()
        .arg("validate")
        .current_dir(dir.path())
        .assert()
        .success();
}

/// config: config-partial-override
// @minter:e2e config-partial-override
#[test]
fn config_partial_override_tests_default() {
    let spec = valid_spec("feat");
    let config = r#"{ "specs": "specifications/" }"#;

    let dir = project_with_config(
        config,
        &[
            ("specifications", &[("feat.spec", spec.as_str())]),
            // tests/ is default — not mentioned in config
            ("tests", &[("a.test.ts", "// @minter:e2e do-thing\n")]),
        ],
    );

    // No tests field in config — should fall back to default tests/
    minter()
        .arg("coverage")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1/1"));
}

// ═══════════════════════════════════════════════════════════════
// CLI override
// ═══════════════════════════════════════════════════════════════

/// config: cli-args-override-config
// @minter:e2e cli-args-override-config
#[test]
fn cli_args_override_config_validate() {
    let spec_in_config_dir = valid_spec("config-spec");
    let spec_in_other_dir = valid_spec("other-spec");
    let config = r#"{ "specs": "specifications/" }"#;

    let dir = project_with_config(
        config,
        &[
            (
                "specifications",
                &[("config-spec.spec", spec_in_config_dir.as_str())],
            ),
            (
                "other-specs",
                &[("other-spec.spec", spec_in_other_dir.as_str())],
            ),
        ],
    );

    // First: config-based validate without args should discover specifications/
    minter()
        .arg("validate")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("config-spec"));

    // Second: explicit CLI path should override config — validate other-specs/ not specifications/
    minter()
        .arg("validate")
        .arg(dir.path().join("other-specs"))
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("other-spec")
                .and(predicate::str::contains("config-spec").not()),
        );
}

/// config: cli-args-override-config
// @minter:e2e cli-args-override-config
#[test]
fn cli_args_override_config_coverage() {
    let spec = valid_spec("feat");
    let config = r#"{ "specs": "specifications/", "tests": ["custom-tests/"] }"#;

    let dir = project_with_config(
        config,
        &[
            ("specifications", &[("feat.spec", spec.as_str())]),
            (
                "custom-tests",
                &[("a.test.ts", "// @minter:e2e do-thing\n")],
            ),
            ("other-specs", &[("feat.spec", spec.as_str())]),
        ],
    );

    // First: config-based coverage without args should use specifications/ and custom-tests/
    minter()
        .arg("coverage")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1/1"));

    // Second: explicit CLI spec_path should override config
    minter()
        .arg("coverage")
        .arg(dir.path().join("other-specs"))
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("feat"));
}

// ═══════════════════════════════════════════════════════════════
// Error cases
// ═══════════════════════════════════════════════════════════════

/// config: reject-invalid-json
// @minter:e2e reject-invalid-json
#[test]
fn reject_invalid_json_validate() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("minter.config.json"), "{ not valid json }").unwrap();

    minter()
        .arg("validate")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("minter.config.json")
                .and(predicate::str::contains("invalid").or(predicate::str::contains("parse"))),
        );
}

/// config: reject-invalid-json
// @minter:e2e reject-invalid-json
#[test]
fn reject_invalid_json_coverage() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("minter.config.json"), "{ not valid json }").unwrap();

    minter()
        .arg("coverage")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("minter.config.json")
                .and(predicate::str::contains("invalid").or(predicate::str::contains("parse"))),
        );
}

/// config: reject-invalid-specs-type
// @minter:e2e reject-invalid-specs-type
#[test]
fn reject_invalid_specs_type_validate() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("minter.config.json"), r#"{ "specs": 42 }"#).unwrap();

    minter()
        .arg("validate")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("specs").and(predicate::str::contains("string")));
}

/// config: reject-invalid-specs-type
// @minter:e2e reject-invalid-specs-type
#[test]
fn reject_invalid_specs_type_coverage() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("minter.config.json"), r#"{ "specs": 42 }"#).unwrap();

    minter()
        .arg("coverage")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("specs").and(predicate::str::contains("string")));
}

/// config: reject-invalid-tests-type
// @minter:e2e reject-invalid-tests-type
#[test]
fn reject_invalid_tests_type_validate() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("minter.config.json"),
        r#"{ "tests": { "unit": "tests/" } }"#,
    )
    .unwrap();

    minter()
        .arg("validate")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("tests"));
}

/// config: reject-invalid-tests-type
// @minter:e2e reject-invalid-tests-type
#[test]
fn reject_invalid_tests_type_coverage() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("minter.config.json"),
        r#"{ "tests": { "unit": "tests/" } }"#,
    )
    .unwrap();

    minter()
        .arg("coverage")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("tests"));
}

/// config: reject-nonexistent-specs-dir
// @minter:e2e reject-nonexistent-specs-dir
#[test]
fn reject_nonexistent_specs_dir_validate() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("minter.config.json"),
        r#"{ "specs": "nonexistent/" }"#,
    )
    .unwrap();

    minter()
        .arg("validate")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent").and(
            predicate::str::contains("not found").or(predicate::str::contains("does not exist")),
        ));
}

/// config: reject-nonexistent-specs-dir
// @minter:e2e reject-nonexistent-specs-dir
#[test]
fn reject_nonexistent_specs_dir_coverage() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("minter.config.json"),
        r#"{ "specs": "nonexistent/" }"#,
    )
    .unwrap();

    minter()
        .arg("coverage")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent").and(
            predicate::str::contains("not found").or(predicate::str::contains("does not exist")),
        ));
}

/// config: reject-nonexistent-tests-dir
// @minter:e2e reject-nonexistent-tests-dir
#[test]
fn reject_nonexistent_tests_dir_validate() {
    let dir = TempDir::new().unwrap();
    let tests_dir = dir.path().join("tests");
    fs::create_dir(&tests_dir).unwrap();

    fs::write(
        dir.path().join("minter.config.json"),
        r#"{ "tests": ["tests/", "nonexistent/"] }"#,
    )
    .unwrap();

    minter()
        .arg("validate")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

/// config: reject-nonexistent-tests-dir
// @minter:e2e reject-nonexistent-tests-dir
#[test]
fn reject_nonexistent_tests_dir_coverage() {
    let dir = TempDir::new().unwrap();
    let tests_dir = dir.path().join("tests");
    fs::create_dir(&tests_dir).unwrap();

    fs::write(
        dir.path().join("minter.config.json"),
        r#"{ "tests": ["tests/", "nonexistent/"] }"#,
    )
    .unwrap();

    minter()
        .arg("coverage")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

/// config: reject-unknown-fields
// @minter:e2e reject-unknown-fields
#[test]
fn reject_unknown_fields_validate() {
    let dir = TempDir::new().unwrap();
    let specs_dir = dir.path().join("specs");
    fs::create_dir(&specs_dir).unwrap();

    fs::write(
        dir.path().join("minter.config.json"),
        r#"{ "specs": "specs/", "output": "dist/" }"#,
    )
    .unwrap();

    minter()
        .arg("validate")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("output").and(predicate::str::contains("unknown")));
}

/// config: reject-unknown-fields
// @minter:e2e reject-unknown-fields
#[test]
fn reject_unknown_fields_coverage() {
    let dir = TempDir::new().unwrap();
    let specs_dir = dir.path().join("specs");
    fs::create_dir(&specs_dir).unwrap();

    fs::write(
        dir.path().join("minter.config.json"),
        r#"{ "specs": "specs/", "output": "dist/" }"#,
    )
    .unwrap();

    minter()
        .arg("coverage")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("output").and(predicate::str::contains("unknown")));
}

// ═══════════════════════════════════════════════════════════════
// Edge cases
// ═══════════════════════════════════════════════════════════════

/// config: empty-config-uses-defaults
// @minter:e2e empty-config-uses-defaults
#[test]
fn empty_config_uses_defaults_validate() {
    let spec = valid_spec("feat");
    let dir = project_with_config("{}", &[("specs", &[("feat.spec", spec.as_str())])]);

    // Empty config {} — should fall back to specs/ and tests/ defaults
    minter()
        .arg("validate")
        .current_dir(dir.path())
        .assert()
        .success();
}

/// config: empty-config-uses-defaults
// @minter:e2e empty-config-uses-defaults
#[test]
fn empty_config_uses_defaults_coverage() {
    let spec = valid_spec("feat");
    let dir = project_with_config(
        "{}",
        &[
            ("specs", &[("feat.spec", spec.as_str())]),
            ("tests", &[("a.test.ts", "// @minter:e2e do-thing\n")]),
        ],
    );

    // Empty config {} — should fall back to specs/ and tests/ defaults
    minter()
        .arg("coverage")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1/1"));
}

/// config: no-default-dirs-without-config
// @minter:e2e no-default-dirs-without-config
#[test]
fn no_default_dirs_without_config_validate() {
    let dir = TempDir::new().unwrap();
    // No minter.config.json, no specs/, no tests/

    minter()
        .arg("validate")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("specs").and(
            predicate::str::contains("not found").or(predicate::str::contains("does not exist")),
        ));
}

/// config: no-default-dirs-without-config
// @minter:e2e no-default-dirs-without-config
#[test]
fn no_default_dirs_without_config_coverage() {
    let dir = TempDir::new().unwrap();
    // No minter.config.json, no specs/, no tests/

    minter()
        .arg("coverage")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("specs").and(
            predicate::str::contains("not found").or(predicate::str::contains("does not exist")),
        ));
}

/// config: no-default-dirs-without-config
// @minter:e2e no-default-dirs-without-config
#[test]
fn no_default_dirs_without_config_graph() {
    let dir = TempDir::new().unwrap();
    // No minter.config.json, no specs/, no tests/

    minter()
        .arg("graph")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("specs").and(
            predicate::str::contains("not found").or(predicate::str::contains("does not exist")),
        ));
}
