spec dependency-resolution v2.0.0
title "Dependency Resolution"

description
  Resolves and validates the full dependency graph for specs. Handles
  name-based resolution across the spec tree, version constraint
  checking, cycle detection, transitive traversal, and duplicate name
  rejection. Includes a persistent graph cache at .minter/graph.json
  that stores content hashes, file paths, and dependency edges to
  avoid re-parsing unchanged files on subsequent runs.

motivation
  A single spec can be valid in isolation but broken in context — it
  depends on a spec that does not exist, or two specs depend on
  incompatible versions of a third. Full graph resolution catches
  these problems. The cache makes repeated resolution fast as the
  spec tree grows.


# Resolution — happy paths

behavior resolve-direct-dependencies [happy_path]
  "Validate the spec then resolve all its direct dependencies, exit 0"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec exists with version 1.2.0 and is valid

  when minter validate --deep specs/a.spec

  then emits stdout
    assert output contains result for a
    assert output contains result for b

  then emits process_exit
    assert code == 0


behavior resolve-by-name-in-tree [happy_path]
  "Find a dependency by scanning the spec tree for spec-name.spec"

  given
    specs/validation/create-note.spec depends on user-auth >= 1.0.0
    specs/auth/user-auth.spec exists with version 1.2.0

  when minter validate --deep specs/validation/create-note.spec

  then emits process_exit
    assert code == 0


behavior resolve-transitive-dependencies [happy_path]
  "Validate the full dependency graph including transitive deps"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec depends on c >= 1.0.0
    specs/c.spec exists with version 1.0.0 and no dependencies

  when minter validate --deep specs/a.spec

  then emits stdout
    assert output contains result for a
    assert output contains result for b
    assert output contains result for c

  then emits process_exit
    assert code == 0


behavior resolve-cross-directory-dependency [happy_path]
  "Resolve a dependency located in a different subdirectory"

  given
    specs/validation/a.spec depends on b >= 1.0.0
    specs/caching/b.spec exists with version 1.0.0

  when minter validate --deep specs/validation/a.spec

  then emits process_exit
    assert code == 0


behavior folder-automatically-resolves-dependencies [happy_path]
  "Folder validation resolves the full graph without needing --deep"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec exists with version 1.0.0
    Both specs are valid

  when minter validate specs/

  then emits stdout
    assert output contains dependency tree for a including b

  then emits process_exit
    assert code == 0


# Version constraint semantics

behavior version-constraint-satisfied [happy_path]
  "The >= constraint is satisfied by equal or higher semver versions"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec exists with version 2.3.0

  when minter validate --deep specs/a.spec

  then emits process_exit
    assert code == 0


behavior version-constraint-exact-match [happy_path]
  "The >= constraint is satisfied by the exact version"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec exists with version 1.0.0

  when minter validate --deep specs/a.spec

  then emits process_exit
    assert code == 0


behavior version-constraint-patch-higher [happy_path]
  "The >= constraint is satisfied by a higher patch version"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec exists with version 1.0.1

  when minter validate --deep specs/a.spec

  then emits process_exit
    assert code == 0


# Resolution — error cases

behavior reject-missing-dependency [error_case]
  "Print error when no file in the spec tree matches the dependency name"

  given
    specs/create-note.spec depends on user-auth >= 1.0.0
    No file named user-auth.spec exists in the spec tree

  when minter validate --deep specs/create-note.spec

  then emits stderr
    assert output contains "create-note"
    assert output contains "user-auth"
    assert output contains "not found"

  then emits process_exit
    assert code == 1


behavior reject-incompatible-version [error_case]
  "Print error showing the version mismatch"

  given
    specs/create-note.spec depends on user-auth >= 2.0.0
    specs/user-auth.spec exists with version 1.5.0

  when minter validate --deep specs/create-note.spec

  then emits stderr
    assert output contains "user-auth"
    assert output contains ">= 2.0.0"
    assert output contains "1.5.0"

  then emits process_exit
    assert code == 1


behavior reject-version-below-constraint [error_case]
  "Reject when the found version is below the constraint"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec exists with version 0.9.0

  when minter validate --deep specs/a.spec

  then emits stderr
    assert output contains "b"
    assert output contains ">= 1.0.0"
    assert output contains "0.9.0"

  then emits process_exit
    assert code == 1


behavior reject-cyclic-dependencies [error_case]
  "Print error showing the cycle path for direct cycles"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec depends on a >= 1.0.0

  when minter validate --deep specs/a.spec

  then emits stderr
    assert output contains "cycle"
    assert output contains "a"
    assert output contains "b"

  then emits process_exit
    assert code == 1


behavior reject-transitive-cycle [error_case]
  "Print error showing the cycle path for transitive cycles"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec depends on c >= 1.0.0
    specs/c.spec depends on a >= 1.0.0

  when minter validate --deep specs/a.spec

  then emits stderr
    assert output contains "cycle"
    assert output contains "a"
    assert output contains "b"
    assert output contains "c"

  then emits process_exit
    assert code == 1


