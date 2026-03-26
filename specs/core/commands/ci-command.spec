spec ci-command v1.1.0
title "CI Command"

description
  The minter ci command is the single CI gate for spec-driven projects.
  It reads minter.config.json (or conventions), loads minter.lock, then
  recomputes hashes and mappings from the current state of specs, NFRs,
  and test files. It compares the recomputed state against the lock and
  reports any divergence. Zero flags required — everything is derived
  from the config and lock. The command performs six checks: spec
  integrity (hash match), NFR integrity (hash match), dependency
  structure (edges match), test integrity (hash match), coverage
  (every behavior has at least one test), and orphan detection (every
  tag references a real spec and behavior). Exit 0 means all checks
  pass. Exit 1 means at least one check failed.

motivation
  Agents must not silently modify specs or tests during implementation.
  The CI command enforces this by comparing the current project state
  against the human-approved lock file. It replaces ad-hoc CI scripts
  with a single deterministic command that validates integrity,
  traceability, and coverage in one pass.

nfr
  operability#ci-friendly-output
  operability#deterministic-output
  operability#zero-config
  reliability#error-completeness

# Full pass — happy path

behavior ci-all-checks-pass [happy_path]
  "Exit 0 when all six checks pass with summary stats"

  given
    minter.lock exists and was generated from the current state
    All specs, NFRs, and test files are unchanged since lock
    Every behavior has at least one test with a valid @minter tag

  when minter ci

  then emits stdout
    assert output contains "spec integrity" with spec and nfr counts
    assert output contains "nfr integrity" with nfr count
    assert output contains "dependency structure" with edge count
    assert output contains "test integrity" with test file count
    assert output contains "coverage" with covered/total and percentage
    assert output contains "orphan" with orphan count
    assert output indicates all checks passed

  then emits process_exit
    assert code == 0


behavior ci-summary-shows-stats [happy_path]
  "Pass lines show correct counts matching the project state"

  given
    A project with known quantities of specs, behaviors, nfrs, deps, and test files
    minter.lock was generated from these files
    All files are unchanged

  when minter ci

  then emits stdout
    assert spec integrity line contains correct spec and nfr counts
    assert nfr integrity line contains correct nfr count
    assert dependency structure line contains correct edge count
    assert test integrity line contains correct test file count
    assert coverage line contains correct covered/total and percentage
    assert orphan line contains 0 orphaned tags

  then emits process_exit
    assert code == 0


behavior ci-reads-config [happy_path]
  "CI reads paths from minter.config.json without any CLI flags"

  given
    minter.config.json contains: { "specs": "specifications/", "tests": ["src/tests/"] }
    minter.lock was generated from these paths
    All files are unchanged

  when minter ci

  then emits process_exit
    assert code == 0


behavior ci-uses-default-conventions [happy_path]
  "CI uses specs/ and tests/ when no config exists"

  given
    No minter.config.json exists
    minter.lock was generated using default conventions
    All files are unchanged

  when minter ci

  then emits process_exit
    assert code == 0


# Spec integrity

behavior detect-spec-hash-mismatch [error_case]
  "Fail when a spec file hash differs from the lock"

  given
    minter.lock contains specs/a.spec with hash sha256:abc
    specs/a.spec has been modified and its current hash is sha256:def

  when minter ci

  then emits stderr
    assert output contains "specs/a.spec"
    assert output contains "hash mismatch" or "modified since last lock"

  then emits process_exit
    assert code == 1


behavior detect-new-spec-not-in-lock [error_case]
  "Fail when a spec file exists on disk but not in the lock"

  given
    minter.lock does not contain an entry for specs/new.spec
    specs/new.spec exists on disk

  when minter ci

  then emits stderr
    assert output contains "specs/new.spec"
    assert output contains "not in lock" or "unlocked"

  then emits process_exit
    assert code == 1


behavior detect-deleted-spec-in-lock [error_case]
  "Fail when a spec in the lock no longer exists on disk"

  given
    minter.lock contains specs/removed.spec
    specs/removed.spec does not exist on disk

  when minter ci

  then emits stderr
    assert output contains "specs/removed.spec"
    assert output contains "missing" or "deleted"

  then emits process_exit
    assert code == 1


# NFR integrity

behavior detect-nfr-hash-mismatch [error_case]
  "Fail when an NFR file hash differs from the lock"

  given
    minter.lock contains specs/nfr/performance.nfr with hash sha256:abc
    specs/nfr/performance.nfr has been modified

  when minter ci

  then emits stderr
    assert output contains "performance.nfr"
    assert output contains "hash mismatch" or "modified since last lock"

  then emits process_exit
    assert code == 1


behavior detect-new-nfr-not-in-lock [error_case]
  "Fail when an NFR file exists on disk but not in the lock"

  given
    minter.lock does not contain specs/nfr/security.nfr
    specs/nfr/security.nfr exists on disk

  when minter ci

  then emits stderr
    assert output contains "security.nfr"
    assert output contains "not in lock" or "unlocked"

  then emits process_exit
    assert code == 1


