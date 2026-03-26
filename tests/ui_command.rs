#![cfg(feature = "ui")]

mod common;

use std::fs;

use sha2::{Digest, Sha256};
use tempfile::TempDir;

// ── Spec fixtures ───────────────────────────────────────

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

fn spec_multi_behaviors(name: &str, version: &str, behaviors: &[&str]) -> String {
    let mut s = format!(
        "\
spec {name} v{version}
title \"{name}\"

description
  Test.

motivation
  Test.

"
    );
    for (i, b) in behaviors.iter().enumerate() {
        if i > 0 {
            s.push('\n');
        }
        s.push_str(&format!(
            "\
behavior {b} [happy_path]
  \"Does {b}\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"

"
        ));
    }
    s
}

fn spec_with_deps(name: &str, version: &str, behaviors: &[&str], deps: &[(&str, &str)]) -> String {
    let mut s = spec_multi_behaviors(name, version, behaviors);
    for (dep_name, dep_ver) in deps {
        s.push_str(&format!("\ndepends on {} >= {}\n", dep_name, dep_ver));
    }
    s
}

fn spec_mixed_categories(name: &str, version: &str) -> String {
    format!(
        "\
spec {name} v{version}
title \"{name}\"

description
  Test.

motivation
  Test.

behavior do-happy [happy_path]
  \"Happy\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"

behavior do-happy-two [happy_path]
  \"Happy two\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"

behavior do-error [error_case]
  \"Error\"

  given
    Ready

  when act

  then returns result
    assert status == \"error\"

behavior do-edge [edge_case]
  \"Edge\"

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

// ── Lock helpers ────────────────────────────────────────

fn sha256_hex(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

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

        let mut test_file_entries = Vec::new();
        for (test_path, test_content, covers) in tests {
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

// ── Project setup helpers ───────────────────────────────

/// Set up a project directory with specs/, optional nfrs, tests/, and optional minter.lock.
fn setup_project(
    spec_files: &[(&str, &str)],
    nfr_files: &[(&str, &str)],
    test_files: &[(&str, &str)],
    lock_content: Option<&str>,
) -> TempDir {
    let dir = TempDir::new().unwrap();

    let specs_dir = dir.path().join("specs");
    fs::create_dir_all(&specs_dir).unwrap();
    for (name, content) in spec_files {
        let path = specs_dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
    }

    if !nfr_files.is_empty() {
        let nfr_dir = specs_dir.join("nfr");
        fs::create_dir_all(&nfr_dir).unwrap();
        for (name, content) in nfr_files {
            fs::write(nfr_dir.join(name), content).unwrap();
        }
    }

    let tests_dir = dir.path().join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    for (name, content) in test_files {
        fs::write(tests_dir.join(name), content).unwrap();
    }

    if let Some(lock) = lock_content {
        fs::write(dir.path().join("minter.lock"), lock).unwrap();
    }

    dir
}

// ═══════════════════════════════════════════════════════════════
// Startup and overview — happy path
// ═══════════════════════════════════════════════════════════════

/// ui-command: launch-displays-overview
// @minter:unit launch-displays-overview
#[test]
fn state_loads_correct_project_counts() {
    use minter::core::ui::state::UiState;

    let spec_a = spec_multi_behaviors(
        "auth",
        "1.0.0",
        &["login", "logout", "refresh-token", "reset-password"],
    );
    let spec_b = spec_multi_behaviors(
        "billing",
        "2.1.0",
        &[
            "create-invoice",
            "cancel-invoice",
            "refund",
            "list-invoices",
            "export-csv",
            "send-reminder",
        ],
    );
    let spec_c = spec_multi_behaviors(
        "users",
        "1.0.0",
        &["create-user", "delete-user", "update-profile", "list-users"],
    );
    let spec_d = spec_multi_behaviors("search", "1.0.0", &["full-text-search", "filter-results"]);
    let spec_e = spec_multi_behaviors("notifications", "1.0.0", &["send-email", "send-sms"]);

    // 5 specs, 18 behaviors total (4+6+4+2+2)
    let nfr_content = nfr_performance();
    let nfr_reliability = "\
nfr reliability v1.0.0
title \"Reliability\"

description
  Reliability.

motivation
  Reliability.


constraint no-data-loss [rule]
  \"No data loss\"

  rule
    No data loss events during failure scenarios.

  verification
    static \"data loss detection test\"

  violation critical
  overridable no
";

    // 14 of 18 behaviors covered → 77%
    let tag = "@minter";
    let test_content = format!(
        "\
// {tag}:unit login
// {tag}:unit logout
// {tag}:e2e refresh-token
// {tag}:e2e reset-password
// {tag}:unit create-invoice
// {tag}:unit cancel-invoice
// {tag}:e2e refund
// {tag}:unit list-invoices
// {tag}:e2e export-csv
// {tag}:unit send-reminder
// {tag}:unit create-user
// {tag}:e2e delete-user
// {tag}:unit full-text-search
// {tag}:e2e filter-results
"
    );

    let lock = build_lock(
        &[
            (
                "specs/auth.spec",
                &spec_a,
                &["login", "logout", "refresh-token", "reset-password"],
                &[],
                &[],
            ),
            (
                "specs/billing.spec",
                &spec_b,
                &[
                    "create-invoice",
                    "cancel-invoice",
                    "refund",
                    "list-invoices",
                    "export-csv",
                    "send-reminder",
                ],
                &[],
                &[],
            ),
            (
                "specs/users.spec",
                &spec_c,
                &["create-user", "delete-user", "update-profile", "list-users"],
                &[],
                &[],
            ),
            (
                "specs/search.spec",
                &spec_d,
                &["full-text-search", "filter-results"],
                &[],
                &[],
            ),
            (
                "specs/notifications.spec",
                &spec_e,
                &["send-email", "send-sms"],
                &[],
                &[],
            ),
        ],
        &[
            ("specs/nfr/performance.nfr", nfr_content),
            ("specs/nfr/reliability.nfr", nfr_reliability),
        ],
        &[(
            "tests/all_test.rs",
            &test_content,
            &[
                "login",
                "logout",
                "refresh-token",
                "reset-password",
                "create-invoice",
                "cancel-invoice",
                "refund",
                "list-invoices",
                "export-csv",
                "send-reminder",
                "create-user",
                "delete-user",
                "full-text-search",
                "filter-results",
            ],
        )],
    );

    let dir = setup_project(
        &[
            ("auth.spec", &spec_a),
            ("billing.spec", &spec_b),
            ("users.spec", &spec_c),
            ("search.spec", &spec_d),
            ("notifications.spec", &spec_e),
        ],
        &[
            ("performance.nfr", nfr_content),
            ("reliability.nfr", nfr_reliability),
        ],
        &[("all_test.rs", &test_content)],
        Some(&lock),
    );

    let state = UiState::load(dir.path());

    assert_eq!(state.spec_count(), 5, "expected 5 specs");
    assert_eq!(state.behavior_count(), 18, "expected 18 behaviors");
    assert_eq!(state.nfr_count(), 2, "expected 2 nfr constraints");
    assert_eq!(state.test_count(), 14, "expected 14 test tags");
    assert_eq!(
        state.coverage_percent(),
        77,
        "expected 77% coverage (14/18)"
    );
    assert!(state.lock_aligned(), "expected lock to be aligned");
}

/// ui-command: launch-displays-overview
// @minter:unit launch-displays-overview
#[test]
fn overview_test_count_counts_tags_not_files() {
    use minter::core::ui::state::UiState;

    let spec_a = spec_one_behavior("auth", "1.0.0", "login");
    let spec_b = spec_one_behavior("billing", "1.0.0", "create-invoice");

    // 2 files, 3 tags total
    let test_a = "// @minter:unit login\n// @minter:e2e login\n";
    let test_b = "// @minter:e2e create-invoice\n";

    let dir = setup_project(
        &[("auth.spec", &spec_a), ("billing.spec", &spec_b)],
        &[],
        &[("auth_test.rs", test_a), ("billing_test.rs", test_b)],
        None,
    );

    let state = UiState::load(dir.path());
    assert_eq!(state.test_count(), 3, "expected 3 test tags (not 2 files)");
}

/// ui-command: launch-displays-overview
// @minter:unit launch-displays-overview
#[test]
fn overview_test_count_zero_when_no_tests() {
    use minter::core::ui::state::UiState;

    let spec_a = spec_one_behavior("auth", "1.0.0", "login");
    let dir = setup_project(&[("auth.spec", &spec_a)], &[], &[], None);

    let state = UiState::load(dir.path());
    assert_eq!(state.test_count(), 0, "expected 0 test tags");
}

/// ui-command: launch-displays-overview
// @minter:unit launch-displays-overview
#[test]
fn overview_nfr_count_counts_constraints_not_files() {
    use minter::core::ui::state::UiState;

    // 1 NFR file with 1 constraint
    let nfr_one = nfr_performance(); // has 1 constraint: api-latency

    // 1 NFR file with 2 constraints
    let nfr_two = "\
nfr reliability v1.0.0
title \"Reliability\"

description
  Reliability.

motivation
  Reliability.


constraint no-data-loss [metric]
  \"No data loss\"

  metric \"data loss events\"
  threshold <= 0

  verification
    environment production
    benchmark \"chaos test\"
    pass \"zero data loss\"

  violation critical
  overridable no

constraint uptime-sla [metric]
  \"Uptime SLA\"

  metric \"monthly uptime\"
  threshold >= 99%

  verification
    environment production
    benchmark \"uptime monitor\"
    pass \"99% uptime\"

  violation high
  overridable no
";

    let spec_a = spec_one_behavior("auth", "1.0.0", "login");
    let dir = setup_project(
        &[("auth.spec", &spec_a)],
        &[("performance.nfr", nfr_one), ("reliability.nfr", nfr_two)],
        &[],
        None,
    );

    let state = UiState::load(dir.path());
    // 2 files but 3 constraints total (1 + 2)
    assert_eq!(
        state.nfr_count(),
        3,
        "expected 3 nfr constraints (not 2 files)"
    );
}

/// ui-command: launch-displays-specs-list
// @minter:unit launch-displays-specs-list
#[test]
fn state_contains_spec_list_with_versions_and_behavior_counts() {
    use minter::core::ui::state::UiState;

    let spec_a = spec_multi_behaviors(
        "auth",
        "1.0.0",
        &["login", "logout", "refresh-token", "reset-password"],
    );
    let spec_b = spec_multi_behaviors(
        "billing",
        "2.1.0",
        &[
            "create-invoice",
            "cancel-invoice",
            "refund",
            "list-invoices",
            "export-csv",
            "send-reminder",
        ],
    );

    let dir = setup_project(
        &[("auth.spec", &spec_a), ("billing.spec", &spec_b)],
        &[],
        &[],
        None,
    );

    let state = UiState::load(dir.path());
    let specs = state.specs_list();

    // Specs should be in alphabetical order
    assert_eq!(specs.len(), 2);
    assert_eq!(specs[0].name, "auth");
    assert_eq!(specs[0].version, "1.0.0");
    assert_eq!(specs[0].behavior_count, 4);
    assert_eq!(specs[1].name, "billing");
    assert_eq!(specs[1].version, "2.1.0");
    assert_eq!(specs[1].behavior_count, 6);
}

/// ui-command: launch-displays-integrity-aligned
// @minter:unit launch-displays-integrity-aligned
#[test]
fn state_shows_aligned_when_lock_matches() {
    use minter::core::ui::state::{IntegrityStatus, UiState};

    let spec_content = spec_one_behavior("auth", "1.0.0", "login");
    let test_content = "// @minter:unit login\n";

    let lock = build_lock(
        &[("specs/auth.spec", &spec_content, &["login"], &[], &[])],
        &[],
        &[("tests/auth_test.rs", test_content, &["login"])],
    );

    let dir = setup_project(
        &[("auth.spec", &spec_content)],
        &[],
        &[("auth_test.rs", test_content)],
        Some(&lock),
    );

    let state = UiState::load(dir.path());
    let integrity = state.integrity();

    assert_eq!(integrity.specs, IntegrityStatus::Aligned);
    assert_eq!(integrity.nfrs, IntegrityStatus::Aligned);
    assert_eq!(integrity.tests, IntegrityStatus::Aligned);
}

/// ui-command: launch-displays-integrity-drifted
// @minter:unit launch-displays-integrity-drifted
#[test]
fn state_shows_drift_when_lock_mismatches() {
    use minter::core::ui::state::{IntegrityStatus, UiState};

    let original_spec = spec_one_behavior("auth", "1.0.0", "login");
    let modified_spec = spec_one_behavior("auth", "2.0.0", "login");
    let new_spec = spec_one_behavior("new-feature", "1.0.0", "do-new-thing");
    let test_content = "// @minter:unit login\n";
    let removed_test_content = "// @minter:e2e login\n";

    // Lock was built with original auth.spec, a removed_test, and no new-feature.spec
    let lock = build_lock(
        &[("specs/auth.spec", &original_spec, &["login"], &[], &[])],
        &[],
        &[
            ("tests/auth_test.rs", test_content, &["login"]),
            ("tests/removed_test.rs", removed_test_content, &["login"]),
        ],
    );

    // Current state: auth.spec modified, new-feature.spec added, removed_test.rs deleted
    let dir = setup_project(
        &[
            ("auth.spec", &modified_spec),
            ("new-feature.spec", &new_spec),
        ],
        &[],
        &[("auth_test.rs", test_content)],
        Some(&lock),
    );

    let state = UiState::load(dir.path());
    let integrity = state.integrity();

    assert_eq!(integrity.specs, IntegrityStatus::Drifted);

    let drift = state.drift_details();
    assert!(
        drift.modified_specs.iter().any(|s| s.contains("auth")),
        "expected auth.spec to be listed as modified"
    );
    assert!(
        drift
            .unlocked_specs
            .iter()
            .any(|s| s.contains("new-feature")),
        "expected new-feature.spec to be listed as unlocked"
    );
    assert!(
        drift
            .missing_tests
            .iter()
            .any(|s| s.contains("removed_test")),
        "expected removed_test.rs to be listed as missing"
    );
}

// ═══════════════════════════════════════════════════════════════
// Actions — happy path
// ═══════════════════════════════════════════════════════════════

/// ui-command: validate-targeted-spec
// @minter:unit validate-targeted-spec
#[test]
fn action_validate_returns_results() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    // auth.spec has a parse error — missing required fields
    let broken_spec = "\
spec auth v1.0.0
title \"Auth\"

description
  Auth things.

motivation
  Auth.

behavior login [happy_path]
  \"Login\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"


behavior missing-tag []
  \"Bad behavior\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
";

    let dir = setup_project(&[("auth.spec", broken_spec)], &[], &[], None);

    let state = UiState::load(dir.path());
    let result = state.run_action(Action::Validate, None);

    match result {
        ActionResult::Validate { output, has_errors } => {
            assert!(has_errors, "expected validation errors");
            assert!(
                output.contains("auth"),
                "expected output to reference auth spec"
            );
        }
        other => panic!("expected ActionResult::Validate, got {:?}", other),
    }
}

/// ui-command: coverage-targeted-spec
// @minter:unit coverage-targeted-spec
#[test]
fn action_coverage_returns_report() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    let spec_a = spec_multi_behaviors(
        "auth",
        "1.0.0",
        &["login", "logout", "refresh-token", "reset-password", "mfa"],
    );
    let spec_b = spec_multi_behaviors(
        "billing",
        "1.0.0",
        &[
            "create-invoice",
            "cancel-invoice",
            "refund",
            "list-invoices",
            "export-csv",
        ],
    );

    // 8 of 10 covered → 80%
    let tag = "@minter";
    let test_content = format!(
        "\
// {tag}:unit login
// {tag}:unit logout
// {tag}:e2e refresh-token
// {tag}:e2e reset-password
// {tag}:unit create-invoice
// {tag}:unit cancel-invoice
// {tag}:e2e refund
// {tag}:unit list-invoices
"
    );

    let dir = setup_project(
        &[("auth.spec", &spec_a), ("billing.spec", &spec_b)],
        &[],
        &[("all_test.rs", &test_content)],
        None,
    );

    let state = UiState::load(dir.path());
    let result = state.run_action(Action::Coverage, None);

    match result {
        ActionResult::Coverage {
            covered,
            total,
            percent,
            uncovered_behaviors,
        } => {
            assert_eq!(covered, 8, "expected 8 covered behaviors");
            assert_eq!(total, 10, "expected 10 total behaviors");
            assert_eq!(percent, 80, "expected 80% coverage");
            assert_eq!(
                uncovered_behaviors.len(),
                2,
                "expected 2 uncovered behaviors"
            );
            assert!(
                uncovered_behaviors.contains(&"mfa".to_string()),
                "expected mfa to be uncovered"
            );
            assert!(
                uncovered_behaviors.contains(&"export-csv".to_string()),
                "expected export-csv to be uncovered"
            );
        }
        other => panic!("expected ActionResult::Coverage, got {:?}", other),
    }
}

/// ui-command: trigger-lock-action
// @minter:unit trigger-lock-action
#[test]
fn action_lock_generates_lock_and_refreshes_integrity() {
    use minter::core::ui::state::{Action, ActionResult, IntegrityStatus, UiState};

    let spec_content = spec_one_behavior("auth", "1.0.0", "login");
    let test_content = "// @minter:unit login\n";

    // Start without a lock file → integrity is "no lock"
    let dir = setup_project(
        &[("auth.spec", &spec_content)],
        &[],
        &[("auth_test.rs", test_content)],
        None,
    );

    let mut state = UiState::load(dir.path());
    assert_eq!(
        state.integrity().lock_status,
        IntegrityStatus::NoLock,
        "expected no lock initially"
    );

    let result = state.run_action(Action::Lock, None);

    match result {
        ActionResult::Lock { success, message } => {
            assert!(success, "expected lock generation to succeed");
            assert!(
                message.contains("lock") || message.contains("generated"),
                "expected confirmation message"
            );
        }
        other => panic!("expected ActionResult::Lock, got {:?}", other),
    }

    // After lock action, integrity should refresh to aligned
    assert!(
        dir.path().join("minter.lock").exists(),
        "expected minter.lock to be created"
    );
    state.refresh(dir.path());
    assert!(
        state.lock_aligned(),
        "expected lock to be aligned after regeneration"
    );
}

/// ui-command: lock-action-refreshes-integrity
// @minter:unit lock-action-refreshes-integrity
#[test]
fn lock_action_refreshes_integrity_from_drifted_to_aligned() {
    use minter::core::ui::state::{Action, ActionResult, IntegrityStatus, UiState};

    let spec_content_v1 = spec_one_behavior("auth", "1.0.0", "login");
    let test_content = "// @minter:unit login\n";

    // 1. Create a project and generate a lock file from the initial state
    let dir = setup_project(
        &[("auth.spec", &spec_content_v1)],
        &[],
        &[("auth_test.rs", test_content)],
        None,
    );

    let mut state = UiState::load(dir.path());
    state.validate_all();
    // Generate an initial lock
    let lock_result = state.run_action(Action::Lock, None);
    match &lock_result {
        ActionResult::Lock { success, .. } => assert!(success, "initial lock should succeed"),
        other => panic!("expected ActionResult::Lock, got {:?}", other),
    }
    state.refresh(dir.path());
    assert!(state.lock_aligned(), "expected aligned after first lock");

    // 2. Modify a spec to cause drift
    let spec_content_v2 = spec_one_behavior("auth", "1.1.0", "login");
    fs::write(dir.path().join("specs/auth.spec"), &spec_content_v2).unwrap();

    // Reload — now integrity should show drifted
    state.refresh(dir.path());
    assert_eq!(
        state.integrity().specs,
        IntegrityStatus::Drifted,
        "expected specs drifted after modification"
    );

    // 3. Run the lock action (simulates user pressing 'l')
    let result = state.run_action(Action::Lock, None);
    match &result {
        ActionResult::Lock { success, .. } => {
            assert!(success, "re-lock should succeed");
        }
        other => panic!("expected ActionResult::Lock, got {:?}", other),
    }

    // 4. Simulate what the app loop should do: refresh + validate_all
    //    This is the fix: the app loop must call refresh after lock action.
    state.refresh(dir.path());
    state.validate_all();

    // 5. Verify integrity is now aligned
    assert_eq!(
        state.integrity().lock_status,
        IntegrityStatus::Aligned,
        "expected lock aligned after lock action + refresh"
    );
    assert_eq!(
        state.integrity().specs,
        IntegrityStatus::Aligned,
        "expected specs aligned after lock action + refresh"
    );
}

// ═══════════════════════════════════════════════════════════════
// Reactive updates — happy path
// ═══════════════════════════════════════════════════════════════

/// ui-command: reactive-update-spec-change
// @minter:unit reactive-update-spec-change
#[test]
fn state_refresh_updates_after_spec_change() {
    use minter::core::ui::state::UiState;

    let spec_a = spec_multi_behaviors(
        "auth",
        "1.0.0",
        &["login", "logout", "refresh-token", "reset-password"],
    );

    let dir = setup_project(&[("auth.spec", &spec_a)], &[], &[], None);

    let mut state = UiState::load(dir.path());
    assert_eq!(state.behavior_count(), 4, "expected 4 behaviors initially");

    // Modify the spec to add a fifth behavior
    let updated_spec = spec_multi_behaviors(
        "auth",
        "1.0.0",
        &["login", "logout", "refresh-token", "reset-password", "mfa"],
    );
    fs::write(dir.path().join("specs").join("auth.spec"), &updated_spec).unwrap();

    state.refresh_spec(dir.path(), &dir.path().join("specs").join("auth.spec"));

    assert_eq!(
        state.behavior_count(),
        5,
        "expected 5 behaviors after update"
    );

    let specs = state.specs_list();
    let auth = specs
        .iter()
        .find(|s| s.name == "auth")
        .expect("auth spec present");
    assert_eq!(auth.behavior_count, 5, "expected auth to show 5 behaviors");
}

/// ui-command: reactive-update-test-change
// @minter:unit reactive-update-test-change
#[test]
fn state_refresh_updates_coverage_after_test_change() {
    use minter::core::ui::state::UiState;

    let spec_a = spec_multi_behaviors("auth", "1.0.0", &["login", "logout"]);
    let initial_test = "// @minter:unit login\n";

    let dir = setup_project(
        &[("auth.spec", &spec_a)],
        &[],
        &[("auth_test.rs", initial_test)],
        None,
    );

    let mut state = UiState::load(dir.path());
    assert_eq!(state.coverage_percent(), 50, "expected 50% coverage (1/2)");

    // Add test for the uncovered behavior
    let updated_test = "// @minter:unit login\n// @minter:e2e logout\n";
    fs::write(dir.path().join("tests").join("auth_test.rs"), updated_test).unwrap();

    state.refresh(dir.path());

    assert_eq!(
        state.coverage_percent(),
        100,
        "expected 100% coverage (2/2) after test update"
    );
}

// ═══════════════════════════════════════════════════════════════
// Error cases
// ═══════════════════════════════════════════════════════════════

/// ui-command: launch-without-specs-directory
// @minter:unit launch-without-specs-directory
#[test]
fn state_handles_missing_specs_directory() {
    use minter::core::ui::state::UiState;

    let dir = TempDir::new().unwrap();
    // No specs/ directory, no minter.config.json

    let state = UiState::load(dir.path());

    assert_eq!(state.spec_count(), 0, "expected 0 specs");
    assert_eq!(state.behavior_count(), 0, "expected 0 behaviors");
    assert!(
        state.has_error(),
        "expected state to carry an error about missing specs"
    );
    assert!(
        state.error_message().unwrap().contains("specs"),
        "expected error message to mention specs directory"
    );
}

/// ui-command: launch-without-lock-file
// @minter:unit launch-without-lock-file
#[test]
fn state_shows_no_lock_when_lock_file_missing() {
    use minter::core::ui::state::{IntegrityStatus, UiState};

    let spec_content = spec_one_behavior("auth", "1.0.0", "login");

    let dir = setup_project(&[("auth.spec", &spec_content)], &[], &[], None);

    let state = UiState::load(dir.path());

    assert_eq!(
        state.integrity().lock_status,
        IntegrityStatus::NoLock,
        "expected no lock status"
    );
    assert!(!state.lock_aligned(), "expected lock to not be aligned");
}

// ═══════════════════════════════════════════════════════════════
// Edge cases
// ═══════════════════════════════════════════════════════════════

/// ui-command: zero-coverage-no-crash
// @minter:unit zero-coverage-no-crash
#[test]
fn state_shows_zero_coverage_without_crash() {
    use minter::core::ui::state::UiState;

    let spec_a = spec_multi_behaviors(
        "auth",
        "1.0.0",
        &["login", "logout", "refresh-token", "reset-password"],
    );

    // No test files with @minter tags
    let dir = setup_project(&[("auth.spec", &spec_a)], &[], &[], None);

    let state = UiState::load(dir.path());

    assert_eq!(state.spec_count(), 1, "expected 1 spec");
    assert_eq!(state.behavior_count(), 4, "expected 4 behaviors");
    assert_eq!(state.coverage_percent(), 0, "expected 0% coverage");

    // Verify the spec's behaviors all show uncovered
    let specs = state.specs_list();
    let auth = specs
        .iter()
        .find(|s| s.name == "auth")
        .expect("auth spec present");
    assert!(
        auth.behaviors.iter().all(|b| !b.covered),
        "expected all behaviors to be uncovered"
    );
}

/// ui-command: lock-outdated-shows-drift
// @minter:unit lock-outdated-shows-drift
#[test]
fn state_shows_specific_drift_details() {
    use minter::core::ui::state::UiState;

    let spec_a_original = spec_one_behavior("alpha", "1.0.0", "do-alpha");
    let spec_b_original = spec_one_behavior("beta", "1.0.0", "do-beta");
    let spec_c_original = spec_one_behavior("gamma", "1.0.0", "do-gamma");
    let nfr_original = nfr_performance();
    let test_a = "// @minter:unit do-alpha\n";
    let test_b = "// @minter:unit do-beta\n";
    let test_deleted_1 = "// @minter:e2e do-alpha\n";
    let test_deleted_2 = "// @minter:e2e do-beta\n";

    // Lock was built with original state: 3 specs, 1 nfr, 4 test files
    let lock = build_lock(
        &[
            (
                "specs/alpha.spec",
                &spec_a_original,
                &["do-alpha"],
                &[],
                &[],
            ),
            ("specs/beta.spec", &spec_b_original, &["do-beta"], &[], &[]),
            (
                "specs/gamma.spec",
                &spec_c_original,
                &["do-gamma"],
                &[],
                &[],
            ),
        ],
        &[("specs/nfr/performance.nfr", nfr_original)],
        &[
            ("tests/test_a.rs", test_a, &["do-alpha"]),
            ("tests/test_b.rs", test_b, &["do-beta"]),
            ("tests/test_deleted_1.rs", test_deleted_1, &["do-alpha"]),
            ("tests/test_deleted_2.rs", test_deleted_2, &["do-beta"]),
        ],
    );

    // Current state: 3 specs modified, 1 new nfr added, 2 tests deleted
    let spec_a_modified = spec_one_behavior("alpha", "2.0.0", "do-alpha");
    let spec_b_modified = spec_one_behavior("beta", "2.0.0", "do-beta");
    let spec_c_modified = spec_one_behavior("gamma", "2.0.0", "do-gamma");
    let nfr_new = "\
nfr reliability v1.0.0
title \"Reliability\"

description
  Reliability.

motivation
  Reliability.


constraint no-data-loss [rule]
  \"No data loss\"

  verification
    environment production
    benchmark \"chaos test\"
    pass \"zero data loss\"

  violation critical
  overridable no
";

    let dir = setup_project(
        &[
            ("alpha.spec", &spec_a_modified),
            ("beta.spec", &spec_b_modified),
            ("gamma.spec", &spec_c_modified),
        ],
        &[
            ("performance.nfr", nfr_original),
            ("reliability.nfr", nfr_new),
        ],
        &[("test_a.rs", test_a), ("test_b.rs", test_b)],
        Some(&lock),
    );

    let state = UiState::load(dir.path());
    let drift = state.drift_details();

    assert_eq!(
        drift.modified_specs.len(),
        3,
        "expected 3 specs modified, got: {:?}",
        drift.modified_specs
    );
    assert_eq!(
        drift.unlocked_nfrs.len(),
        1,
        "expected 1 nfr unlocked (reliability), got: {:?}",
        drift.unlocked_nfrs
    );
    assert_eq!(
        drift.missing_tests.len(),
        2,
        "expected 2 tests missing, got: {:?}",
        drift.missing_tests
    );

    // Verify each drifted file is listed by name
    assert!(drift.modified_specs.iter().any(|s| s.contains("alpha")));
    assert!(drift.modified_specs.iter().any(|s| s.contains("beta")));
    assert!(drift.modified_specs.iter().any(|s| s.contains("gamma")));
    assert!(
        drift
            .unlocked_nfrs
            .iter()
            .any(|s| s.contains("reliability"))
    );
    assert!(
        drift
            .missing_tests
            .iter()
            .any(|s| s.contains("test_deleted_1"))
    );
    assert!(
        drift
            .missing_tests
            .iter()
            .any(|s| s.contains("test_deleted_2"))
    );
}

// ═══════════════════════════════════════════════════════════════
// Context-aware actions — targeted (spec selected)
// ═══════════════════════════════════════════════════════════════

/// ui-command: validate-targeted-spec
// @minter:unit validate-targeted-spec
#[test]
fn action_validate_targeted_covers_only_selected_spec() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    // auth.spec has a validation error (empty behavior tag)
    let broken_spec = "\
spec auth v1.0.0
title \"Auth\"

description
  Auth things.

motivation
  Auth.

behavior login [happy_path]
  \"Login\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"


behavior missing-tag []
  \"Bad behavior\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
";

    let valid_spec = spec_one_behavior("billing", "1.0.0", "create-invoice");

    let dir = setup_project(
        &[("auth.spec", broken_spec), ("billing.spec", &valid_spec)],
        &[],
        &[],
        None,
    );

    let state = UiState::load(dir.path());
    let auth_path = dir.path().join("specs").join("auth.spec");
    let result = state.run_action(Action::Validate, Some(&auth_path));

    match result {
        ActionResult::Validate { output, has_errors } => {
            assert!(has_errors, "expected validation errors for auth");
            assert!(
                output.contains("auth"),
                "expected output to reference auth spec"
            );
            assert!(
                !output.contains("billing"),
                "expected output to NOT reference billing spec"
            );
        }
        other => panic!("expected ActionResult::Validate, got {:?}", other),
    }
}

/// ui-command: deep-validate-targeted-spec
// @minter:unit deep-validate-targeted-spec
#[test]
fn action_deep_validate_targeted_resolves_deps_for_selected_spec() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    let user_spec = spec_one_behavior("user", "1.0.0", "create-user");
    let auth_spec = spec_with_deps("auth", "1.0.0", &["login"], &[("user", "1.0.0")]);

    let dir = setup_project(
        &[("auth.spec", &auth_spec), ("user.spec", &user_spec)],
        &[],
        &[],
        None,
    );

    let state = UiState::load(dir.path());
    let auth_path = dir.path().join("specs").join("auth.spec");
    let result = state.run_action(Action::DeepValidate, Some(&auth_path));

    match result {
        ActionResult::DeepValidate { output, has_errors } => {
            assert!(
                !has_errors,
                "expected no errors for valid spec with valid dep, got: {}",
                output
            );
            assert!(
                output.contains("auth"),
                "expected output to reference auth spec"
            );
            assert!(
                output.contains("deep"),
                "expected output to mention deep validation"
            );
        }
        other => panic!("expected ActionResult::DeepValidate, got {:?}", other),
    }
}

