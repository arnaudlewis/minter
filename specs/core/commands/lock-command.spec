spec lock-command v1.0.0
title "Lock Command"

description
  The minter lock command produces a minter.lock file at the project
  root. The lock captures the full integrity snapshot of the project:
  SHA-256 hashes for every spec, NFR, and test file, the dependency
  graph structure, and the behavior-to-test traceability matrix built
  from @minter tags. The lock reuses the graph cache for spec and NFR
  hashes and the coverage tag scanner for test mapping. It reads
  project paths from minter.config.json or falls back to conventions
  (specs/, tests/). The lock file is the source of truth for minter ci.

motivation
  In spec-driven development, specs and tests must not be modified
  silently during implementation. The lock file freezes the approved
  state of specs, tests, and their relationships. CI compares the
  current state against the lock to detect tampering, drift, or
  coverage gaps. The lock is a human-only command — agents implement
  against it but never regenerate it.

nfr
  reliability#no-silent-data-loss
  operability#ci-friendly-output
  operability#deterministic-output

# Lock generation — happy path

behavior generate-lock-file [happy_path]
  "Generate minter.lock at the project root with spec, NFR, and test hashes"

  given
    specs/a.spec has 2 behaviors: do-thing and do-other
    specs/nfr/performance.nfr exists with 1 constraint
    tests/a_test.rs contains // @minter:unit do-thing
    tests/b_test.rs contains // @minter:e2e do-other

  when minter lock

  then emits file minter.lock
    assert file is valid JSON
    assert file contains "version"
    assert file contains spec entry for specs/a.spec with hash
    assert file contains nfr entry for specs/nfr/performance.nfr with hash
    assert file contains test_files entries with hashes
    assert file contains covers arrays mapping behaviors to tests

  then emits stdout
    assert output contains "lock"

  then emits process_exit
    assert code == 0


behavior lock-contains-spec-hashes [happy_path]
  "Each spec entry in the lock contains a SHA-256 content hash"

  given
    specs/a.spec exists with known content

  when minter lock

  then emits file minter.lock
    assert specs/a.spec entry has a hash field
    assert hash is a SHA-256 hex string


behavior lock-contains-behaviors [happy_path]
  "Each spec entry lists all behavior names"

  given
    specs/a.spec has 3 behaviors: do-thing, do-other, do-more

  when minter lock

  then emits file minter.lock
    assert specs/a.spec entry has behaviors array
    assert behaviors contains "do-thing"
    assert behaviors contains "do-other"
    assert behaviors contains "do-more"


behavior lock-contains-dependencies [happy_path]
  "Each spec entry lists its dependency edges"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec exists with version 1.0.0

  when minter lock

  then emits file minter.lock
    assert specs/a.spec entry has dependencies array
    assert dependencies contains "specs/b.spec"


behavior lock-contains-nfr-refs [happy_path]
  "Each spec entry lists its NFR references"

  given
    specs/a.spec references performance#api-latency
    specs/nfr/performance.nfr exists

  when minter lock

  then emits file minter.lock
    assert specs/a.spec entry has nfrs array
    assert nfrs contains "performance#api-latency"


behavior lock-contains-nfr-hashes [happy_path]
  "Each NFR file has its own entry with a SHA-256 content hash"

  given
    specs/nfr/performance.nfr exists with known content

  when minter lock

  then emits file minter.lock
    assert nfrs section contains specs/nfr/performance.nfr
    assert entry has hash field


behavior lock-contains-test-file-hashes [happy_path]
  "Each test file mapped to a spec has a SHA-256 content hash"

  given
    specs/a.spec has behavior do-thing
    tests/a_test.rs contains // @minter:unit do-thing

  when minter lock

  then emits file minter.lock
    assert specs/a.spec test_files contains tests/a_test.rs
    assert tests/a_test.rs entry has hash field
    assert hash is a SHA-256 hex string


behavior lock-contains-covers-mapping [happy_path]
  "Each test file entry lists which behaviors it covers"

  given
    specs/a.spec has behaviors do-thing and do-other
    tests/a_test.rs contains // @minter:unit do-thing
    tests/a_test.rs contains // @minter:e2e do-other

  when minter lock

  then emits file minter.lock
    assert tests/a_test.rs covers array contains "do-thing"
    assert tests/a_test.rs covers array contains "do-other"


behavior lock-reads-config [happy_path]
  "Lock reads specs and test paths from minter.config.json"

  given
    minter.config.json contains: { "specs": "specifications/", "tests": ["src/tests/"] }
    specifications/a.spec has behavior do-thing
    src/tests/a_test.rs contains // @minter:unit do-thing

  when minter lock

  then emits file minter.lock
    assert file contains specifications/a.spec
    assert file contains src/tests/a_test.rs

  then emits process_exit
    assert code == 0


