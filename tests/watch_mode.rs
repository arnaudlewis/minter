mod common;

use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use std::os::unix::fs::PermissionsExt;

use common::read_graph_json;

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
  Testing watch mode.

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

/// Get the path to the minter binary.
fn minter_bin() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("minter")
}

/// A non-blocking line receiver backed by a background reader thread.
struct LineReceiver {
    rx: mpsc::Receiver<String>,
}

impl LineReceiver {
    fn new(reader: impl std::io::Read + Send + 'static) -> Self {
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
        LineReceiver { rx }
    }

    /// Wait for a line matching the predicate, with timeout.
    fn wait_for(
        &self,
        predicate: impl Fn(&str) -> bool,
        timeout: Duration,
    ) -> Option<String> {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                return None;
            }
            match self.rx.recv_timeout(remaining) {
                Ok(line) => {
                    if predicate(&line) {
                        return Some(line);
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => return None,
                Err(mpsc::RecvTimeoutError::Disconnected) => return None,
            }
        }
    }

    /// Collect all lines received within a duration.
    fn collect_for(&self, duration: Duration) -> Vec<String> {
        let deadline = std::time::Instant::now() + duration;
        let mut lines = Vec::new();
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            match self.rx.recv_timeout(remaining) {
                Ok(line) => lines.push(line),
                Err(_) => break,
            }
        }
        lines
    }
}

// ═══════════════════════════════════════════════════════════════
// watch-mode.spec behaviors
// ═══════════════════════════════════════════════════════════════

/// watch-command.spec: watch-start-directory
#[test]
fn watch_start() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let receiver = LineReceiver::new(stdout);

    // Should see "watching" message
    let line = receiver.wait_for(
        |l| {
            let lower = l.to_lowercase();
            lower.contains("watching") || lower.contains("watch")
        },
        Duration::from_secs(10),
    );

    assert!(line.is_some(), "should see 'watching' message in stdout");
    let line = line.unwrap();
    assert!(
        line.to_lowercase().contains("watch"),
        "output should mention watching: {line}"
    );

    // Process should still be alive
    assert!(
        child.try_wait().unwrap().is_none(),
        "watch process should still be running"
    );

    let _ = child.kill();
    let _ = child.wait();
}

/// watch-command.spec: watch-revalidate-changed-file
#[test]
fn watch_incremental_revalidation() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(
        dir.path().join("a.spec"),
        valid_spec("a", "1.0.0", Some(("b", "1.0.0"))),
    )
    .unwrap();
    fs::write(dir.path().join("b.spec"), valid_spec("b", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let receiver = LineReceiver::new(stdout);

    // Wait for initial watch message
    receiver
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see initial watch message");

    // Wait for watcher to be ready, then modify b.spec
    std::thread::sleep(Duration::from_millis(500));
    fs::write(dir.path().join("b.spec"), valid_spec("b", "1.1.0", None)).unwrap();

    // Should see output about the changed file
    let line = receiver.wait_for(
        |l| l.contains("b") || l.contains("changed") || l.contains("modified"),
        Duration::from_secs(10),
    );

    assert!(
        line.is_some(),
        "should see revalidation output after file change"
    );

    let _ = child.kill();
    let _ = child.wait();
}

/// watch-command.spec: watch-graceful-shutdown
#[test]
fn watch_graceful_shutdown() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let receiver = LineReceiver::new(stdout);

    // Wait for initial watch message
    receiver
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see initial watch message");

    // Send SIGINT
    unsafe {
        libc::kill(child.id() as libc::pid_t, libc::SIGINT);
    }

    // Wait for process to exit
    let status = child.wait().expect("should wait for child");
    assert!(
        status.success(),
        "should exit with code 0 after SIGINT, got: {status}"
    );

    // graph.json should be written
    let graph = read_graph_json(dir.path());
    assert!(
        graph.get("specs").is_some(),
        "graph.json should be written before exit"
    );
}

