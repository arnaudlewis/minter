spec watch-mode v1.0.0
title "Watch Mode"

description
  A long-running mode that watches a specs directory for file changes and
  triggers incremental re-validation automatically. Keeps the dependency
  graph hot in memory so authors get instant feedback while editing specs.

motivation
  During spec authoring, running validate manually after every edit is
  tedious. Watch mode provides a continuous feedback loop: save a file,
  see validation results immediately. By keeping the graph in memory and
  using incremental validation, re-validation is near-instant even for
  large spec trees.

# Startup

behavior watch-start [happy_path]
  "Start watching the specs directory and validate on file changes"

  given
    A directory with valid .spec files

  when specval watch specs/

  then emits stdout
    assert output indicates watch mode is active
    assert output contains the directory being watched

  then
    assert the process remains running and watching for changes


# Re-validation on change

behavior watch-incremental-revalidation [happy_path]
  "Re-validate only the changed file and its dependents on file save"

  given
    Watch mode is running with a valid graph in memory
    specs/b.spec is modified and saved

  when file change detected

  then emits stdout
    assert output shows which file changed
    assert output shows the re-validated dependency tree
    assert only b and its dependents appear in the output


# Re-validation after fix

behavior watch-revalidate-after-fix [edge_case]
  "Show validation result when a previously broken spec is fixed"

  given
    Watch mode is running with a valid graph in memory
    specs/a.spec was valid on startup
    specs/a.spec is then modified to introduce a parse error

  when specs/a.spec is fixed and saved again

  then emits stdout
    assert output shows the changed file event
    assert output shows a success checkmark for the now-valid spec


# Shutdown

behavior watch-graceful-shutdown [happy_path]
  "Stop the watcher cleanly on interrupt signal"

  given
    Watch mode is running

  when SIGINT received

  then
    assert the graph is written to .specval/graph.json before exit

  then emits process_exit
    assert code == 0


# File events

behavior watch-detect-new-file [edge_case]
  "Integrate a newly created spec file during watch mode"

  given
    Watch mode is running with a valid graph in memory

  when a new file specs/d.spec is created

  then emits stdout
    assert output shows the new file was detected
    assert output shows d integrated into the graph

  then
    assert d is added to the in-memory graph with its edges


behavior watch-report-broken-deps-on-delete [error_case]
  "Report broken dependencies when a watched spec file is deleted"

  given
    Watch mode is running with a valid graph in memory
    specs/a.spec depends on b >= 1.0.0

  when specs/b.spec is deleted

  then emits stdout
    assert output shows which file was deleted

  then emits stderr
    assert output mentions broken dependency from a to b

  then
    assert b is removed from the in-memory graph


# Colored output

behavior watch-colored-output [happy_path]
  "Use ANSI colors to distinguish event types and validation results"

  given
    Watch mode is running

  when file changes are detected and validated

  then emits stdout
    assert success lines use green for the checkmark
    assert semantic failure lines use red for the cross mark
    assert parse error lines use red for the cross mark
    assert changed file events use yellow
    assert new file events use cyan
    assert deleted file events use red
    assert the watching banner uses cyan


# Debouncing

behavior handle-rapid-successive-changes [edge_case]
  "Debounce rapid file changes to avoid redundant validation runs"

  given
    Watch mode is running
    A spec file is saved multiple times in quick succession

  when multiple file change events fire within a short window

  then
    assert only one validation cycle runs for the batch of changes
    assert the final state of all changed files is validated

depends on incremental-validation >= 1.0.0