behavior reject-invalid-dependency-spec [error_case]
  "Print error when a dependency spec exists but fails validation"

  given
    specs/create-note.spec depends on broken >= 1.0.0
    specs/broken.spec exists but has parse or semantic errors

  when minter validate --deep specs/create-note.spec

  then emits stderr
    assert output contains "broken"

  then emits process_exit
    assert code == 1


behavior reject-duplicate-spec-names [error_case]
  "Error when two spec files in different subdirectories share the same name"

  given
    specs/sub1/my-feature.spec exists
    specs/sub2/my-feature.spec exists with the same stem name

  when minter validate --deep specs/

  then emits stderr
    assert output contains "my-feature"
    assert output contains "sub1"
    assert output contains "sub2"

  then emits process_exit
    assert code == 1


# Resolution — edge cases

behavior skip-deps-when-spec-invalid [error_case]
  "Skip dependency resolution when the spec itself fails validation"

  given
    A .spec file that fails parse or semantic validation
    The spec has depends on declarations

  when minter validate --deep

  then emits stderr
    assert output contains spec validation errors
    assert output does not contain dependency resolution results

  then emits process_exit
    assert code == 1


behavior handle-no-dependencies [edge_case]
  "Exit 0 when the spec has no depends on declarations"

  given
    A .spec file with no depends on lines

  when minter validate --deep

  then emits process_exit
    assert code == 0


behavior report-all-resolution-errors [error_case]
  "Report all unresolved dependencies, not just the first"

  given
    specs/a.spec depends on missing-one >= 1.0.0 and missing-two >= 1.0.0
    Neither missing-one nor missing-two exist in the spec tree

  when minter validate --deep specs/a.spec

  then emits stderr
    assert output contains "missing-one"
    assert output contains "missing-two"

  then emits process_exit
    assert code == 1


# Graph cache — storage

behavior cache-directory-location [happy_path]
  "Create .minter/graph.json at the current working directory"

  given
    The specs live in a subdirectory (e.g. specs/)
    The CLI is invoked from the project root
    No .minter directory exists

  when minter validate --deep specs/a.spec

  then
    assert .minter/graph.json is created at the current working directory
    assert .minter is not created inside the specs directory


behavior cache-cold-start-creates-directory [happy_path]
  "Create .minter directory when it does not exist"

  given
    The project has no .minter directory
    specs/a.spec is a valid spec

  when minter validate --deep specs/a.spec

  then
    assert .minter directory is created at the current working directory
    assert graph.json is written inside it


# Graph cache — loading and updating

behavior cache-produces-correct-results [happy_path]
  "Produce correct validation results when cache is present and files unchanged"

  given
    A valid .minter/graph.json exists from a previous --deep run
    No spec files have changed since the graph was written
    specs/a.spec depends on b >= 1.0.0

  when minter validate --deep specs/a.spec

  then emits stdout
    assert output contains "a"
    assert output contains "b"

  then emits process_exit
    assert code == 0


behavior cache-revalidates-modified-and-dependents [happy_path]
  "Re-validate a modified spec and its dependents on subsequent run"

  given
    A valid .minter/graph.json exists from a previous --deep run
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec has been modified since the last run

  when minter validate --deep specs/

  then emits stdout
    assert output contains result for b
    assert output contains result for a

  then emits process_exit
    assert code == 0


behavior cache-integrates-new-files [happy_path]
  "Include a newly added spec in results on subsequent run"

  given
    A valid .minter/graph.json exists from a previous --deep run
    A new file specs/d.spec is added to the directory

  when minter validate --deep specs/

  then emits stdout
    assert output contains result for d

  then emits process_exit
    assert code == 0


behavior validate-without-deep-ignores-graph [edge_case]
  "Single file validation produces correct results without touching graph"

  given
    .minter/graph.json exists with a cached graph
    specs/a.spec is a valid spec

  when minter validate specs/a.spec

  then emits stdout
    assert output contains "a"

  then emits process_exit
    assert code == 0


# Graph cache — error recovery

behavior rebuild-on-corrupted-graph [edge_case]
  "Rebuild the graph from scratch when graph.json contains invalid JSON"

  given
    .minter/graph.json exists but contains invalid JSON
    specs/a.spec is a valid spec with no dependencies

  when minter validate --deep specs/a.spec

  then emits stderr
    assert output contains "corrupted"
    assert output contains "rebuilding"

  then emits stdout
    assert output contains "a"

  then emits process_exit
    assert code == 0


behavior rebuild-on-schema-mismatch [edge_case]
  "Rebuild when graph.json has valid JSON but an incompatible schema"

  given
    .minter/graph.json exists with valid JSON but missing required fields
    specs/a.spec is a valid spec with no dependencies

  when minter validate --deep specs/a.spec

  then emits stderr
    assert output contains "incompatible"

  then emits stdout
    assert output contains "a"

  then emits process_exit
    assert code == 0


depends on validate-command >= 2.0.0