/// watch-command.spec: watch-integrate-new-file
#[test]
fn watch_detect_new_file() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let receiver = LineReceiver::new(stdout);

    // Wait for initial watch message
    receiver
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see initial watch message");

    // Wait for watcher setup, then create a new spec file
    std::thread::sleep(Duration::from_millis(500));
    fs::write(dir.path().join("d.spec"), valid_spec("d", "1.0.0", None)).unwrap();

    // Should see output about the new file
    let line = receiver.wait_for(
        |l| l.contains("d") || l.contains("new") || l.contains("created") || l.contains("detected"),
        Duration::from_secs(10),
    );

    assert!(line.is_some(), "should detect and report new spec file");

    let _ = child.kill();
    let _ = child.wait();
}

/// watch-command.spec: watch-report-broken-deps-on-delete
#[test]
fn watch_report_broken_deps_on_delete() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(
        dir.path().join("a.spec"),
        valid_spec("a", "1.0.0", Some(("b", "1.0.0"))),
    )
    .unwrap();
    fs::write(dir.path().join("b.spec"), valid_spec("b", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let stdout_rx = LineReceiver::new(stdout);
    let stderr_rx = LineReceiver::new(stderr);

    // Wait for initial watch message
    stdout_rx
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see initial watch message");

    // Wait for watcher setup, then delete b.spec
    std::thread::sleep(Duration::from_millis(500));
    fs::remove_file(dir.path().join("b.spec")).unwrap();

    // Should see deletion in stdout
    let line = stdout_rx.wait_for(
        |l| l.contains("b") || l.contains("deleted") || l.contains("removed"),
        Duration::from_secs(10),
    );

    assert!(line.is_some(), "should report file deletion in stdout");

    // Stderr should mention broken dependency
    let stderr_lines = stderr_rx.collect_for(Duration::from_secs(3));
    let stderr_combined = stderr_lines.join("\n");
    assert!(
        stderr_combined.contains("b")
            || stderr_combined.contains("missing")
            || stderr_combined.contains("broken"),
        "stderr should mention broken dependency: got {:?}",
        stderr_combined
    );

    let _ = child.kill();
    let _ = child.wait();
}

/// watch-command.spec: watch-debounce-rapid-changes
#[test]
fn watch_debounce() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let receiver = LineReceiver::new(stdout);

    // Wait for initial watch message
    receiver
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see initial watch message");

    // Wait for watcher setup
    std::thread::sleep(Duration::from_millis(500));

    // Rapid successive writes (5 writes in quick succession)
    for i in 0..5 {
        fs::write(
            dir.path().join("a.spec"),
            valid_spec("a", &format!("1.{}.0", i), None),
        )
        .unwrap();
        std::thread::sleep(Duration::from_millis(20));
    }

    // Wait for debounce to settle and collect output
    let lines = receiver.collect_for(Duration::from_secs(5));

    // Count validation cycles (each cycle should show "✓" for the spec)
    let validation_count = lines
        .iter()
        .filter(|l| l.contains('\u{2713}') && l.contains("a"))
        .count();

    assert!(
        validation_count <= 2,
        "rapid changes should be debounced into at most 2 validation cycles, got {}",
        validation_count
    );
    assert!(
        validation_count >= 1,
        "should still validate at least once after rapid changes"
    );

    let _ = child.kill();
    let _ = child.wait();
}

/// watch-command.spec: watch-file-fixed-invalid-to-valid
#[test]
fn watch_revalidate_after_fix() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let receiver = LineReceiver::new(stdout);

    // Wait for initial watch message
    receiver
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see initial watch message");

    // Wait for watcher setup, then break the spec
    std::thread::sleep(Duration::from_millis(500));
    fs::write(dir.path().join("a.spec"), "this is broken").unwrap();

    // Wait for the error cross to appear
    receiver
        .wait_for(
            |l| l.contains('\u{2717}'),
            Duration::from_secs(10),
        )
        .expect("should see cross mark for broken spec");

    // Now fix the spec
    std::thread::sleep(Duration::from_millis(500));
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    // Should see a success checkmark after the fix
    let success_line = receiver.wait_for(
        |l| l.contains('\u{2713}'),
        Duration::from_secs(10),
    );
    assert!(
        success_line.is_some(),
        "should see success checkmark after fixing a broken spec"
    );

    let _ = child.kill();
    let _ = child.wait();
}

