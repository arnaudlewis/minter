spec mcp-response-format v1.0.0
title "MCP Response Format"

description
  The single source of truth for all MCP tool response structure: JSON
  field names, types, nesting, error representation, and content format.
  Every MCP tool behavior references this spec for response shape rules.
  This is the MCP equivalent of cli-display — where cli-display
  defines checkmarks, ANSI colors, and tree connectors for the terminal,
  this spec defines JSON schemas, field conventions, and error objects
  for agent consumers.

motivation
  Agents parsing MCP tool responses depend on a stable, predictable JSON
  structure. If field names drift between tools, or error representation
  varies, agents must special-case every tool. Centralizing all response
  format rules into one spec ensures consistent structure across validate,
  inspect, graph, and all other tools.

nfr
  operability#deterministic-output
  operability#json-response-schema


# Validate result structure

behavior validate-pass-result-fields [happy_path]
  "A passing spec result contains file, name, version, type, status, count, and empty errors"

  given
    A spec named my-feature at version 1.2.0 with 12 behaviors
    The spec passes all validation

  when format validation result for my-feature

  then returns tool_result
    assert results[0].file == "specs/my-feature.spec"
    assert results[0].name == "my-feature"
    assert results[0].version == "1.2.0"
    assert results[0].type == "spec"
    assert results[0].status == "pass"
    assert results[0].behavior_count == 12
    assert results[0].errors is empty


behavior validate-fail-result-fields [error_case]
  "A failing spec result contains status fail and a non-empty errors array"

  given
    A spec named broken-feature at version 2.0.0
    The spec fails validation with errors at lines 5 and 12

  when format validation result for broken-feature

  then returns tool_result
    assert results[0].name == "broken-feature"
    assert results[0].version == "2.0.0"
    assert results[0].status == "fail"
    assert results[0].errors is not empty


behavior validate-nfr-result-fields [happy_path]
  "An NFR result uses constraint_count instead of behavior_count and includes category"

  given
    An NFR file with category performance at version 1.0.0 with 4 constraints
    The NFR passes all validation

  when format validation result for performance

  then returns tool_result
    assert results[0].name == "performance"
    assert results[0].version == "1.0.0"
    assert results[0].type == "nfr"
    assert results[0].status == "pass"
    assert results[0].constraint_count == 4


behavior validate-summary-fields [happy_path]
  "Every validate response includes a summary with total, passed, and failed counts"

  given
    A directory containing 3 specs where 2 pass and 1 fails

  when format validation result for directory

  then returns tool_result
    assert summary.total == 3
    assert summary.passed == 2
    assert summary.failed == 1


behavior validate-deep-dependency-fields [happy_path]
  "Results in deep mode include a dependencies array per spec"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec has no dependencies
    Both specs are valid and deep mode is enabled

  when format validation result for a in deep mode

  then returns tool_result
    assert results for a contains dependencies array
    assert dependencies[0].name == "b"
    assert dependencies[0].constraint == ">= 1.0.0"
    assert results for b contains empty dependencies array


behavior validate-inline-result-omits-file [edge_case]
  "Results from inline content validation omit the file field"

  given
    Inline spec content is validated (no file path provided)

  when format validation result for inline content

  then returns tool_result
    assert results[0].file is not present
    assert results[0].name is_present
    assert results[0].status is_present


# Error object structure

behavior error-object-fields [error_case]
  "Each error object contains line, column when available, and message"

  given
    A spec with a parse error at line 5 column 3

  when format validation error

  then returns tool_result
    assert error.line == 5
    assert error.message is_present
    assert error message describes what is wrong


behavior error-object-includes-file-path [error_case]
  "Error objects for file-based validation include the file path"

  given
    specs/broken.spec has a validation error

  when format validation error for broken.spec

  then returns tool_result
    assert error.file == "specs/broken.spec"


# Inspect result structure