# Dependency structure

behavior detect-dependency-change [error_case]
  "Fail when dependency edges differ from the lock"

  given
    minter.lock records specs/a.spec depends on specs/b.spec
    specs/a.spec now depends on specs/c.spec instead

  when minter ci

  then emits stderr
    assert output contains "specs/a.spec"
    assert output contains "dependency" or "structure"

  then emits process_exit
    assert code == 1


# Test integrity

behavior detect-test-hash-mismatch [error_case]
  "Fail when a test file hash differs from the lock"

  given
    minter.lock contains tests/a_test.rs with hash sha256:abc
    tests/a_test.rs has been modified

  when minter ci

  then emits stderr
    assert output contains "tests/a_test.rs"
    assert output contains "hash mismatch" or "modified since last lock"

  then emits process_exit
    assert code == 1


behavior detect-new-test-not-in-lock [error_case]
  "Fail when a test file with @minter tags exists but is not in the lock"

  given
    minter.lock does not contain tests/new_test.rs
    tests/new_test.rs contains // @minter:unit do-thing

  when minter ci

  then emits stderr
    assert output contains "tests/new_test.rs"
    assert output contains "not in lock" or "unlocked"

  then emits process_exit
    assert code == 1


behavior detect-deleted-test-in-lock [error_case]
  "Fail when a test file in the lock no longer exists on disk"

  given
    minter.lock contains tests/removed_test.rs
    tests/removed_test.rs does not exist on disk

  when minter ci

  then emits stderr
    assert output contains "tests/removed_test.rs"
    assert output contains "missing" or "deleted"

  then emits process_exit
    assert code == 1


# Coverage

behavior detect-uncovered-behavior [error_case]
  "Fail when a behavior has no test coverage"

  given
    minter.lock contains specs/a.spec with behaviors do-thing and do-other
    tests/a_test.rs covers only do-thing
    No test covers do-other

  when minter ci

  then emits stderr
    assert output contains "do-other"
    assert output contains "uncovered" or "no test coverage"

  then emits process_exit
    assert code == 1


# Orphan detection

behavior detect-orphaned-tag [error_case]
  "Fail when a test tag references a behavior not in any spec"

  given
    minter.lock contains specs/a.spec with behavior do-thing
    tests/a_test.rs contains // @minter:unit nonexistent-behavior

  when minter ci

  then emits stderr
    assert output contains "nonexistent-behavior"
    assert output contains "orphan" or "unknown"

  then emits process_exit
    assert code == 1


# Missing lock file

behavior reject-missing-lock [error_case]
  "Fail when minter.lock does not exist"

  given
    No minter.lock file exists at the project root

  when minter ci

  then emits stderr
    assert output contains "minter.lock"
    assert output contains "not found" or "missing"
    assert output contains "minter lock"

  then emits process_exit
    assert code == 1


behavior reject-corrupted-lock [error_case]
  "Fail when minter.lock is not valid JSON"

  given
    minter.lock contains invalid JSON

  when minter ci

  then emits stderr
    assert output contains "minter.lock"
    assert output contains "invalid" or "corrupted"

  then emits process_exit
    assert code == 1


# Report format

behavior report-all-failures [edge_case]
  "Report all check failures at once, not just the first"

  given
    specs/a.spec has been modified (hash mismatch)
    tests/a_test.rs has been modified (hash mismatch)
    behavior do-other has no test coverage

  when minter ci

  then emits stderr
    assert output contains "specs/a.spec"
    assert output contains "tests/a_test.rs"
    assert output contains "do-other"

  then emits process_exit
    assert code == 1


behavior report-check-summary [edge_case]
  "Display a summary showing which checks passed/failed with stats"

  given
    Spec integrity passes
    Test integrity fails (one file modified)
    Coverage passes

  when minter ci

  then emits stdout
    assert output shows each check with pass/fail status and counts
    assert passing checks include parenthesized stats
    assert failing checks show FAIL without stats

  then emits process_exit
    assert code == 1


behavior ignore-untagged-test-files [edge_case]
  "Test files without @minter tags are not tracked in the lock and not checked"

  given
    tests/helper.rs contains no @minter tags
    tests/helper.rs is not in minter.lock
    tests/helper.rs has been modified

  when minter ci

  then
    assert tests/helper.rs is not flagged
    assert test integrity check passes for this file


behavior ci-multi-test-dirs [happy_path]
  "CI passes when lock includes files from all configured test directories"

  given
    minter.config.json contains: { "specs": "specs/", "tests": ["tests/", "benches/"] }
    minter.lock was generated from these paths including benches/ benchmark files
    All files are unchanged

  when minter ci

  then emits stdout
    assert output contains "pass test integrity"

  then emits process_exit
    assert code == 0


depends on config >= 1.0.0
depends on lock-command >= 1.0.0
