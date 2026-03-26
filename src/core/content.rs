/// Spec-driven development methodology reference text.
pub fn methodology() -> &'static str {
    "\
Spec-Driven Development Methodology
====================================

Specs are the source of truth for every feature. Each spec declares
behaviors — the atomic unit of work. 1 behavior maps to 1 test. Every
behavior belongs to a category: happy_path, error_case, or edge_case.


Non-Functional Requirements (NFRs)
-----------------------------------

NFRs define non-functional quality attributes as constraints. Each
constraint is either a metric (measurable threshold) or a rule
(binary pass/fail policy).

There are seven NFR categories:
  performance, reliability, security, observability,
  scalability, cost, operability


Cross-Reference Binding
-----------------------

Specs reference NFR constraints via an `nfr` section. There are two
binding levels:

  spec-level    — applies the constraint to all behaviors in the spec
  behavior-level — pins a specific anchor constraint to one behavior

References come in three forms:

  category                           whole-file reference
  category#constraint                anchor reference
  category#constraint operator value override (behavior-level only)


Whole-File vs Anchor References
-------------------------------

A whole-file reference (just `category`) imports every constraint from
the corresponding .nfr file. An anchor reference (`category#constraint`)
targets a single named constraint via the `#` anchor syntax.


Containment Rule
----------------

Every category referenced at behavior-level must also appear in the
spec-level `nfr` section. This containment rule ensures spec-level
declarations act as a table of contents for the NFR categories in scope.


Override Rules
--------------

A behavior-level reference may override the default threshold of a
metric constraint. Overrides are only allowed when:

  - The constraint is marked `overridable yes`
  - The constraint is a metric (not a rule)
  - The override operator matches the original threshold operator
  - The override value is stricter than the default


Test Generation
---------------

Each NFR reference in a spec emits a test obligation. The validate
command checks that all references resolve to real constraints and
that overrides satisfy the rules above. Test runners generate one
test per behavior per bound constraint.


Workflow
--------

The spec-driven workflow has five phases. Complete each phase
before moving to the next.

Phase 1 — Write Specs
  Use scaffold, format, and validate to author specs.
  Scaffold a skeleton, fill in behaviors, validate until green.
  The spec is complete before any implementation begins.

Phase 2 — Write Documentation (optional)
  Derive docs from the spec. Skip if not needed.

Phase 3 — Write Red Tests
  1 behavior = 1 test. Write all tests first. Every test must fail
  before any implementation begins. Tag every test with @minter:<type>
  to link it to its spec behavior. Run minter coverage to verify
  completeness. See: minter guide coverage

Phase 4 — Implement (TDD)
  Make one test green at a time. Do not skip ahead.

Phase 5 — All Green
  All tests pass. Run a final validate to confirm spec compliance."
}

/// Functional requirement (FR) DSL grammar reference text.
pub fn fr_grammar() -> &'static str {
    "\
Functional Requirement (FR) Spec Grammar
=========================================

spec <name> v<version>
title \"<title>\"

description
  <free text>

motivation
  <free text>

nfr
  <category>                              whole-file reference
  <category>#<constraint>                 anchor reference

behavior <name> [<category>]
  \"<description>\"

  nfr
    <category>#<constraint>                         anchor reference
    <category>#<constraint> <operator> <value>       override (stricter)

  given
    <precondition text>
    @<alias> = <Entity> {{ <key>: <value>, ... }}

  when <action-name>
    <input> = <value>
    <input> = @<alias>.<field>

  then returns <channel>
    assert <field> == <value>
    assert <field> contains <value>
    assert <field> is_present
    assert <field> in_range <min>..<max>
    assert <field> matches_pattern <pattern>
    assert <field> >= <value>

  then emits <channel>
    assert <field> == <value>

  then side_effect
    assert <prose description>

depends on <spec-name> >= <version>

Categories: happy_path, error_case, edge_case"
}

/// Non-functional requirement (NFR) DSL grammar reference text.
pub fn nfr_grammar() -> &'static str {
    "\
Non-Functional Requirement (NFR) Spec Grammar
===============================================

