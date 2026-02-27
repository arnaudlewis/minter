spec mcp-agent-guidance v1.2.0
title "MCP Agent Guidance"

description
  Defines the agent-only guidance layer of the minter MCP server. This
  layer shapes how AI agents practice spec-driven development through
  three mechanisms: an initialization tool that onboards agents into the
  methodology before any other tool call, a guide tool that provides
  condensed reference on workflow, spec authoring, requirements smells,
  and NFR design, and next_steps directives embedded in every tool
  response that enforce the correct phase sequence. Specs are the source
  of truth at all cost. Every behavior produces a red e2e test before
  implementation. Code is a derived artifact — never the starting point.

motivation
  Without explicit guidance delivered at tool-call time, agents treat
  minter as a bag of independent utilities — scaffolding specs without
  filling in behaviors, validating without writing tests, or jumping
  straight to implementation. The CLI serves humans who internalize the
  methodology over time. Agents need the methodology injected into every
  interaction. This spec ensures the MCP server actively enforces
  spec-driven development: the workflow is spec, then tests, then code
  — never out of order, never optional.

nfr
  operability#deterministic-output
  operability#mcp-protocol-compliance


# Initialization

behavior initialize-includes-methodology-instructions [happy_path]
  "The MCP handshake includes instructions that establish the spec-driven mindset"

  given
    An MCP host connects to the minter-mcp binary over stdio
    The host sends an initialize request

  when initialize

  then returns server_info
    assert instructions is_present
    assert instructions contains "specs are the source of truth"
    assert instructions contains workflow phase sequence
    assert instructions contains "call initialize_minter before using any other tool"


behavior initialize-minter-returns-full-methodology [happy_path]
  "The initialize_minter tool returns the complete development methodology and workflow"

  given
    The MCP server has been initialized
    The agent has not called any other minter tool

  when tools/call initialize_minter

  then returns tool_result
    assert content describes spec-driven development principles
    assert content contains the five workflow phases in order
    assert content contains "1 behavior = 1 test"
    assert content contains "TDD is mandatory"
    assert content contains "specs are the source of truth"


behavior initialize-minter-lists-available-tools [happy_path]
  "The methodology includes a summary of each tool and when to use it in the workflow"

  given
    The agent calls initialize_minter

  when tools/call initialize_minter

  then returns tool_result
    assert content lists validate tool with workflow context
    assert content lists scaffold tool with workflow context
    assert content lists format tool with workflow context
    assert content lists inspect tool with workflow context
    assert content lists graph tool with workflow context
    assert content lists initialize_minter tool with workflow context
    assert content lists guide tool with workflow context


# Guide tool

behavior guide-workflow-phases [happy_path]
  "Return the condensed development workflow phases as static reference"

  given
    The agent needs to understand the correct development sequence

  when tools/call guide
    topic = "workflow"

  then returns tool_result
    assert content describes phase 1 as write specs using scaffold and format
    assert content describes phase 2 as write documentation derived from specs
    assert content marks phase 2 as optional
    assert content describes phase 3 as write e2e tests that must be red
    assert content states 1 behavior equals 1 test
    assert content describes phase 4 as implement using TDD with mandatory unit tests
    assert content describes phase 5 as all tests green


behavior guide-spec-authoring [happy_path]
  "Return condensed spec authoring guidance covering granularity, decomposition, and NFR integration"

  given
    The agent needs to understand how to write well-formed specs

  when tools/call guide
    topic = "authoring"

  then returns tool_result
    assert content describes right granularity for a behavior
    assert content describes when to split a spec
    assert content describes when to merge behaviors
    assert content describes NFR referencing and when each category applies
    assert content describes classification of work types
    assert content describes too-coarse and too-fine signals
    assert content describes project type calibration


behavior guide-requirements-smells [happy_path]
  "Return condensed requirements smell detection reference"

  given
    The agent needs to identify weak or ambiguous requirements

  when tools/call guide
    topic = "smells"

  then returns tool_result
    assert content describes ambiguity smell
    assert content describes NASA forbidden words
    assert content describes compound behavior smell
    assert content describes implementation leakage smell
    assert each smell has signal and action


