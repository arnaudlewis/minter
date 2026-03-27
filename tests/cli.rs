mod common;

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use common::{
    VALID_NFR, VALID_SPEC, minter, minter_bin, temp_dir_with_spec_and_nfrs, temp_dir_with_specs,
    temp_nfr, temp_spec,
};
use predicates::prelude::*;

/// Wait for a line matching a predicate from a background reader, with timeout.
fn wait_for_line(
    rx: &mpsc::Receiver<String>,
    predicate: impl Fn(&str) -> bool,
    timeout: Duration,
) -> Option<String> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            return None;
        }
        match rx.recv_timeout(remaining) {
            Ok(line) => {
                if predicate(&line) {
                    return Some(line);
                }
            }
            Err(_) => return None,
        }
    }
}

/// Spawn a background thread that reads lines and sends them over a channel.
fn spawn_line_reader(reader: impl std::io::Read + Send + 'static) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let buf = BufReader::new(reader);
        for line in buf.lines() {
            match line {
                Ok(l) => {
                    if tx.send(l).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    rx
}

// ═══════════════════════════════════════════════════════════════
// Happy paths (cli.spec)
// ═══════════════════════════════════════════════════════════════

// @minter:e2e show-help
#[test]
fn show_help() {
    // --help flag prints usage with all eight commands
    minter()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("minter"))
        .stdout(predicate::str::contains("validate"))
        .stdout(predicate::str::contains("watch"))
        .stdout(predicate::str::contains("format"))
        .stdout(predicate::str::contains("scaffold"))
        .stdout(predicate::str::contains("inspect"))
        .stdout(predicate::str::contains("graph"))
        .stdout(predicate::str::contains("guide"))
        .stdout(predicate::str::contains("coverage"))
        .stdout(predicate::str::contains("lock"))
        .stdout(predicate::str::contains("ci"))
        .stdout(predicate::str::contains("web"));

    // No arguments also prints usage
    minter()
        .assert()
        .success()
        .stdout(predicate::str::contains("minter"));
}

// @minter:e2e show-version
#[test]
fn show_version() {
    minter()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

// @minter:e2e route-validate-file
#[test]
fn validate_command_routing() {
    let (_dir, path) = temp_spec("valid", VALID_SPEC);
    minter().arg("validate").arg(&path).assert().success();
}

// @minter:e2e route-validate-file-deep
#[test]
fn validate_deep_flag_routing() {
    let (_dir, path) = temp_spec("valid", VALID_SPEC);
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&path)
        .assert()
        .success();
}

// ═══════════════════════════════════════════════════════════════
// Error cases (cli.spec)
// ═══════════════════════════════════════════════════════════════

// @minter:e2e reject-unknown-command
#[test]
fn reject_unknown_command() {
    // clap reports the unknown command name; it does not list all valid commands
    minter()
        .arg("frobnicate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("frobnicate"));
}

// @minter:e2e reject-missing-required-argument
#[test]
fn reject_no_files() {
    // When no config and no default dirs exist, validate reports an error
    let dir = tempfile::TempDir::new().unwrap();
    minter()
        .arg("validate")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// @minter:e2e reject-non-spec-extension
#[test]
fn reject_non_spec_extension() {
    let (_dir, path) = common::temp_file("readme.md", "not a spec");
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(".spec"));
}

// @minter:e2e route-validate-nfr-file
#[test]
fn route_validate_nfr_file() {
    let (_dir, path) = temp_nfr("perf", VALID_NFR);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("performance"));
}

// @minter:e2e route-inspect-nfr
#[test]
fn route_inspect_nfr() {
    let (_dir, path) = temp_nfr("perf", VALID_NFR);
    minter()
        .arg("inspect")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("performance"))
        .stdout(predicate::str::contains("constraint"));
}

// @minter:e2e reject-unknown-flag
#[test]
fn reject_unknown_flag() {
    minter()
        .arg("validate")
        .arg("--frobnicate")
        .arg("file.spec")
        .assert()
        .failure()
        .stderr(predicate::str::contains("frobnicate"));
}

// ═══════════════════════════════════════════════════════════════
// Edge cases (cli.spec)
// ═══════════════════════════════════════════════════════════════

// @minter:e2e handle-mixed-valid-invalid-files
#[test]
fn handle_mixed_valid_invalid_files() {
    let (_dir, path) = temp_spec("valid", VALID_SPEC);
    minter()
        .arg("validate")
        .arg(&path)
        .arg("nonexistent.spec")
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent.spec"));
}

// ═══════════════════════════════════════════════════════════════
// New command routing tests (cli.spec)
// ═══════════════════════════════════════════════════════════════

// @minter:e2e route-validate-folder
#[test]
fn route_validate_folder() {
    let (_dir, dir_path) = temp_dir_with_specs(&[("route-val", VALID_SPEC)]);
    minter()
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("test-spec"));
}