behavior inspect-spec-result-fields [happy_path]
  "An inspect result for a spec contains categories, dependencies, and assertion types"

  given
    specs/my-feature.spec has 4 happy_path, 2 error_case, 1 edge_case
    The spec depends on user-auth >= 1.0.0
    The spec uses assertion types equals and contains

  when format inspect result for my-feature

  then returns tool_result
    assert name == "my-feature"
    assert type == "spec"
    assert behavior_count == 7
    assert categories.happy_path == 4
    assert categories.error_case == 2
    assert categories.edge_case == 1
    assert dependencies[0].name == "user-auth"
    assert dependencies[0].constraint == ">= 1.0.0"
    assert assertion_types contains "equals"
    assert assertion_types contains "contains"


behavior inspect-nfr-result-fields [happy_path]
  "An inspect result for an NFR contains category, constraint count, and type distribution"

  given
    specs/performance.nfr has 3 metric and 1 rule constraints

  when format inspect result for performance

  then returns tool_result
    assert name == "performance"
    assert type == "nfr"
    assert category == "performance"
    assert constraint_count == 4
    assert types.metric == 3
    assert types.rule == 1


behavior inspect-no-dependencies-field [edge_case]
  "An inspect result for a spec with no dependencies has an empty array"

  given
    specs/standalone.spec has no depends on declarations

  when format inspect result for standalone

  then returns tool_result
    assert dependencies is empty


# Graph result structure

behavior graph-full-result-fields [happy_path]
  "A full graph result contains specs array and edges array"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec depends on c >= 1.0.0
    specs/c.spec has no dependencies

  when format graph result

  then returns tool_result
    assert specs contains entry with name "a" and file path and version
    assert specs contains entry with name "b" and file path and version
    assert specs contains entry with name "c" and file path and version
    assert edges contains entry with from "a" and to "b" and constraint ">= 1.0.0"
    assert edges contains entry with from "b" and to "c" and constraint ">= 1.0.0"


behavior graph-impacted-result-fields [happy_path]
  "An impacted graph result contains the target name and impacted specs array"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/c.spec depends on b >= 1.0.0

  when format graph impacted result for b

  then returns tool_result
    assert target == "b"
    assert impacted contains entry with name "a"
    assert impacted contains entry with name "c"


behavior graph-no-edges-result [edge_case]
  "A graph with no dependencies has specs but an empty edges array"

  given
    specs/a.spec has no dependencies
    specs/b.spec has no dependencies

  when format graph result

  then returns tool_result
    assert specs contains entry for a
    assert specs contains entry for b
    assert edges is empty


# Text content responses

behavior text-content-format [happy_path]
  "Scaffold, format, and guide tools return plain text content"

  given
    A tool that produces text output (scaffold, format, or guide)

  when format text result

  then returns tool_result
    assert content is a text string
    assert content is not wrapped in JSON structure
    assert content is directly usable as spec text or reference


# Tool error responses

behavior tool-error-structure [error_case]
  "Tool errors set isError true and include an actionable message"

  given
    A tool call that cannot execute (e.g. nonexistent file path)

  when format tool error

  then returns tool_result
    assert isError == true
    assert error message describes what went wrong
    assert error message includes the input that caused the failure


behavior tool-error-lists-valid-options [error_case]
  "Tool errors for invalid enum inputs list the valid options"

  given
    A tool call with an invalid type or category argument

  when format tool error for invalid enum

  then returns tool_result
    assert isError == true
    assert error message contains the invalid value provided
    assert error message lists the valid options


# Cross-cutting format rules

behavior no-ansi-in-responses [edge_case]
  "No tool response contains ANSI escape sequences or terminal control characters"

  given
    Any tool is called with valid input through the MCP server

  when format any tool result

  then returns tool_result
    assert content does not contain ANSI escape sequences
    assert content does not contain "\x1b[" or "\033["


behavior snake-case-field-names [edge_case]
  "All JSON field names use snake_case convention"

  given
    Any tool returns a structured JSON response

  when format any structured result

  then returns tool_result
    assert all field names use snake_case
    assert no field names use camelCase or PascalCase


behavior consistent-count-field-naming [edge_case]
  "Specs use behavior_count, NFRs use constraint_count — never mixed"

  given
    A validate or inspect result for a spec file
    A validate or inspect result for an NFR file

  when format results for both types

  then returns tool_result
    assert spec results use behavior_count field
    assert nfr results use constraint_count field
    assert spec results do not contain constraint_count
    assert nfr results do not contain behavior_count


depends on nfr-grammar >= 1.0.0
