spec watch-command v2.1.0
title "Watch Command"

description
  The minter watch command starts a long-running process that watches
  a file or directory for changes and triggers incremental re-validation
  automatically. Keeps the dependency graph hot in memory so authors
  get instant feedback while editing specs. Handles file additions,
  deletions, modifications, and moves by updating the in-memory graph
  and re-validating only affected specs and their dependents.

motivation
  During spec authoring, running validate manually after every edit is
  tedious. Watch mode provides a continuous feedback loop: save a file,
  see validation results immediately. By keeping the graph in memory and
  using incremental validation, re-validation is near-instant even for
  large spec trees.

nfr
  performance#watch-revalidation-latency
  reliability#no-silent-data-loss
  reliability#crash-safe-persistence
  operability#ci-friendly-output


# Startup

behavior watch-start-directory [happy_path]
  "Start watching a directory, display banner, and run initial validation"

  given
    A directory with valid .spec files including specs/a.spec

  when minter watch specs/

  then emits stdout
    assert output contains "watching"
    assert output contains "specs/"
    assert output contains "a"

  then
    assert the process does not emit an exit code


behavior watch-start-single-file [happy_path]
  "Start watching a single file and all specs that depend on it"

  given
    specs/b.spec exists and is valid
    specs/a.spec depends on b >= 1.0.0

  when minter watch specs/b.spec

  then emits stdout
    assert output contains "watching"
    assert output contains "b.spec"
    assert output contains "b"

  then
    assert the process does not emit an exit code


behavior watch-initial-validation [happy_path]
  "Run full validation on startup before waiting for changes"

  given
    specs/a.spec is valid
    specs/b.spec has a validation error

  when minter watch specs/

  then emits stdout
    assert output contains "a"
    assert output contains "b"

  then emits stderr
    assert output contains validation error for b


# Re-validation on changes

behavior watch-revalidate-changed-file [happy_path]
  "Re-validate changed file and its dependents on save"

  given
    Watch mode is running with a valid graph in memory
    specs/b.spec is modified and saved
    specs/a.spec depends on b >= 1.0.0

  when file change detected for specs/b.spec

  then emits stdout
    assert output contains "b.spec"
    assert output contains "a"
    assert output contains "b"


behavior watch-revalidate-subdirectory [happy_path]
  "Detect and re-validate file changes in subdirectories"

  given
    Watch mode is running on specs/
    specs/validation/a.spec exists and is valid

  when specs/validation/a.spec is modified

  then emits stdout
    assert output contains "validation/a.spec"
    assert output contains "a"


behavior watch-file-regresses-valid-to-invalid [happy_path]
  "Show failure result when a previously valid spec becomes invalid"

  given
    Watch mode is running
    specs/a.spec was valid on startup

  when specs/a.spec is modified to introduce a parse error

  then emits stdout
    assert output contains "a"

  then emits stderr
    assert output contains parse error message with line number


behavior watch-file-fixed-invalid-to-valid [happy_path]
  "Show success result when a previously broken spec is fixed"

  given
    Watch mode is running
    specs/a.spec had a validation error

  when specs/a.spec is fixed and saved

  then emits stdout
    assert output contains "a"

  then emits stderr
    assert output is empty for a


behavior watch-dependency-added [happy_path]
  "Re-validate when a dependency is added to an existing spec"

  given
    Watch mode is running
    specs/a.spec had no dependencies
    specs/b.spec exists and is valid

  when specs/a.spec is modified to add: depends on b >= 1.0.0

  then emits stdout
    assert output contains "a"
    assert output contains "b"


behavior watch-dependency-removed [happy_path]
  "Re-validate when a dependency is removed from an existing spec"

  given
    Watch mode is running
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec exists and is valid

  when specs/a.spec is modified to remove the depends on b line

  then emits stdout
    assert output contains "a"
    assert output does not contain "b" in the dependency tree


# File events

behavior watch-integrate-new-file [happy_path]
  "Integrate a newly created spec file into the graph and validate it"

  given
    Watch mode is running with a valid graph in memory

  when a new file specs/d.spec is created

  then emits stdout
    assert output contains "d.spec"
    assert output contains validation result for d