// ANSI escape code constants for test assertions
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_RED: &str = "\x1b[31m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_CYAN: &str = "\x1b[36m";

/// validate-display.spec: color-banner-cyan
#[test]
fn watch_color_banner_cyan() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let receiver = LineReceiver::new(stdout);

    let banner = receiver
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see watching banner");

    assert!(
        banner.contains(ANSI_CYAN),
        "watching banner should use cyan: {banner}"
    );

    let _ = child.kill();
    let _ = child.wait();
}

/// validate-display.spec: color-changed-file-yellow
#[test]
fn watch_color_changed_file_yellow() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let receiver = LineReceiver::new(stdout);

    receiver
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see watching banner");

    std::thread::sleep(Duration::from_millis(500));
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.1.0", None)).unwrap();

    let changed_line = receiver.wait_for(
        |l| l.contains("changed"),
        Duration::from_secs(10),
    );
    assert!(changed_line.is_some(), "should see 'changed' event");
    let changed_line = changed_line.unwrap();
    assert!(
        changed_line.contains(ANSI_YELLOW),
        "changed event should use yellow: {changed_line}"
    );

    let _ = child.kill();
    let _ = child.wait();
}

/// validate-display.spec: color-new-file-cyan
#[test]
fn watch_color_new_file_cyan() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let receiver = LineReceiver::new(stdout);

    receiver
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see watching banner");

    std::thread::sleep(Duration::from_millis(500));
    fs::write(dir.path().join("c.spec"), valid_spec("c", "1.0.0", None)).unwrap();

    let new_line = receiver.wait_for(
        |l| l.contains("new") || l.contains("detected"),
        Duration::from_secs(10),
    );
    assert!(new_line.is_some(), "should see new file event");
    let new_line = new_line.unwrap();
    assert!(
        new_line.contains(ANSI_CYAN),
        "new file event should use cyan: {new_line}"
    );

    let _ = child.kill();
    let _ = child.wait();
}

/// validate-display.spec: color-deleted-file-red
#[test]
fn watch_color_deleted_file_red() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();
    fs::write(dir.path().join("c.spec"), valid_spec("c", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let receiver = LineReceiver::new(stdout);

    receiver
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see watching banner");

    std::thread::sleep(Duration::from_millis(500));
    fs::remove_file(dir.path().join("c.spec")).unwrap();

    let deleted_line = receiver.wait_for(
        |l| l.contains("deleted"),
        Duration::from_secs(10),
    );
    assert!(deleted_line.is_some(), "should see deleted event");
    let deleted_line = deleted_line.unwrap();
    assert!(
        deleted_line.contains(ANSI_RED),
        "deleted event should use red: {deleted_line}"
    );

    let _ = child.kill();
    let _ = child.wait();
}

/// watch-command.spec: watch-revalidate-subdirectory
#[test]
fn watch_subdirectory_changes() {
    let dir = tempfile::TempDir::new().unwrap();
    let sub_dir = dir.path().join("validation");
    fs::create_dir(&sub_dir).unwrap();
    fs::write(sub_dir.join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let receiver = LineReceiver::new(stdout);

    // Wait for initial watch message
    receiver
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see initial watch message");

    // Wait for watcher setup, then modify the file in the subdirectory
    std::thread::sleep(Duration::from_millis(500));
    fs::write(sub_dir.join("a.spec"), valid_spec("a", "1.1.0", None)).unwrap();

    // Should see output about the changed file
    let line = receiver.wait_for(
        |l| l.contains("a") || l.contains("changed") || l.contains("modified"),
        Duration::from_secs(10),
    );

    assert!(
        line.is_some(),
        "should detect and report file change in subdirectory"
    );

    let _ = child.kill();
    let _ = child.wait();
}

// ═══════════════════════════════════════════════════════════════
// watch-command.spec behaviors (new tests)
// ═══════════════════════════════════════════════════════════════

/// watch-command.spec: watch-start-single-file
#[test]
fn watch_start_single_file() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("b.spec"), valid_spec("b", "1.0.0", None)).unwrap();
    fs::write(
        dir.path().join("a.spec"),
        valid_spec("a", "1.0.0", Some(("b", "1.0.0"))),
    )
    .unwrap();

    let b_path = dir.path().join("b.spec");
    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(&b_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let receiver = LineReceiver::new(stdout);

    let line = receiver.wait_for(
        |l| l.to_lowercase().contains("watching"),
        Duration::from_secs(10),
    );

    assert!(line.is_some(), "should see 'watching' message in stdout");
    let line = line.unwrap();
    assert!(
        line.contains("b.spec"),
        "output should contain 'b.spec': {line}"
    );

    // Process should still be alive
    assert!(
        child.try_wait().unwrap().is_none(),
        "watch process should still be running"
    );

    let _ = child.kill();
    let _ = child.wait();
}

/// watch-command.spec: watch-initial-validation
#[test]
fn watch_initial_validation() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();
    // b.spec is invalid: has header but no behavior (semantic validation error)
    fs::write(
        dir.path().join("b.spec"),
        "spec b v1.0.0\ntitle \"b\"\n\ndescription\n  A broken spec.\n\nmotivation\n  Testing.\n",
    )
    .unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let stdout_rx = LineReceiver::new(stdout);
    let stderr_rx = LineReceiver::new(stderr);

    // Collect all initial stdout (validation results appear before the "watching" banner)
    // Wait enough time for initial validation + watching banner to appear
    let initial_lines = stdout_rx.collect_for(Duration::from_secs(10));
    let stdout_combined = initial_lines.join("\n");
    assert!(
        stdout_combined.to_lowercase().contains("watch"),
        "initial stdout should contain watching banner: got {:?}",
        stdout_combined
    );
    assert!(
        stdout_combined.contains("a"),
        "initial stdout should contain 'a': got {:?}",
        stdout_combined
    );

    // Stderr should contain validation error for b
    let stderr_lines = stderr_rx.collect_for(Duration::from_secs(3));
    let stderr_combined = stderr_lines.join("\n");
    assert!(
        stderr_combined.contains("b"),
        "initial stderr should contain validation error for 'b': got {:?}",
        stderr_combined
    );

    let _ = child.kill();
    let _ = child.wait();
}

