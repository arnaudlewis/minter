spec mcp-server v1.3.0
title "MCP Server"

description
  The minter MCP server exposes minter capabilities as tools over the
  Model Context Protocol using stdio transport. It enables AI agents
  and MCP-compatible hosts to validate specs, inspect metadata, generate
  scaffolds, retrieve grammar references, query dependency graphs, and
  discover the spec-driven methodology. The server shares the same core
  logic as the CLI but delegates response formatting to mcp-response-format.
  Distributed as a second binary (minter-mcp) in the same crate, so
  cargo install minter provides both the CLI and the MCP server.

motivation
  The CLI is designed for human authors in a terminal. AI agents
  consuming minter through MCP need structured JSON responses, not
  colored text with exit codes. Exposing the same core capabilities
  through MCP enables agents to integrate minter into their workflows
  — validating specs they author, discovering the methodology, and
  querying the dependency graph — without parsing terminal output.
  The MCP server makes minter a first-class tool in any agent
  toolchain that supports the Model Context Protocol.

nfr
  operability#deterministic-output
  operability#mcp-protocol-compliance
  operability#input-schema-accuracy
  security#max-file-size
  security#max-inline-content-size
  security#file-extension-validation
  security#permission-denied-handling
  performance#cache-skip-unchanged


# Server lifecycle

behavior initialize-server [happy_path]
  "Return server identity and capabilities on MCP initialize handshake"

  given
    An MCP host connects to the minter-mcp binary over stdio
    The host sends an initialize request with a supported protocol version

  when initialize

  then returns server_info
    assert name == "minter"
    assert version matches_pattern "^\\d+\\.\\d+\\.\\d+"
    assert capabilities contains tools

behavior list-tools [happy_path]
  "Return all seven tool definitions with descriptions and input schemas"

  given
    The MCP server has been initialized

  when tools/list

  then returns tool_list
    assert tool_count == 7
    assert tools contains tool named "validate"
    assert tools contains tool named "inspect"
    assert tools contains tool named "scaffold"
    assert tools contains tool named "format"
    assert tools contains tool named "graph"
    assert tools contains tool named "initialize_minter"
    assert tools contains tool named "guide"
    assert each tool has a description
    assert each tool has an inputSchema


# Validate tool

behavior validate-file-pass [happy_path]
  "Return pass result when a valid spec file is validated"

  given
    specs/my-feature.spec is valid with 5 behaviors at version 1.0.0

  when tools/call validate
    path = "specs/my-feature.spec"

  then returns tool_result
    assert result contains entry for my-feature
    assert result status is pass

behavior validate-file-fail [happy_path]
  "Return fail result with error details when a spec has validation errors"

  given
    specs/broken.spec has validation errors at lines 5 and 12

  when tools/call validate
    path = "specs/broken.spec"

  then returns tool_result
    assert result contains entry for broken
    assert result status is fail
    assert result errors is not empty

behavior validate-directory [happy_path]
  "Return results for all discovered spec and nfr files in a directory"

  given
    A directory containing:
    specs/a.spec (valid)
    specs/sub/b.spec (valid)
    specs/performance.nfr (valid)

  when tools/call validate
    path = "specs/"

  then returns tool_result
    assert result contains entry for a
    assert result contains entry for b
    assert result contains entry for performance

behavior validate-deep-mode [happy_path]
  "Resolve the dependency graph when deep mode is enabled"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec exists with version 1.2.0 and is valid

  when tools/call validate
    path = "specs/a.spec"
    deep = true

  then returns tool_result
    assert result contains entry for a
    assert result contains entry for b

behavior validate-nfr-file [happy_path]
  "Return result with NFR metadata when an nfr file is validated"

  given
    specs/performance.nfr is valid with 4 constraints

  when tools/call validate
    path = "specs/performance.nfr"

  then returns tool_result
    assert result contains entry for performance
    assert result status is pass

behavior validate-inline-content [happy_path]
  "Validate spec content passed directly without requiring a file on disk"

  given
    No file path is provided
    Content is a valid spec string

  when tools/call validate
    content = "spec my-inline v1.0.0\ntitle \"Inline\"\n..."
    content_type = "spec"

  then returns tool_result
    assert result contains entry for my-inline
    assert result status is pass

behavior validate-nonexistent-path [error_case]
  "Return error when the specified path does not exist"

  given
    The path nonexistent.spec does not exist on disk

  when tools/call validate
    path = "nonexistent.spec"

  then returns tool_result
    assert isError == true
    assert error message contains "nonexistent.spec"