behavior watch-report-broken-deps-on-delete [error_case]
  "Report broken dependencies when a watched spec file is deleted"

  given
    Watch mode is running
    specs/a.spec depends on b >= 1.0.0

  when specs/b.spec is deleted

  then emits stdout
    assert output contains "b.spec"

  then emits stderr
    assert output contains "a"
    assert output contains "b"
    assert output contains "missing" or "not found"


behavior watch-rebuild-stale-graph-entries [edge_case]
  "Rebuild graph entries when files are moved to a different directory"

  given
    Watch mode is running
    specs/old/a.spec is moved to specs/new/a.spec

  when file events detected for the move

  then emits stdout
    assert output contains "new/a.spec"
    assert output contains validation result for a


# Multiple simultaneous changes

behavior watch-multiple-files-changed [edge_case]
  "Handle multiple files changing simultaneously"

  given
    Watch mode is running
    specs/a.spec and specs/b.spec are both modified within a short window

  when file change events fire for both files

  then emits stdout
    assert output contains "a"
    assert output contains "b"


# Debouncing

behavior watch-debounce-rapid-changes [edge_case]
  "Debounce rapid file changes to avoid redundant validation"

  given
    Watch mode is running
    specs/a.spec is saved three times in rapid succession

  when multiple file change events fire within a short window

  then emits stdout
    assert exactly one validation result appears for a per batch


# Shutdown

behavior watch-graceful-shutdown [happy_path]
  "Stop the watcher and persist the graph on interrupt signal"

  given
    Watch mode is running

  when SIGINT received

  then
    assert .minter/graph.json is written before exit

  then emits process_exit
    assert code == 0


# Error cases

behavior watch-nonexistent-path [error_case]
  "Print error when the watched path does not exist"

  given
    The specified path does not exist on disk

  when minter watch nonexistent/

  then emits stderr
    assert output contains "nonexistent"

  then emits process_exit
    assert code == 1


behavior watch-permission-denied-on-directory [error_case]
  "Print error when the watched directory is not readable"

  given
    The specified directory exists but the user lacks read permission

  when minter watch restricted/

  then emits stderr
    assert output contains "permission"

  then emits process_exit
    assert code == 1


behavior watch-empty-directory [error_case]
  "Print error when the watched directory contains no spec files"

  given
    The specified directory exists but contains no .spec files

  when minter watch empty-dir/

  then emits stderr
    assert output contains "no spec files"

  then emits process_exit
    assert code == 1


behavior watch-file-becomes-unreadable [error_case]
  "Report error for a file that becomes unreadable mid-watch"

  given
    Watch mode is running
    specs/a.spec was readable on startup

  when specs/a.spec permissions change to remove read access

  then emits stderr
    assert output contains "a.spec"
    assert output contains "permission" or "read"

  then
    assert the watch process continues running


behavior watch-graph-persist-failure-on-shutdown [error_case]
  "Warn when graph cannot be persisted on shutdown"

  given
    Watch mode is running
    .minter/ directory is not writable

  when SIGINT received

  then emits stderr
    assert output contains "graph" or ".minter"
    assert output contains "write" or "persist"

  then emits process_exit
    assert code == 0


# NFR watch support

behavior watch-discover-nfr-files [happy_path]
  "Include .nfr files in the initial directory scan and validation"

  given
    A directory containing specs/a.spec and specs/performance.nfr
    Both files are valid

  when minter watch specs/

  then emits stdout
    assert output contains "watching"
    assert output contains "a"
    assert output contains "performance"


behavior watch-revalidate-changed-nfr [happy_path]
  "Re-validate when a .nfr file is modified and saved"

  given
    Watch mode is running with specs/performance.nfr in the watch set
    specs/performance.nfr is modified and saved

  when file change detected for specs/performance.nfr

  then emits stdout
    assert output contains "performance.nfr"
    assert output contains validation result for performance


behavior watch-integrate-new-nfr-file [happy_path]
  "Add a newly created .nfr file to the watch set and validate it"

  given
    Watch mode is running with a valid graph in memory

  when a new file specs/security.nfr is created

  then emits stdout
    assert output contains "security.nfr"
    assert output contains validation result for security


depends on dependency-resolution >= 2.0.0
depends on cli-display >= 2.0.0