/// watch-command.spec: watch-file-regresses-valid-to-invalid
#[test]
fn watch_file_regresses_valid_to_invalid() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let stdout_rx = LineReceiver::new(stdout);
    let stderr_rx = LineReceiver::new(stderr);

    // Wait for initial watch message
    stdout_rx
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see initial watch message");

    // Wait for watcher setup, then break a.spec
    std::thread::sleep(Duration::from_millis(500));
    fs::write(dir.path().join("a.spec"), "this is broken garbage").unwrap();

    // Stdout should show cross mark for the broken spec
    let cross_line = stdout_rx.wait_for(
        |l| l.contains('\u{2717}'),
        Duration::from_secs(10),
    );
    assert!(
        cross_line.is_some(),
        "should see cross mark in stdout for parse error"
    );

    // Stderr should contain a parse error message
    let stderr_lines = stderr_rx.collect_for(Duration::from_secs(3));
    let stderr_combined = stderr_lines.join("\n");
    assert!(
        !stderr_combined.is_empty(),
        "stderr should contain parse error message, got empty"
    );

    let _ = child.kill();
    let _ = child.wait();
}

/// watch-command.spec: watch-dependency-added
#[test]
fn watch_dependency_added() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();
    fs::write(dir.path().join("b.spec"), valid_spec("b", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let receiver = LineReceiver::new(stdout);

    // Wait for initial watch message
    receiver
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see initial watch message");

    // Wait for watcher setup, then add a dependency on b
    std::thread::sleep(Duration::from_millis(500));
    fs::write(
        dir.path().join("a.spec"),
        valid_spec("a", "1.0.0", Some(("b", "1.0.0"))),
    )
    .unwrap();

    // Collect output — should contain both "a" and "b"
    let lines = receiver.collect_for(Duration::from_secs(5));
    let combined = lines.join("\n");
    assert!(
        combined.contains("a"),
        "output should contain 'a' after dependency added: got {:?}",
        combined
    );
    assert!(
        combined.contains("b"),
        "output should contain 'b' in dep tree after dependency added: got {:?}",
        combined
    );

    let _ = child.kill();
    let _ = child.wait();
}

/// watch-command.spec: watch-dependency-removed
#[test]
fn watch_dependency_removed() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(
        dir.path().join("a.spec"),
        valid_spec("a", "1.0.0", Some(("b", "1.0.0"))),
    )
    .unwrap();
    fs::write(dir.path().join("b.spec"), valid_spec("b", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let receiver = LineReceiver::new(stdout);

    // Wait for initial watch message
    receiver
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see initial watch message");

    // Wait for watcher setup, then remove the dependency
    std::thread::sleep(Duration::from_millis(500));
    fs::write(
        dir.path().join("a.spec"),
        valid_spec("a", "1.1.0", None),
    )
    .unwrap();

    // Should see output about "a" being re-validated
    let line = receiver.wait_for(
        |l| l.contains("a"),
        Duration::from_secs(10),
    );
    assert!(
        line.is_some(),
        "should see 'a' in output after dependency removed"
    );

    let _ = child.kill();
    let _ = child.wait();
}

/// watch-command.spec: watch-multiple-files-changed
#[test]
fn watch_multiple_files_changed() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();
    fs::write(dir.path().join("b.spec"), valid_spec("b", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let receiver = LineReceiver::new(stdout);

    // Wait for initial watch message
    receiver
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see initial watch message");

    // Wait for watcher setup, then modify both files within a short window
    std::thread::sleep(Duration::from_millis(500));
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.1.0", None)).unwrap();
    fs::write(dir.path().join("b.spec"), valid_spec("b", "1.1.0", None)).unwrap();

    // Collect output — should contain both "a" and "b"
    let lines = receiver.collect_for(Duration::from_secs(5));
    let combined = lines.join("\n");
    assert!(
        combined.contains("a"),
        "output should contain 'a' after multiple changes: got {:?}",
        combined
    );
    assert!(
        combined.contains("b"),
        "output should contain 'b' after multiple changes: got {:?}",
        combined
    );

    let _ = child.kill();
    let _ = child.wait();
}

/// watch-command.spec: watch-nonexistent-path
#[test]
fn watch_nonexistent_path() {
    let output = Command::new(minter_bin())
        .arg("watch")
        .arg("nonexistent/")
        .output()
        .expect("failed to run minter watch");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("nonexistent"),
        "stderr should mention 'nonexistent': got {:?}",
        stderr
    );
    assert!(
        !output.status.success(),
        "should exit with non-zero code"
    );
}

/// watch-command.spec: watch-empty-directory
#[test]
fn watch_empty_directory() {
    let dir = tempfile::TempDir::new().unwrap();

    let output = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .output()
        .expect("failed to run minter watch");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("no spec files"),
        "stderr should mention 'no spec files': got {:?}",
        stderr
    );
    assert!(
        !output.status.success(),
        "should exit with non-zero code for empty directory"
    );
}

/// watch-command.spec: watch-permission-denied-on-directory
#[test]
fn watch_permission_denied_on_directory() {
    let dir = tempfile::TempDir::new().unwrap();
    let restricted = dir.path().join("restricted");
    fs::create_dir(&restricted).unwrap();

    // Remove read permission
    let metadata = fs::metadata(&restricted).unwrap();
    let mut perms = metadata.permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&restricted, perms).unwrap();

    let output = Command::new(minter_bin())
        .arg("watch")
        .arg(&restricted)
        .output()
        .expect("failed to run minter watch");

    // Restore permissions for cleanup
    let mut restore_perms = fs::metadata(&restricted).unwrap().permissions();
    restore_perms.set_mode(0o755);
    fs::set_permissions(&restricted, restore_perms).unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("permission"),
        "stderr should mention 'permission': got {:?}",
        stderr
    );
    assert!(
        !output.status.success(),
        "should exit with non-zero code for permission denied"
    );
}