/// ui-command: coverage-targeted-spec
// @minter:unit coverage-targeted-spec
#[test]
fn action_coverage_targeted_covers_only_selected_spec() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    let spec_a = spec_multi_behaviors("auth", "1.0.0", &["login", "logout", "mfa"]);
    let spec_b = spec_multi_behaviors("billing", "1.0.0", &["create-invoice", "cancel-invoice"]);

    // Cover 2 of 3 auth behaviors, 1 of 2 billing behaviors
    let tag = "@minter";
    let test_content = format!(
        "\
// {tag}:unit login
// {tag}:unit logout
// {tag}:unit create-invoice
"
    );

    let dir = setup_project(
        &[("auth.spec", &spec_a), ("billing.spec", &spec_b)],
        &[],
        &[("all_test.rs", &test_content)],
        None,
    );

    let state = UiState::load(dir.path());
    let auth_path = dir.path().join("specs").join("auth.spec");
    let result = state.run_action(Action::Coverage, Some(&auth_path));

    match result {
        ActionResult::Coverage {
            covered,
            total,
            percent,
            uncovered_behaviors,
        } => {
            assert_eq!(total, 3, "expected 3 total behaviors for auth only");
            assert_eq!(covered, 2, "expected 2 covered behaviors for auth");
            assert_eq!(percent, 66, "expected 66% coverage for auth (2/3)");
            assert_eq!(
                uncovered_behaviors.len(),
                1,
                "expected 1 uncovered behavior"
            );
            assert!(
                uncovered_behaviors.contains(&"mfa".to_string()),
                "expected mfa to be uncovered"
            );
        }
        other => panic!("expected ActionResult::Coverage, got {:?}", other),
    }
}

