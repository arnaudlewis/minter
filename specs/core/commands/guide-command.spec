spec guide-command v1.2.0
title "Guide Command"

description
  The minter guide command outputs targeted reference content for
  spec-driven development. It accepts a topic argument and returns
  the corresponding guide: workflow phases, spec authoring, requirements
  smells, NFR design, context management, or the full methodology.
  The output is designed for both human authors and AI agents.

motivation
  An agent consuming minter as a tool has access to format (grammar)
  and validate (verification), but no way to discover the methodology
  itself or get targeted guidance on specific topics. A human author
  needs quick access to reference material without reading the full
  methodology every time. The guide command makes both the methodology
  and topic-specific guidance machine-discoverable.

nfr
  operability#ci-friendly-output
  operability#zero-config


# Methodology topic (replaces explain)

behavior guide-methodology-prints-methodology [happy_path]
  "Print the complete spec-driven methodology to stdout and exit 0"

  given
    The guide subcommand is invoked with topic methodology

  when minter guide methodology

  then emits stdout
    assert output contains "spec"
    assert output contains "NFR"
    assert output contains "behavior"
    assert output contains "constraint"

  then emits process_exit
    assert code == 0


behavior guide-methodology-describes-spec-role [happy_path]
  "Explain that specs are the source of truth with behaviors as the atomic unit"

  given
    The guide subcommand is invoked with topic methodology

  when minter guide methodology

  then emits stdout
    assert output contains "source of truth"
    assert output contains "behavior"
    assert output contains "one behavior" or "1 behavior"
    assert output contains "one test" or "1 test"


behavior guide-methodology-describes-nfr-constraints [happy_path]
  "Explain NFR constraints as non-negotiable quality attributes"

  given
    The guide subcommand is invoked with topic methodology

  when minter guide methodology

  then emits stdout
    assert output contains "non-functional"
    assert output contains "constraint"
    assert output contains "metric"
    assert output contains "rule"


behavior guide-methodology-lists-nfr-categories [happy_path]
  "List all seven NFR categories"

  given
    The guide subcommand is invoked with topic methodology

  when minter guide methodology

  then emits stdout
    assert output contains "performance"
    assert output contains "reliability"
    assert output contains "security"
    assert output contains "observability"
    assert output contains "scalability"
    assert output contains "cost"
    assert output contains "operability"


behavior guide-methodology-describes-spec-level-binding [happy_path]
  "Explain that spec-level nfr sections bind constraints to all behaviors in the spec"

  given
    The guide subcommand is invoked with topic methodology

  when minter guide methodology

  then emits stdout
    assert output contains "spec-level"
    assert output contains "all behaviors"


behavior guide-methodology-describes-behavior-level-binding [happy_path]
  "Explain that behavior-level nfr sections pin specific constraints to one behavior"

  given
    The guide subcommand is invoked with topic methodology

  when minter guide methodology

  then emits stdout
    assert output contains "behavior-level"
    assert output contains "anchor"


behavior guide-methodology-describes-whole-file-vs-anchor [happy_path]
  "Explain the difference between whole-file and anchor references"

  given
    The guide subcommand is invoked with topic methodology

  when minter guide methodology

  then emits stdout
    assert output contains "whole-file"
    assert output contains "anchor"
    assert output contains "#"


behavior guide-methodology-describes-containment-rule [happy_path]
  "Explain that behavior-level categories must be declared at spec level"

  given
    The guide subcommand is invoked with topic methodology

  when minter guide methodology

  then emits stdout
    assert output contains "containment"
    assert output contains "spec-level"


behavior guide-methodology-describes-override-rules [happy_path]
  "Explain override mechanics: overridable, metric-only, same operator, stricter"

  given
    The guide subcommand is invoked with topic methodology

  when minter guide methodology

  then emits stdout
    assert output contains "override"
    assert output contains "stricter"
    assert output contains "overridable"
    assert output contains "metric"


behavior guide-methodology-describes-test-emission [happy_path]
  "Explain that NFR references emit test obligations"

  given
    The guide subcommand is invoked with topic methodology

  when minter guide methodology

  then emits stdout
    assert output contains "test"
    assert output contains "emit" or "generate"


