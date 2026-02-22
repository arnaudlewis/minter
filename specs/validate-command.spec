spec validate-command v1.0.0
title "Validate Command"

description
  The specval validate command reads one or more .spec files, parses
  the DSL, runs semantic validation on the parsed model, and reports
  pass/fail with detailed error diagnostics to stdout/stderr.

motivation
  This is the deterministic gate in the spec-driven pipeline. Every spec
  must pass through specval validate before reaching downstream agents.
  Without this gate, spec inconsistencies propagate silently.


behavior validate-valid-spec [happy_path]
  "Report success and exit 0 when a spec passes all validation"

  given
    A .spec file with valid DSL syntax and valid semantics

  when validate

  then emits stdout
    assert output contains the result line for the spec

  then emits process_exit
    assert code == 0


behavior validate-multiple-files-all-valid [happy_path]
  "Exit 0 when all file arguments are valid"

  given
    Multiple .spec file arguments that all pass validation

  when validate

  then emits stdout
    assert output contains result for each file

  then emits process_exit
    assert code == 0


behavior validate-directory [happy_path]
  "Discover and validate all .spec files when given a directory path"

  given
    A directory containing one or more .spec files

  when validate specs/

  then
    assert all .spec files in the directory are discovered
    assert each discovered file is parsed and validated

  then emits stdout
    assert output contains result for each discovered file

  then emits process_exit
    assert code == 0


behavior validate-directory-with-invalid [happy_path]
  "Exit 1 when a directory contains at least one invalid spec"

  given
    A directory containing multiple .spec files
    At least one .spec file in the directory has validation errors

  when validate specs/

  then emits stdout
    assert output contains result for every discovered file

  then emits process_exit
    assert code == 1


behavior reject-duplicate-behavior-names [error_case]
  "Reject specs with duplicate behavior names"

  given
    A .spec file containing two or more behaviors with identical names

  when validate

  then emits stderr
    assert output mentions the duplicate behavior name

  then emits process_exit
    assert code == 1


behavior reject-unresolved-alias [error_case]
  "Reject specs with unresolved alias references"

  given
    A .spec file where a behavior input uses a from reference that
    does not match any alias declared in that behavior's given section

  when validate

  then emits stderr
    assert output mentions the unresolved alias name

  then emits process_exit
    assert code == 1


behavior reject-duplicate-aliases [error_case]
  "Reject specs with duplicate alias names in a behavior"

  given
    A .spec file where a single behavior has two preconditions
    declaring the same alias name

  when validate

  then emits stderr
    assert output mentions the duplicate alias name

  then emits process_exit
    assert code == 1


behavior reject-invalid-identity-name [error_case]
  "Reject specs with non-kebab-case names"

  given
    A .spec file where the spec name does not match the kebab-case
    pattern

  when validate

  then emits stderr
    assert output mentions the invalid name value
    assert output mentions kebab-case

  then emits process_exit
    assert code == 1


behavior reject-invalid-semver [error_case]
  "Reject specs with invalid semantic version strings"

  given
    A .spec file where the spec version is not a valid semantic
    version string

  when validate

  then emits stderr
    assert output mentions the invalid version value
    assert output mentions semver

  then emits process_exit
    assert code == 1


behavior reject-no-happy-path [error_case]
  "Reject specs with no happy_path behavior"

  given
    A .spec file where no behavior has category happy_path

  when validate

  then emits stderr
    assert output mentions happy_path

  then emits process_exit
    assert code == 1


behavior handle-nonexistent-file [edge_case]
  "Print error with the file path and exit 1"

  given
    The specified file path does not exist on disk

  when validate

  then emits stderr
    assert output contains the file path

  then emits process_exit
    assert code == 1


behavior handle-empty-directory [edge_case]
  "Print error when directory contains no .spec files"

  given
    A directory that exists but contains no .spec files

  when validate empty-dir/

  then emits stderr
    assert output mentions no spec files found
    assert output contains the directory path

  then emits process_exit
    assert code == 1


behavior handle-nonexistent-directory [edge_case]
  "Print error when directory does not exist"

  given
    The specified directory path does not exist on disk

  when validate nonexistent-dir/

  then emits stderr
    assert output contains the directory path

  then emits process_exit
    assert code == 1


behavior handle-unreadable-file [edge_case]
  "Print error with the file path and exit 1"

  given
    The .spec file exists but is not readable due to file permissions

  when validate

  then emits stderr
    assert output contains the file path
    assert output mentions permission denied

  then emits process_exit
    assert code == 1


behavior handle-empty-file [edge_case]
  "Print error and exit 1 when the file is empty"

  given
    The .spec file exists but contains no data

  when validate

  then emits stderr
    assert output mentions the file is empty

  then emits process_exit
    assert code == 1


behavior report-all-errors [edge_case]
  "Report all errors at once, not just the first"

  given
    A .spec file with multiple independent errors

  when validate

  then emits stderr
    assert output contains all error messages with line numbers

  then emits process_exit
    assert code == 1


behavior skip-semantic-when-parse-fails [edge_case]
  "Only report parse errors when DSL syntax is invalid"

  given
    A .spec file that fails parsing and would also fail semantic
    validation if parsing succeeded

  when validate

  then emits stderr
    assert output contains parse errors
    assert output does not contain semantic errors

  then emits process_exit
    assert code == 1


behavior exit-1-when-any-file-invalid [edge_case]
  "Exit 1 when at least one file in a multi-file run is invalid"

  given
    Multiple .spec file arguments where at least one is invalid

  when validate

  then emits stdout
    assert output contains result for each file

  then emits process_exit
    assert code == 1


behavior validate-all-files-independently [edge_case]
  "Report results for every file even when some fail"

  given
    Multiple .spec file arguments where the first file is invalid

  when validate

  then emits stdout
    assert output contains result for every file, not just the first


depends on dsl-format >= 1.0.0
depends on validate-display >= 1.0.0