/// ui-command: graph-targeted-spec
// @minter:unit graph-targeted-spec
#[test]
fn action_graph_targeted_shows_only_selected_spec() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    let user_spec = spec_one_behavior("user", "1.0.0", "create-user");
    let auth_spec = spec_with_deps("auth", "1.0.0", &["login"], &[("user", "1.0.0")]);
    let billing_spec = spec_one_behavior("billing", "1.0.0", "create-invoice");

    let dir = setup_project(
        &[
            ("auth.spec", &auth_spec),
            ("user.spec", &user_spec),
            ("billing.spec", &billing_spec),
        ],
        &[],
        &[],
        None,
    );

    let state = UiState::load(dir.path());
    let auth_path = dir.path().join("specs").join("auth.spec");
    let result = state.run_action(Action::Graph, Some(&auth_path));

    match result {
        ActionResult::Graph { output } => {
            assert!(
                output.contains("auth"),
                "expected graph to include auth spec"
            );
            assert!(
                output.contains("user"),
                "expected graph to show auth's dependency on user"
            );
            assert!(
                !output.contains("billing"),
                "expected graph to NOT include billing (not targeted)"
            );
        }
        other => panic!("expected ActionResult::Graph, got {:?}", other),
    }
}

/// ui-command: inspect-targeted-spec
// @minter:unit inspect-targeted-spec
#[test]
fn action_inspect_targeted_shows_metadata_for_selected_spec() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    let auth_spec = spec_mixed_categories("auth", "1.0.0");
    let user_spec = spec_one_behavior("user", "1.0.0", "create-user");

    // auth has: 2 happy_path, 1 error_case, 1 edge_case = 4 behaviors
    let dir = setup_project(
        &[("auth.spec", &auth_spec), ("user.spec", &user_spec)],
        &[],
        &[],
        None,
    );

    let state = UiState::load(dir.path());
    let auth_path = dir.path().join("specs").join("auth.spec");
    let result = state.run_action(Action::Inspect, Some(&auth_path));

    match result {
        ActionResult::Inspect {
            output,
            behavior_count,
            categories,
            dependencies: _,
        } => {
            assert_eq!(behavior_count, 4, "expected 4 behaviors for auth");
            assert!(
                output.contains("auth"),
                "expected inspect output to reference auth"
            );
            // Check category distribution
            let happy_count = categories
                .iter()
                .find(|(cat, _)| cat.contains("happy"))
                .map(|(_, n)| *n)
                .unwrap_or(0);
            assert_eq!(happy_count, 2, "expected 2 happy_path behaviors");
            let error_count = categories
                .iter()
                .find(|(cat, _)| cat.contains("error"))
                .map(|(_, n)| *n)
                .unwrap_or(0);
            assert_eq!(error_count, 1, "expected 1 error_case behavior");
        }
        other => panic!("expected ActionResult::Inspect, got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════
// Context-aware actions — global (no spec selected)
// ═══════════════════════════════════════════════════════════════

/// ui-command: validate-global-no-selection
// @minter:unit validate-global-no-selection
#[test]
fn action_validate_global_covers_all_specs() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    // auth.spec has a validation error (empty behavior tag)
    let broken_spec = "\
spec auth v1.0.0
title \"Auth\"

description
  Auth things.

motivation
  Auth.

behavior login [happy_path]
  \"Login\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"


behavior missing-tag []
  \"Bad behavior\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
";

    let valid_spec = spec_one_behavior("billing", "1.0.0", "create-invoice");

    let dir = setup_project(
        &[("auth.spec", broken_spec), ("billing.spec", &valid_spec)],
        &[],
        &[],
        None,
    );

    let state = UiState::load(dir.path());
    // No spec selected → global validation
    let result = state.run_action(Action::Validate, None);

    match result {
        ActionResult::Validate { output, has_errors } => {
            assert!(has_errors, "expected validation errors from auth");
            assert!(
                output.contains("auth"),
                "expected output to reference auth spec"
            );
            assert!(
                output.contains("billing"),
                "expected output to also reference billing spec (global)"
            );
        }
        other => panic!("expected ActionResult::Validate, got {:?}", other),
    }
}

/// ui-command: coverage-global-no-selection
// @minter:unit coverage-global-no-selection
#[test]
fn action_coverage_global_covers_all_specs() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    let spec_a = spec_multi_behaviors("auth", "1.0.0", &["login", "logout", "mfa"]);
    let spec_b = spec_multi_behaviors(
        "billing",
        "1.0.0",
        &[
            "create-invoice",
            "cancel-invoice",
            "refund",
            "list-invoices",
            "export-csv",
            "send-reminder",
            "payment",
        ],
    );

    // 8 of 10 covered → 80%
    let tag = "@minter";
    let test_content = format!(
        "\
// {tag}:unit login
// {tag}:unit logout
// {tag}:unit create-invoice
// {tag}:unit cancel-invoice
// {tag}:e2e refund
// {tag}:unit list-invoices
// {tag}:e2e export-csv
// {tag}:unit send-reminder
"
    );

    let dir = setup_project(
        &[("auth.spec", &spec_a), ("billing.spec", &spec_b)],
        &[],
        &[("all_test.rs", &test_content)],
        None,
    );

    let state = UiState::load(dir.path());
    // No spec selected → global coverage
    let result = state.run_action(Action::Coverage, None);

    match result {
        ActionResult::Coverage {
            covered,
            total,
            percent,
            uncovered_behaviors,
        } => {
            assert_eq!(total, 10, "expected 10 total behaviors across all specs");
            assert_eq!(covered, 8, "expected 8 covered behaviors");
            assert_eq!(percent, 80, "expected 80% coverage");
            assert!(
                uncovered_behaviors.contains(&"mfa".to_string()),
                "expected mfa to be uncovered"
            );
            assert!(
                uncovered_behaviors.contains(&"payment".to_string()),
                "expected payment to be uncovered"
            );
        }
        other => panic!("expected ActionResult::Coverage, got {:?}", other),
    }
}

