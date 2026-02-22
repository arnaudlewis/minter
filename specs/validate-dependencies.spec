spec validate-dependencies v1.0.0
title "Validate Dependencies"

description
  The --deps flag extends specval validate. It first validates the spec
  itself (parse + semantic rules), then resolves and validates its
  dependencies. It checks that every depends on reference points to a
  sibling .spec file in the same directory, that versions are compatible,
  that each dependency is itself valid, and that the graph has no cycles.

motivation
  A single spec can be valid in isolation but broken in context — it
  depends on a spec that does not exist, or two specs depend on
  incompatible versions of a third. This command catches those problems
  before downstream agents attempt to use the spec graph.


behavior validate-spec-then-resolve-deps [happy_path]
  "Validate the spec itself then resolve all its dependencies, exit 0"

  given
    A valid .spec file with depends on declarations
    All referenced specs exist as sibling .spec files with compatible versions

  when validate --deps

  then emits stdout
    assert output contains spec validation result
    assert output contains count of dependencies resolved
    assert output contains "all dependencies resolved"

  then emits process_exit
    assert code == 0


behavior resolve-by-sibling-name [happy_path]
  "Find a dependency by looking for spec-name.spec in the same directory"

  given
    A file specs/create-note.spec that depends on user-auth >= 1.0.0
    A file specs/user-auth.spec exists with version 1.2.0

  when validate --deps specs/create-note.spec

  then emits process_exit
    assert code == 0


behavior resolve-transitive-dependencies [happy_path]
  "Validate the full dependency graph, not just direct dependencies"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec depends on c >= 1.0.0
    specs/c.spec exists with version 1.0.0 and no dependencies

  when validate --deps specs/a.spec

  then emits stdout
    assert output contains all three specs

  then emits process_exit
    assert code == 0


behavior reject-missing-dependency [error_case]
  "Print error when no sibling file matches the dependency name"

  given
    A file specs/create-note.spec that depends on user-auth >= 1.0.0
    No file specs/user-auth.spec exists

  when validate --deps specs/create-note.spec

  then emits stderr
    assert output mentions create-note
    assert output mentions user-auth
    assert output mentions the directory searched

  then emits process_exit
    assert code == 1


behavior reject-incompatible-version [error_case]
  "Print error showing the version mismatch"

  given
    A file specs/create-note.spec that depends on user-auth >= 2.0.0
    A file specs/user-auth.spec exists with version 1.5.0

  when validate --deps specs/create-note.spec

  then emits stderr
    assert output mentions user-auth
    assert output mentions the required version
    assert output mentions the found version

  then emits process_exit
    assert code == 1


behavior reject-cyclic-dependencies [error_case]
  "Print error showing the cycle path"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec depends on a >= 1.0.0

  when validate --deps specs/a.spec

  then emits stderr
    assert output mentions cycle
    assert output mentions both spec names

  then emits process_exit
    assert code == 1


behavior reject-invalid-dependency-spec [error_case]
  "Print error when a dependency spec exists but fails validation"

  given
    specs/create-note.spec depends on broken >= 1.0.0
    specs/broken.spec exists but has parse or semantic errors

  when validate --deps specs/create-note.spec

  then emits stderr
    assert output mentions broken
    assert output mentions validation failure

  then emits process_exit
    assert code == 1


behavior skip-deps-when-spec-invalid [edge_case]
  "Skip dependency resolution when the spec itself fails validation"

  given
    A .spec file that fails parse or semantic validation
    The spec has depends on declarations

  when validate --deps

  then emits stderr
    assert output contains spec validation errors
    assert output does not contain dependency resolution results

  then emits process_exit
    assert code == 1


behavior handle-no-dependencies [edge_case]
  "Exit 0 when the spec has no depends on declarations"

  given
    A .spec file with no depends on lines

  when validate --deps

  then emits stdout
    assert output contains "no dependencies"

  then emits process_exit
    assert code == 0


behavior report-all-resolution-errors [edge_case]
  "Report all unresolved dependencies, not just the first"

  given
    A .spec file with multiple depends on lines where several are unresolved

  when validate --deps

  then emits stderr
    assert output contains all unresolved dependency names

  then emits process_exit
    assert code == 1


depends on validate-command >= 1.0.0
