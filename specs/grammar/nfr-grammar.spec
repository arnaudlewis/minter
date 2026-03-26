spec nfr-grammar v1.1.0
title "NFR Grammar"

description
  Defines the grammar and structure of the .nfr DSL. Every keyword,
  section, block structure, operator, and syntax rule that the parser
  must recognize and enforce for non-functional requirement files.

motivation
  The NFR DSL is the authoring interface for non-functional requirement
  specs. A strict, well-defined format ensures consistency across all
  NFR files and enables reliable parsing. This spec is the contract for
  what constitutes a well-formed .nfr file.

# Header section

behavior parse-nfr-declaration [happy_path]
  "Parse the nfr declaration with category and version"

  given
    A .nfr file starting with: nfr performance v1.0.0

  when parse

  then
    assert category == "performance"
    assert version == "1.0.0"


behavior parse-nfr-category-validated [happy_path]
  "Accept all seven valid NFR categories"

  given
    A .nfr file with category set to each of: performance, reliability,
    security, observability, scalability, cost, operability

  when parse

  then
    assert category is one of the 7 valid values


behavior reject-invalid-nfr-category [error_case]
  "Reject a category not in the fixed list"

  given
    A .nfr file starting with: nfr banana v1.0.0

  when parse

  then emits stderr
    assert output contains "banana"
    assert output contains "category"

  then emits process_exit
    assert code == 1


behavior parse-nfr-title [happy_path]
  "Parse the title as a quoted string"

  given
    A .nfr file with: title "Performance Requirements"

  when parse

  then
    assert title == "Performance Requirements"


behavior parse-nfr-description-block [happy_path]
  "Parse a multiline indented description block"

  given
    A .nfr file with:
    description
      This is line one.
      This is line two.

  when parse

  then
    assert description == "This is line one. This is line two."


behavior parse-nfr-motivation-block [happy_path]
  "Parse a multiline indented motivation block"

  given
    A .nfr file with:
    motivation
      Reason line one.
      Reason line two.

  when parse

  then
    assert motivation == "Reason line one. Reason line two."


behavior reject-nfr-missing-version [error_case]
  "Reject an nfr declaration with no version"

  given
    A .nfr file starting with: nfr performance

  when parse

  then emits stderr
    assert output contains "version"

  then emits process_exit
    assert code == 1


# Constraint declaration

behavior parse-metric-constraint-declaration [happy_path]
  "Parse a metric constraint with name and type"

  given
    A .nfr file containing:
    constraint api-response-time [metric]
      "API endpoints must respond within acceptable latency bounds"

  when parse

  then
    assert constraint name == "api-response-time"
    assert constraint type == "metric"
    assert constraint description == "API endpoints must respond within acceptable latency bounds"


behavior parse-rule-constraint-declaration [happy_path]
  "Parse a rule constraint with name and type"

  given
    A .nfr file containing:
    constraint no-n-plus-one [rule]
      "No endpoint may issue unbounded database calls"

  when parse

  then
    assert constraint name == "no-n-plus-one"
    assert constraint type == "rule"
    assert constraint description == "No endpoint may issue unbounded database calls"


behavior reject-unknown-constraint-type [error_case]
  "Reject a constraint with a type other than metric or rule"

  given
    A .nfr file containing:
    constraint my-constraint [banana]
      "Some description"

  when parse

  then emits stderr
    assert output contains "banana"
    assert output contains "metric" or "rule"

  then emits process_exit
    assert code == 1


behavior reject-constraint-without-description [error_case]
  "Reject a constraint that has no quoted description string"

  given
    A .nfr file containing:
    constraint my-constraint [metric]
    metric "Something"

  when parse

  then emits stderr
    assert output contains line number
    assert output mentions missing constraint description

  then emits process_exit
    assert code == 1


behavior reject-non-kebab-constraint-name [error_case]
  "Reject a constraint name that is not kebab-case"

  given
    A .nfr file containing:
    constraint MyConstraint [metric]
      "Some description"

  when parse

  then emits stderr
    assert output contains "MyConstraint"
    assert output contains "kebab-case"

  then emits process_exit
    assert code == 1


# Metric fields

behavior parse-metric-field [happy_path]
  "Parse the metric field as a quoted string"

  given
    A metric constraint with: metric "HTTP response time, p95"

  when parse

  then
    assert metric == "HTTP response time, p95"


behavior parse-threshold-less-than [happy_path]
  "Parse a threshold with the less-than operator"

  given
    A metric constraint with: threshold < 500ms

  when parse

  then
    assert threshold operator == "<"
    assert threshold value == "500ms"


behavior parse-threshold-greater-than [happy_path]
  "Parse a threshold with the greater-than operator"

  given
    A metric constraint with: threshold > 100

  when parse

  then
    assert threshold operator == ">"
    assert threshold value == "100"


