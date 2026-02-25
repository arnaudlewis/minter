spec cli-display v2.1.0
title "Validation Display"

description
  The single source of truth for all visual output: result lines,
  dependency tree rendering, ANSI colors, indicator symbols, and
  channel separation between stdout and stderr. Every spec that
  produces user-facing output references this spec for display rules.

motivation
  Validation output format was previously scattered across multiple
  specs. Centralizing all display concerns into one spec ensures
  consistent presentation across single-file, multi-file, directory,
  dependency-tree, and watch mode output.

nfr
  operability#deterministic-output
  operability#ci-friendly-output


# Success output

behavior display-success-line [happy_path]
  "Print a checkmark line with name, version, and behavior count"

  given
    A spec named my-feature at version 1.2.0 with 12 behaviors
    The spec passes all validation

  when minter validate specs/my-feature.spec

  then emits stdout
    assert output contains "✓ my-feature v1.2.0 (12 behaviors)"


behavior display-singular-behavior-count [happy_path]
  "Use singular form when spec has exactly one behavior"

  given
    A spec named single-case at version 1.0.0 with 1 behavior
    The spec passes all validation

  when minter validate specs/single-case.spec

  then emits stdout
    assert output contains "✓ single-case v1.0.0 (1 behavior)"


# Failure output

behavior display-failure-line [error_case]
  "Print a cross mark line with name and version on validation failure"

  given
    A spec named broken-feature at version 2.0.0
    The spec fails validation

  when minter validate specs/broken-feature.spec

  then emits stdout
    assert output contains "✗ broken-feature v2.0.0"

  then emits process_exit
    assert code == 1


behavior display-errors-on-stderr [error_case]
  "Print error details to stderr, not stdout"

  given
    A spec that fails validation with errors at line 5 and line 12

  when minter validate

  then emits stderr
    assert output contains "5"
    assert output contains "12"

  then emits stdout
    assert output does not contain "error"


# Validation result colors

behavior color-success-checkmark-green [happy_path]
  "Render the success checkmark in green using ANSI escape"

  given
    A spec that passes validation
    The output terminal supports ANSI colors

  when minter validate specs/valid.spec

  then emits stdout
    assert the ✓ character is wrapped in ANSI green escape sequences


behavior color-failure-cross-red [error_case]
  "Render the failure cross mark in red using ANSI escape"

  given
    A spec that fails validation
    The output terminal supports ANSI colors

  when minter validate specs/broken.spec

  then emits stdout
    assert the ✗ character is wrapped in ANSI red escape sequences


# File event colors (used by watch mode)

behavior color-changed-file-yellow [happy_path]
  "Render changed file events in yellow"

  given
    A file change event is being displayed
    The output terminal supports ANSI colors

  when display file change event

  then emits stdout
    assert the changed file path is wrapped in ANSI yellow escape sequences


behavior color-new-file-cyan [happy_path]
  "Render new file events in cyan"

  given
    A new file event is being displayed
    The output terminal supports ANSI colors

  when display new file event

  then emits stdout
    assert the new file path is wrapped in ANSI cyan escape sequences


behavior color-deleted-file-red [happy_path]
  "Render deleted file events in red"

  given
    A file deletion event is being displayed
    The output terminal supports ANSI colors

  when display deleted file event

  then emits stdout
    assert the deleted file path is wrapped in ANSI red escape sequences


behavior color-banner-cyan [happy_path]
  "Render UI banners in cyan"

  given
    A status banner is being displayed (e.g. watching banner)
    The output terminal supports ANSI colors

  when display banner

  then emits stdout
    assert the banner text is wrapped in ANSI cyan escape sequences


# Dependency tree

behavior display-dependency-tree [happy_path]
  "Show the root spec with its dependencies as a tree using connectors"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec depends on c >= 1.0.0
    specs/c.spec has no dependencies
    All specs are valid

  when minter validate --deep specs/a.spec

  then emits stdout
    assert output shows a as the root with its result line
    assert output shows b as a child of a using tree connectors
    assert output shows c as a child of b using tree connectors
    assert tree connectors use the characters: │ ├── └──