// @minter:e2e route-format
#[test]
fn route_format() {
    minter()
        .arg("format")
        .arg("spec")
        .assert()
        .success()
        .stdout(predicate::str::contains("spec"))
        .stdout(predicate::str::contains("behavior"))
        .stdout(predicate::str::contains("given"));
}

// @minter:e2e route-scaffold-spec
#[test]
fn route_scaffold_spec() {
    minter()
        .arg("scaffold")
        .arg("spec")
        .assert()
        .success()
        .stdout(predicate::str::contains("spec"))
        .stdout(predicate::str::contains("title"))
        .stdout(predicate::str::contains("behavior"));
}

// @minter:e2e route-scaffold-nfr
#[test]
fn route_scaffold_nfr() {
    minter()
        .arg("scaffold")
        .arg("nfr")
        .arg("performance")
        .assert()
        .success()
        .stdout(predicate::str::contains("performance"));
}

// @minter:e2e route-inspect
#[test]
fn route_inspect() {
    let spec = "\
spec inspect-route v1.0.0
title \"Inspect Route\"

description
  Testing inspect routing.

motivation
  Routing.

behavior one [happy_path]
  \"First\"

  given
    Ready

  when act

  then returns r
    assert x == \"1\"

behavior two [happy_path]
  \"Second\"

  given
    Ready

  when act

  then returns r
    assert x == \"2\"

behavior three [error_case]
  \"Third\"

  given
    Ready

  when act

  then returns r
    assert x == \"3\"
";
    let (_dir, path) = temp_spec("inspect-route", spec);
    minter()
        .arg("inspect")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("3 behaviors"));
}

// @minter:e2e route-graph
#[test]
fn route_graph() {
    let spec_a = "\
spec graph-a v1.0.0
title \"Graph A\"

description
  A.

motivation
  A.

behavior do-a [happy_path]
  \"Does A\"

  given
    Ready

  when act

  then returns r
    assert x == \"1\"
";
    let spec_b = "\
spec graph-b v1.0.0
title \"Graph B\"

description
  B.

motivation
  B.

behavior do-b [happy_path]
  \"Does B\"

  given
    Ready

  when act

  then returns r
    assert x == \"1\"
";
    let (_dir, dir_path) = temp_dir_with_specs(&[("graph-a", spec_a), ("graph-b", spec_b)]);
    minter()
        .arg("graph")
        .arg(&dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("graph-a"))
        .stdout(predicate::str::contains("graph-b"));
}

// @minter:e2e route-graph-impacted
#[test]
fn route_graph_impacted() {
    let spec_a = "\
spec imp-a v1.0.0
title \"Imp A\"

description
  A.

motivation
  A.

behavior do-a [happy_path]
  \"Does A\"

  given
    Ready

  when act

  then returns r
    assert x == \"1\"

depends on imp-b >= 1.0.0
";
    let spec_b = "\
spec imp-b v1.0.0
title \"Imp B\"

description
  B.

motivation
  B.

behavior do-b [happy_path]
  \"Does B\"

  given
    Ready

  when act

  then returns r
    assert x == \"1\"
";
    let (_dir, dir_path) = temp_dir_with_specs(&[("imp-a", spec_a), ("imp-b", spec_b)]);
    minter()
        .arg("graph")
        .arg(&dir_path)
        .arg("--impacted")
        .arg("imp-b")
        .assert()
        .success()
        .stdout(predicate::str::contains("imp-a"));
}

