spec validate-command v2.1.0
title "Validate Command"

description
  The minter validate command reads one or more .spec files or
  directories, parses the DSL, runs semantic validation, and reports
  pass/fail with detailed error diagnostics. Single file validation
  is isolated (syntax + semantic only). The --deep flag enables full
  dependency graph resolution for a single file. Folder validation
  is always deep — all files plus the full dependency graph.

motivation
  This is the deterministic gate in the spec-driven pipeline. Every
  spec must pass through minter validate before reaching downstream
  agents. Without this gate, spec inconsistencies propagate silently.

nfr
  performance#validation-latency
  performance#directory-validation-scaling
  performance#no-redundant-file-reads
  performance#single-discovery-pass
  performance#cache-skip-unchanged
  performance#large-tree-validation-scaling
  reliability#error-completeness
  reliability#no-silent-data-loss
  operability#ci-friendly-output


behavior validate-valid-spec [happy_path]
  "Report success and exit 0 when a spec passes all validation"

  given
    A .spec file named my-feature at version 1.0.0 with 5 behaviors
    The file has valid DSL syntax and valid semantics

  when minter validate specs/my-feature.spec

  then emits stdout
    assert output contains "my-feature"
    assert output contains "v1.0.0"
    assert output contains "5 behaviors"

  then emits process_exit
    assert code == 0


behavior validate-multiple-files-all-valid [happy_path]
  "Exit 0 when all file arguments are valid"

  given
    specs/a.spec and specs/b.spec both pass validation

  when minter validate specs/a.spec specs/b.spec

  then emits stdout
    assert output contains "a"
    assert output contains "b"

  then emits process_exit
    assert code == 0


behavior validate-single-file-is-isolated [happy_path]
  "Single file validation runs syntax and semantic checks only, no graph"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec does not exist
    specs/a.spec has valid syntax and semantics

  when minter validate specs/a.spec

  then emits stdout
    assert output contains "a"

  then emits process_exit
    assert code == 0


behavior validate-deep-single-file [happy_path]
  "The --deep flag enables dependency graph resolution for a single file"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec exists with version 1.0.0 and is valid

  when minter validate --deep specs/a.spec

  then emits stdout
    assert output contains "a"
    assert output contains "b"

  then emits process_exit
    assert code == 0


behavior discover-specs-in-directory [happy_path]
  "Discover all .spec files recursively when given a directory path"

  given
    A directory containing:
    specs/a.spec
    specs/sub/b.spec
    specs/sub/deep/c.spec

  when minter validate specs/

  then emits stdout
    assert output contains "a"
    assert output contains "b"
    assert output contains "c"

  then emits process_exit
    assert code == 0


behavior validate-directory-is-always-deep [happy_path]
  "Folder validation always resolves the full dependency graph"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec exists with version 1.0.0
    Both specs are valid

  when minter validate specs/

  then emits stdout
    assert output contains "a"
    assert output contains "b"

  then emits process_exit
    assert code == 0


# Error cases

behavior reject-duplicate-behavior-names [error_case]
  "Reject specs with duplicate behavior names"

  given
    A .spec file containing two behaviors both named do-thing

  when minter validate

  then emits stderr
    assert output contains "do-thing"
    assert output contains "duplicate"

  then emits process_exit
    assert code == 1


behavior reject-unresolved-alias [error_case]
  "Reject specs with unresolved alias references"

  given
    A .spec file where a when input references @missing_alias.id
    No alias named missing_alias is declared in that behavior's given

  when minter validate

  then emits stderr
    assert output contains "missing_alias"

  then emits process_exit
    assert code == 1


behavior reject-duplicate-aliases [error_case]
  "Reject specs with duplicate alias names in a behavior"

  given
    A .spec file where a single behavior declares @user twice in its given

  when minter validate

  then emits stderr
    assert output contains "user"
    assert output contains "duplicate"

  then emits process_exit
    assert code == 1


behavior reject-invalid-identity-name [error_case]
  "Reject specs with non-kebab-case names"

  given
    A .spec file with: spec MyFeature v1.0.0

  when minter validate

  then emits stderr
    assert output contains "MyFeature"
    assert output contains "kebab-case"

  then emits process_exit
    assert code == 1


behavior reject-invalid-semver [error_case]
  "Reject specs with invalid semantic version strings"

  given
    A .spec file with: spec my-feature v1.0

  when minter validate

  then emits stderr
    assert output contains "1.0"
    assert output contains "semver"

  then emits process_exit
    assert code == 1


behavior reject-no-happy-path [error_case]
  "Reject specs with no happy_path behavior"

  given
    A .spec file where all behaviors have category error_case or edge_case

  when minter validate

  then emits stderr
    assert output contains "happy_path"

  then emits process_exit
    assert code == 1


behavior validate-directory-with-invalid [error_case]
  "Exit 1 when a directory contains at least one invalid spec"

  given
    A directory containing specs/valid.spec and specs/broken.spec
    specs/broken.spec has validation errors

  when minter validate specs/

  then emits stdout
    assert output contains "valid"
    assert output contains "broken"

  then emits process_exit
    assert code == 1


behavior reject-nonexistent-file [error_case]
  "Print error with the file path and exit 1"

  given
    The specified file path does not exist on disk

  when minter validate nonexistent.spec

  then emits stderr
    assert output contains "nonexistent.spec"

  then emits process_exit
    assert code == 1