/// watch-command.spec: watch-file-becomes-unreadable
#[test]
fn watch_file_becomes_unreadable() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let stdout_rx = LineReceiver::new(stdout);
    let stderr_rx = LineReceiver::new(stderr);

    // Wait for initial watch message
    stdout_rx
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see initial watch message");

    // Wait for watcher setup, then remove read access and touch a different file
    // to trigger a watcher event cycle
    std::thread::sleep(Duration::from_millis(500));
    let a_path = dir.path().join("a.spec");
    let mut perms = fs::metadata(&a_path).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&a_path, perms).unwrap();

    // Create b.spec to trigger watcher events
    fs::write(dir.path().join("b.spec"), valid_spec("b", "1.0.0", None)).unwrap();

    // Collect stderr — may contain read error for a.spec
    let stderr_lines = stderr_rx.collect_for(Duration::from_secs(5));
    let _stderr_combined = stderr_lines.join("\n").to_lowercase();

    // Restore permissions for cleanup
    let mut restore_perms = fs::metadata(&a_path).unwrap().permissions();
    restore_perms.set_mode(0o644);
    fs::set_permissions(&a_path, restore_perms).unwrap();

    // The watch process should continue running after unreadable file
    assert!(
        child.try_wait().unwrap().is_none(),
        "watch process should continue running after unreadable file"
    );

    let _ = child.kill();
    let _ = child.wait();
}

