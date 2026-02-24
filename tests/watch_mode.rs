mod common;

use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

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

/// watch-mode.spec: watch-start
/// stdout shows "watching" + dir path, process stays alive.
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

/// watch-mode.spec: watch-incremental-revalidation
/// Modify file → stdout shows changed file + tree for affected deps only.
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

/// watch-mode.spec: watch-graceful-shutdown
/// SIGINT → graph.json written, exit 0.
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

/// watch-mode.spec: watch-detect-new-file
/// Create new .spec → stdout shows detection + integration.
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

/// watch-mode.spec: watch-report-broken-deps-on-delete
/// Delete dep → stdout shows deletion, stderr shows broken dep.
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

/// watch-mode.spec: handle-rapid-successive-changes
/// Rapid saves → single validation cycle.
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

/// watch-mode.spec: watch-revalidate-after-fix
/// A spec that was valid, then broken, then fixed should show the success checkmark.
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

/// watch-mode.spec: watch-colored-output
/// Event types and validation results use ANSI colors.
#[test]
fn watch_colored_output() {
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

    // Wait for "watching" banner — should use cyan
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

    // Wait for watcher setup, then modify a.spec to trigger re-validation
    std::thread::sleep(Duration::from_millis(500));
    fs::write(dir.path().join("a.spec"), valid_spec("a", "1.1.0", None)).unwrap();

    // "changed:" should use yellow
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

    // "✓" line should use green
    let success_line = receiver.wait_for(
        |l| l.contains('\u{2713}'),
        Duration::from_secs(5),
    );
    assert!(success_line.is_some(), "should see success checkmark");
    let success_line = success_line.unwrap();
    assert!(
        success_line.contains(ANSI_GREEN),
        "success checkmark should use green: {success_line}"
    );

    // Create a new file — "detected new file" should use cyan
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

    // Write a broken spec to trigger a parse error — "✗" should use red
    std::thread::sleep(Duration::from_millis(500));
    fs::write(
        dir.path().join("c.spec"),
        "this is not a valid spec at all",
    )
    .unwrap();

    // Wait for changed/new event for c, then look for the red cross
    let parse_fail_line = receiver.wait_for(
        |l| l.contains('\u{2717}'),
        Duration::from_secs(10),
    );
    assert!(parse_fail_line.is_some(), "should see cross mark on parse error");
    let parse_fail_line = parse_fail_line.unwrap();
    assert!(
        parse_fail_line.contains(ANSI_RED),
        "parse error cross mark should use red: {parse_fail_line}"
    );

    // Delete a file — "deleted:" should use red
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

/// watch-mode.spec: watch-subdirectory-changes
/// Modify a file in a subdirectory → stdout shows changed + result.
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