behavior validate-mixed-results [edge_case]
  "Return both pass and fail results in the same response"

  given
    specs/valid.spec passes validation
    specs/broken.spec fails validation

  when tools/call validate
    path = "specs/"

  then returns tool_result
    assert result contains entry for valid with status pass
    assert result contains entry for broken with status fail

behavior validate-reject-non-spec-extension [error_case]
  "Return error when a file does not have a .spec or .nfr extension"

  nfr
    security#file-extension-validation

  given
    A file path that does not end in .spec or .nfr

  when tools/call validate
    path = "readme.md"

  then returns tool_result
    assert isError == true
    assert error message contains ".spec"
    assert error message contains ".nfr"

behavior validate-reject-oversized-file [error_case]
  "Return error when the file exceeds the maximum size limit"

  nfr
    security#max-file-size

  given
    A .spec file that is larger than 10MB

  when tools/call validate
    path = "huge.spec"

  then returns tool_result
    assert isError == true
    assert error message contains "10MB"

behavior validate-reject-oversized-content [error_case]
  "Return error when inline content exceeds the maximum size limit"

  nfr
    security#max-inline-content-size

  given
    An inline content string larger than 10MB

  when tools/call validate
    content = "<10MB+ string>"
    content_type = "spec"

  then returns tool_result
    assert isError == true
    assert error message contains "10MB"

behavior validate-reject-unreadable-file [error_case]
  "Return error with clean message when file permissions deny read access"

  nfr
    security#permission-denied-handling

  given
    A .spec file exists but is not readable due to file permissions

  when tools/call validate
    path = "unreadable.spec"

  then returns tool_result
    assert isError == true
    assert error message contains "permission"
    assert error message contains "unreadable.spec"

behavior validate-content-takes-precedence [edge_case]
  "When both path and content are provided, content is used and path is ignored"

  given
    specs/a.spec exists on disk with name a
    Inline content defines a different spec named inline-override

  when tools/call validate
    path = "specs/a.spec"
    content = "spec inline-override v1.0.0\ntitle \"Override\"\n..."
    content_type = "spec"

  then returns tool_result
    assert result contains entry for inline-override
    assert result does not contain entry for a

behavior validate-reject-unknown-content-type [error_case]
  "Return error when the content_type is not spec or nfr"

  given
    Inline content is provided with an unrecognized content_type

  when tools/call validate
    content = "some content"
    content_type = "banana"

  then returns tool_result
    assert isError == true
    assert error message contains "banana"

behavior validate-require-path-or-content [error_case]
  "Return error when neither path nor content is provided"

  given
    No path and no content parameters are provided

  when tools/call validate

  then returns tool_result
    assert isError == true
    assert error message contains "path"
    assert error message contains "content"


# Inspect tool

behavior inspect-spec-file [happy_path]
  "Return metadata for a spec file including categories and dependencies"

  given
    specs/my-feature.spec is valid with 7 behaviors and 2 dependencies

  when tools/call inspect
    path = "specs/my-feature.spec"

  then returns tool_result
    assert result contains metadata for my-feature
    assert result contains category distribution
    assert result contains dependency list

behavior inspect-nfr-file [happy_path]
  "Return metadata for an NFR file including constraint type distribution"

  given
    specs/performance.nfr is valid with 3 metric and 1 rule constraints

  when tools/call inspect
    path = "specs/performance.nfr"

  then returns tool_result
    assert result contains metadata for performance
    assert result contains type distribution

behavior inspect-inline-content [happy_path]
  "Return metadata for spec content passed directly without a file"

  given
    No file path is provided
    Content is a valid spec string with 3 behaviors

  when tools/call inspect
    content = "spec my-inline v1.0.0\n..."
    content_type = "spec"

  then returns tool_result
    assert result contains metadata for my-inline

behavior inspect-invalid-file [error_case]
  "Return validation errors when the inspected file is invalid"

  given
    specs/broken.spec has parse or semantic errors

  when tools/call inspect
    path = "specs/broken.spec"

  then returns tool_result
    assert isError == true
    assert result contains validation errors

behavior inspect-nonexistent-file [error_case]
  "Return error when the file does not exist"

  given
    The file specs/missing.spec does not exist on disk

  when tools/call inspect
    path = "specs/missing.spec"

  then returns tool_result
    assert isError == true
    assert error message contains "missing.spec"

behavior inspect-reject-non-spec-extension [error_case]
  "Return error when the file does not have a .spec or .nfr extension"

  nfr
    security#file-extension-validation

  given
    A file path that does not end in .spec or .nfr

  when tools/call inspect
    path = "config.yaml"

  then returns tool_result
    assert isError == true
    assert error message contains ".spec"
    assert error message contains ".nfr"

