spec inspect-command v1.1.0
title "Inspect Command"

description
  The minter inspect command reads a single .spec file, validates it,
  and displays structured metadata: behavior count, category distribution,
  dependency list with version constraints, and assertion types used.

motivation
  When reviewing or debugging a spec, authors need a quick summary of
  its structure without reading the full file. The inspect command
  provides this at a glance.

nfr
  operability#ci-friendly-output


behavior inspect-behavior-count [happy_path]
  "Display the total number of behaviors in the spec"

  given
    specs/my-feature.spec is valid with 8 behaviors

  when minter inspect specs/my-feature.spec

  then emits stdout
    assert output contains "8 behaviors"

  then emits process_exit
    assert code == 0


behavior inspect-category-distribution [happy_path]
  "Display the count of behaviors per category"

  given
    specs/my-feature.spec is valid with 4 happy_path, 2 error_case, 1 edge_case

  when minter inspect specs/my-feature.spec

  then emits stdout
    assert output contains "happy_path" and "4"
    assert output contains "error_case" and "2"
    assert output contains "edge_case" and "1"

  then emits process_exit
    assert code == 0


behavior inspect-dependencies [happy_path]
  "Display dependency names and version constraints"

  given
    specs/my-feature.spec depends on user-auth >= 1.0.0 and billing >= 2.0.0

  when minter inspect specs/my-feature.spec

  then emits stdout
    assert output contains "user-auth" and ">= 1.0.0"
    assert output contains "billing" and ">= 2.0.0"

  then emits process_exit
    assert code == 0


behavior inspect-assertion-types [happy_path]
  "Display the distinct assertion types used across all behaviors"

  given
    specs/my-feature.spec uses assertions of type equals, contains, and is_present

  when minter inspect specs/my-feature.spec

  then emits stdout
    assert output contains "equals"
    assert output contains "contains"
    assert output contains "is_present"

  then emits process_exit
    assert code == 0


behavior inspect-invalid-spec [error_case]
  "Display validation errors on stderr when the spec is invalid"

  given
    specs/broken.spec has parse or semantic errors

  when minter inspect specs/broken.spec

  then emits stderr
    assert output contains validation error messages

  then emits process_exit
    assert code == 1


behavior inspect-nonexistent-file [error_case]
  "Display file path in error when the file does not exist"

  given
    The file specs/missing.spec does not exist on disk

  when minter inspect specs/missing.spec

  then emits stderr
    assert output contains "missing.spec"

  then emits process_exit
    assert code == 1


behavior inspect-no-dependencies [edge_case]
  "Indicate that the spec has no dependencies"

  given
    specs/standalone.spec is valid with no depends on declarations

  when minter inspect specs/standalone.spec

  then emits stdout
    assert output contains "no dependencies"

  then emits process_exit
    assert code == 0


# NFR inspection

behavior inspect-nfr-constraint-count [happy_path]
  "Display the total number of constraints in an NFR file"

  given
    specs/performance.nfr is valid with 4 constraints

  when minter inspect specs/performance.nfr

  then emits stdout
    assert output contains "4 constraints"

  then emits process_exit
    assert code == 0


behavior inspect-nfr-type-distribution [happy_path]
  "Display the count of metric vs rule constraints"

  given
    specs/performance.nfr is valid with 3 metric and 2 rule constraints

  when minter inspect specs/performance.nfr

  then emits stdout
    assert output contains "metric" and "3"
    assert output contains "rule" and "2"

  then emits process_exit
    assert code == 0


behavior inspect-nfr-category [happy_path]
  "Display the NFR category"

  given
    specs/performance.nfr is valid with category performance

  when minter inspect specs/performance.nfr

  then emits stdout
    assert output contains "performance"

  then emits process_exit
    assert code == 0


behavior inspect-nfr-no-dependencies [edge_case]
  "Indicate that an NFR file has no dependencies"

  given
    specs/performance.nfr is a valid NFR file with no depends on declarations

  when minter inspect specs/performance.nfr

  then emits stdout
    assert output contains "no dependencies"

  then emits process_exit
    assert code == 0


depends on spec-grammar >= 1.1.0
depends on validate-command >= 2.0.0
depends on nfr-grammar >= 1.0.0
