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
    assert output contains spec name
    assert output contains "valid"

  then emits process_exit
    assert code == 0


behavior validate-prints-summary [happy_path]
  "Print spec metadata summary to stdout on successful validation"

  given
    A .spec file that passes all validation

  when validate

  then emits stdout
    assert output contains spec name
    assert output contains spec version
    assert output contains behavior count


behavior validate-multiple-files-all-valid [happy_path]
  "Exit 0 when all file arguments are valid"

  given
    Multiple .spec file arguments that all pass validation

  when validate

  then emits stdout
    assert output contains result for each file

  then emits process_exit
    assert code == 0


behavior reject-duplicate-behavior-names [error_case]
  "Print error identifying the duplicate name and exit 1"

  given
    A .spec file containing two or more behaviors with identical names

  when validate

  then emits stderr
    assert output mentions the duplicate behavior name

  then emits process_exit
    assert code == 1


behavior reject-unresolved-alias [error_case]
  "Print error identifying the unresolved alias and exit 1"

  given
    A .spec file where a behavior input uses a from reference that
    does not match any alias declared in that behavior's given section

  when validate

  then emits stderr
    assert output mentions the unresolved alias name

  then emits process_exit
    assert code == 1


behavior reject-duplicate-aliases [error_case]
  "Print error identifying the duplicate alias and exit 1"

  given
    A .spec file where a single behavior has two preconditions
    declaring the same alias name

  when validate

  then emits stderr
    assert output mentions the duplicate alias name

  then emits process_exit
    assert code == 1


behavior reject-invalid-identity-name [error_case]
  "Print error showing the invalid name and expected format, exit 1"

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
  "Print error showing the invalid version and expected format, exit 1"

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
  "Print error stating at least one happy_path is required, exit 1"

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