behavior parse-threshold-less-or-equal [happy_path]
  "Parse a threshold with the less-or-equal operator"

  given
    A metric constraint with: threshold <= 99.9%

  when parse

  then
    assert threshold operator == "<="
    assert threshold value == "99.9%"


behavior parse-threshold-greater-or-equal [happy_path]
  "Parse a threshold with the greater-or-equal operator"

  given
    A metric constraint with: threshold >= 99.9%

  when parse

  then
    assert threshold operator == ">="
    assert threshold value == "99.9%"


behavior parse-threshold-equals [happy_path]
  "Parse a threshold with the equals operator"

  given
    A metric constraint with: threshold == 100%

  when parse

  then
    assert threshold operator == "=="
    assert threshold value == "100%"


behavior reject-threshold-invalid-operator [error_case]
  "Reject a threshold with an invalid operator"

  given
    A metric constraint with: threshold != 500ms

  when parse

  then emits stderr
    assert output contains "!="
    assert output contains "operator"

  then emits process_exit
    assert code == 1


behavior parse-metric-verification-block [happy_path]
  "Parse a metric verification block with all required fields"

  given
    A metric constraint with:
    verification
      environment staging, production
      benchmark "100 concurrent requests per endpoint"
      dataset "Production-representative volume"
      pass "p95 < threshold"

  when parse

  then
    assert verification environment == ["staging", "production"]
    assert verification benchmark == "100 concurrent requests per endpoint"
    assert verification dataset == "Production-representative volume"
    assert verification pass == "p95 < threshold"


behavior parse-metric-verification-without-dataset [happy_path]
  "Parse a metric verification block without the optional dataset field"

  given
    A metric constraint with:
    verification
      environment all
      benchmark "Assert response Content-Length on representative queries"
      pass "No response exceeds threshold"

  when parse

  then
    assert verification environment == ["all"]
    assert verification benchmark is_present
    assert verification dataset is not present
    assert verification pass is_present


# Rule fields

behavior parse-rule-text-block [happy_path]
  "Parse a rule text block with multiline content"

  given
    A rule constraint with:
    rule
      No endpoint may issue more than a fixed number of database
      or service calls regardless of result set size.

  when parse

  then
    assert rule text contains "No endpoint may issue more than a fixed number"
    assert rule text contains "or service calls regardless of result set size."


behavior parse-rule-verification-static-only [happy_path]
  "Parse a rule verification block with only static checks"

  given
    A rule constraint with:
    verification
      static "Query count per request path does not scale with input size"

  when parse

  then
    assert verification static == "Query count per request path does not scale with input size"
    assert verification runtime is not present


behavior parse-rule-verification-runtime-only [happy_path]
  "Parse a rule verification block with only runtime checks"

  given
    A rule constraint with:
    verification
      runtime "Call every endpoint without auth header, assert 401"

  when parse

  then
    assert verification runtime == "Call every endpoint without auth header, assert 401"
    assert verification static is not present


behavior parse-rule-verification-both [happy_path]
  "Parse a rule verification block with both static and runtime checks"

  given
    A rule constraint with:
    verification
      static "Every database query includes tenant filter"
      runtime "Authenticate as tenant A, attempt to access tenant B data"

  when parse

  then
    assert verification static is_present
    assert verification runtime is_present


behavior reject-rule-verification-empty [error_case]
  "Reject a rule verification block with neither static nor runtime"

  given
    A rule constraint with:
    verification

  when parse

  then emits stderr
    assert output contains "verification"
    assert output mentions missing static or runtime

  then emits process_exit
    assert code == 1


# Shared fields

behavior parse-violation-severity [happy_path]
  "Parse all four valid violation severity levels"

  given
    Constraints with violation set to each of: critical, high, medium, low

  when parse

  then
    assert violation severity is one of the 4 valid values


behavior reject-invalid-violation-severity [error_case]
  "Reject a violation severity not in the valid list"

  given
    A constraint with: violation banana

  when parse

  then emits stderr
    assert output contains "banana"
    assert output contains "violation"

  then emits process_exit
    assert code == 1


behavior parse-overridable-values [happy_path]
  "Parse both valid overridable values"

  given
    Constraints with overridable set to: yes and no

  when parse

  then
    assert overridable is one of yes or no


behavior reject-invalid-overridable-value [error_case]
  "Reject an overridable value not yes or no"

  given
    A constraint with: overridable maybe

  when parse

  then emits stderr
    assert output contains "maybe"
    assert output contains "overridable"

  then emits process_exit
    assert code == 1


# Structural errors