nfr <category> v<version>
title \"<title>\"

description
  <free text>

motivation
  <free text>


constraint <name> [metric]
  \"<description>\"

  metric \"<what is measured>\"
  threshold <operator> <value>

  verification
    environment <env1>, <env2>, ...
    benchmark \"<procedure>\"
    dataset \"<data requirements>\"
    pass \"<criteria>\"

  violation <critical|high|medium|low>
  overridable <yes|no>


constraint <name> [rule]
  \"<description>\"

  rule
    <free text>

  verification
    static \"<check description>\"
    runtime \"<check description>\"

  violation <critical|high|medium|low>
  overridable <yes|no>

Categories: performance, reliability, security, observability, scalability, cost, operability
Threshold operators: <, >, <=, >=, =="
}

/// Functional requirement (FR) scaffold template text.
pub fn fr_scaffold() -> &'static str {
    "\
spec my-feature v0.1.0
title \"My Feature\"

description
  Describe what this feature does.

motivation
  Explain why this feature is needed.

nfr
  performance

behavior do-something [happy_path]
  \"The system does something successfully\"

  given
    The system is in a valid state

  when perform-action

  then returns result
    assert status == \"success\"
"
}

/// Non-functional requirement (NFR) scaffold template text with category interpolation.
pub fn nfr_scaffold(category: &str) -> String {
    let title = crate::model::capitalize(category);
    format!(
        "\
nfr {category} v0.1.0
title \"{title} Requirements\"

description
  Describe the {category} requirements.

motivation
  Explain why these {category} requirements matter.


constraint example-constraint [metric]
  \"Describe what this constraint measures\"

  metric \"The metric being measured\"
  threshold < 1s

  verification
    environment all
    benchmark \"Describe the benchmark procedure\"
    pass \"Describe what constitutes passing\"

  violation medium
  overridable yes
"
    )
}

/// Agent onboarding: spec-driven development principles, workflow phases, and tool summary.
pub fn initialize_minter() -> &'static str {
    "\
Spec-Driven Development — Agent Onboarding
============================================

Principles
----------

1. Specs are the source of truth — never code.
2. TDD is mandatory — every behavior gets a failing test before implementation.
3. 1 behavior = 1 test — no more, no less.
4. NFRs are constraints, not guidelines — they are enforced, not suggested.
5. The spec is complete before any code is written.


Workflow Phases
---------------

Phase 1 — Write Specs
  Use `scaffold` to generate a skeleton, `format` to review the grammar.
  Fill in every behavior with given/when/then. Add NFR references.
  Validate with `validate`. Do not proceed until the spec passes.

Phase 2 — Write Documentation (optional)
  Derive user-facing documentation from the spec.

Phase 3 — Write Red Tests
  Write one e2e test per behavior. Tests MUST fail (red).
  Tag every test with @minter:<type> to link it to its spec behavior.
  Run `coverage` to verify completeness. See: `guide` coverage topic.
  Do not write implementation code yet.

Phase 4 — Implement (TDD)
  Write the minimum code to make one test pass at a time.
  Unit tests are mandatory alongside implementation.

Phase 5 — All Green
  All e2e and unit tests pass. Run `validate` one final time.


Available Tools
---------------

  validate    — Check a spec or directory for parse/semantic errors.
                Use after writing or editing a spec.
  scaffold    — Generate a blank spec or NFR skeleton.
                Use at the start of Phase 1 to bootstrap a new spec.
  format      — Return the spec or NFR DSL grammar reference.
                Use when unsure about syntax during authoring.
  inspect     — Return metadata and coverage analysis for a spec.
                Use to check behavior category distribution.
  graph       — Query the dependency graph across specs.
                Use to understand impact of changes.
  initialize_minter — This tool. Call it first before using any other tool.
  guide       — Condensed reference on workflow, authoring, smells, NFR design,
                or context management.
                Use when you need a quick refresher mid-workflow."
}

/// Condensed workflow phase reference for the guide tool.
pub fn guide_workflow() -> &'static str {
    "\
Workflow Phases
===============

