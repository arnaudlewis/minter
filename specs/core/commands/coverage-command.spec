spec coverage-command v1.4.0
title "Coverage Command"

description
  The minter coverage command scans project files for @minter tags in
  comments and cross-references them against the spec graph to produce
  a behavior coverage report. Tags placed in test files declare which
  spec behaviors a test covers. The format is
  @minter:<type> <behavior>... for behavioral tests (any type name is
  accepted) and @minter:benchmark #<category>#<constraint>... for NFR
  benchmarks. The benchmark type is special — it maps to NFR constraints
  instead of behavior names. All other types are treated as behavioral
  tags. The command takes a spec path as its positional argument and
  walks the current working directory for tags by default. The --scan
  flag narrows the scan to specific directories. NFR coverage for
  behavioral tests is derived from the spec graph — if a covered
  behavior references an NFR constraint, that constraint has indirect
  coverage. Fully covered specs are collapsed to a single summary line
  by default; --verbose expands all specs to show individual behaviors.
  No configuration file is required.

motivation
  Specs are the source of truth. Tests are derived from specs. But
  today there is no backlink from tests to specs. There is no way to
  answer: which behaviors are covered by tests? Which are not? What
  type of tests cover them? The coverage command closes this loop with
  lightweight tags and a read-only report against the spec graph.


nfr
  operability#ci-friendly-output
  operability#deterministic-output
  operability#zero-config
  reliability#error-completeness


# Tag scanning — happy path

behavior report-full-coverage [happy_path]
  "Report 100% coverage and exit 0 when every behavior has at least one test"

  given
    specs/a.spec has 2 behaviors: do-thing and do-other
    A file contains // @minter:unit do-thing
    Another file contains // @minter:e2e do-other

  when minter coverage specs/

  then emits stdout
    assert output contains "2/2"
    assert output contains "100"

  then emits process_exit
    assert code == 0


behavior report-partial-coverage [happy_path]
  "Report uncovered behaviors and exit 1 when some behaviors lack tests"

  given
    specs/a.spec has 3 behaviors: do-thing, do-other, and do-missing
    A file contains // @minter:unit do-thing
    Another file contains // @minter:e2e do-other
    No file references do-missing

  when minter coverage specs/

  then emits stdout
    assert output contains "do-thing"
    assert output contains "do-other"
    assert output contains "do-missing"
    assert output contains "uncovered"
    assert output contains "2/3"

  then emits process_exit
    assert code == 1


behavior group-by-spec [happy_path]
  "Display coverage results grouped by spec name and version"

  given
    specs/a.spec v1.0.0 has behavior do-thing
    specs/b.spec v2.0.0 has behavior do-other
    Both behaviors are covered by test tags

  when minter coverage specs/

  then emits stdout
    assert output contains "a v1.0.0"
    assert output contains "b v2.0.0"

  then emits process_exit
    assert code == 0


behavior show-test-types [happy_path]
  "Display which test types cover each behavior"

  given
    specs/a.spec has behavior do-thing
    One file contains // @minter:unit do-thing
    Another file contains // @minter:e2e do-thing

  when minter coverage specs/

  then emits stdout
    assert output contains "unit"
    assert output contains "e2e"


behavior show-summary [happy_path]
  "Display a summary section with total, covered count, percentage, and per-type counts"

  given
    specs/a.spec has 3 behaviors
    2 behaviors are covered (one by unit, one by unit and e2e)
    1 behavior is uncovered

  when minter coverage specs/

  then emits stdout
    assert output contains "2/3"
    assert output contains "66"
    assert output contains "unit"
    assert output contains "e2e"


behavior multiple-ids-in-one-tag [happy_path]
  "A single tag can reference multiple behavior IDs"

  given
    specs/a.spec has behaviors do-thing and do-other
    A file contains // @minter:e2e do-thing do-other

  when minter coverage specs/

  then emits stdout
    assert output contains "2/2"

  then emits process_exit
    assert code == 0


behavior scan-double-slash-comments [happy_path]
  "Detect @minter tags in // style comments"

  given
    specs/a.spec has behavior do-thing
    A .ts file contains // @minter:unit do-thing

  when minter coverage specs/

  then emits stdout
    assert output contains "unit"


behavior scan-hash-comments [happy_path]
  "Detect @minter tags in # style comments"

  given
    specs/a.spec has behavior do-thing
    A .py file contains # @minter:unit do-thing

  when minter coverage specs/

  then emits stdout
    assert output contains "unit"


# Scan scoping

