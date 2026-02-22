spec graph-cache v1.0.0
title "Graph Cache"

description
  A persistent dependency graph stored in .specval/graph.json at the current
  working directory (project root), not inside the specs directory. Caches
  parse results, content hashes, and dependency edges so that subsequent
  validate --deps runs can skip re-parsing unchanged files.

motivation
  Every validate --deps run currently re-parses all sibling specs from scratch.
  As the spec count grows, this becomes slow and wasteful. A persistent graph
  with content hashing lets the tool know which files have changed without
  re-reading and re-parsing every spec on every invocation.

# Storage location

behavior specval-directory-at-cwd [happy_path]
  "Create .specval at the current working directory, not inside the specs directory"

  given
    The specs live in a subdirectory (e.g. specs/)
    The CLI is invoked from the project root

  when validate --deps specs/

  then
    assert .specval/graph.json is created at the current working directory
    assert .specval is not created inside the specs directory


# Cold start

behavior build-graph-cold-start [happy_path]
  "Build the full dependency graph when no cached graph exists"

  given
    A directory with valid .spec files and no .specval/graph.json

  when validate --deps specs/

  then
    assert .specval directory is created at the current working directory
    assert graph.json is written containing all discovered specs
    assert graph.json contains a content hash for each spec file
    assert graph.json contains forward and reverse dependency edges

  then emits process_exit
    assert code == 0


behavior empty-specval-directory [happy_path]
  "Create .specval directory at the current working directory if it does not exist"

  given
    The project has no .specval directory

  when validate --deps specs/a.spec

  then
    assert .specval directory is created at the current working directory
    assert graph.json is written inside it


# Cache loading

behavior load-cached-graph [happy_path]
  "Load the cached graph and skip re-parsing unchanged files"

  given
    A valid .specval/graph.json exists
    No spec files have changed since the graph was written

  when validate --deps specs/a.spec

  then
    assert unchanged specs are not re-parsed from disk
    assert validation results are served from the cached graph
    assert graph.json is not rewritten

  then emits process_exit
    assert code == 0


behavior write-updated-graph [happy_path]
  "Persist the updated graph after incremental validation"

  given
    A valid .specval/graph.json exists
    One or more spec files have changed

  when validate --deps specs/a.spec

  then
    assert graph.json is written with updated content hashes
    assert graph.json contains updated dependency edges
    assert the graph remains loadable for the next run


# No-deps bypass

behavior validate-without-deps-ignores-graph [edge_case]
  "The graph is not loaded or updated when --deps is not used"

  given
    .specval/graph.json exists with a cached graph

  when specval validate specs/a.spec

  then
    assert the spec is validated using parse and semantic rules only
    assert graph.json is not read or modified


# Error recovery

behavior rebuild-on-corrupted-graph [error_case]
  "Rebuild the graph from scratch when graph.json is corrupted"

  given
    .specval/graph.json exists but contains invalid JSON

  when validate --deps specs/a.spec

  then emits stderr
    assert output warns that the cached graph is corrupted
    assert output mentions rebuilding from scratch

  then
    assert the graph is rebuilt by parsing all spec files
    assert graph.json is overwritten with a valid graph

  then emits process_exit
    assert code reflects the validation result of the specs


behavior rebuild-on-schema-mismatch [error_case]
  "Rebuild when graph.json has valid JSON but an incompatible schema"

  given
    .specval/graph.json exists with valid JSON but missing required fields

  when validate --deps specs/a.spec

  then emits stderr
    assert output warns that the cached graph has an incompatible format

  then
    assert the graph is rebuilt from scratch
    assert graph.json is overwritten with the current schema

depends on validate-dependencies >= 1.0.0