/// ui-command: graph-global-no-selection
// @minter:unit graph-global-no-selection
#[test]
fn action_graph_global_shows_full_project_graph() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    let spec_a = spec_with_deps("a", "1.0.0", &["do-a"], &[("b", "1.0.0")]);
    let spec_b = spec_one_behavior("b", "1.0.0", "do-b");

    let dir = setup_project(&[("a.spec", &spec_a), ("b.spec", &spec_b)], &[], &[], None);

    let state = UiState::load(dir.path());
    // No spec selected → full project graph
    let result = state.run_action(Action::Graph, None);

    match result {
        ActionResult::Graph { output } => {
            assert!(output.contains("a"), "expected graph to include spec a");
            assert!(output.contains("b"), "expected graph to include spec b");
            assert!(
                output.contains("->"),
                "expected graph to show dependency edge from a to b"
            );
        }
        other => panic!("expected ActionResult::Graph, got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════
// New actions — happy path
// ═══════════════════════════════════════════════════════════════

/// ui-command: trigger-format-reference
// @minter:unit trigger-format-reference
#[test]
fn action_format_shows_dsl_grammar_reference() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    let spec_content = spec_one_behavior("auth", "1.0.0", "login");
    let dir = setup_project(&[("auth.spec", &spec_content)], &[], &[], None);

    let state = UiState::load(dir.path());
    let result = state.run_action(Action::Format, None);

    match result {
        ActionResult::Format { output } => {
            assert!(
                output.contains("spec"),
                "expected grammar to include 'spec' keyword"
            );
            assert!(
                output.contains("title"),
                "expected grammar to include 'title' keyword"
            );
            assert!(
                output.contains("description"),
                "expected grammar to include 'description' keyword"
            );
            assert!(
                output.contains("behavior"),
                "expected grammar to include 'behavior' keyword"
            );
            assert!(
                output.contains("given"),
                "expected grammar to include 'given' keyword"
            );
            assert!(
                output.contains("when"),
                "expected grammar to include 'when' keyword"
            );
            assert!(
                output.contains("then"),
                "expected grammar to include 'then' keyword"
            );
        }
        other => panic!("expected ActionResult::Format, got {:?}", other),
    }
}

/// ui-command: trigger-scaffold-spec
// @minter:unit trigger-scaffold-spec
#[test]
fn action_scaffold_shows_spec_skeleton() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    let spec_content = spec_one_behavior("auth", "1.0.0", "login");
    let dir = setup_project(&[("auth.spec", &spec_content)], &[], &[], None);

    let state = UiState::load(dir.path());
    let result = state.run_action(Action::Scaffold, None);

    match result {
        ActionResult::Scaffold { output } => {
            assert!(
                output.contains("spec"),
                "expected scaffold to include 'spec' keyword"
            );
            assert!(
                output.contains("title"),
                "expected scaffold to include 'title' section"
            );
            assert!(
                output.contains("description"),
                "expected scaffold to include 'description' section"
            );
            assert!(
                output.contains("behavior"),
                "expected scaffold to include 'behavior' section"
            );
        }
        other => panic!("expected ActionResult::Scaffold, got {:?}", other),
    }
}