behavior display-first-occurrence-expanded [happy_path]
  "Show full result line with version and behavior count on first occurrence"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec has no dependencies
    Both specs are valid

  when minter validate --deep specs/a.spec

  then emits stdout
    assert b appears with its version and behavior count on first occurrence


behavior display-repeated-dep-dimmed [happy_path]
  "Show repeated dependencies in dimmed text using ANSI dim escape"

  given
    specs/a.spec depends on b >= 1.0.0 and c >= 1.0.0
    specs/c.spec depends on b >= 1.0.0
    specs/b.spec has no dependencies
    All specs are valid

  when minter validate --deep specs/a.spec

  then emits stdout
    assert b is expanded at its shallowest depth in the tree
    assert b appears with ANSI dim escape sequence at deeper occurrences
    assert the dimmed occurrence does not repeat b's subtree


behavior display-repeated-dep-preserves-status [happy_path]
  "Show the correct checkmark or cross mark on dimmed repeated dependencies"

  given
    specs/a.spec depends on b >= 1.0.0 and c >= 1.0.0
    specs/c.spec depends on b >= 1.0.0
    specs/b.spec fails validation

  when minter validate --deep specs/a.spec

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

  when minter validate --deep specs/

  then emits stdout
    assert a appears as the root with its full tree including b
    assert b does not appear again as a separate root-level entry


behavior display-tree-error-on-stderr [error_case]
  "Print error details to stderr when a dependency fails in the tree"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec fails validation

  when minter validate --deep specs/a.spec

  then emits stderr
    assert output contains the validation error for b
    assert errors are printed only once even if b appears multiple times

  then emits process_exit
    assert code == 1


# Deep validation messaging

behavior display-dependency-count [happy_path]
  "Show the number of dependencies resolved during deep validation"

  given
    specs/a.spec depends on b >= 1.0.0 and c >= 1.0.0
    Both dependencies resolve successfully

  when minter validate --deep specs/a.spec

  then emits stdout
    assert output contains "2" as the dependency count


# No-color mode

behavior no-color-mode [edge_case]
  "Suppress ANSI escape sequences when NO_COLOR env is set or output is not a tty"

  given
    The NO_COLOR environment variable is set to any value
    A spec that passes validation

  when minter validate specs/valid.spec

  then emits stdout
    assert output contains "✓"
    assert output does not contain ANSI escape sequences


# Very long names

behavior display-long-spec-name [edge_case]
  "Display full spec name without truncation even when very long"

  given
    A spec named my-very-long-feature-name-that-exceeds-typical-width at version 1.0.0
    The spec passes all validation with 5 behaviors

  when minter validate specs/my-very-long-feature-name-that-exceeds-typical-width.spec

  then emits stdout
    assert output contains "✓ my-very-long-feature-name-that-exceeds-typical-width v1.0.0 (5 behaviors)"


# Channel separation

# NFR display

behavior display-nfr-success-line [happy_path]
  "Print a checkmark line with category, version, and constraint count for NFR"

  given
    An NFR file with category performance at version 1.0.0 with 4 constraints
    The NFR file passes all validation

  when minter validate specs/performance.nfr

  then emits stdout
    assert output contains "✓ performance v1.0.0 (4 constraints)"


behavior display-nfr-singular-constraint-count [happy_path]
  "Use singular form when NFR has exactly one constraint"

  given
    An NFR file with category security at version 1.0.0 with 1 constraint
    The NFR file passes all validation

  when minter validate specs/security.nfr

  then emits stdout
    assert output contains "✓ security v1.0.0 (1 constraint)"


behavior display-nfr-failure-line [error_case]
  "Print a cross mark line with category and version on NFR validation failure"

  given
    An NFR file with category performance at version 2.0.0
    The NFR file fails validation

  when minter validate specs/performance.nfr

  then emits stdout
    assert output contains "✗ performance v2.0.0"

  then emits process_exit
    assert code == 1


# Channel separation

behavior separate-result-and-errors [happy_path]
  "Result lines go to stdout, error details go to stderr"

  given
    A spec named clean at version 1.0.0 with 3 behaviors
    The spec passes all validation

  when minter validate specs/clean.spec

  then emits stdout
    assert output contains "clean"
    assert output contains "v1.0.0"

  then emits stderr
    assert output is empty


depends on nfr-dsl-format >= 1.0.0