behavior guide-nfr-design [happy_path]
  "Return condensed NFR design reference covering categories, constraints, and referencing"

  given
    The agent needs to understand NFR structure and integration

  when tools/call guide
    topic = "nfr"

  then returns tool_result
    assert content lists seven NFR categories
    assert content describes metric and rule constraint types
    assert content describes three-level referencing
    assert content describes coverage gap detection by concern
    assert content describes FR/NFR classification axes
    assert content describes override rules


behavior guide-context-management [happy_path]
  "Return condensed context management protocol for lazy loading specs and NFRs"

  given
    The agent needs to understand how to manage context when working with specs

  when tools/call guide
    topic = "context"

  then returns tool_result
    assert content describes lazy loading sequence
    assert content describes guard rails for context management
    assert content describes scoping to subgraph
    assert content describes structure before content principle


behavior guide-unknown-topic [error_case]
  "Return error listing valid topics when given an unknown topic"

  given
    The agent calls guide with an unrecognized topic

  when tools/call guide
    topic = "banana"

  then returns tool_result
    assert isError == true
    assert error message contains "banana"
    assert error message lists "workflow" as valid topic
    assert error message lists "authoring" as valid topic
    assert error message lists "smells" as valid topic
    assert error message lists "nfr" as valid topic
    assert error message lists "context" as valid topic


# Workflow enforcement via next_steps

behavior next-steps-after-scaffold [happy_path]
  "Scaffold tool response includes next step to fill in behaviors and validate"

  given
    The agent calls scaffold with type spec

  when tools/call scaffold
    type = "spec"

  then returns tool_result
    assert content contains spec skeleton
    assert next_steps is_present
    assert next_steps contains "fill in behaviors for each user-observable outcome"
    assert next_steps contains "validate the spec with the validate tool"


behavior next-steps-after-validate-pass [happy_path]
  "Validate tool response on success includes next step to write red e2e tests"

  given
    specs/my-feature.spec is valid with 3 behaviors

  when tools/call validate
    path = "specs/my-feature.spec"

  then returns tool_result
    assert result status is pass
    assert next_steps is_present
    assert next_steps contains "write one e2e test per behavior"
    assert next_steps contains "tests must fail (red) before implementation"


behavior next-steps-after-validate-fail [happy_path]
  "Validate tool response on failure includes next step to fix errors"

  given
    specs/broken.spec has validation errors

  when tools/call validate
    path = "specs/broken.spec"

  then returns tool_result
    assert result status is fail
    assert next_steps is_present
    assert next_steps contains "fix the errors listed above"
    assert next_steps contains "re-validate"


behavior next-steps-after-format [happy_path]
  "Format tool response includes next step to use the grammar for spec authoring"

  given
    The agent calls format with type spec

  when tools/call format
    type = "spec"

  then returns tool_result
    assert content contains spec grammar
    assert next_steps is_present
    assert next_steps contains "use this grammar to write your spec"


behavior next-steps-after-inspect [happy_path]
  "Inspect tool response includes next step based on coverage analysis"

  given
    specs/my-feature.spec has 3 happy_path and 0 error_case behaviors

  when tools/call inspect
    path = "specs/my-feature.spec"

  then returns tool_result
    assert next_steps is_present
    assert next_steps contains "add error_case behaviors for each happy path"


behavior next-steps-after-nfr-scaffold [happy_path]
  "NFR scaffold response includes next step to define constraints and reference from specs"

  given
    The agent calls scaffold with type nfr and category performance

  when tools/call scaffold
    type = "nfr"
    category = "performance"

  then returns tool_result
    assert next_steps is_present
    assert next_steps contains "define metric or rule constraints"
    assert next_steps contains "reference from functional specs using nfr section"


# Tool listing

behavior list-tools-includes-agent-guidance [happy_path]
  "The tools/list response includes agent-only guidance tools"

  given
    The MCP server has been initialized

  when tools/list

  then returns tool_list
    assert tools contains tool named "initialize_minter"
    assert tools contains tool named "guide"
    assert initialize_minter description contains "MUST be called before"
    assert guide description contains "spec-driven development practices"


depends on mcp-server >= 1.3.0
depends on mcp-response-format >= 1.0.0