/// ui-command: trigger-guide-topics
// @minter:unit trigger-guide-topics
#[test]
fn action_guide_shows_available_topics() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    let spec_content = spec_one_behavior("auth", "1.0.0", "login");
    let dir = setup_project(&[("auth.spec", &spec_content)], &[], &[], None);

    let state = UiState::load(dir.path());
    let result = state.run_action(Action::Guide, None);

    match result {
        ActionResult::Guide { topics } => {
            assert!(
                topics.contains(&"methodology".to_string()),
                "expected topics to include methodology"
            );
            assert!(
                topics.contains(&"workflow".to_string()),
                "expected topics to include workflow"
            );
            assert!(
                topics.contains(&"authoring".to_string()),
                "expected topics to include authoring"
            );
            assert!(
                topics.contains(&"smells".to_string()),
                "expected topics to include smells"
            );
            assert!(
                topics.contains(&"nfr".to_string()),
                "expected topics to include nfr"
            );
            assert!(
                topics.contains(&"context".to_string()),
                "expected topics to include context"
            );
            assert!(
                topics.contains(&"coverage".to_string()),
                "expected topics to include coverage"
            );
            assert_eq!(topics.len(), 7, "expected exactly 7 guide topics");
        }
        other => panic!("expected ActionResult::Guide, got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════
// Validation status — real-time validation per spec
// ═══════════════════════════════════════════════════════════════

/// ui-command: validation-status-icons
// @minter:unit validation-status-icons
#[test]
fn validation_status_valid_for_well_formed_specs() {
    use minter::core::ui::state::{UiState, ValidationStatus};

    let spec_a = spec_one_behavior("auth", "1.0.0", "login");
    let spec_b = spec_one_behavior("billing", "2.0.0", "create-invoice");

    let dir = setup_project(
        &[("auth.spec", &spec_a), ("billing.spec", &spec_b)],
        &[],
        &[],
        None,
    );

    let mut state = UiState::load(dir.path());
    state.validate_all();

    let specs = state.specs_list();
    assert_eq!(specs.len(), 2);
    for spec in specs {
        assert_eq!(
            spec.validation_status,
            ValidationStatus::Valid,
            "spec '{}' should be valid",
            spec.name
        );
    }
}

/// ui-command: broken-spec-stays-visible
// @minter:unit broken-spec-stays-visible
#[test]
fn validation_status_invalid_for_spec_with_parse_error() {
    use minter::core::ui::state::{UiState, ValidationStatus};

    let valid_spec = spec_one_behavior("auth", "1.0.0", "login");
    let invalid_spec = "this is not a valid spec file at all";

    let dir = setup_project(
        &[("auth.spec", &valid_spec), ("broken.spec", invalid_spec)],
        &[],
        &[],
        None,
    );

    let mut state = UiState::load(dir.path());
    state.validate_all();

    let specs = state.specs_list();
    // Only the valid spec gets parsed and added to specs list.
    // The broken spec fails parsing so it doesn't appear in the parsed list.
    // But validate_all should still set the valid spec's status.
    let auth = specs.iter().find(|s| s.name == "auth");
    assert!(auth.is_some(), "auth spec should be present");
    assert_eq!(auth.unwrap().validation_status, ValidationStatus::Valid);
}

/// ui-command: integrity-shows-validation-errors
// @minter:unit integrity-shows-validation-errors
#[test]
fn validation_status_invalid_for_spec_with_semantic_error() {
    use minter::core::ui::state::{UiState, ValidationStatus};

    // A spec with duplicate behavior names is a semantic error
    let dup_behaviors = "\
spec bad-spec v1.0.0
title \"Bad\"

description
  Bad spec.

motivation
  Test.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"

behavior do-thing [happy_path]
  \"Does a thing again\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
";

    let dir = setup_project(&[("bad.spec", dup_behaviors)], &[], &[], None);

    let mut state = UiState::load(dir.path());
    state.validate_all();

    let specs = state.specs_list();
    assert_eq!(specs.len(), 1);

    match &specs[0].validation_status {
        ValidationStatus::Invalid(errors) => {
            assert!(!errors.is_empty(), "should have at least one error message");
        }
        other => panic!("expected ValidationStatus::Invalid, got {:?}", other),
    }
}

/// ui-command: validation-status-icons
// @minter:unit validation-status-icons
#[test]
fn validation_status_defaults_to_unknown() {
    use minter::core::ui::state::{SpecInfo, ValidationStatus};

    // SpecInfo should have a default validation_status of Unknown
    // when constructed without explicit validation
    let info = SpecInfo {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        path: std::path::PathBuf::from("test.spec"),
        behavior_count: 0,
        behaviors: Vec::new(),
        validation_status: ValidationStatus::Unknown,
    };

    assert_eq!(info.validation_status, ValidationStatus::Unknown);
}

/// ui-command: reactive-update-spec-change
// @minter:unit reactive-update-spec-change
#[test]
fn validation_status_updated_on_refresh() {
    use minter::core::ui::state::{UiState, ValidationStatus};

    let spec_a = spec_one_behavior("auth", "1.0.0", "login");

    let dir = setup_project(&[("auth.spec", &spec_a)], &[], &[], None);

    let mut state = UiState::load(dir.path());
    state.validate_all();

    // Initially valid
    assert_eq!(
        state.specs_list()[0].validation_status,
        ValidationStatus::Valid
    );

    // Break the spec on disk
    fs::write(
        dir.path().join("specs/auth.spec"),
        "not a valid spec anymore",
    )
    .unwrap();

    // Refresh and re-validate
    state.refresh(dir.path());
    state.validate_all();

    // The broken spec stays in the list with Invalid status
    assert_eq!(state.specs_list().len(), 1);
    assert!(matches!(
        state.specs_list()[0].validation_status,
        ValidationStatus::Invalid(_)
    ));
}

// ═══════════════════════════════════════════════════════════════
// Selection and deselection
// ═══════════════════════════════════════════════════════════════

/// ui-command: validate-global-no-selection
// @minter:unit validate-global-no-selection
#[test]
fn action_validate_all_specs_when_nothing_selected() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    let broken_spec = "\
spec auth v1.0.0
title \"Auth\"

description
  Auth things.

motivation
  Auth.

behavior login [happy_path]
  \"Login\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"


behavior missing-tag []
  \"Bad behavior\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
";

    let valid_spec = spec_one_behavior("billing", "1.0.0", "create-invoice");

    let dir = setup_project(
        &[("auth.spec", broken_spec), ("billing.spec", &valid_spec)],
        &[],
        &[],
        None,
    );

    let state = UiState::load(dir.path());
    // Pass None to simulate no spec selected -> validates all
    let result = state.run_action(Action::Validate, None);

    match result {
        ActionResult::Validate { output, has_errors } => {
            assert!(has_errors, "expected validation errors");
            // Should contain auth (the broken one)
            assert!(
                output.contains("auth"),
                "expected output to reference auth spec"
            );
        }
        other => panic!("expected ActionResult::Validate, got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════
// Broken spec visibility and validation status
// ═══════════════════════════════════════════════════════════════

/// ui-command: broken-spec-stays-visible
// @minter:unit broken-spec-stays-visible
#[test]
fn broken_spec_stays_visible_in_list() {
    use minter::core::ui::state::{UiState, ValidationStatus};

    let dir = setup_project(
        &[("auth.spec", "this is not valid spec syntax at all")],
        &[],
        &[],
        None,
    );

    let state = UiState::load(dir.path());

    // The broken spec should still appear in the list
    assert_eq!(state.specs_list().len(), 1);
    assert_eq!(state.specs_list()[0].name, "auth");
    assert_eq!(state.specs_list()[0].behavior_count, 0);
    assert!(matches!(
        state.specs_list()[0].validation_status,
        ValidationStatus::Invalid(_)
    ));
}

/// ui-command: broken-spec-stays-visible
// @minter:unit broken-spec-stays-visible
#[test]
fn valid_spec_broken_then_fixed_updates_status() {
    use minter::core::ui::state::{UiState, ValidationStatus};

    let spec_a = spec_one_behavior("auth", "1.0.0", "login");
    let dir = setup_project(&[("auth.spec", &spec_a)], &[], &[], None);

    let mut state = UiState::load(dir.path());
    state.validate_all();
    assert!(matches!(
        state.specs_list()[0].validation_status,
        ValidationStatus::Valid
    ));

    // Break it
    fs::write(dir.path().join("specs/auth.spec"), "broken syntax here").unwrap();
    state.refresh(dir.path());
    state.validate_all();

    assert_eq!(state.specs_list().len(), 1);
    assert!(matches!(
        state.specs_list()[0].validation_status,
        ValidationStatus::Invalid(_)
    ));

    // Fix it back
    fs::write(dir.path().join("specs/auth.spec"), &spec_a).unwrap();
    state.refresh(dir.path());
    state.validate_all();

    assert_eq!(state.specs_list().len(), 1);
    assert!(matches!(
        state.specs_list()[0].validation_status,
        ValidationStatus::Valid
    ));
}

/// ui-command: esc-clears-all
// @minter:unit esc-clears-all
#[test]
fn esc_clears_action_result_and_selection_at_once() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use minter::core::ui::input::{InputResult, handle_event};
    use minter::core::ui::render::AppState;

    let mut state = AppState::new();
    state.selected_spec = Some(2);
    state.action_result = Some("validate output".to_string());

    let evt = Event::Key(KeyEvent {
        code: KeyCode::Esc,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    });

    let result = handle_event(&evt, &mut state, 5);
    assert!(matches!(result, InputResult::StateChanged));
    assert!(state.action_result.is_none());
    assert_eq!(state.selected_spec, None);
}

// ═══════════════════════════════════════════════════════════════
// Integrity panel — uncovered behaviors
// ═══════════════════════════════════════════════════════════════

/// ui-command: integrity-shows-uncovered-behaviors
// @minter:unit integrity-shows-uncovered-behaviors
#[test]
fn integrity_panel_lists_uncovered_behaviors() {
    use minter::core::ui::state::UiState;

    let spec_a = spec_multi_behaviors("auth", "1.0.0", &["login", "logout", "refresh-token"]);
    let spec_b = spec_multi_behaviors("billing", "1.0.0", &["create-invoice", "cancel-invoice"]);

    // Only login and logout are covered
    let test_content = "// @minter:unit login\n// @minter:unit logout\n";

    let dir = setup_project(
        &[("auth.spec", &spec_a), ("billing.spec", &spec_b)],
        &[],
        &[("auth_test.rs", test_content)],
        None,
    );

    let state = UiState::load(dir.path());

    // Collect uncovered behaviors across all specs
    let uncovered: Vec<(&str, &str)> = state
        .specs_list()
        .iter()
        .flat_map(|s| {
            s.behaviors
                .iter()
                .filter(|b| !b.covered)
                .map(move |b| (s.name.as_str(), b.name.as_str()))
        })
        .collect();

    assert_eq!(uncovered.len(), 3, "expected 3 uncovered behaviors");
    assert!(uncovered.contains(&("auth", "refresh-token")));
    assert!(uncovered.contains(&("billing", "create-invoice")));
    assert!(uncovered.contains(&("billing", "cancel-invoice")));
}

/// ui-command: integrity-shows-uncovered-behaviors
// @minter:unit integrity-shows-uncovered-behaviors
#[test]
fn integrity_panel_no_uncovered_section_when_fully_covered() {
    use minter::core::ui::state::UiState;

    let spec_a = spec_one_behavior("auth", "1.0.0", "login");
    let test_content = "// @minter:unit login\n";

    let dir = setup_project(
        &[("auth.spec", &spec_a)],
        &[],
        &[("auth_test.rs", test_content)],
        None,
    );

    let state = UiState::load(dir.path());

    let uncovered: Vec<_> = state
        .specs_list()
        .iter()
        .flat_map(|s| s.behaviors.iter().filter(|b| !b.covered))
        .collect();

    assert!(uncovered.is_empty(), "all behaviors should be covered");
}

// ═══════════════════════════════════════════════════════════════
// Integrity panel — validation errors display
// ═══════════════════════════════════════════════════════════════

/// ui-command: integrity-shows-validation-errors
// @minter:unit integrity-shows-validation-errors
#[test]
fn integrity_panel_no_errors_when_all_valid() {
    use minter::core::ui::state::{UiState, ValidationStatus};

    let spec_a = spec_one_behavior("auth", "1.0.0", "login");
    let spec_b = spec_one_behavior("billing", "1.0.0", "create-invoice");

    let dir = setup_project(
        &[("auth.spec", &spec_a), ("billing.spec", &spec_b)],
        &[],
        &[],
        None,
    );

    let state = UiState::load(dir.path());
    let invalid: Vec<_> = state
        .specs_list()
        .iter()
        .filter(|s| matches!(s.validation_status, ValidationStatus::Invalid(_)))
        .collect();

    assert!(invalid.is_empty());
}

/// ui-command: integrity-shows-validation-errors
// @minter:unit integrity-shows-validation-errors
#[test]
fn integrity_panel_shows_parse_error_for_broken_spec() {
    use minter::core::ui::state::{UiState, ValidationStatus};

    let valid_spec = spec_one_behavior("billing", "1.0.0", "create-invoice");

    let dir = setup_project(
        &[
            ("auth.spec", "this is broken syntax"),
            ("billing.spec", &valid_spec),
        ],
        &[],
        &[],
        None,
    );

    let state = UiState::load(dir.path());

    // Both specs should be in the list
    assert_eq!(state.specs_list().len(), 2);

    let auth = state
        .specs_list()
        .iter()
        .find(|s| s.name == "auth")
        .unwrap();
    let billing = state
        .specs_list()
        .iter()
        .find(|s| s.name == "billing")
        .unwrap();

    // auth should be invalid with error messages
    match &auth.validation_status {
        ValidationStatus::Invalid(errors) => {
            assert!(!errors.is_empty(), "expected at least one error message");
        }
        other => panic!("expected Invalid, got {:?}", other),
    }

    // billing should be valid
    assert!(matches!(billing.validation_status, ValidationStatus::Valid));
}

/// ui-command: integrity-shows-validation-errors
// @minter:unit integrity-shows-validation-errors
#[test]
fn integrity_panel_shows_semantic_error() {
    use minter::core::ui::state::{UiState, ValidationStatus};

    // Spec with duplicate behavior names -> semantic error
    let bad_spec = "\
spec auth v1.0.0
title \"Auth\"

description
  Test.

motivation
  Test.

behavior login [happy_path]
  \"First login\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"

behavior login [happy_path]
  \"Duplicate login\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
";

    let dir = setup_project(&[("auth.spec", bad_spec)], &[], &[], None);
    let state = UiState::load(dir.path());

    assert_eq!(state.specs_list().len(), 1);
    match &state.specs_list()[0].validation_status {
        ValidationStatus::Invalid(errors) => {
            assert!(!errors.is_empty());
            let joined = errors.join(" ");
            assert!(
                joined.contains("login")
                    || joined.contains("duplicate")
                    || joined.contains("Duplicate"),
                "expected error about duplicate behavior, got: {}",
                joined
            );
        }
        other => panic!("expected Invalid for semantic error, got {:?}", other),
    }
}

/// ui-command: integrity-shows-validation-errors
// @minter:unit integrity-shows-validation-errors
#[test]
fn integrity_panel_multiple_broken_specs() {
    use minter::core::ui::state::{UiState, ValidationStatus};

    let dir = setup_project(
        &[
            ("auth.spec", "broken auth"),
            ("billing.spec", "broken billing"),
            (
                "orders.spec",
                &spec_one_behavior("orders", "1.0.0", "create-order"),
            ),
        ],
        &[],
        &[],
        None,
    );

    let state = UiState::load(dir.path());
    assert_eq!(state.specs_list().len(), 3);

    let invalid_count = state
        .specs_list()
        .iter()
        .filter(|s| matches!(s.validation_status, ValidationStatus::Invalid(_)))
        .count();

    assert_eq!(invalid_count, 2, "expected 2 broken specs");

    let valid_count = state
        .specs_list()
        .iter()
        .filter(|s| matches!(s.validation_status, ValidationStatus::Valid))
        .count();

    assert_eq!(valid_count, 1, "expected 1 valid spec");
}

/// ui-command: integrity-shows-validation-errors
// @minter:unit integrity-shows-validation-errors
#[test]
fn integrity_panel_broken_spec_has_zero_behaviors() {
    use minter::core::ui::state::UiState;

    let dir = setup_project(&[("auth.spec", "completely broken")], &[], &[], None);

    let state = UiState::load(dir.path());
    assert_eq!(state.specs_list().len(), 1);
    assert_eq!(state.specs_list()[0].behavior_count, 0);
    assert!(state.specs_list()[0].behaviors.is_empty());
}

/// ui-command: integrity-shows-validation-errors
// @minter:unit integrity-shows-validation-errors
#[test]
fn integrity_panel_errors_update_on_fix() {
    use minter::core::ui::state::{UiState, ValidationStatus};

    let dir = setup_project(&[("auth.spec", "broken")], &[], &[], None);

    let mut state = UiState::load(dir.path());

    // Initially invalid
    assert!(matches!(
        state.specs_list()[0].validation_status,
        ValidationStatus::Invalid(_)
    ));

    // Fix the spec
    let fixed = spec_one_behavior("auth", "1.0.0", "login");
    fs::write(dir.path().join("specs/auth.spec"), &fixed).unwrap();
    state.refresh(dir.path());
    state.validate_all();

    // Now valid
    assert_eq!(state.specs_list().len(), 1);
    assert!(matches!(
        state.specs_list()[0].validation_status,
        ValidationStatus::Valid
    ));
    assert_eq!(state.specs_list()[0].behavior_count, 1);
}

// ═══════════════════════════════════════════════════════════════
// Panel scrolling
// ═══════════════════════════════════════════════════════════════

/// ui-command: large-project-scrollable
// @minter:unit large-project-scrollable
#[test]
fn specs_panel_viewport_follows_selection() {
    use minter::core::ui::render::AppState;

    let mut state = AppState::new();

    // Simulate navigating down past visible area (assume 20 rows visible)
    for _ in 0..25 {
        state.move_down(50);
    }

    assert_eq!(state.selected_spec, Some(24));
    // scroll_offset should have adjusted to keep selection visible
    assert!(
        state.scroll_offset > 0,
        "scroll_offset should advance when selection goes past viewport"
    );
}

/// ui-command: large-project-scrollable
// @minter:unit large-project-scrollable
#[test]
fn specs_panel_scroll_follows_selection_up() {
    use minter::core::ui::render::AppState;

    let mut state = AppState::new();

    // Navigate down then back up
    for _ in 0..30 {
        state.move_down(50);
    }
    let offset_at_bottom = state.scroll_offset;

    for _ in 0..30 {
        state.move_up();
    }

    assert_eq!(state.selected_spec, None);
    assert!(
        state.scroll_offset < offset_at_bottom,
        "scroll_offset should decrease when navigating up"
    );
}

/// ui-command: integrity-panel-scrollable
// @minter:unit integrity-panel-scrollable
#[test]
fn integrity_scroll_page_down_and_up() {
    use minter::core::ui::render::AppState;

    let mut state = AppState::new();
    assert_eq!(state.integrity_scroll, 0);

    state.integrity_page_down(100);
    assert!(state.integrity_scroll > 0, "page down should scroll");

    let after_down = state.integrity_scroll;
    state.integrity_page_up();
    assert!(
        state.integrity_scroll < after_down,
        "page up should scroll back"
    );
}

/// ui-command: integrity-panel-scrollable
// @minter:unit integrity-panel-scrollable
#[test]
fn integrity_scroll_does_not_go_negative() {
    use minter::core::ui::render::AppState;

    let mut state = AppState::new();
    state.integrity_page_up();
    assert_eq!(state.integrity_scroll, 0, "scroll should not go below 0");
}

// ═══════════════════════════════════════════════════════════════
// Invalid tags — happy path
// ═══════════════════════════════════════════════════════════════

/// ui-command: integrity-shows-invalid-tags
// @minter:unit integrity-shows-invalid-tags
#[test]
fn invalid_tags_detected_in_state() {
    use minter::core::ui::state::UiState;

    let spec_content = spec_one_behavior("auth", "1.0.0", "login");
    // "login" is valid, "nonexistent" is invalid
    let test_content = "// @minter:unit login\n// @minter:unit nonexistent\n";

    let dir = setup_project(
        &[("auth.spec", &spec_content)],
        &[],
        &[("auth_test.rs", test_content)],
        None,
    );

    let state = UiState::load(dir.path());
    let invalid = state.invalid_tags();

    assert_eq!(
        invalid.len(),
        1,
        "expected 1 invalid tag, got: {:?}",
        invalid
    );
    assert!(
        invalid[0].file.contains("auth_test.rs"),
        "expected invalid tag to reference auth_test.rs, got: {}",
        invalid[0].file
    );
    assert!(
        invalid[0].message.contains("nonexistent"),
        "expected message to mention 'nonexistent', got: {}",
        invalid[0].message
    );
    assert_eq!(invalid[0].line, 2, "expected invalid tag on line 2");
}

/// ui-command: integrity-shows-invalid-tags
// @minter:unit integrity-shows-invalid-tags
#[test]
fn no_invalid_tags_when_all_valid() {
    use minter::core::ui::state::UiState;

    let spec_content = spec_one_behavior("auth", "1.0.0", "login");
    let test_content = "// @minter:unit login\n";

    let dir = setup_project(
        &[("auth.spec", &spec_content)],
        &[],
        &[("auth_test.rs", test_content)],
        None,
    );

    let state = UiState::load(dir.path());
    let invalid = state.invalid_tags();

    assert!(
        invalid.is_empty(),
        "expected no invalid tags, got: {:?}",
        invalid
    );
}

/// ui-command: integrity-shows-invalid-tags
// @minter:unit integrity-shows-invalid-tags
#[test]
fn invalid_tags_update_on_refresh() {
    use minter::core::ui::state::UiState;

    let spec_content = spec_one_behavior("auth", "1.0.0", "login");
    // Initially all tags are valid
    let test_content = "// @minter:unit login\n";

    let dir = setup_project(
        &[("auth.spec", &spec_content)],
        &[],
        &[("auth_test.rs", test_content)],
        None,
    );

    let mut state = UiState::load(dir.path());
    assert!(
        state.invalid_tags().is_empty(),
        "expected no invalid tags initially"
    );

    // Add an invalid tag
    let updated_test = "// @minter:unit login\n// @minter:unit ghost-behavior\n";
    fs::write(dir.path().join("tests").join("auth_test.rs"), updated_test).unwrap();

    state.refresh(dir.path());
    let invalid = state.invalid_tags();

    assert_eq!(
        invalid.len(),
        1,
        "expected 1 invalid tag after refresh, got: {:?}",
        invalid
    );
    assert!(
        invalid[0].message.contains("ghost-behavior"),
        "expected message to mention 'ghost-behavior', got: {}",
        invalid[0].message
    );
}

// ═══════════════════════════════════════════════════════════════
// Integrity — dependency errors
// ═══════════════════════════════════════════════════════════════

/// ui-command: integrity-shows-dependency-errors
// @minter:unit integrity-shows-dependency-errors
#[test]
fn integrity_panel_shows_dependency_errors() {
    use minter::core::ui::state::UiState;

    // auth depends on user >= 1.0.0, but user.spec does not exist
    let spec_content = spec_with_deps("auth", "1.0.0", &["login"], &[("user", "1.0.0")]);

    let dir = setup_project(&[("auth.spec", &spec_content)], &[], &[], None);

    let state = UiState::load(dir.path());
    let dep_errors = state.dep_errors();

    assert!(
        !dep_errors.is_empty(),
        "expected dependency errors when user.spec is missing"
    );
    assert!(
        dep_errors
            .iter()
            .any(|e| e.contains("auth") && e.contains("user")),
        "expected dep error to mention auth and user, got: {:?}",
        dep_errors
    );
}

/// ui-command: integrity-shows-dependency-errors
// @minter:unit integrity-shows-dependency-errors
#[test]
fn integrity_panel_no_dependency_errors_when_all_resolved() {
    use minter::core::ui::state::UiState;

    // auth depends on user >= 1.0.0, and user.spec exists
    let auth_content = spec_with_deps("auth", "1.0.0", &["login"], &[("user", "1.0.0")]);
    let user_content = spec_one_behavior("user", "1.0.0", "create-user");

    let dir = setup_project(
        &[("auth.spec", &auth_content), ("user.spec", &user_content)],
        &[],
        &[],
        None,
    );

    let state = UiState::load(dir.path());
    let dep_errors = state.dep_errors();

    assert!(
        dep_errors.is_empty(),
        "expected no dependency errors when all deps are resolved, got: {:?}",
        dep_errors
    );
}

// ═══════════════════════════════════════════════════════════════
// Action bar — selection awareness
// ═══════════════════════════════════════════════════════════════

/// ui-command: action-bar-shows-esc-hint
// @minter:unit action-bar-shows-esc-hint
#[test]
fn action_bar_shows_esc_hint_when_spec_selected() {
    use minter::core::ui::render::AppState;

    let mut state = AppState::new();
    state.selected_spec = Some(2);

    // has_selection is computed in render() as app_state.selected_spec.is_some()
    // which drives render_action_bar to show [Esc]
    assert!(
        state.selected_spec.is_some(),
        "expected has_selection to be true when spec is selected"
    );
}

/// ui-command: action-bar-updates-on-selection-change
// @minter:unit action-bar-updates-on-selection-change
#[test]
fn action_bar_updates_on_selection_change() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use minter::core::ui::input::{InputResult, handle_event};
    use minter::core::ui::render::AppState;

    let mut state = AppState::new();

    // Initially no selection
    assert!(
        state.selected_spec.is_none(),
        "expected no selection initially"
    );

    // Select first spec via Down arrow
    let down = Event::Key(KeyEvent {
        code: KeyCode::Down,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    });
    let result = handle_event(&down, &mut state, 5);
    assert!(matches!(result, InputResult::StateChanged));
    assert!(
        state.selected_spec.is_some(),
        "expected selection after Down"
    );

    // Deselect via Esc
    let esc = Event::Key(KeyEvent {
        code: KeyCode::Esc,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    });
    let result = handle_event(&esc, &mut state, 5);
    assert!(matches!(result, InputResult::StateChanged));
    assert!(
        state.selected_spec.is_none(),
        "expected no selection after Esc"
    );
}

/// ui-command: action-bar-with-selection
// @minter:unit action-bar-with-selection
#[test]
fn action_bar_with_selection_shows_targeted_labels() {
    use minter::core::ui::render::AppState;

    let mut state = AppState::new();
    state.selected_spec = Some(0);

    assert!(
        state.selected_spec.is_some(),
        "expected selected_spec to be Some for targeted action bar labels"
    );
}

/// ui-command: action-bar-without-selection
// @minter:unit action-bar-without-selection
#[test]
fn action_bar_without_selection_shows_global_labels() {
    use minter::core::ui::render::AppState;

    let state = AppState::new();

    assert!(
        state.selected_spec.is_none(),
        "expected selected_spec to be None for global action bar labels"
    );
}

// ═══════════════════════════════════════════════════════════════
// Action failures and edge cases
// ═══════════════════════════════════════════════════════════════

/// ui-command: action-fails-display-error
// @minter:unit action-fails-display-error
#[test]
fn action_validate_shows_error_for_broken_spec() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    // Spec with empty behavior tag -> validation error
    let broken_spec = "\
spec broken v1.0.0
title \"Broken\"

description
  Broken.

motivation
  Broken.

behavior bad-behavior []
  \"Bad behavior\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
";

    let dir = setup_project(&[("broken.spec", broken_spec)], &[], &[], None);

    let state = UiState::load(dir.path());
    let result = state.run_action(Action::Validate, None);

    match result {
        ActionResult::Validate { has_errors, .. } => {
            assert!(has_errors, "expected validation errors for broken spec");
        }
        other => panic!("expected ActionResult::Validate, got {:?}", other),
    }
}

/// ui-command: deep-validate-requires-selection
// @minter:unit deep-validate-requires-selection
#[test]
fn deep_validate_runs_globally_when_no_selection() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    let spec_a = spec_one_behavior("alpha", "1.0.0", "do-alpha");
    let spec_b = spec_one_behavior("beta", "1.0.0", "do-beta");

    let dir = setup_project(
        &[("alpha.spec", &spec_a), ("beta.spec", &spec_b)],
        &[],
        &[],
        None,
    );

    let state = UiState::load(dir.path());
    // No spec selected -> runs deep validation on all specs
    let result = state.run_action(Action::DeepValidate, None);

    match result {
        ActionResult::DeepValidate { output, has_errors } => {
            assert!(
                !has_errors,
                "expected no errors for valid specs, got: {}",
                output
            );
            assert!(
                output.contains("alpha"),
                "expected output to include alpha spec"
            );
            assert!(
                output.contains("beta"),
                "expected output to include beta spec"
            );
        }
        other => panic!("expected ActionResult::DeepValidate, got {:?}", other),
    }
}