behavior guide-methodology-shows-reference-syntax [happy_path]
  "Show the three forms of NFR reference syntax with examples"

  given
    The guide subcommand is invoked with topic methodology

  when minter guide methodology

  then emits stdout
    assert output contains "category"
    assert output contains "category#constraint"
    assert output contains "category#constraint operator value"


behavior guide-methodology-describes-workflow-phases [happy_path]
  "Describe the five-phase spec-driven workflow"

  given
    The guide subcommand is invoked with topic methodology

  when minter guide methodology

  then emits stdout
    assert output contains "Phase 1"
    assert output contains "Phase 2"
    assert output contains "Phase 3"
    assert output contains "Phase 4"
    assert output contains "Phase 5"
    assert output contains "spec" or "Spec"
    assert output contains "test" or "Test"


behavior guide-methodology-specs-before-code [happy_path]
  "Emphasize that specs are complete before any code is written"

  given
    The guide subcommand is invoked with topic methodology

  when minter guide methodology

  then emits stdout
    assert output contains "before"
    assert output contains "implementation" or "code"


behavior guide-methodology-red-tests [happy_path]
  "Explain that all tests must fail before implementation begins"

  given
    The guide subcommand is invoked with topic methodology

  when minter guide methodology

  then emits stdout
    assert output contains "fail"
    assert output contains "1 behavior = 1 test" or "one behavior = one test"


# Topic-specific guides

behavior guide-workflow-topic [happy_path]
  "Return the condensed workflow phases guide"

  given
    The guide subcommand is invoked with topic workflow

  when minter guide workflow

  then emits stdout
    assert output contains "Phase 1"
    assert output contains "Phase 5"

  then emits process_exit
    assert code == 0


behavior guide-authoring-topic [happy_path]
  "Return the condensed spec authoring guide"

  given
    The guide subcommand is invoked with topic authoring

  when minter guide authoring

  then emits stdout
    assert output contains "Right Granularity"
    assert output contains "Phase Collapse"
    assert output contains "Writing Specs from Code"
    assert output contains "Entity Format"

  then emits process_exit
    assert code == 0


behavior guide-smells-topic [happy_path]
  "Return the condensed requirements smell detection guide"

  given
    The guide subcommand is invoked with topic smells

  when minter guide smells

  then emits stdout
    assert output contains "Ambiguity"
    assert output contains "Observer Test"
    assert output contains "Swap Test"
    assert output contains "Phase Leak"

  then emits process_exit
    assert code == 0


behavior guide-nfr-topic [happy_path]
  "Return the condensed NFR design guide"

  given
    The guide subcommand is invoked with topic nfr

  when minter guide nfr

  then emits stdout
    assert output contains "Seven Fixed Categories"
    assert output contains "Decision Tree"

  then emits process_exit
    assert code == 0


behavior guide-context-topic [happy_path]
  "Return the condensed context management guide"

  given
    The guide subcommand is invoked with topic context

  when minter guide context

  then emits stdout
    assert output contains "Lazy Loading Sequence"

  then emits process_exit
    assert code == 0


behavior guide-coverage-topic [happy_path]
  "Return the condensed coverage tagging guide"

  given
    The guide subcommand is invoked with topic coverage

  when minter guide coverage

  then emits stdout
    assert output contains "Coverage Tagging"
    assert output contains "@minter"
    assert output contains "unit"
    assert output contains "e2e"
    assert output contains "benchmark"
    assert output contains "Qualified Names"
    assert output contains "Common Mistakes"

  then emits process_exit
    assert code == 0


# Error cases

behavior guide-unknown-topic [error_case]
  "Print error listing valid topics when given an unknown topic"

  given
    The guide subcommand is invoked with an unrecognized topic

  when minter guide banana

  then emits stderr
    assert output contains "banana"

  then emits process_exit
    assert code == 1


behavior guide-missing-topic [happy_path]
  "List available topics with descriptions when no topic argument is provided"

  given
    The guide subcommand is invoked with no arguments

  when minter guide

  then emits stdout
    assert output contains "workflow"
    assert output contains "authoring"
    assert output contains "smells"
    assert output contains "nfr"
    assert output contains "context"
    assert output contains "methodology"
    assert output contains "coverage"

  then emits process_exit
    assert code == 0


depends on nfr-grammar >= 1.0.0
depends on nfr-cross-reference >= 1.0.0