Phase 1 — Write Specs
  Tools: scaffold, format, validate
  Scaffold a skeleton → fill in behaviors → validate until green.

Phase 2 — Write Documentation (optional)
  Derive docs from the spec. Skip if not needed.

Phase 3 — Write Red Tests
  1 behavior = 1 test. All tests must fail before implementation.
  Tag every test with @minter:<type>. Run minter coverage to verify.
  See: minter guide coverage

Phase 4 — Implement (TDD)
  Make one test green at a time. Unit tests are mandatory.

Phase 5 — All Green
  All tests pass. Final validate."
}

/// Condensed spec authoring guidance for the guide tool.
pub fn guide_authoring() -> &'static str {
    "\
Spec Authoring Guide
====================

Right Granularity
  A behavior describes one user-observable outcome. If the description
  needs \"and\", split it into two behaviors. Aim for 1–5 assertions per
  postcondition block. Each behavior should be self-contained and have
  clear pass/fail criteria.

Too Coarse — split when:
  - Description contains multiple verbs joined by \"and\"
  - Given section has more than 5 preconditions
  - More than 8 assertions in a single postcondition block
  - Multiple distinct actions implied but only one expressed

Too Fine — merge when:
  - Specifies implementation detail (technology choices, internal algorithms)
  - Delivers no observable user/agent value on its own
  - Tests a single field validation that belongs as an assertion in a larger behavior

Project Type Calibration
  API specs    → endpoint + verb level (field validation as assertions)
  CLI specs    → command/subcommand level
  UI specs     → interaction level (not screen, not component)
  Worker specs → event/message level
  Agent specs  → reasoning/action level

When to Split a Spec
  Split when a spec exceeds ~15 behaviors or covers multiple bounded
  contexts. Each spec should map to one capability.

When to Merge Behaviors
  Merge when two behaviors share the same given/when/then structure
  and differ only in a single input value — use a parameterized behavior.

NFR Referencing
  Add an `nfr` section at spec-level for each applicable category.
  Pin specific constraints at behavior-level with `category#constraint`.
  Categories: performance (latency, throughput), reliability (uptime,
  recovery), security (auth, encryption), observability (logging, tracing),
  scalability (load, capacity), cost (budget, efficiency),
  operability (deployment, configuration).

Work Type Classification
  new feature   — new spec with all behaviors
  enhancement   — add behaviors to existing spec, bump minor version
  bug fix       — add error_case behavior that reproduces the bug
  refactor      — no spec change (behavior is preserved)
  deprecation   — mark behaviors as removed, bump major version

Phase Collapse Rule
  When a system has sequential internal phases (validate → process →
  cleanup), do NOT write one behavior per phase. Instead:
  1. Identify the end-state observable to the user
  2. Write ONE behavior describing that end-state
  3. Internal phases become implementation, not spec
  The \"when\" trigger should be the completion event, not each phase.

Writing Specs from Code
  When inferring specs from existing source code:
  1. Read the code to understand the domain — then stop reading code
  2. Ask: \"What does a user/caller observe before and after?\"
  3. Write behaviors answering ONLY that question
  4. If a behavior mentions an internal component name (Lambda, queue,
     database table, cache key, worker function), it's a smell — rewrite
  Technical constraints found in code (retry counts, timeout values,
  ID formats, token limits, model names) go in NFR files, not FR behaviors.