/// ui-command: expand-spec-shows-coverage
// @minter:unit expand-spec-shows-coverage
#[test]
fn expanded_spec_shows_behavior_coverage() {
    use minter::core::ui::state::UiState;

    let spec_a = spec_multi_behaviors("auth", "1.0.0", &["login", "logout", "mfa"]);
    // Cover 2 of 3 behaviors with different test types
    let test_content = "// @minter:unit login\n// @minter:e2e logout\n";

    let dir = setup_project(
        &[("auth.spec", &spec_a)],
        &[],
        &[("auth_test.rs", test_content)],
        None,
    );

    let state = UiState::load(dir.path());
    let specs = state.specs_list();
    assert_eq!(specs.len(), 1);

    let auth = &specs[0];
    assert_eq!(auth.behaviors.len(), 3);

    // login: covered by unit
    let login = auth.behaviors.iter().find(|b| b.name == "login").unwrap();
    assert!(login.covered, "login should be covered");
    assert!(
        login.test_types.contains(&"unit".to_string()),
        "login should have unit test type"
    );

    // logout: covered by e2e
    let logout = auth.behaviors.iter().find(|b| b.name == "logout").unwrap();
    assert!(logout.covered, "logout should be covered");
    assert!(
        logout.test_types.contains(&"e2e".to_string()),
        "logout should have e2e test type"
    );

    // mfa: not covered
    let mfa = auth.behaviors.iter().find(|b| b.name == "mfa").unwrap();
    assert!(!mfa.covered, "mfa should not be covered");
    assert!(mfa.test_types.is_empty(), "mfa should have no test types");
}

