spec validate-display v1.0.0
title "Validation Display"

description
  Defines the output format for specval validation results. Covers single-spec
  result lines, dependency tree rendering, and channel separation between
  stdout and stderr. This spec is the single source of truth for how
  validation results are presented to the user.

motivation
  Validation output format was previously implicit and scattered across
  validate-command behaviors. Extracting it into a dedicated spec ensures
  consistent presentation across single-file, multi-file, directory, and
  dependency-tree validation modes.

# Success output

behavior display-success-line [happy_path]
  "Print a checkmark line with name, version, and behavior count"

  given
    A spec named my-feature at version 1.2.0 with 12 behaviors
    The spec passes all validation

  when validate specs/my-feature.spec

  then emits stdout
    assert output contains a line matching: ✓ my-feature v1.2.0 (12 behaviors)


behavior display-singular-behavior-count [happy_path]
  "Use singular form when spec has exactly one behavior"

  given
    A spec named single-case at version 1.0.0 with 1 behavior
    The spec passes all validation

  when validate specs/single-case.spec

  then emits stdout
    assert output contains a line matching: ✓ single-case v1.0.0 (1 behavior)


# Failure output

behavior display-failure-line [error_case]
  "Print a cross mark line with name and version on validation failure"

  given
    A spec named broken-feature at version 2.0.0
    The spec fails validation

  when validate specs/broken-feature.spec

  then emits stdout
    assert output contains a line matching: ✗ broken-feature v2.0.0


behavior display-errors-on-stderr [error_case]
  "Print error details to stderr, not stdout"

  given
    A spec that fails validation with one or more errors

  when validate

  then emits stderr
    assert output contains each error message
    assert output contains line numbers for located errors

  then emits stdout
    assert output does not contain error details


# Dependency tree

behavior display-dependency-tree [happy_path]
  "Show the root spec with its dependencies as a tree using connectors"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec depends on c >= 1.0.0
    specs/c.spec has no dependencies
    All specs are valid

  when validate --deps specs/a.spec

  then emits stdout
    assert output shows a as the root with its result line
    assert output shows b as a child of a using tree connectors
    assert output shows c as a child of b using tree connectors
    assert tree connectors use the characters: pipe, tee, and elbow


behavior display-first-occurrence-expanded [happy_path]
  "Show full result line with version and behavior count on first occurrence"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec has no dependencies
    Both specs are valid

  when validate --deps specs/a.spec

  then emits stdout
    assert b appears with its version and behavior count on first occurrence


behavior display-repeated-dep-dimmed [happy_path]
  "Show repeated dependencies in dimmed text without expanding their subtree"

  given
    specs/a.spec depends on b >= 1.0.0 and c >= 1.0.0
    specs/c.spec depends on b >= 1.0.0
    specs/b.spec has no dependencies
    All specs are valid

  when validate --deps specs/a.spec

  then emits stdout
    assert b is expanded at its shallowest depth in the tree
    assert b appears in dimmed text at deeper occurrences
    assert the dimmed occurrence does not repeat b's subtree


behavior display-repeated-dep-preserves-status [happy_path]
  "Show the correct checkmark or cross mark on dimmed repeated dependencies"

  given
    specs/a.spec depends on b >= 1.0.0 and c >= 1.0.0
    specs/c.spec depends on b >= 1.0.0
    specs/b.spec fails validation

  when validate --deps specs/a.spec

  then emits stdout
    assert b shows ✗ on first occurrence
    assert b shows ✗ in dimmed text on subsequent occurrences


behavior skip-already-shown-root [happy_path]
  "Skip root-level display for specs already shown in another spec's tree"

  given
    A directory containing specs a.spec, b.spec
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec has no dependencies
    All specs are valid

  when validate --deps specs/

  then emits stdout
    assert a appears as the root with its full tree including b
    assert b does not appear again as a separate root-level entry


behavior display-tree-error-on-stderr [error_case]
  "Print error details to stderr when a dependency fails in the tree"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec fails validation

  when validate --deps specs/a.spec

  then emits stderr
    assert output contains the validation error for b
    assert errors are printed only once even if b appears multiple times

  then emits process_exit
    assert code == 1


# Channel separation

behavior separate-result-and-errors [happy_path]
  "Result lines go to stdout, error details go to stderr"

  given
    A spec that passes validation

  when validate

  then emits stdout
    assert output contains the result line

  then emits stderr
    assert output is empty
