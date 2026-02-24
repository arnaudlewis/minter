spec graph-command v1.0.0
title "Graph Command"

description
  The minter graph command displays the full dependency graph for all
  specs in a directory. With the --impacted flag, it displays the
  reverse dependency chain for a named spec — every spec that directly
  or transitively depends on it.

motivation
  Understanding the dependency graph is essential for impact analysis.
  When changing a spec, authors need to know which other specs will be
  affected. The graph command makes this visible.


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


depends on dependency-resolution >= 2.0.0
