spec graph-command v1.4.0
title "Graph Command"

description
  The minter graph command displays the full dependency graph for all
  specs in a directory. A summary header shows the total spec count
  and deduplicated behavior count across the entire graph. With the
  --impacted flag, it displays the reverse dependency chain for a
  named spec — every spec that directly or transitively depends on it.
  NFR references are displayed as tagged nodes in the tree grouped by
  category, with individual anchors shown as sub-items. NFR categories
  can be used as targets for --impacted analysis.

motivation
  Understanding the dependency graph is essential for impact analysis.
  When changing a spec, authors need to know which other specs will be
  affected. The graph command makes this visible.

nfr
  operability#deterministic-output
  operability#ci-friendly-output


behavior display-summary-header [happy_path]
  "Display total specs, behaviors, and referenced NFRs/constraints as a summary header"

  given
    specs/a.spec has 2 behaviors and references performance#api-response-time
    specs/b.spec has 3 behaviors and references reliability (whole-file, 1 constraint)
    nfr/performance.nfr has 2 constraints
    nfr/reliability.nfr has 1 constraint
    nfr/security.nfr has 3 constraints but is not referenced by any spec

  when minter graph specs/

  then emits stdout
    assert first line contains "2 specs, 5 behaviors, 2 nfr categories, 2 nfrs"
    assert summary counts only NFR categories and nfrs referenced by specs
    assert unreferenced security.nfr is excluded from counts

  then emits process_exit
    assert code == 0


behavior display-full-graph [happy_path]
  "Display all specs and their dependency edges"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec depends on c >= 1.0.0
    specs/c.spec has no dependencies

  when minter graph specs/

  then emits stdout
    assert output contains "a"
    assert output contains "b"
    assert output contains "c"
    assert output contains edge from a to b
    assert output contains edge from b to c

  then emits process_exit
    assert code == 0


behavior display-impacted-specs [happy_path]
  "Display direct reverse dependencies of a named spec"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/c.spec depends on b >= 1.0.0
    specs/b.spec has no dependencies

  when minter graph --impacted b specs/

  then emits stdout
    assert output contains "a"
    assert output contains "c"

  then emits process_exit
    assert code == 0


behavior display-transitive-impacted [happy_path]
  "Display transitive reverse dependencies of a named spec"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec depends on c >= 1.0.0
    specs/c.spec has no dependencies

  when minter graph --impacted c specs/

  then emits stdout
    assert output contains "b"
    assert output contains "a"

  then emits process_exit
    assert code == 0


behavior impacted-unknown-spec [error_case]
  "Print error when the named spec is not found in the graph"

  given
    specs/a.spec exists
    No spec named nonexistent exists in the spec tree

  when minter graph --impacted nonexistent specs/

  then emits stderr
    assert output contains "nonexistent"
    assert output contains "not found"

  then emits process_exit
    assert code == 1


behavior graph-no-specs [error_case]
  "Print error when no spec files are found in the directory"

  given
    An empty directory with no .spec files

  when minter graph empty-dir/

  then emits stderr
    assert output contains "no spec files found"

  then emits process_exit
    assert code == 1


behavior graph-persists-cache [happy_path]
  "Persist the graph cache after building the dependency graph"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec has no dependencies
    No prior .minter/graph.json exists

  when minter graph specs/

  then emits file .minter/graph.json
    assert file contains "a"
    assert file contains "b"

  then emits process_exit
    assert code == 0


behavior graph-no-dependencies [edge_case]
  "List all specs with no edges when none have dependencies"

  given
    specs/a.spec has no dependencies
    specs/b.spec has no dependencies

  when minter graph specs/

  then emits stdout
    assert output contains "a"
    assert output contains "b"
    assert output does not contain edge indicators

  then emits process_exit
    assert code == 0


# Graph display — NFR references

behavior display-nfr-refs-in-tree [happy_path]
  "NFR categories are parent nodes tagged [nfr] with anchors listed as sub-items"

  given
    specs/a.spec references performance#api-response-time, performance#cache-hit-ratio, and reliability
    nfr/performance.nfr exists with version 1.1.0
    nfr/reliability.nfr exists with version 1.0.0

  when minter graph specs/

  then emits stdout
    assert output contains "[nfr] performance v1.1.0"
    assert output contains "#api-response-time" as sub-item under performance
    assert output contains "#cache-hit-ratio" as sub-item under performance
    assert "[nfr] reliability" appears as a leaf with no anchor sub-items
    assert NFR categories are sorted alphabetically
    assert anchor sub-items render as plain text, not dimmed
    assert only the "#" character on each anchor is colored magenta, the anchor name is plain text


behavior display-nfr-refs-hidden-when-spec-dimmed [happy_path]
  "When a spec is dimmed (repeated), its NFR refs are omitted entirely"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/c.spec depends on b >= 1.0.0
    specs/b.spec references performance NFR
    nfr/performance.nfr exists

  when minter graph specs/

  then emits stdout
    assert the second occurrence of b is dimmed
    assert its NFR refs do not appear under the dimmed occurrence


behavior display-nfr-anchors-show-referenced-count [happy_path]
  "When a spec references specific anchors, the constraint count reflects only the referenced anchors"

  given
    specs/a.spec references performance#api-response-time and performance#cache-hit-ratio at spec level
    nfr/performance.nfr exists with version 1.0.0 and 4 constraints

  when minter graph specs/

  then emits stdout
    assert output contains "[nfr] performance v1.0.0 (2 constraints)"
    assert output contains "#api-response-time" as sub-item
    assert output contains "#cache-hit-ratio" as sub-item
    assert output does not contain "(4 constraints)" for performance


behavior display-nfr-whole-file-absorbs-anchors [happy_path]
  "When a spec has both a whole-file ref and behavior-level anchors for the same NFR category, display as whole-file leaf with full count"

  given
    specs/a.spec references performance (whole-file) at spec level
    specs/a.spec behavior references performance#api-response-time at behavior level
    nfr/performance.nfr exists with version 1.0.0 and 3 constraints

  when minter graph specs/

  then emits stdout
    assert output contains "[nfr] performance v1.0.0 (3 constraints)"
    assert output does not contain "#api-response-time" as sub-item
    assert performance node is a leaf (no children)


behavior impacted-by-nfr [happy_path]
  "minter graph --impacted <nfr-category> lists all specs referencing that NFR category"

  given
    specs/a.spec references performance NFR
    specs/b.spec references performance NFR
    specs/c.spec does not reference performance
    nfr/performance.nfr exists

  when minter graph --impacted performance specs/

  then emits stdout
    assert output contains "a"
    assert output contains "b"
    assert output does not contain "c"

  then emits process_exit
    assert code == 0


behavior single-file-resolves-nfr-from-sibling-dirs [edge_case]
  "When given a single spec file, NFR files in sibling directories are discovered and resolved"

  given
    features/spec-a.spec references performance#api-response-time
    nfr/performance.nfr exists with version 1.0.0 and 1 constraint
    The spec and nfr live in sibling subdirectories under a common root

  when minter graph features/spec-a.spec

  then emits stdout
    assert summary contains "1 nfr"
    assert output contains "[nfr] performance v1.0.0"
    assert output contains "#api-response-time"

  then emits process_exit
    assert code == 0


depends on dependency-resolution >= 2.1.0