behavior scope-scan-with-flag [happy_path]
  "Scan only the specified directories when --scan is provided"

  given
    specs/a.spec has behavior do-thing
    tests/unit/a.test.ts contains // @minter:unit do-thing
    tests/e2e/a.spec.ts contains // @minter:e2e do-thing

  when minter coverage specs/ --scan tests/unit/

  then emits stdout
    assert output contains "unit"
    assert output does not contain "e2e"


behavior multiple-scan-flags [happy_path]
  "Accept multiple --scan flags to scan several directories"

  given
    specs/a.spec has behavior do-thing
    src/a.rs contains // @minter:unit do-thing
    benches/a.rs contains // @minter:benchmark #performance#api-latency
    nfr/performance.nfr exists with constraint api-latency

  when minter coverage specs/ --scan src/ --scan benches/

  then emits stdout
    assert output contains "unit"
    assert output contains "benchmark"


behavior single-spec-file [happy_path]
  "Report coverage for a single spec when a file path is given"

  given
    specs/a.spec has 2 behaviors: do-thing and do-other
    specs/b.spec has 1 behavior: unrelated
    A file contains // @minter:unit do-thing

  when minter coverage specs/a.spec

  then emits stdout
    assert output contains "do-thing"
    assert output contains "do-other"
    assert output does not contain "unrelated"


behavior skip-gitignored-paths [happy_path]
  "Respect .gitignore and skip ignored directories during scan"

  given
    specs/a.spec has behavior do-thing
    .gitignore contains node_modules/
    node_modules/dep/test.js contains // @minter:unit do-thing
    tests/a.test.ts contains // @minter:e2e do-thing

  when minter coverage specs/

  then emits stdout
    assert output contains "e2e"
    assert output does not contain "unit"


# NFR derived coverage

behavior derive-nfr-from-covered-behavior [happy_path]
  "Show NFR coverage as derived when its linked behavior is covered"

  given
    specs/a.spec has behavior do-thing with nfr performance#api-latency
    A file contains // @minter:e2e do-thing
    nfr/performance.nfr exists with constraint api-latency

  when minter coverage specs/

  then emits stdout
    assert output contains "performance#api-latency"
    assert output contains "derived"


behavior derive-nfr-uncovered-from-uncovered-behavior [happy_path]
  "Report NFR constraint as uncovered when its linked behavior is uncovered"

  given
    specs/a.spec has behavior do-thing with nfr performance#api-latency
    No file references do-thing

  when minter coverage specs/

  then emits stdout
    assert output contains "performance#api-latency"
    assert output contains "uncovered"


# Benchmark NFR coverage

behavior report-benchmark-nfr [happy_path]
  "Display direct NFR coverage from benchmark tags"

  given
    specs/a.spec has 1 behavior: do-thing
    A file contains // @minter:benchmark #performance#api-latency
    nfr/performance.nfr exists with constraint api-latency

  when minter coverage specs/

  then emits stdout
    assert output contains "performance#api-latency"
    assert output contains "benchmark"


# Compact display

behavior collapse-fully-covered-spec [happy_path]
  "Display fully covered specs as a single line with count and aggregated test types"

  given
    specs/a.spec has 2 behaviors: do-thing and do-other
    A file contains // @minter:unit do-thing
    Another file contains // @minter:e2e do-other

  when minter coverage specs/

  then emits stdout
    assert output contains "Behavior Coverage"
    assert output contains "a v1.0.0"
    assert output contains "2/2"
    assert output contains "unit"
    assert output contains "e2e"
    assert output does not contain "do-thing"
    assert output does not contain "do-other"


behavior expand-partially-covered-spec [happy_path]
  "Expand specs with uncovered behaviors to show individual behavior lines"

  given
    specs/a.spec has 3 behaviors: do-thing, do-other, and do-missing
    A file contains // @minter:unit do-thing
    Another file contains // @minter:e2e do-other
    No file references do-missing

  when minter coverage specs/

  then emits stdout
    assert output contains "do-thing"
    assert output contains "do-other"
    assert output contains "do-missing"
    assert output contains "uncovered"


behavior verbose-expands-all [happy_path]
  "Expand all specs when --verbose is provided, even when fully covered"

  given
    specs/a.spec has 2 behaviors: do-thing and do-other
    A file contains // @minter:unit do-thing
    Another file contains // @minter:e2e do-other

  when minter coverage specs/ --verbose

  then emits stdout
    assert output contains "do-thing"
    assert output contains "do-other"
    assert output contains "2/2"

  then emits process_exit
    assert code == 0


# JSON output