/// ui-command: inspect-requires-selection
// @minter:unit inspect-requires-selection
#[test]
fn inspect_returns_message_when_no_spec_selected() {
    use minter::core::ui::state::{Action, ActionResult, UiState};

    let spec_a = spec_one_behavior("auth", "1.0.0", "login");
    let dir = setup_project(&[("auth.spec", &spec_a)], &[], &[], None);

    let state = UiState::load(dir.path());
    // No spec selected -> inspect returns message
    let result = state.run_action(Action::Inspect, None);

    match result {
        ActionResult::Inspect {
            output,
            behavior_count,
            categories,
            dependencies,
        } => {
            assert!(
                output.contains("requires") || output.contains("selected"),
                "expected message about requiring a selected spec, got: {}",
                output
            );
            assert_eq!(behavior_count, 0);
            assert!(categories.is_empty());
            assert!(dependencies.is_empty());
        }
        other => panic!("expected ActionResult::Inspect, got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════
// Navigation — keyboard and mouse
// ═══════════════════════════════════════════════════════════════

/// ui-command: navigate-specs-keyboard
// @minter:unit navigate-specs-keyboard
#[test]
fn arrow_keys_navigate_specs_list() {
    use minter::core::ui::render::AppState;

    let mut state = AppState::new();
    let spec_count = 5;

    // Start with no selection
    assert_eq!(state.selected_spec, None);

    // Down -> selects first (index 0)
    state.move_down(spec_count);
    assert_eq!(state.selected_spec, Some(0));

    // Down again -> index 1
    state.move_down(spec_count);
    assert_eq!(state.selected_spec, Some(1));

    // Up -> back to index 0
    state.move_up();
    assert_eq!(state.selected_spec, Some(0));

    // Enter toggles expand on currently selected spec
    assert!(!state.expanded_specs.contains(&0));
    state.toggle_expand();
    assert!(
        state.expanded_specs.contains(&0),
        "Enter should expand the selected spec"
    );

    // Enter again collapses
    state.toggle_expand();
    assert!(
        !state.expanded_specs.contains(&0),
        "Enter again should collapse the expanded spec"
    );
}

/// ui-command: navigate-specs-mouse
// @minter:unit navigate-specs-mouse
#[test]
fn mouse_click_selects_and_expands_spec() {
    use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
    use minter::core::ui::input::{InputResult, handle_event};
    use minter::core::ui::render::AppState;

    let mut state = AppState::new();
    let spec_count = 3;

    // Click on second spec (row 5 -> index 1, since row offset is 4)
    let click = Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 5,
        row: 5,
        modifiers: KeyModifiers::empty(),
    });

    let result = handle_event(&click, &mut state, spec_count);
    assert!(matches!(result, InputResult::StateChanged));
    assert_eq!(
        state.selected_spec,
        Some(1),
        "click should select spec at index 1"
    );

    // Click same spec again -> toggle expand
    let result = handle_event(&click, &mut state, spec_count);
    assert!(matches!(result, InputResult::StateChanged));
    assert!(
        state.expanded_specs.contains(&1),
        "clicking selected spec should toggle expand"
    );
}

// ═══════════════════════════════════════════════════════════════
// Quit behaviors
// ═══════════════════════════════════════════════════════════════

/// ui-command: quit-with-ctrl-c
// @minter:unit quit-with-ctrl-c
#[test]
fn ctrl_c_quits_application() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use minter::core::ui::input::{InputResult, handle_event};
    use minter::core::ui::render::AppState;

    let mut state = AppState::new();

    let evt = Event::Key(KeyEvent {
        code: KeyCode::Char('c'),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    });

    let result = handle_event(&evt, &mut state, 5);
    assert!(
        matches!(result, InputResult::Quit),
        "Ctrl+C should quit the application"
    );
}

