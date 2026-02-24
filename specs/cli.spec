spec cli v2.0.0
title "CLI Interface"

description
  The minter command-line interface. Defines the top-level command
  routing, argument parsing, help output, version display, and global
  behaviors like exit codes and error formatting. Each behavior asserts
  that the correct subcommand is reached with the correct arguments
  and that the correct exit code and output channel are used. No
  command-internal logic — that lives in each command's spec.

motivation
  The CLI is the entry point for all minter functionality. Clear
  argument parsing, helpful error messages, and predictable exit codes
  are essential for both human users and CI pipelines.


# Help and version

behavior show-help [happy_path]
  "Print usage information listing all six commands"

  given
    No arguments provided, or --help flag is used

  when minter --help

  then emits stdout
    assert output contains "minter"
    assert output contains "validate"
    assert output contains "watch"
    assert output contains "format"
    assert output contains "scaffold"
    assert output contains "inspect"
    assert output contains "graph"

  then emits process_exit
    assert code == 0


behavior show-version [happy_path]
  "Print the minter version in semver format"

  given
    The --version flag is used

  when minter --version

  then emits stdout
    assert output matches_pattern "^\\d+\\.\\d+\\.\\d+"

  then emits process_exit
    assert code == 0


# Validate routing

behavior route-validate-file [happy_path]
  "Route to isolated validation for a single file"

  given
    specs/a.spec is a valid .spec file

  when minter validate specs/a.spec

  then emits stdout
    assert output contains "a"

  then emits process_exit
    assert code == 0


behavior route-validate-file-deep [happy_path]
  "Route to deep validation with --deep flag on a single file"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec exists with version 1.0.0 and is valid

  when minter validate --deep specs/a.spec

  then emits stdout
    assert output contains "a"
    assert output contains "b"

  then emits process_exit
    assert code == 0


behavior route-validate-folder [happy_path]
  "Route to deep validation for a directory (always graph-aware)"

  given
    A directory containing specs/a.spec and specs/b.spec
    Both specs are valid

  when minter validate specs/

  then emits stdout
    assert output contains "a"
    assert output contains "b"

  then emits process_exit
    assert code == 0


# Watch routing

behavior route-watch-folder [happy_path]
  "Route to watch mode for a directory"

  given
    A directory containing valid .spec files

  when minter watch specs/

  then emits stdout
    assert output contains "watching"
    assert output contains "specs/"

  then
    assert the process does not emit an exit code


behavior route-watch-file [happy_path]
  "Route to watch mode for a single file and its dependents"

  given
    specs/a.spec is a valid .spec file

  when minter watch specs/a.spec

  then emits stdout
    assert output contains "watching"
    assert output contains "a.spec"

  then
    assert the process does not emit an exit code


# Format routing

behavior route-format [happy_path]
  "Route to the format command with the specified type"

  given
    The format subcommand is used with type fr

  when minter format fr

  then emits stdout
    assert output contains "spec"
    assert output contains "behavior"
    assert output contains "given"

  then emits process_exit
    assert code == 0


# Scaffold routing

behavior route-scaffold-fr [happy_path]
  "Route to the scaffold command for functional requirements"

  given
    The scaffold subcommand is used with type fr

  when minter scaffold fr

  then emits stdout
    assert output contains "spec"
    assert output contains "title"
    assert output contains "behavior"

  then emits process_exit
    assert code == 0


behavior route-scaffold-nfr [happy_path]
  "Route to the scaffold command for non-functional requirements with category"

  given
    The scaffold subcommand is used with type nfr and category performance

  when minter scaffold nfr performance

  then emits stdout
    assert output contains "performance"

  then emits process_exit
    assert code == 0


# Inspect routing

behavior route-inspect [happy_path]
  "Route to the inspect command for a single file"

  given
    specs/a.spec is a valid .spec file with 3 behaviors

  when minter inspect specs/a.spec

  then emits stdout
    assert output contains "3 behaviors"

  then emits process_exit
    assert code == 0


# Graph routing

behavior route-graph [happy_path]
  "Route to the graph command for a directory"

  given
    A directory containing specs/a.spec and specs/b.spec

  when minter graph specs/

  then emits stdout
    assert output contains "a"
    assert output contains "b"

  then emits process_exit
    assert code == 0


behavior route-graph-impacted [happy_path]
  "Route to the graph command with --impacted flag"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec exists with version 1.0.0

  when minter graph --impacted b specs/

  then emits stdout
    assert output contains "a"

  then emits process_exit
    assert code == 0


# Error cases

behavior reject-unknown-command [error_case]
  "Print error for an unrecognized subcommand"

  given
    An unrecognized subcommand is provided

  when minter frobnicate

  then emits stderr
    assert output contains "frobnicate"
    assert output contains "validate"
    assert output contains "watch"
    assert output contains "format"
    assert output contains "scaffold"
    assert output contains "inspect"
    assert output contains "graph"

  then emits process_exit
    assert code == 1


behavior reject-unknown-flag [error_case]
  "Print error for an unrecognized flag"

  given
    An unrecognized flag is provided to a known command

  when minter validate --frobnicate specs/a.spec

  then emits stderr
    assert output contains "--frobnicate"

  then emits process_exit
    assert code == 1


behavior reject-missing-required-argument [error_case]
  "Print error when a command is used with no required arguments"

  given
    The validate subcommand is used with no file or directory arguments

  when minter validate

  then emits stderr
    assert output contains "validate"

  then emits process_exit
    assert code == 1


behavior reject-non-spec-extension [error_case]
  "Print error when a file does not have a .spec extension"

  given
    A file argument that does not end in .spec

  when minter validate readme.md

  then emits stderr
    assert output contains ".spec"

  then emits process_exit
    assert code == 1


# Edge cases

behavior accept-deep-on-folder-as-noop [edge_case]
  "Accept --deep flag on folder without error (folder is always deep)"

  given
    A directory containing specs/a.spec and specs/b.spec
    Both specs are valid

  when minter validate --deep specs/

  then emits stdout
    assert output contains "a"
    assert output contains "b"

  then emits process_exit
    assert code == 0


behavior handle-mixed-valid-invalid-files [error_case]
  "Process all files even when some do not exist or are invalid"

  given
    specs/exists.spec is a valid .spec file
    specs/missing.spec does not exist on disk

  when minter validate specs/exists.spec specs/missing.spec

  then emits stdout
    assert output contains "exists"

  then emits stderr
    assert output contains "missing.spec"

  then emits process_exit
    assert code == 1


depends on validate-command >= 2.0.0
depends on dependency-resolution >= 2.0.0
depends on watch-command >= 2.0.0
depends on format-command >= 1.0.0
depends on scaffold-command >= 1.0.0
depends on inspect-command >= 1.0.0
depends on graph-command >= 1.0.0