behavior json-output [happy_path]
  "Produce machine-readable JSON when --format json is specified"

  given
    specs/a.spec has 2 behaviors: do-thing and do-other
    A file contains // @minter:unit do-thing
    No file references do-other

  when minter coverage specs/ --format json

  then emits stdout
    assert output contains "total_behaviors"
    assert output contains "covered_behaviors"
    assert output contains "coverage_percentage"
    assert output contains "do-thing"
    assert output contains "do-other"
    assert output contains "uncovered"

  then emits process_exit
    assert code == 1


# Tag validation — error cases

behavior reject-unknown-behavior-id [error_case]
  "Report error when a tag references a behavior that does not exist in any spec"

  given
    specs/a.spec has behavior do-thing
    A file contains // @minter:unit nonexistent-behavior

  when minter coverage specs/

  then emits stderr
    assert output contains "nonexistent-behavior"
    assert output contains "unknown"

  then emits process_exit
    assert code == 1


behavior reject-unknown-nfr-constraint [error_case]
  "Report error when a benchmark tag references an NFR constraint that does not exist"

  given
    specs/a.spec has 1 behavior: do-thing
    nfr/performance.nfr exists with constraint api-latency
    A file contains // @minter:benchmark #performance#nonexistent

  when minter coverage specs/

  then emits stderr
    assert output contains "nonexistent"
    assert output contains "unknown"

  then emits process_exit
    assert code == 1


behavior reject-missing-type [error_case]
  "Report error when a tag has @minter without a type suffix"

  given
    specs/a.spec has 1 behavior: do-thing
    A file contains // @minter do-thing

  when minter coverage specs/

  then emits stderr
    assert output contains "@minter"
    assert output contains "type"

  then emits process_exit
    assert code == 1


behavior reject-behavior-in-benchmark [error_case]
  "Report error when a benchmark tag contains behavior IDs instead of NFR refs"

  given
    specs/a.spec has behavior do-thing
    A file contains // @minter:benchmark do-thing

  when minter coverage specs/

  then emits stderr
    assert output contains "do-thing"
    assert output contains "benchmark"

  then emits process_exit
    assert code == 1


behavior reject-nfr-in-behavioral-tag [error_case]
  "Report error when a non-benchmark tag contains NFR refs"

  given
    specs/a.spec has 1 behavior: do-thing
    A file contains // @minter:unit #performance#api-latency

  when minter coverage specs/

  then emits stderr
    assert output contains "performance"
    assert output contains "benchmark"

  then emits process_exit
    assert code == 1


behavior reject-nonexistent-spec-path [error_case]
  "Print error when the spec path does not exist"

  given
    The directory nonexistent/ does not exist on disk

  when minter coverage nonexistent/

  then emits stderr
    assert output contains "nonexistent"

  then emits process_exit
    assert code == 1


behavior reject-no-specs-in-path [error_case]
  "Print error when the spec path contains no .spec files"

  given
    An empty directory with no .spec files

  when minter coverage empty-dir/

  then emits stderr
    assert output contains "no spec files found"

  then emits process_exit
    assert code == 1


behavior reject-nonexistent-scan-path [error_case]
  "Print error when a --scan directory does not exist"

  given
    specs/a.spec has 1 behavior: do-thing
    The directory nonexistent/ does not exist on disk

  when minter coverage specs/ --scan nonexistent/

  then emits stderr
    assert output contains "nonexistent"

  then emits process_exit
    assert code == 1


behavior reject-invalid-format [error_case]
  "Print error when --format value is not a recognized output format"

  given
    specs/a.spec has 1 behavior: do-thing

  when minter coverage specs/ --format xml

  then emits stderr
    assert output contains "xml"
    assert output contains "invalid"

  then emits process_exit
    assert code == 1


behavior report-tag-errors-with-location [error_case]
  "Include file path and line number in tag validation errors"

  given
    specs/a.spec has 1 behavior: do-thing
    tests/a.test.ts line 5 contains // @minter:unit nonexistent-behavior

  when minter coverage specs/

  then emits stderr
    assert output contains "a.test.ts"
    assert output contains "5"

  then emits process_exit
    assert code == 1


# Edge cases

behavior warn-empty-tag [edge_case]
  "Emit a warning but still produce the coverage report when a tag has no IDs"

  given
    specs/a.spec has behavior do-thing
    A file contains // @minter:unit do-thing
    Another file contains // @minter:e2e

  when minter coverage specs/

  then emits stderr
    assert output contains "empty"

  then emits stdout
    assert output contains "1/1"

  then emits process_exit
    assert code == 0


