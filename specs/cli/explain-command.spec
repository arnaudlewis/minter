spec explain-command v1.1.0
title "Explain Command"

description
  The minter explain command outputs the spec-driven development
  methodology as a single structured reference document. It describes
  what specs are, what NFRs are, how they connect through cross-references,
  what this means for test generation, and the five-phase workflow from
  spec authoring through TDD to green tests. The output is designed
  for both human authors and AI agents consuming minter through MCP.

motivation
  An agent consuming minter as a tool has access to format (grammar)
  and validate (verification), but no way to discover the methodology
  itself — the rules governing how specs and NFRs interact, why NFR
  constraints are non-negotiable, or how cross-references drive test
  generation. A human author needs the workflow phases to know when
  specs end and implementation begins. The explain command makes both
  the methodology and the workflow machine-discoverable.

nfr
  operability#ci-friendly-output
  operability#zero-config


# Core output

behavior explain-prints-methodology [happy_path]
  "Print the complete spec-driven methodology to stdout and exit 0"

  given
    The explain subcommand is invoked with no arguments

  when minter explain

  then emits stdout
    assert output contains "spec"
    assert output contains "NFR"
    assert output contains "behavior"
    assert output contains "constraint"

  then emits process_exit
    assert code == 0


# Spec role

behavior explain-describes-spec-role [happy_path]
  "Explain that specs are the source of truth with behaviors as the atomic unit"

  given
    The explain subcommand is invoked

  when minter explain

  then emits stdout
    assert output contains "source of truth"
    assert output contains "behavior"
    assert output contains "one behavior" or "1 behavior"
    assert output contains "one test" or "1 test"


# NFR role

behavior explain-describes-nfr-constraints [happy_path]
  "Explain NFR constraints as non-negotiable quality attributes"

  given
    The explain subcommand is invoked

  when minter explain

  then emits stdout
    assert output contains "non-functional"
    assert output contains "constraint"
    assert output contains "metric"
    assert output contains "rule"


behavior explain-lists-nfr-categories [happy_path]
  "List all seven NFR categories"

  given
    The explain subcommand is invoked

  when minter explain

  then emits stdout
    assert output contains "performance"
    assert output contains "reliability"
    assert output contains "security"
    assert output contains "observability"
    assert output contains "scalability"
    assert output contains "cost"
    assert output contains "operability"


# Cross-reference binding

behavior explain-describes-spec-level-binding [happy_path]
  "Explain that spec-level nfr sections bind constraints to all behaviors in the spec"

  given
    The explain subcommand is invoked

  when minter explain

  then emits stdout
    assert output contains "spec-level"
    assert output contains "all behaviors"


behavior explain-describes-behavior-level-binding [happy_path]
  "Explain that behavior-level nfr sections pin specific constraints to one behavior"

  given
    The explain subcommand is invoked

  when minter explain

  then emits stdout
    assert output contains "behavior-level"
    assert output contains "anchor"


behavior explain-describes-whole-file-vs-anchor [happy_path]
  "Explain the difference between whole-file and anchor references"

  given
    The explain subcommand is invoked

  when minter explain

  then emits stdout
    assert output contains "whole-file"
    assert output contains "anchor"
    assert output contains "#"


# Validation rules

behavior explain-describes-containment-rule [happy_path]
  "Explain that behavior-level categories must be declared at spec level"

  given
    The explain subcommand is invoked

  when minter explain

  then emits stdout
    assert output contains "containment"
    assert output contains "spec-level"


behavior explain-describes-override-rules [happy_path]
  "Explain override mechanics: overridable, metric-only, same operator, stricter"

  given
    The explain subcommand is invoked

  when minter explain

  then emits stdout
    assert output contains "override"
    assert output contains "stricter"
    assert output contains "overridable"
    assert output contains "metric"


# Test generation

behavior explain-describes-test-emission [happy_path]
  "Explain that NFR references emit test obligations"

  given
    The explain subcommand is invoked

  when minter explain

  then emits stdout
    assert output contains "test"
    assert output contains "emit" or "generate"


# Reference syntax

behavior explain-shows-reference-syntax [happy_path]
  "Show the three forms of NFR reference syntax with examples"

  given
    The explain subcommand is invoked

  when minter explain

  then emits stdout
    assert output contains "category"
    assert output contains "category#constraint"
    assert output contains "category#constraint operator value"


# Workflow

behavior explain-describes-workflow-phases [happy_path]
  "Describe the five-phase spec-driven workflow: specs, docs, red tests, implement, green"

  given
    The explain subcommand is invoked

  when minter explain

  then emits stdout
    assert output contains "Phase 1"
    assert output contains "Phase 2"
    assert output contains "Phase 3"
    assert output contains "Phase 4"
    assert output contains "Phase 5"
    assert output contains "spec" or "Spec"
    assert output contains "test" or "Test"


behavior explain-workflow-specs-before-code [happy_path]
  "Emphasize that specs are complete before any code is written"

  given
    The explain subcommand is invoked

  when minter explain

  then emits stdout
    assert output contains "before"
    assert output contains "implementation" or "code"


behavior explain-workflow-red-tests [happy_path]
  "Explain that all tests must fail before implementation begins"

  given
    The explain subcommand is invoked

  when minter explain

  then emits stdout
    assert output contains "fail"
    assert output contains "1 behavior = 1 test" or "one behavior = one test"


depends on nfr-grammar >= 1.0.0
depends on nfr-cross-reference >= 1.0.0
