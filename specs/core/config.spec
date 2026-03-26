spec config v1.0.0
title "Project Configuration"

description
  Minter uses convention-over-configuration to locate project resources.
  By default it looks for specs in specs/ and tests in tests/. A
  minter.config.json file at the project root overrides these defaults.
  Commands that need paths — validate, coverage, graph, lock, ci — read
  from this config instead of requiring CLI arguments. When no config
  file exists, conventions apply silently. The config file is optional;
  minter never creates it automatically.

motivation
  Today every command requires explicit path arguments. This is verbose,
  error-prone, and forces users to remember conventions. A single config
  file centralizes project layout and eliminates repetitive flags. It
  also enables the lock and ci commands to discover resources without
  manual input.

nfr
  operability#zero-config
  reliability#error-completeness


# Config loading — happy path

behavior load-default-conventions [happy_path]
  "Use specs/ and tests/ when no minter.config.json exists"

  given
    No minter.config.json file exists at the project root
    A specs/ directory contains valid .spec files
    A tests/ directory contains files with @minter tags

  when any command that reads config is invoked without explicit paths

  then
    assert specs are discovered from specs/
    assert tests are scanned from tests/


behavior load-config-file [happy_path]
  "Read paths from minter.config.json when it exists"

  given
    minter.config.json at project root contains:
    { "specs": "specifications/", "tests": ["tests/e2e/", "tests/unit/"] }
    A specifications/ directory contains valid .spec files

  when any command that reads config is invoked without explicit paths

  then
    assert specs are discovered from specifications/
    assert tests are scanned from tests/e2e/ and tests/unit/


behavior config-specs-string [happy_path]
  "Accept specs as a single directory string"

  given
    minter.config.json contains: { "specs": "my-specs/" }

  when config is loaded

  then
    assert specs directory resolves to my-specs/


behavior config-tests-array [happy_path]
  "Accept tests as an array of directory strings"

  given
    minter.config.json contains: { "tests": ["tests/unit/", "tests/e2e/", "benches/"] }

  when config is loaded

  then
    assert test directories resolve to tests/unit/, tests/e2e/, and benches/


behavior config-tests-single-string [happy_path]
  "Accept tests as a single directory string for convenience"

  given
    minter.config.json contains: { "tests": "tests/" }

  when config is loaded

  then
    assert test directories resolve to tests/


behavior config-partial-override [happy_path]
  "Use defaults for fields not specified in config"

  given
    minter.config.json contains: { "tests": ["src/"] }
    No specs field is present in the config

  when config is loaded

  then
    assert specs directory falls back to default specs/
    assert test directories resolve to src/


# CLI override

behavior cli-args-override-config [happy_path]
  "Explicit CLI path arguments take precedence over config"

  given
    minter.config.json contains: { "specs": "specifications/" }
    A different directory other-specs/ contains valid .spec files

  when minter validate other-specs/

  then
    assert validation targets other-specs/, not specifications/


# Error cases

behavior reject-invalid-json [error_case]
  "Print error when minter.config.json is not valid JSON"

  given
    minter.config.json contains invalid JSON syntax

  when any command that reads config is invoked

  then emits stderr
    assert output contains "minter.config.json"
    assert output contains "invalid" or "parse"

  then emits process_exit
    assert code == 1


behavior reject-invalid-specs-type [error_case]
  "Print error when specs field is not a string"

  given
    minter.config.json contains: { "specs": 42 }

  when any command that reads config is invoked

  then emits stderr
    assert output contains "specs"
    assert output contains "string"

  then emits process_exit
    assert code == 1


behavior reject-invalid-tests-type [error_case]
  "Print error when tests field is neither a string nor an array of strings"

  given
    minter.config.json contains: { "tests": { "unit": "tests/" } }

  when any command that reads config is invoked

  then emits stderr
    assert output contains "tests"

  then emits process_exit
    assert code == 1


behavior reject-nonexistent-specs-dir [error_case]
  "Print error when configured specs directory does not exist"

  given
    minter.config.json contains: { "specs": "nonexistent/" }
    The directory nonexistent/ does not exist on disk

  when any command that reads config is invoked

  then emits stderr
    assert output contains "nonexistent"
    assert output contains "not found" or "does not exist"

  then emits process_exit
    assert code == 1


behavior reject-nonexistent-tests-dir [error_case]
  "Print error when a configured test directory does not exist"

  given
    minter.config.json contains: { "tests": ["tests/", "nonexistent/"] }
    The directory nonexistent/ does not exist on disk

  when any command that reads config is invoked

  then emits stderr
    assert output contains "nonexistent"

  then emits process_exit
    assert code == 1


behavior reject-unknown-fields [error_case]
  "Print error when config contains unrecognized fields"

  given
    minter.config.json contains: { "specs": "specs/", "output": "dist/" }

  when any command that reads config is invoked

  then emits stderr
    assert output contains "output"
    assert output contains "unknown"

  then emits process_exit
    assert code == 1


# Edge cases

behavior empty-config-uses-defaults [edge_case]
  "An empty JSON object uses all defaults"

  given
    minter.config.json contains: {}

  when config is loaded

  then
    assert specs directory falls back to default specs/
    assert test directories fall back to default tests/


behavior no-default-dirs-without-config [edge_case]
  "When no config exists and default directories do not exist, commands that require paths report an error"

  given
    No minter.config.json exists
    Neither specs/ nor tests/ directories exist

  when a command that requires specs is invoked without explicit paths

  then emits stderr
    assert output contains "specs"
    assert output contains "not found" or "does not exist"

  then emits process_exit
    assert code == 1