behavior lock-uses-default-conventions [happy_path]
  "Lock uses specs/ and tests/ when no config exists"

  given
    No minter.config.json exists
    specs/a.spec has behavior do-thing
    tests/a_test.rs contains // @minter:unit do-thing

  when minter lock

  then emits file minter.lock
    assert file contains specs/a.spec
    assert file contains tests/a_test.rs


behavior lock-deterministic [happy_path]
  "Running lock twice on unchanged files produces identical output"

  given
    specs/a.spec and tests/a_test.rs exist and are unchanged

  when minter lock is run twice

  then
    assert both minter.lock files are byte-identical


behavior lock-updates-existing [happy_path]
  "Overwrite existing minter.lock with fresh snapshot"

  given
    minter.lock already exists from a previous run
    specs/a.spec has been modified since last lock

  when minter lock

  then emits file minter.lock
    assert file reflects the current state of specs/a.spec
    assert hash differs from previous lock


behavior lock-multiple-specs [happy_path]
  "Lock includes all specs in the configured directory"

  given
    specs/a.spec has 2 behaviors
    specs/sub/b.spec has 3 behaviors
    specs/sub/deep/c.spec has 1 behavior

  when minter lock

  then emits file minter.lock
    assert file contains specs/a.spec
    assert file contains specs/sub/b.spec
    assert file contains specs/sub/deep/c.spec


behavior lock-test-covers-multiple-specs [happy_path]
  "A test file covering behaviors from multiple specs appears under each"

  given
    specs/a.spec has behavior do-thing
    specs/b.spec has behavior do-other
    tests/shared_test.rs contains // @minter:e2e do-thing
    tests/shared_test.rs contains // @minter:e2e do-other

  when minter lock

  then emits file minter.lock
    assert specs/a.spec test_files contains tests/shared_test.rs with covers ["do-thing"]
    assert specs/b.spec test_files contains tests/shared_test.rs with covers ["do-other"]


# Error cases

behavior reject-invalid-specs [error_case]
  "Fail when any spec file has validation errors"

  given
    specs/broken.spec has parse errors

  when minter lock

  then emits stderr
    assert output contains "broken"
    assert output contains validation error

  then emits process_exit
    assert code == 1


behavior reject-tag-errors [error_case]
  "Fail when test tags reference nonexistent behaviors"

  given
    specs/a.spec has behavior do-thing
    tests/a_test.rs contains // @minter:unit nonexistent-behavior

  when minter lock

  then emits stderr
    assert output contains "nonexistent-behavior"

  then emits process_exit
    assert code == 1


behavior reject-no-specs-found [error_case]
  "Fail when no spec files are found in configured paths"

  given
    specs/ directory exists but contains no .spec files

  when minter lock

  then emits stderr
    assert output contains "no spec files found"

  then emits process_exit
    assert code == 1


# Edge cases

behavior lock-no-tests [edge_case]
  "Generate lock with empty test_files when no tests exist"

  given
    specs/a.spec has behavior do-thing
    No test files contain @minter tags

  when minter lock

  then emits file minter.lock
    assert specs/a.spec entry has empty test_files

  then emits process_exit
    assert code == 0


behavior lock-no-nfrs [edge_case]
  "Generate lock with empty nfrs section when no NFR files exist"

  given
    specs/a.spec has behavior do-thing with no NFR references
    No .nfr files exist

  when minter lock

  then emits file minter.lock
    assert nfrs section is empty

  then emits process_exit
    assert code == 0


behavior lock-atomic-write [edge_case]
  "Write lock via temp file and rename to prevent corruption"

  given
    specs/a.spec exists

  when minter lock

  then
    assert minter.lock is written atomically (temp file + rename)
    assert no partial minter.lock is observable on crash


behavior lock-scans-all-test-dirs [happy_path]
  "Lock scans all test directories from config, including benchmark dirs"

  given
    minter.config.json contains: { "specs": "specs/", "tests": ["tests/", "benches/"] }
    specs/a.spec has behavior do-thing
    tests/a_test.rs contains // @minter:unit do-thing
    benches/perf_test.rs contains // @minter:benchmark #performance#api-latency

  when minter lock

  then emits file minter.lock
    assert file contains tests/a_test.rs
    assert file contains benches/perf_test.rs in benchmark_files section

  then emits process_exit
    assert code == 0


depends on config >= 1.0.0
depends on coverage-command >= 1.3.0
depends on graph-command >= 1.4.0