// @minter:e2e accept-deep-on-folder-as-noop
#[test]
fn accept_deep_on_folder_as_noop() {
    let (_dir, dir_path) = temp_dir_with_specs(&[("deep-noop", VALID_SPEC)]);
    minter()
        .arg("validate")
        .arg("--deep")
        .arg(&dir_path)
        .assert()
        .success();
}

// @minter:e2e route-watch-folder
#[test]
fn route_watch_folder() {
    let (_dir, dir_path) = temp_dir_with_specs(&[("watch-route", VALID_SPEC)]);

    let mut child = Command::new(minter_bin())
        .arg("watch")
        .arg(&dir_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let rx = spawn_line_reader(stdout);

    let line = wait_for_line(
        &rx,
        |l| l.to_lowercase().contains("watching"),
        Duration::from_secs(10),
    );

    assert!(line.is_some(), "should see 'watching' message in stdout");

    // Process should still be alive (long-running watch)
    assert!(
        child.try_wait().unwrap().is_none(),
        "watch process should still be running"
    );

    let _ = child.kill();
    let _ = child.wait();
}

// @minter:e2e route-validate-mixed-directory
#[test]
fn route_validate_mixed_directory() {
    let spec_content = "\
spec mixed-route v1.0.0
title \"Mixed Route\"

description
  A spec with nfr references.

motivation
  Testing mixed directory routing.

nfr
  performance#api-response-time

behavior do-thing [happy_path]
  \"Do the thing\"

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, dir_path) =
        temp_dir_with_spec_and_nfrs("mixed-route", spec_content, &[("performance", VALID_NFR)]);

    let output = minter()
        .env("NO_COLOR", "1")
        .arg("validate")
        .arg(&dir_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("mixed-route"),
        "Should validate .spec file, got: {stdout}"
    );
    assert!(
        stdout.contains("performance"),
        "Should validate .nfr file, got: {stdout}"
    );
}

// @minter:e2e route-watch-file
#[test]
fn route_watch_file() {
    let (_dir, path) = temp_spec("watch-file-route", VALID_SPEC);

    let mut child = Command::new(minter_bin())
        .arg("watch")
        .arg(&path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let rx = spawn_line_reader(stdout);

    let line = wait_for_line(
        &rx,
        |l| l.to_lowercase().contains("watching"),
        Duration::from_secs(10),
    );

    assert!(line.is_some(), "should see 'watching' message in stdout");
    let line = line.unwrap();
    assert!(
        line.contains("watch-file-route") || line.contains(".spec"),
        "output should reference the watched file: {line}"
    );

    // Process should still be alive (long-running watch)
    assert!(
        child.try_wait().unwrap().is_none(),
        "watch process should still be running"
    );

    let _ = child.kill();
    let _ = child.wait();
}

// @minter:e2e route-guide
#[test]
fn route_guide() {
    minter()
        .args(&["guide", "methodology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("spec"))
        .stdout(predicate::str::contains("NFR"));
}

// @minter:e2e route-coverage-directory
#[test]
fn route_coverage_directory() {
    let dir = tempfile::TempDir::new().unwrap();
    let spec_dir = dir.path().join("specs");
    std::fs::create_dir(&spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("a.spec"),
        "\
spec a v1.0.0
title \"A\"

description
  A.

motivation
  A.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
",
    )
    .unwrap();
    std::fs::write(dir.path().join("a.test.ts"), "// @minter:unit do-thing\n").unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("a v1.0.0"))
        .stdout(predicate::str::contains("1/1"));
}

// @minter:e2e route-coverage-file
#[test]
fn route_coverage_file() {
    let dir = tempfile::TempDir::new().unwrap();
    let spec_path = dir.path().join("a.spec");
    std::fs::write(
        &spec_path,
        "\
spec a v1.0.0
title \"A\"

description
  A.

motivation
  A.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
",
    )
    .unwrap();
    std::fs::write(dir.path().join("a.test.ts"), "// @minter:unit do-thing\n").unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_path)
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("a v1.0.0"))
        .stdout(predicate::str::contains("1/1"));
}