behavior inspect-reject-oversized-file [error_case]
  "Return error when the file exceeds the maximum size limit"

  nfr
    security#max-file-size

  given
    A .spec file that is larger than 10MB

  when tools/call inspect
    path = "huge.spec"

  then returns tool_result
    assert isError == true
    assert error message contains "10MB"


# Scaffold tool

behavior scaffold-spec-template [happy_path]
  "Return a functional requirement skeleton as text content"

  given
    The scaffold tool is called with type spec

  when tools/call scaffold
    type = "spec"

  then returns tool_result
    assert content contains spec skeleton with all required sections

behavior scaffold-nfr-template [happy_path]
  "Return an NFR skeleton for the specified category as text content"

  given
    The scaffold tool is called with type nfr and category performance

  when tools/call scaffold
    type = "nfr"
    category = "performance"

  then returns tool_result
    assert content contains NFR skeleton for performance

behavior scaffold-unknown-category [error_case]
  "Return error listing valid categories when given an unknown NFR category"

  given
    The scaffold tool is called with an unrecognized category

  when tools/call scaffold
    type = "nfr"
    category = "banana"

  then returns tool_result
    assert isError == true
    assert error message contains "banana"
    assert error message lists valid categories

behavior scaffold-nfr-missing-category [error_case]
  "Return error when type is nfr but no category is provided"

  given
    The scaffold tool is called with type nfr and no category

  when tools/call scaffold
    type = "nfr"

  then returns tool_result
    assert isError == true
    assert error message contains "category"

behavior scaffold-unknown-type [error_case]
  "Return error when the scaffold type is not spec or nfr"

  given
    The scaffold tool is called with an unrecognized type

  when tools/call scaffold
    type = "banana"

  then returns tool_result
    assert isError == true
    assert error message contains "banana"


# Format tool

behavior format-spec-grammar [happy_path]
  "Return the functional requirement DSL grammar as text content"

  given
    The format tool is called with type spec

  when tools/call format
    type = "spec"

  then returns tool_result
    assert content contains spec grammar reference

behavior format-nfr-grammar [happy_path]
  "Return the non-functional requirement DSL grammar as text content"

  given
    The format tool is called with type nfr

  when tools/call format
    type = "nfr"

  then returns tool_result
    assert content contains NFR grammar reference

behavior format-unknown-type [error_case]
  "Return error listing valid types when given an unknown format type"

  given
    The format tool is called with an unrecognized type

  when tools/call format
    type = "banana"

  then returns tool_result
    assert isError == true
    assert error message contains "banana"
    assert error message lists valid types


# Graph tool

behavior graph-full-dependencies [happy_path]
  "Return all specs and dependency edges as structured data"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec depends on c >= 1.0.0
    specs/c.spec has no dependencies

  when tools/call graph
    path = "specs/"

  then returns tool_result
    assert result contains specs a, b, and c
    assert result contains edges from a to b and b to c

behavior graph-impacted-specs [happy_path]
  "Return reverse dependencies of a named spec"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/c.spec depends on b >= 1.0.0

  when tools/call graph
    path = "specs/"
    impacted = "b"

  then returns tool_result
    assert result contains a and c as impacted by b

behavior graph-impacted-transitive [happy_path]
  "Return transitive reverse dependencies when a spec is depended on indirectly"

  given
    specs/a.spec depends on b >= 1.0.0
    specs/b.spec depends on c >= 1.0.0
    specs/c.spec has no dependencies

  when tools/call graph
    path = "specs/"
    impacted = "c"

  then returns tool_result
    assert result contains b and a as impacted by c

behavior graph-unknown-spec [error_case]
  "Return error when the named spec is not found in the graph"

  given
    No spec named nonexistent exists in the spec tree

  when tools/call graph
    path = "specs/"
    impacted = "nonexistent"

  then returns tool_result
    assert isError == true
    assert error message contains "nonexistent"

behavior graph-empty-directory [error_case]
  "Return error when no spec files are found in the directory"

  given
    An empty directory with no .spec files

  when tools/call graph
    path = "empty-dir/"

  then returns tool_result
    assert isError == true
    assert error message contains "no spec files found"

behavior graph-nonexistent-directory [error_case]
  "Return error when the graph directory does not exist"

  given
    The directory path does not exist on disk

  when tools/call graph
    path = "nonexistent-dir/"

  then returns tool_result
    assert isError == true
    assert error message contains "nonexistent-dir"


depends on validate-command >= 2.1.0
depends on inspect-command >= 1.1.0
depends on scaffold-command >= 1.1.0
depends on format-command >= 1.1.0
depends on graph-command >= 1.3.0
depends on mcp-response-format >= 1.0.0