Entity Format Guidance
  Entity names (@repo, @user, @input) aid readability — keep them.
  Entity field shapes risk prescribing the data model. Rules:
  - Use domain-meaningful field names (id, repositoryId), not
    implementation names (mongoId, coreId, executionArn)
  - Keep entity shapes minimal — only fields referenced in when/then
  - If a field name maps to a specific technology, abstract it:
    Bad:  {{ repositoryMongoId: \"repo-789\" }}
    Good: {{ id: \"repo-789\" }}"
}

/// Condensed requirements smell detection reference for the guide tool.
pub fn guide_smells() -> &'static str {
    "\
Requirements Smell Detection
=============================

Detect these smells in specs and fix before writing tests.

Ambiguity
  Signal: Subjective language, ambiguous pronouns, vague modifiers.
  Action: Rewrite with measurable criteria and explicit subjects.

Non-verifiability
  Signal: Cannot define a concrete test for the assertion.
  Action: Sharpen into testable criteria or move to a guideline.

NASA Forbidden Words
  Signal: flexible, easy, sufficient, safe, adequate, user-friendly,
  fast, portable, robust, or any -ly/-ize adverb/verb.
  Action: Replace with quantified constraint or measurable threshold.

Compound Behavior
  Signal: \"and\" joining distinct actions in a behavior description.
  Action: Split into separate behaviors, one action each.

Missing Error Cases
  Signal: Only happy_path behaviors specified, no error_case.
  Action: Add error_case and edge_case behaviors for each happy path.

Implicit Preconditions
  Signal: When block assumes state not declared in the given section.
  Action: Make every precondition explicit in the given block.

Prose Assertion Overuse
  Signal: More than 20% of assertions are prose (non-typed).
  Action: Convert to typed assertion operators where possible.

Missing NFR References
  Signal: Data access with no security NFR, API with no performance NFR.
  Action: Add appropriate NFR constraint references.

Implementation Leakage
  Signal: Spec describes HOW not WHAT (\"uses HashMap\", \"calls REST\").
  Detection: Apply three tests to each behavior:

  Observer Test — ask \"who observes this outcome?\"
    Valid: API caller, end user, ops team (via dashboards → NFR).
    Invalid: internal function, downstream step, queue consumer, DB trigger.
    If only internal components observe it → collapse into user-observable
    outcome, move to NFR, or drop.

  Swap Test — ask \"if I replaced the technology, would this behavior break?\"
    \"SQS fan-out\" → swap to HTTP → breaks → SMELL
    \"All documents searchable\" → swap anything → holds → GOOD
    If swapping implementation breaks the description, it's a leak.

  Action: Rewrite at behavioral level — describe observable outcomes.

Phase Leak
  Signal: Sequential behaviors matching internal processing phases
  (e.g., paginate-records, process-batch, purge-stale).
  Action: Collapse into one behavior per user-observable end-state.
  The \"when\" trigger is the completion event, not each phase.
  Bad: paginate-generates-batches, worker-processes-batch, purge-deletes-stale
  Good: all-documents-indexed, stale-documents-removed"
}

/// Condensed context management protocol reference for the guide tool.
pub fn guide_context() -> &'static str {
    "\
Context Management Protocol
============================

Specs form a dependency graph: specs reference sub-specs and link to NFRs.
Loading everything at once will bloat your context. Follow this discipline:


The Lazy Loading Sequence
-------------------------

1. Get the subgraph structure first.
   Call minter's graph tool scoped to your target spec. This returns the
   dependency tree — IDs, names, relationships — without full content.
   This is lightweight.

2. Load the root spec's full content.
   Read the spec you're directly working on.

3. Load NFRs on demand.
   As you encounter a behavior that references an NFR, load that specific
   NFR. Don't front-load all NFRs.

4. Load sub-specs only when you reach them.
   If the root spec depends on a sub-spec, load it when you're working on
   a behavior that references it — not before.


Guard Rails
-----------

- Never load the full project graph.
  Always scope to the target spec's subgraph.

- Never load more than 3 specs/NFRs in a single step.
  If the subgraph is larger, work through it incrementally.

- Structure before content.
  Always read the graph topology first, then pull content for what you
  need right now.

- If you're unsure whether you need a dependency, you don't need it yet.
  Load it when you do."
}

/// Condensed NFR design reference for the guide tool.
pub fn guide_nfr() -> &'static str {
    "\
NFR Design Guide
=================

Seven Fixed Categories
  performance   — latency, throughput, resource usage, response time
  reliability   — availability, consistency, fault tolerance, recovery
  security      — isolation, authentication, authorization, encryption
  observability — logging, metrics, tracing, alerting, health checks
  scalability   — load limits, growth paths, concurrency, ceilings
  cost          — infrastructure budgets, per-unit economics, efficiency
  operability   — deployment, IaC, rollback, maintenance, CI/CD

Not every project needs all seven. The categories are fixed — only the
constraints within them change.


Two Constraint Types

  Metric — quantitative, threshold-based. Has a measurable value and a
  pass/fail threshold (e.g. latency p95 < 500ms, availability > 99.9%).

  Rule — structural, binary pass/fail. An invariant that holds or doesn't
  (e.g. all data access scoped by repository_id, no secrets in source).

Every constraint must include a verification method specific enough to
generate a test.


Three-Level Referencing

  Spec-level    — whole NFR file or specific anchor in frontmatter.
                  Applies to all behaviors. Tests emitted for all constraints.
  Behavior-level — specific constraint pinned to one behavior, with optional
                  threshold override (must be stricter than default).
  Global        — defined in the NFR file itself. Applies to all specs
                  that reference the file.


Coverage Gap Detection

  When a functional spec involves these concerns, flag missing NFRs:

  Data access (read/write)    → security (isolation, auth)
  API endpoints               → performance (latency budgets)
  State changes               → reliability (consistency, recovery)
  External service calls      → reliability (fault tolerance)
  User-facing operations      → performance (response time)
  Infrastructure changes      → operability (deployment, rollback)
  Batch/bulk operations       → cost (limits), scalability (concurrency)
  Logging/metrics emission    → observability (structured logging)


FR/NFR Classification

  Use three axes to classify requirements:

  Scope         — FR: affects one feature. NFR: cross-cuts many features.
  Architecture  — FR: doesn't shape architecture. NFR: architecturally significant.
  Constraint    — FR: what to build. NFR: how well it must work.

  Requirements with both FR and NFR characteristics (auth, encryption,
  accessibility) should be decomposed: behavior in FR spec, quality
  constraint in NFR spec.


FR/NFR Decision Tree

  Is this a constraint on HOW the system works?
    YES → Does it apply to a single behavior?
      YES → NFR constraint pinned to that behavior
      NO  → NFR constraint at spec level
    NO → Is the outcome observable by the API caller or end user?
      YES → FR behavior
      NO  → Either NFR or drop it

  Technical constraints from code (retry counts, timeout values, ID
  formats, token limits, model names, dimension counts) are NFR
  material, not FR behaviors.


Override Rules

  - Only constraints marked `overridable yes` can be overridden
  - Only metric constraints (not rules) support overrides
  - Override operator must match original threshold operator
  - Override value must be stricter than default
  - Overrides are behavior-level only (not spec-level)"
}

/// Coverage tagging guide for the guide tool.
pub fn guide_coverage() -> &'static str {
    concat!(
        "\
Coverage Tagging Guide
======================

Purpose
  The @minter tag links a test to the spec behaviors it covers. Minter's
  coverage command scans these tags and cross-references them against the
  spec graph to produce a coverage report. Tags are declarations — they
  state \"this test covers this behavior,\" nothing more.

Tag Format
  Behavioral:   // @minter:<type> <behavior> [<behavior>...]
  Benchmark:    // @minter:benchmark #<category>#<constraint> [...]
  Types: unit, integration, e2e, benchmark
  Comment styles: // (C-family, Rust, Go) or # (Python, Ruby, Shell)

Placement
  Place the tag immediately above the test block it annotates.
  One tag per test block. If a describe has sub-describes with different
  coverage, use separate tags on each.

Behavioral Tags (unit, integration, e2e)
  Reference behavior names from specs. Space-separated, no commas.
  Do NOT add NFR refs — NFR coverage is derived automatically from the
  spec graph.
  // @minter:unit validate-valid-spec
  // @minter:e2e report-full-coverage report-partial-coverage
  # @minter:integration reject-unknown-behavior-id

Benchmark Tags
  Reference NFR constraints only. Format: #category#constraint.
  Do NOT add behavior IDs.
  // @minter:benchmark #performance#validation-latency

Choosing the Type
  unit         — tests a single function or module in isolation
  integration  — tests multiple components working together
  e2e          — tests the full system from the user's entry point
  benchmark    — measures an NFR constraint (latency, throughput, scaling)

Qualified Names
  If two specs share a behavior name, qualify with spec-name/behavior-name:
    // @",
        "minter:unit billing-webhooks/handle-error

Discovering Behavior Names
  minter inspect specs/my-feature.spec   — lists all behavior names
  minter graph specs/                    — shows the full spec tree

When to Tag
  Tag every test that exercises a spec behavior. Tests that don't map to
  a spec behavior should not be tagged. When spec behaviors change,
  update the tags.

Common Mistakes
  NFR on behavioral tag  — redundant, coverage is derived automatically
  Behavior on benchmark  — benchmarks are for NFR constraints only
  Missing colon          — @minter unit (wrong) vs @minter:unit (right)
  Using commas           — @minter:unit a, b (wrong) vs @minter:unit a b
  Invented names         — IDs must match spec behavior names exactly
  Tagging everything     — only tag tests that map to spec behaviors"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// content: methodology-contains-key-sections
    #[test]
    fn methodology_contains_key_sections() {
        let text = methodology();
        assert!(text.starts_with("Spec-Driven Development Methodology"));
        assert!(text.contains("Non-Functional Requirements"));
        assert!(text.contains("Cross-Reference Binding"));
        assert!(text.contains("Override Rules"));
        assert!(text.contains("Test Generation"));
        assert!(text.contains("Workflow"));
    }

    /// content: methodology-contains-workflow-phases
    #[test]
    fn methodology_contains_workflow_phases() {
        let text = methodology();
        assert!(text.contains("Phase 1"));
        assert!(text.contains("Phase 2"));
        assert!(text.contains("Phase 3"));
        assert!(text.contains("Phase 4"));
        assert!(text.contains("Phase 5"));
        assert!(text.contains("before any implementation"));
        assert!(text.contains("1 behavior = 1 test"));
        assert!(text.contains("fail"));
    }

    /// content: fr-grammar-contains-nfr-ref-syntax
    #[test]
    fn fr_grammar_contains_nfr_ref_syntax() {
        let text = fr_grammar();
        assert!(text.starts_with("Functional Requirement (FR) Spec Grammar"));
        assert!(text.contains("whole-file reference"));
        assert!(text.contains("anchor reference"));
        assert!(text.contains("override"));
        assert!(text.contains("nfr"));
    }

    /// content: fr-scaffold-contains-nfr-section
    #[test]
    fn fr_scaffold_contains_nfr_section() {
        let text = fr_scaffold();
        assert!(text.starts_with("spec my-feature"));
        assert!(text.contains("nfr"));
        assert!(text.contains("performance"));
    }

    /// content: nfr-scaffold-interpolates-all-categories
    #[test]
    fn nfr_scaffold_interpolates_all_categories() {
        for category in crate::model::VALID_NFR_CATEGORIES {
            let text = nfr_scaffold(category);
            let expected_header = format!("nfr {category} v0.1.0");
            assert!(
                text.starts_with(&expected_header),
                "scaffold for {category} should start with '{expected_header}'"
            );
            let title = crate::model::capitalize(category);
            let expected_title = format!("{title} Requirements");
            assert!(
                text.contains(&expected_title),
                "scaffold for {category} should contain '{expected_title}'"
            );
        }
    }

    // ── initialize_minter ─────────────────────────────────

    /// content: initialize-minter-contains-principles
    #[test]
    fn initialize_minter_contains_principles() {
        let text = initialize_minter();
        assert!(text.contains("Principles"));
        assert!(text.contains("Specs are the source of truth"));
        assert!(text.contains("TDD is mandatory"));
        assert!(text.contains("1 behavior = 1 test"));
    }

    /// content: initialize-minter-contains-workflow-phases
    #[test]
    fn initialize_minter_contains_workflow_phases() {
        let text = initialize_minter();
        assert!(text.contains("Workflow Phases"));
        assert!(text.contains("Phase 1"));
        assert!(text.contains("Phase 2"));
        assert!(text.contains("Phase 3"));
        assert!(text.contains("Phase 4"));
        assert!(text.contains("Phase 5"));
    }

    /// content: initialize-minter-lists-tools
    #[test]
    fn initialize_minter_lists_tools() {
        let text = initialize_minter();
        assert!(text.contains("validate"));
        assert!(text.contains("scaffold"));
        assert!(text.contains("format"));
        assert!(text.contains("inspect"));
        assert!(text.contains("graph"));
        assert!(text.contains("initialize_minter"));
        assert!(text.contains("guide"));
    }

    // ── guide ─────────────────────────────────────────────

    /// content: guide-workflow-contains-phases
    #[test]
    fn guide_workflow_contains_phases() {
        let text = guide_workflow();
        assert!(text.contains("Phase 1"));
        assert!(text.contains("Phase 2"));
        assert!(text.contains("Phase 3"));
        assert!(text.contains("Phase 4"));
        assert!(text.contains("Phase 5"));
        assert!(text.contains("scaffold"));
        assert!(text.contains("validate"));
    }

    /// content: guide-authoring-contains-topics
    #[test]
    fn guide_authoring_contains_topics() {
        let text = guide_authoring();
        assert!(text.contains("Right Granularity"));
        assert!(text.contains("When to Split"));
        assert!(text.contains("When to Merge"));
        assert!(text.contains("NFR Referencing"));
        assert!(text.contains("Work Type Classification"));
        assert!(text.contains("Too Coarse"));
        assert!(text.contains("Too Fine"));
        assert!(text.contains("API"));
        assert!(text.contains("CLI"));
        assert!(text.contains("Phase Collapse"));
        assert!(text.contains("Writing Specs from Code"));
        assert!(text.contains("Entity Format"));
    }

    /// content: guide-smells-contains-smell-types
    #[test]
    fn guide_smells_contains_smell_types() {
        let text = guide_smells();
        assert!(text.contains("Ambiguity"));
        assert!(text.contains("NASA"));
        assert!(text.contains("Compound"));
        assert!(text.contains("Implementation"));
        assert!(text.contains("Signal"));
        assert!(text.contains("Action"));
        assert!(text.contains("Observer Test"));
        assert!(text.contains("Swap Test"));
        assert!(text.contains("Phase Leak"));
    }

    /// content: guide-nfr-contains-sections
    #[test]
    fn guide_nfr_contains_sections() {
        let text = guide_nfr();
        // Seven categories
        assert!(text.contains("performance"));
        assert!(text.contains("reliability"));
        assert!(text.contains("security"));
        assert!(text.contains("observability"));
        assert!(text.contains("scalability"));
        assert!(text.contains("cost"));
        assert!(text.contains("operability"));
        // Constraint types
        assert!(text.contains("Metric"));
        assert!(text.contains("Rule"));
        // Referencing levels
        assert!(text.contains("Spec-level"));
        assert!(text.contains("Behavior-level"));
        // Coverage gaps
        assert!(text.contains("Coverage"));
        // Classification
        assert!(text.contains("Classification"));
        // Decision tree
        assert!(text.contains("Decision Tree"));
    }

    /// content: guide-context-contains-sections
    #[test]
    fn guide_context_contains_sections() {
        let text = guide_context();
        assert!(text.contains("Lazy Loading Sequence"));
        assert!(text.contains("Guard Rails"));
        assert!(text.contains("subgraph"));
        assert!(text.contains("graph tool"));
        assert!(text.contains("NFR"));
        assert!(text.contains("Never load the full project graph"));
        assert!(text.contains("Structure before content"));
    }

    /// content: guide-coverage-contains-sections
    #[test]
    fn guide_coverage_contains_sections() {
        let text = guide_coverage();
        assert!(text.contains("Coverage Tagging"));
        assert!(text.contains("@minter"));
        assert!(text.contains("unit"));
        assert!(text.contains("e2e"));
        assert!(text.contains("benchmark"));
        assert!(text.contains("Qualified Names"));
        assert!(text.contains("Common Mistakes"));
    }
}