// @minter:e2e route-coverage-with-scan
#[test]
fn route_coverage_with_scan() {
    let dir = tempfile::TempDir::new().unwrap();
    let spec_dir = dir.path().join("specs");
    std::fs::create_dir(&spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("a.spec"),
        "\
spec a v1.0.0
title \"A\"

description
  A.

motivation
  A.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
",
    )
    .unwrap();
    let tests_dir = dir.path().join("tests");
    std::fs::create_dir(&tests_dir).unwrap();
    std::fs::write(tests_dir.join("a.test.ts"), "// @minter:unit do-thing\n").unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .arg("--scan")
        .arg(&tests_dir)
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("a v1.0.0"))
        .stdout(predicate::str::contains("unit"));
}

// @minter:e2e route-coverage-with-format
#[test]
fn route_coverage_with_format() {
    let dir = tempfile::TempDir::new().unwrap();
    let spec_dir = dir.path().join("specs");
    std::fs::create_dir(&spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("a.spec"),
        "\
spec a v1.0.0
title \"A\"

description
  A.

motivation
  A.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
",
    )
    .unwrap();
    std::fs::write(dir.path().join("a.test.ts"), "// @minter:unit do-thing\n").unwrap();

    minter()
        .arg("coverage")
        .arg(&spec_dir)
        .arg("--format")
        .arg("json")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("total_behaviors"));
}

// @minter:e2e route-watch-nfr-file
#[test]
fn route_watch_nfr_file() {
    let (_dir, path) = temp_nfr("perf", VALID_NFR);

    let mut child = Command::new(minter_bin())
        .arg("watch")
        .arg(&path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let rx = spawn_line_reader(stdout);

    let line = wait_for_line(
        &rx,
        |l| l.to_lowercase().contains("watching"),
        Duration::from_secs(10),
    );

    assert!(line.is_some(), "should see 'watching' message in stdout");
    let line = line.unwrap();
    assert!(
        line.contains("performance.nfr") || line.contains(".nfr"),
        "output should reference the watched .nfr file: {line}"
    );

    assert!(
        child.try_wait().unwrap().is_none(),
        "watch process should still be running"
    );

    let _ = child.kill();
    let _ = child.wait();
}

// ═══════════════════════════════════════════════════════════════
// Lock / CI / Web routing (cli.spec)
// ═══════════════════════════════════════════════════════════════

/// cli: route-lock
// @minter:e2e route-lock
#[test]
fn route_lock() {
    let dir = tempfile::TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    std::fs::create_dir(&spec_dir).unwrap();
    std::fs::write(spec_dir.join("a.spec"), VALID_SPEC).unwrap();

    let test_dir = dir.path().join("tests");
    std::fs::create_dir(&test_dir).unwrap();
    std::fs::write(test_dir.join("a_test.rs"), "// @minter:unit do-thing\n").unwrap();

    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();
}

/// cli: route-ci
// @minter:e2e route-ci
#[test]
fn route_ci() {
    let dir = tempfile::TempDir::new().unwrap();

    let spec_dir = dir.path().join("specs");
    std::fs::create_dir(&spec_dir).unwrap();
    std::fs::write(spec_dir.join("a.spec"), VALID_SPEC).unwrap();

    let test_dir = dir.path().join("tests");
    std::fs::create_dir(&test_dir).unwrap();
    std::fs::write(test_dir.join("a_test.rs"), "// @minter:unit do-thing\n").unwrap();

    // Generate a lock file first
    minter()
        .arg("lock")
        .current_dir(dir.path())
        .assert()
        .success();

    // Then run CI
    minter()
        .arg("ci")
        .current_dir(dir.path())
        .assert()
        .success();
}

/// cli: route-web
// @minter:e2e route-web
#[test]
fn route_web() {
    // web starts a server, so we test via --help to verify routing
    minter()
        .args(&["web", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("web"));
}