behavior info-duplicate-coverage [edge_case]
  "Report duplicate count when the same behavior is covered by multiple tests of the same type"

  given
    specs/a.spec has behavior do-thing
    tests/a.test.ts contains // @minter:unit do-thing
    tests/b.test.ts contains // @minter:unit do-thing

  when minter coverage specs/ --verbose

  then emits stdout
    assert output contains "do-thing"
    assert output contains "unit"
    assert output contains "x2"
    assert output contains "duplicate"

  then emits process_exit
    assert code == 0


behavior no-tags-found [edge_case]
  "Report all behaviors as uncovered when no @minter tags exist in any file"

  given
    specs/a.spec has 2 behaviors: do-thing and do-other
    No file in the project contains any @minter tags

  when minter coverage specs/

  then emits stdout
    assert output contains "do-thing"
    assert output contains "do-other"
    assert output contains "uncovered"
    assert output contains "0/2"

  then emits process_exit
    assert code == 1


behavior disambiguate-with-qualified-name [edge_case]
  "Accept spec-name/behavior-name to disambiguate when two specs share a behavior name"

  given
    specs/a.spec has behavior handle-error
    specs/b.spec has behavior handle-error
    A file contains // @minter:unit a/handle-error

  when minter coverage specs/

  then emits stdout
    assert output contains "a"
    assert output contains "handle-error"
    assert output contains "unit"


behavior reject-ambiguous-unqualified-name [edge_case]
  "Report error when an unqualified behavior name matches behaviors in multiple specs"

  given
    specs/a.spec has behavior handle-error
    specs/b.spec has behavior handle-error
    A file contains // @minter:unit handle-error

  when minter coverage specs/

  then emits stderr
    assert output contains "handle-error"
    assert output contains "ambiguous"

  then emits process_exit
    assert code == 1


behavior report-all-tag-errors [edge_case]
  "Report all tag validation errors at once, not just the first"

  given
    specs/a.spec has 1 behavior: do-thing
    File a.test.ts line 3 contains // @minter:unit nonexistent-one
    File b.test.ts line 7 contains // @minter:unit nonexistent-two

  when minter coverage specs/

  then emits stderr
    assert output contains "nonexistent-one"
    assert output contains "nonexistent-two"
    assert output contains "a.test.ts"
    assert output contains "b.test.ts"


behavior json-errors [edge_case]
  "Produce JSON error output when --format json encounters tag errors"

  given
    specs/a.spec has 1 behavior: do-thing
    A file contains // @minter:unit nonexistent-behavior

  when minter coverage specs/ --format json

  then emits stdout
    assert output contains "errors"
    assert output contains "nonexistent-behavior"

  then emits process_exit
    assert code == 1


behavior mixed-valid-and-invalid-tags [edge_case]
  "Report tag errors on stderr and produce no coverage report when any tag is invalid"

  given
    specs/a.spec has 2 behaviors: do-thing and do-other
    A file contains // @minter:unit do-thing
    Another file contains // @minter:unit nonexistent-behavior

  when minter coverage specs/

  then emits stderr
    assert output contains "nonexistent-behavior"

  then emits process_exit
    assert code == 1


behavior accept-arbitrary-tag-type [happy_path]
  "Accept any tag type, not just a predefined list"

  given
    specs/a.spec has behavior do-thing
    A file contains // @minter:acceptance do-thing

  when minter coverage specs/

  then emits stdout
    assert output contains "acceptance"
    assert output contains "1/1"

  then emits process_exit
    assert code == 0


behavior accept-multiple-custom-types [happy_path]
  "Accept multiple custom tag types and report each"

  given
    specs/a.spec has behaviors do-thing and do-other
    A file contains // @minter:smoke do-thing
    Another file contains // @minter:property do-other

  when minter coverage specs/

  then emits stdout
    assert output contains "smoke"
    assert output contains "property"
    assert output contains "2/2"

  then emits process_exit
    assert code == 0


behavior accept-uppercase-tag-type [edge_case]
  "Accept tag types with uppercase characters"

  given
    specs/a.spec has behavior do-thing
    A file contains // @minter:SMOKE do-thing

  when minter coverage specs/

  then emits stdout
    assert output contains "SMOKE"

  then emits process_exit
    assert code == 0


behavior accept-single-char-tag-type [edge_case]
  "Accept single character tag types"

  given
    specs/a.spec has behavior do-thing
    A file contains // @minter:a do-thing

  when minter coverage specs/

  then emits stdout
    assert output contains "1/1"

  then emits process_exit
    assert code == 0


depends on spec-grammar >= 1.1.0
depends on nfr-grammar >= 1.0.0
depends on cli-display >= 2.0.0
