spec incremental-validation v1.0.0
title "Incremental Validation"

description
  Uses the cached dependency graph to detect which spec files have changed
  and re-validates only those files and their dependents. Handles file
  additions, deletions, and moves gracefully by updating the graph.

motivation
  With a cached graph in place, the tool can compare content hashes to find
  changed files and walk reverse dependency edges to identify affected specs.
  This avoids re-parsing and re-validating the entire spec tree on every run,
  keeping validation fast as the number of specs grows.

# Change detection

behavior detect-changed-file [happy_path]
  "Re-validate only the changed file and its dependents"

  given
    A valid .specval/graph.json exists
    specs/b.spec has been modified since the graph was written
    specs/a.spec depends on b >= 1.0.0 and has not changed

  when validate --deps specs/a.spec

  then
    assert specs/b.spec is re-parsed and re-validated
    assert specs/a.spec is re-validated because it depends on b
    assert other unchanged specs with no dependency on b are skipped
    assert graph.json is updated with the new hash for b

  then emits process_exit
    assert code == 0


# File additions

behavior integrate-new-spec-file [edge_case]
  "Add a newly created spec file to the existing graph"

  given
    .specval/graph.json exists with a cached graph
    A new file specs/d.spec is created on disk

  when validate --deps specs/a.spec
    where specs/a.spec now depends on d >= 1.0.0

  then
    assert d is parsed, validated, and added to the graph
    assert graph.json is updated with d and its edges

  then emits process_exit
    assert code == 0


# File deletions

behavior reject-broken-deps-after-deletion [error_case]
  "Report broken dependencies when a spec file has been deleted"

  given
    specs/a.spec depends on b >= 1.0.0
    .specval/graph.json records b as a known spec
    specs/b.spec has been deleted from disk

  when validate --deps specs/a.spec

  then
    assert b is removed from the graph

  then emits stderr
    assert output mentions that b is missing
    assert output mentions that a depends on the missing spec

  then emits process_exit
    assert code == 1


# File moves

behavior rebuild-when-files-moved [edge_case]
  "Rebuild the graph when cached paths no longer match disk"

  given
    .specval/graph.json references file paths that no longer exist
    The spec files have been moved to a different directory

  when validate --deps specs/a.spec

  then
    assert the stale graph entries are discarded
    assert the graph is rebuilt from the current directory

  then emits process_exit
    assert code reflects the validation result of the specs

depends on graph-cache >= 1.0.0