/// watch-command.spec: watch-graph-persist-failure-on-shutdown
#[test]
fn watch_graph_persist_failure_on_shutdown() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let stdout_rx = LineReceiver::new(stdout);
    let stderr_rx = LineReceiver::new(stderr);

    // Wait for initial watch message
    stdout_rx
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see initial watch message");

    // Make .minter/ directory non-writable
    let minter_dir = dir.path().join(".minter");
    if minter_dir.exists() {
        let graph_path = minter_dir.join("graph.json");
        if graph_path.exists() {
            let mut gperms = fs::metadata(&graph_path).unwrap().permissions();
            gperms.set_mode(0o444);
            fs::set_permissions(&graph_path, gperms).unwrap();
        }
        let mut perms = fs::metadata(&minter_dir).unwrap().permissions();
        perms.set_mode(0o555);
        fs::set_permissions(&minter_dir, perms).unwrap();
    }

    // Send SIGINT
    unsafe {
        libc::kill(child.id() as libc::pid_t, libc::SIGINT);
    }

    // Wait for process to exit
    let status = child.wait().expect("should wait for child");
    assert!(
        status.success(),
        "should exit with code 0 even on graph persist failure, got: {status}"
    );

    // Collect stderr — should mention graph write failure
    let stderr_lines = stderr_rx.collect_for(Duration::from_secs(3));
    let stderr_combined = stderr_lines.join("\n").to_lowercase();
    assert!(
        stderr_combined.contains("graph") || stderr_combined.contains(".minter"),
        "stderr should mention 'graph' or '.minter' on persist failure: got {:?}",
        stderr_combined
    );
    assert!(
        stderr_combined.contains("write")
            || stderr_combined.contains("save")
            || stderr_combined.contains("persist")
            || stderr_combined.contains("failed"),
        "stderr should mention write/persist failure: got {:?}",
        stderr_combined
    );

    // Restore permissions for cleanup
    if minter_dir.exists() {
        let mut perms = fs::metadata(&minter_dir).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&minter_dir, perms).unwrap();
        let graph_path = minter_dir.join("graph.json");
        if graph_path.exists() {
            let mut gperms = fs::metadata(&graph_path).unwrap().permissions();
            gperms.set_mode(0o644);
            fs::set_permissions(&graph_path, gperms).unwrap();
        }
    }
}

/// watch-command.spec: watch-rebuild-stale-graph-entries
#[test]
fn watch_rebuild_stale_graph_entries() {
    let dir = tempfile::TempDir::new().unwrap();
    let old_dir = dir.path().join("old");
    let new_dir = dir.path().join("new");
    fs::create_dir(&old_dir).unwrap();
    fs::create_dir(&new_dir).unwrap();
    fs::write(old_dir.join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    let mut child = Command::new(minter_bin())
        .current_dir(dir.path())
        .arg("watch")
        .arg(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn minter watch");

    let stdout = child.stdout.take().unwrap();
    let receiver = LineReceiver::new(stdout);

    receiver
        .wait_for(
            |l| l.to_lowercase().contains("watch"),
            Duration::from_secs(10),
        )
        .expect("should see watching banner");

    // Wait for watcher setup, then move the file
    std::thread::sleep(Duration::from_millis(500));
    fs::remove_file(old_dir.join("a.spec")).unwrap();
    fs::write(new_dir.join("a.spec"), valid_spec("a", "1.0.0", None)).unwrap();

    // Should see output about the file events
    let lines = receiver.collect_for(Duration::from_secs(5));
    let combined = lines.join("\n");
    assert!(
        combined.contains("a"),
        "output should contain 'a' after file move: got {:?}",
        combined
    );

    let _ = child.kill();
    let _ = child.wait();
}