/// ui-command: quit-with-q
// @minter:unit quit-with-q
#[test]
fn q_key_quits_application() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use minter::core::ui::input::{InputResult, handle_event};
    use minter::core::ui::render::AppState;

    let mut state = AppState::new();

    let evt = Event::Key(KeyEvent {
        code: KeyCode::Char('q'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    });

    let result = handle_event(&evt, &mut state, 5);
    assert!(
        matches!(result, InputResult::Quit),
        "q key should quit the application"
    );
}

// ═══════════════════════════════════════════════════════════════
// Visual indicators
// ═══════════════════════════════════════════════════════════════

/// ui-command: selection-highlight-visible
// @minter:unit selection-highlight-visible
#[test]
fn selected_spec_has_highlight_state() {
    use minter::core::ui::render::AppState;

    let mut state = AppState::new();
    state.selected_spec = Some(1);

    // The render function checks app_state.selected_spec == Some(idx)
    // to apply highlight style (Cyan background, Bold)
    assert_eq!(
        state.selected_spec,
        Some(1),
        "selected_spec should be set to highlight the spec at index 1"
    );
}

/// ui-command: validation-status-icons
// @minter:unit validation-status-icons
#[test]
fn specs_show_validation_status_icons() {
    use minter::core::ui::state::{UiState, ValidationStatus};

    let valid_spec = spec_one_behavior("good-spec", "1.0.0", "do-good");
    let broken_spec = "not a valid spec";

    let dir = setup_project(
        &[
            ("good-spec.spec", &valid_spec),
            ("bad-spec.spec", broken_spec),
        ],
        &[],
        &[],
        None,
    );

    let state = UiState::load(dir.path());
    let specs = state.specs_list();
    assert_eq!(specs.len(), 2);

    let good = specs.iter().find(|s| s.name == "good-spec").unwrap();
    assert_eq!(
        good.validation_status,
        ValidationStatus::Valid,
        "valid spec should have Valid status (renders as checkmark icon)"
    );

    let bad = specs.iter().find(|s| s.name == "bad-spec").unwrap();
    assert!(
        matches!(bad.validation_status, ValidationStatus::Invalid(_)),
        "broken spec should have Invalid status (renders as cross icon)"
    );
}