behavior reject-empty-directory [error_case]
  "Print error when directory contains no .spec files"

  given
    A directory that exists but contains no .spec files

  when minter validate empty-dir/

  then emits stderr
    assert output contains "no spec files found"
    assert output contains "empty-dir"

  then emits process_exit
    assert code == 1


behavior reject-nonexistent-directory [error_case]
  "Print error when directory does not exist"

  given
    The specified directory path does not exist on disk

  when minter validate nonexistent-dir/

  then emits stderr
    assert output contains "nonexistent-dir"

  then emits process_exit
    assert code == 1


behavior reject-unreadable-file [error_case]
  "Print error with the file path when permissions deny read access"

  given
    The .spec file exists but is not readable due to file permissions

  when minter validate unreadable.spec

  then emits stderr
    assert output contains "unreadable.spec"
    assert output contains "permission"

  then emits process_exit
    assert code == 1


behavior reject-empty-file [error_case]
  "Print error and exit 1 when the file is empty"

  given
    The .spec file exists but contains no data

  when minter validate empty.spec

  then emits stderr
    assert output contains "empty"

  then emits process_exit
    assert code == 1


# Edge cases

behavior report-all-errors [edge_case]
  "Report all errors at once, not just the first"

  given
    A .spec file with three independent errors at lines 5, 12, and 20

  when minter validate

  then emits stderr
    assert output contains "5"
    assert output contains "12"
    assert output contains "20"


behavior skip-semantic-when-parse-fails [edge_case]
  "Only report parse errors when DSL syntax is invalid"

  given
    A .spec file that fails parsing and would also fail semantic
    validation if parsing succeeded

  when minter validate

  then emits stderr
    assert output contains parse error messages
    assert output does not contain "duplicate" or "kebab-case" or "happy_path"


behavior exit-1-when-any-file-invalid [error_case]
  "Report results for all files and exit 1 when at least one is invalid"

  given
    specs/valid.spec passes validation
    specs/broken.spec fails validation

  when minter validate specs/valid.spec specs/broken.spec

  then emits stdout
    assert output contains "valid"
    assert output contains "broken"

  then emits process_exit
    assert code == 1


behavior validate-all-files-independently [error_case]
  "Report results for every file even when the first file fails"

  given
    specs/broken.spec fails validation
    specs/valid.spec passes validation

  when minter validate specs/broken.spec specs/valid.spec

  then emits stdout
    assert output contains "broken"
    assert output contains "valid"

  then emits process_exit
    assert code == 1


# NFR validation

behavior validate-valid-nfr [happy_path]
  "Report success and exit 0 when an NFR file passes all validation"

  given
    A .nfr file with category performance at version 1.0.0 with 3 constraints
    The file has valid NFR DSL syntax and valid semantics

  when minter validate specs/performance.nfr

  then emits stdout
    assert output contains "performance"
    assert output contains "v1.0.0"
    assert output contains "3 constraints"

  then emits process_exit
    assert code == 0


behavior validate-nfr-single-file-is-isolated [happy_path]
  "Single NFR file validation has no dependency graph"

  given
    A valid .nfr file with category performance
    No other spec or nfr files exist in the directory

  when minter validate specs/performance.nfr

  then emits stdout
    assert output contains "performance"

  then emits process_exit
    assert code == 0


behavior discover-nfr-in-directory [happy_path]
  "Discover .nfr files recursively when given a directory path"

  given
    A directory containing:
    specs/performance.nfr
    specs/sub/security.nfr

  when minter validate specs/

  then emits stdout
    assert output contains "performance"
    assert output contains "security"

  then emits process_exit
    assert code == 0


behavior validate-mixed-spec-and-nfr-directory [happy_path]
  "Validate both .spec and .nfr files when discovered in same directory"

  given
    A directory containing:
    specs/my-feature.spec (valid)
    specs/performance.nfr (valid)

  when minter validate specs/

  then emits stdout
    assert output contains "my-feature"
    assert output contains "performance"

  then emits process_exit
    assert code == 0


behavior reject-invalid-nfr [error_case]
  "Exit 1 when an NFR file has validation errors"

  given
    A .nfr file with missing required fields

  when minter validate specs/broken.nfr

  then emits stderr
    assert output contains validation error messages

  then emits process_exit
    assert code == 1


behavior validate-cross-references-in-directory [happy_path]
  "Validate FR-to-NFR cross-references when directory contains both types"

  given
    A directory containing:
    specs/my-feature.spec with nfr section referencing performance
    specs/performance.nfr declaring category performance
    All references resolve correctly

  when minter validate specs/

  then emits stdout
    assert output contains "my-feature"
    assert output contains "performance"

  then emits process_exit
    assert code == 0


behavior reject-broken-cross-reference [error_case]
  "Exit 1 when an FR spec references a nonexistent NFR category or anchor"

  given
    A directory containing:
    specs/my-feature.spec with nfr section referencing reliability
    No .nfr file declares category reliability

  when minter validate specs/

  then emits stderr
    assert output contains "reliability"
    assert output contains "not found" or "missing"

  then emits process_exit
    assert code == 1


depends on spec-grammar >= 1.1.0
depends on cli-display >= 2.0.0
depends on nfr-grammar >= 1.0.0
depends on nfr-cross-reference >= 1.0.0