behavior reject-missing-nfr-declaration [error_case]
  "Reject a file that does not start with an nfr declaration"

  given
    A .nfr file that begins with: title "My NFR"
    without a preceding nfr declaration line

  when parse

  then emits stderr
    assert output contains line number
    assert output mentions missing nfr declaration

  then emits process_exit
    assert code == 1


behavior reject-nfr-missing-title [error_case]
  "Reject an NFR file that has no title line"

  given
    A .nfr file with an nfr declaration and description but no title line

  when parse

  then emits stderr
    assert output mentions missing title

  then emits process_exit
    assert code == 1


behavior reject-nfr-missing-description [error_case]
  "Reject an NFR file that has no description block"

  given
    A .nfr file with an nfr declaration and title but no description block

  when parse

  then emits stderr
    assert output mentions missing description

  then emits process_exit
    assert code == 1


behavior reject-nfr-missing-motivation [error_case]
  "Reject an NFR file that has no motivation block"

  given
    A .nfr file with an nfr declaration, title, and description but no
    motivation block

  when parse

  then emits stderr
    assert output mentions missing motivation

  then emits process_exit
    assert code == 1


behavior reject-nfr-no-constraints [error_case]
  "Reject an NFR file with no constraint declarations"

  given
    A .nfr file with a valid header but no constraint blocks

  when parse

  then emits stderr
    assert output mentions missing constraint

  then emits process_exit
    assert code == 1


behavior reject-duplicate-constraint-names [error_case]
  "Reject an NFR file with duplicate constraint names"

  given
    A .nfr file containing two constraints both named api-response-time

  when parse

  then emits stderr
    assert output contains "api-response-time"
    assert output contains "duplicate"

  then emits process_exit
    assert code == 1


behavior reject-metric-missing-required-fields [error_case]
  "Reject a metric constraint missing metric, threshold, or verification"

  given
    A metric constraint with the metric field but no threshold and no
    verification block

  when parse

  then emits stderr
    assert output mentions missing required fields

  then emits process_exit
    assert code == 1


behavior reject-metric-verification-missing-required [error_case]
  "Reject a metric verification block missing environment, benchmark, or pass"

  given
    A metric verification block with environment but no benchmark and no pass

  when parse

  then emits stderr
    assert output mentions missing verification fields

  then emits process_exit
    assert code == 1


behavior reject-rule-missing-required-fields [error_case]
  "Reject a rule constraint missing rule text or verification"

  given
    A rule constraint with the rule text but no verification block

  when parse

  then emits stderr
    assert output mentions missing required fields

  then emits process_exit
    assert code == 1


behavior reject-nfr-tab-indentation [error_case]
  "Reject tabs as indentation characters in NFR files"

  given
    A .nfr file where a constraint description is indented with a tab character

  when parse

  then emits stderr
    assert output contains line number
    assert output mentions invalid indentation

  then emits process_exit
    assert code == 1


# Edge cases

behavior parse-multiple-constraints [edge_case]
  "Parse an NFR file with multiple constraints of mixed types"

  given
    A .nfr file with three constraints: two metric and one rule

  when parse

  then
    assert constraint count == 3
    assert constraint 1 type == "metric"
    assert constraint 2 type == "metric"
    assert constraint 3 type == "rule"


behavior parse-multiple-verification-lines [edge_case]
  "Parse multiple benchmark, dataset, or pass lines in a metric verification"

  given
    A metric constraint with verification containing:
    verification
      environment staging, production
      benchmark "Load test at 100 RPS"
      benchmark "Spike test at 500 RPS"
      dataset "Production-representative volume"
      pass "p95 < threshold at 100 RPS"
      pass "No errors above 1% at 500 RPS"

  when parse

  then
    assert verification benchmark count == 2
    assert verification pass count == 2


behavior parse-environment-all [edge_case]
  "Parse environment all as a valid environment value"

  given
    A metric constraint with: environment all

  when parse

  then
    assert verification environment == ["all"]


behavior ignore-nfr-comments [happy_path]
  "Lines starting with # are treated as comments and ignored"

  given
    A .nfr file with:
    nfr performance v1.0.0
    # This is a comment
    title "Performance Requirements"

  when parse

  then
    assert category == "performance"
    assert title == "Performance Requirements"


behavior ignore-nfr-blank-lines [happy_path]
  "Blank lines between sections and constraints are ignored"

  given
    A .nfr file with three blank lines between the title and description

  when parse

  then
    assert title is_present
    assert description is_present


behavior reject-trailing-content [error_case]
  "Reject non-whitespace content after the last valid constraint"

  given
    An NFR file with valid structure followed by unexpected text after the last constraint

  when minter validate is run

  then
    assert validation fails with an error indicating unexpected trailing content
    assert the error includes the line number where the trailing content starts
