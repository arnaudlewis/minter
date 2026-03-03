mod common;

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use common::{
    VALID_NFR, VALID_SPEC, minter, temp_dir_with_spec_and_nfrs, temp_dir_with_specs, temp_nfr,
    temp_spec,
};
use predicates::prelude::*;

/// Get the path to the minter binary.
fn minter_bin() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin!("minter").to_path_buf()
}

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

/// cli.spec: show-help
#[test]
fn show_help() {
    // --help flag prints usage with all seven commands
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
        .stdout(predicate::str::contains("guide"));

    // No arguments also prints usage
    minter()
        .assert()
        .success()
        .stdout(predicate::str::contains("minter"));
}

/// cli.spec: show-version
#[test]
fn show_version() {
    minter()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

/// cli.spec: route-validate-file — routes to validate with file args
#[test]
fn validate_command_routing() {
    let (_dir, path) = temp_spec("valid", VALID_SPEC);
    minter().arg("validate").arg(&path).assert().success();
}

/// cli.spec: route-validate-file-deep — routes to validate with --deep
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

/// cli.spec: reject-unknown-command
#[test]
fn reject_unknown_command() {
    minter()
        .arg("frobnicate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("frobnicate"));
}

/// cli.spec: reject-missing-required-argument
#[test]
fn reject_no_files() {
    minter()
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not())
        .stderr(predicate::str::contains("validate"));
}

/// cli.spec: reject-non-spec-extension
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

/// cli.spec: route-validate-nfr-file
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

/// cli.spec: route-inspect-nfr
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

/// cli.spec: reject-unknown-flag
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

/// cli.spec: handle-mixed-valid-invalid-files
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

/// cli.spec: route-validate-folder
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

/// cli.spec: route-format
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

/// cli.spec: route-scaffold-spec
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

/// cli.spec: route-scaffold-nfr
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

/// cli.spec: route-inspect
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

/// cli.spec: route-graph
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

/// cli.spec: route-graph-impacted
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

/// cli.spec: accept-deep-on-folder-as-noop
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

/// cli.spec: route-watch-folder
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

/// cli.spec: route-validate-mixed-directory
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

/// cli.spec: route-watch-file
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

/// cli.spec: route-guide
#[test]
fn route_guide() {
    minter()
        .args(&["guide", "methodology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("spec"))
        .stdout(predicate::str::contains("NFR"));
}

/// cli.spec: route-watch-nfr-file
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
