spec cli v1.0.0
title "CLI Interface"

description
  The specval command-line interface. Defines available commands, argument
  parsing, help output, version display, and global behavior like exit
  codes and error formatting.

motivation
  The CLI is the entry point for all specval functionality. Clear argument
  parsing, helpful error messages, and predictable exit codes are essential
  for both human users and CI pipelines.


behavior show-help [happy_path]
  "Print usage information and available commands"

  given
    No arguments provided, or --help flag is used

  when specval

  then emits stdout
    assert output contains "specval"
    assert output contains "validate"
    assert output contains available flags

  then emits process_exit
    assert code == 0


behavior show-version [happy_path]
  "Print the specval version"

  given
    The --version flag is used

  when specval --version

  then emits stdout
    assert output contains version number in semver format

  then emits process_exit
    assert code == 0


behavior validate-command [happy_path]
  "Route to the validate command with file arguments"

  given
    The validate subcommand is used with one or more .spec file paths

  when specval validate file.spec

  then
    assert the validate command runs on the provided files


behavior validate-deps-flag [happy_path]
  "Validate the spec then resolve and validate its dependencies"

  given
    The validate subcommand is used with the --deps flag

  when specval validate --deps file.spec

  then
    assert the spec itself is validated first
    assert dependency resolution runs only if spec validation passes


behavior reject-unknown-command [error_case]
  "Print error for an unrecognized subcommand"

  given
    An unrecognized subcommand is provided

  when specval frobnicate

  then emits stderr
    assert output mentions the unrecognized command
    assert output suggests available commands

  then emits process_exit
    assert code == 1


behavior reject-no-files [error_case]
  "Print error when validate is called with no file arguments"

  given
    The validate subcommand is used with no file paths

  when specval validate

  then emits stderr
    assert output mentions that file arguments are required

  then emits process_exit
    assert code == 1


behavior reject-non-spec-extension [error_case]
  "Print error when a file does not have a .spec extension"

  given
    A file argument that does not end in .spec

  when specval validate readme.md

  then emits stderr
    assert output mentions the expected .spec extension

  then emits process_exit
    assert code == 1


behavior reject-unknown-flag [error_case]
  "Print error for an unrecognized flag"

  given
    An unrecognized flag is provided

  when specval validate --frobnicate file.spec

  then emits stderr
    assert output mentions the unrecognized flag

  then emits process_exit
    assert code == 1


behavior handle-mixed-valid-invalid-files [edge_case]
  "Process all files even when some do not exist"

  given
    Multiple file arguments where some exist and some do not

  when specval validate exists.spec missing.spec

  then emits stdout
    assert output contains result for existing file

  then emits stderr
    assert output contains error for missing file

  then emits process_exit
    assert code == 1


depends on validate-command >= 1.0.0
depends on validate-dependencies >= 1.0.0
depends on watch-mode >= 1.0.0
