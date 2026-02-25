# NFR DSL Format — Grammar Reference

**Status:** Locked
**Version:** 1.0.0
**Extension:** `.nfr`
**Purpose:** Defines the grammar, structure, and validation rules for non-functional requirement spec files parsed by minter.

---

## Grammar

```
nfr <category> v<version>
title "<title>"

description
  <free text>

motivation
  <free text>


constraint <name> [metric]
  "<description>"

  metric "<what is measured>"
  threshold <operator> <value>

  verification
    environment <env1>, <env2>, ...
    benchmark "<procedure>"
    dataset "<data requirements>"
    pass "<criteria>"

  violation <critical|high|medium|low>
  overridable <yes|no>


constraint <name> [rule]
  "<description>"

  rule
    <free text>

  verification
    static "<check description>"
    runtime "<check description>"

  violation <critical|high|medium|low>
  overridable <yes|no>
```

---

## File Header

Every `.nfr` file starts with a header declaring category, version, title, description, and motivation.

### `nfr` declaration

```
nfr <category> v<version>
```

- **category** — one of the 7 fixed categories (see Categories below)
- **version** — semver string without the `v` prefix internally (e.g., `v1.0.0` parses as `1.0.0`)

The category is enforced by the parser. The filename does not matter — the category inside the file is authoritative.

### `title`

```
title "<quoted string>"
```

Short human-readable name for this NFR file.

### `description` and `motivation`

```
description
  <indented free text, one or more lines>

motivation
  <indented free text, one or more lines>
```

Same text-block pattern as the FR DSL. Keyword on its own line, content indented by 2+ spaces on following lines.

---

## Constraints

Constraints are the core unit of the NFR format. They replace behaviors from the FR DSL. Each constraint defines a single testable non-functional requirement.

### Declaration

```
constraint <name> [<type>]
  "<description>"
```

- **name** — kebab-case identifier, unique within the file. Serves as the anchor for cross-references from FR specs (e.g., `performance#api-response-time`).
- **type** — either `metric` or `rule`
- **description** — quoted string on the next line, indented by 2 spaces

At least one constraint is required per file.

---

## Metric Constraints

Metric constraints define thresholds verified by benchmarks. They are environment-dependent — meaningful results require representative infrastructure and data.

### Structure

```
constraint <name> [metric]
  "<description>"

  metric "<what is measured>"
  threshold <operator> <value>

  verification
    environment <env1>, <env2>, ...
    benchmark "<procedure>"
    dataset "<data requirements>"
    pass "<criteria>"

  violation <severity>
  overridable <yes|no>
```

### Fields

**`metric`** — Quoted string describing what is measured. Short descriptor, one line.
```
metric "HTTP response time, p95"
metric "Sustained requests per second without degradation"
metric "Time from source write to downstream availability"
```

**`threshold`** — Operator followed by a value. The operator makes the direction explicit, which is critical for override validation.
```
threshold < 500ms
threshold > 100
threshold >= 99.9%
threshold < 500KB
```

Valid operators: `<`, `>`, `<=`, `>=`, `==`

Units are part of the value string (e.g., `500ms`, `99.9%`, `100`). The parser captures operator and value as raw strings. Semantic validation of units is separate from parsing.

**Conditional thresholds:** When a constraint has different thresholds for different contexts (e.g., reads vs writes), the NFR file declares the most permissive default. FR specs narrow it per-behavior using override syntax. The NFR does not know about domain-specific operation types — the functional spec does.

Example: API response time is `< 1s` as the default. An FR behavior for a fast-mode endpoint overrides it to `< 500ms`. The NFR file only says `threshold < 1s`.

### Metric Verification

```
verification
  environment <env1>, <env2>, ...
  benchmark "<procedure>"
  dataset "<data requirements>"
  pass "<criteria>"
```

- **environment** — required. Comma-separated list of environments where this metric produces meaningful results. Common values: `staging`, `production`, `all`. When `all`, the metric can run in any environment including local/CI.
- **benchmark** — required. Quoted string describing the benchmark procedure.
- **dataset** — optional. Quoted string describing data requirements. Omit when not applicable.
- **pass** — required. Quoted string describing what constitutes passing.

Multiple `benchmark`, `dataset`, or `pass` lines are allowed when a constraint requires several verification steps.

---

## Rule Constraints

Rule constraints define structural invariants verified by static analysis, architectural tests, or integration tests. They are environment-independent — they run everywhere.

### Structure

```
constraint <name> [rule]
  "<description>"

  rule
    <free text>

  verification
    static "<check description>"
    runtime "<check description>"

  violation <severity>
  overridable <yes|no>
```

### Fields

**`rule`** — Text block (keyword on its own line, indented content below). The invariant statement. Can be multi-line.
```
rule
  No endpoint may issue more than a fixed number of database
  or service calls regardless of result set size. If a loop
  contains a query, it must be refactored to batch or join.
```

### Rule Verification

```
verification
  static "<check description>"
  runtime "<check description>"
```

- **static** — optional. Quoted string describing a static analysis or code review check.
- **runtime** — optional. Quoted string describing a runtime or integration test check.
- At least one of `static` or `runtime` is required.

Multiple `static` or `runtime` lines are allowed when a constraint requires several verification steps.

---

## Shared Fields

These fields appear on both metric and rule constraints.

### `violation`

```
violation <severity>
```

Severity levels: `critical`, `high`, `medium`, `low`

Indicates the impact when the constraint is violated. Used by CI/CD pipelines to determine blocking vs warning behavior.

### `overridable`

```
overridable <yes|no>
```

- **yes** — FR specs may override the default threshold (metrics) or adjust scope. Overrides must be stricter (same direction), never relaxed.
- **no** — system invariant. Cannot be overridden by any FR spec. Attempts to override produce a validation error.

---

## Categories

7 fixed categories. The taxonomy does not change per project — which constraints are activated does.

| Category | Scope |
|----------|-------|
| `performance` | Latency, throughput, resource usage, efficiency rules |
| `reliability` | Availability, consistency, fault tolerance, recovery |
| `security` | Isolation, authentication, authorization, data protection |
| `observability` | Logging, metrics, tracing, alerting |
| `scalability` | Load limits, growth paths, resource ceilings |
| `cost` | Infrastructure budgets, per-unit economics |
| `operability` | Deployment, IaC, rollback, maintenance |

Not every project needs all 7. A prototype might only activate performance and security. But if a spec involves data access and no security NFR exists, that is a flag.

---

## Formatting Rules

Inherited from the FR DSL:

- **Indentation:** 2-space indent. Tabs are rejected.
- **Comments:** Lines starting with `#` are ignored.
- **Blank lines:** Tolerated between sections and constraints. Ignored by the parser.
- **Constraint names:** kebab-case (e.g., `api-response-time`, `no-n-plus-one`).

### Nesting depth

| Level | Indent | Content |
|-------|--------|---------|
| 0 | none | `nfr`, `title`, `constraint` declarations |
| 1 | 2 spaces | Constraint body: description, `metric`, `threshold`, `verification`, `violation`, `overridable`, `rule` keyword |
| 2 | 4 spaces | Verification sub-fields: `environment`, `benchmark`, `dataset`, `pass`, `static`, `runtime`. Also `rule` text content. |

---

## Validation Rules

Structural validation (parser-level):

1. File must start with `nfr <category> v<version>`
2. Category must be one of the 7 valid values
3. `title`, `description`, `motivation` are required
4. At least one `constraint` is required
5. Constraint names must be unique within the file
6. Constraint type must be `metric` or `rule`
7. Metric constraints require: `metric`, `threshold`, `verification` with `environment` + `benchmark` + `pass`, `violation`, `overridable`
8. Rule constraints require: `rule`, `verification` with at least one of `static`/`runtime`, `violation`, `overridable`
9. `violation` must be one of: `critical`, `high`, `medium`, `low`
10. `overridable` must be `yes` or `no`
11. `threshold` must start with a valid operator: `<`, `>`, `<=`, `>=`, `==`
12. Tab indentation anywhere in the file is an error

---

## Full Example: performance.nfr

```
nfr performance v1.0.0
title "Performance Requirements"

description
  Defines the performance constraints for the system including
  response times, throughput targets, and efficiency rules.

motivation
  Performance directly impacts user experience and system
  reliability. These constraints ensure the system meets
  its SLA commitments.


# Latency

constraint api-response-time [metric]
  "API endpoints must respond within acceptable latency bounds"

  metric "HTTP response time, p95"
  threshold < 1s

  verification
    environment staging, production
    benchmark "100 concurrent requests per endpoint"
    dataset "Production-representative volume"
    pass "p95 < threshold"

  violation high
  overridable yes


constraint data-freshness [metric]
  "Data must be available downstream within target window after write"

  metric "Time from source write to downstream availability"
  threshold < 5s

  verification
    environment staging, production
    benchmark "Write document, poll until readable, measure delta"
    pass "p95 < threshold"

  violation high
  overridable yes


constraint throughput [metric]
  "System must sustain target request rate without degradation"

  metric "Sustained requests per second without degradation"
  threshold > 100

  verification
    environment staging, production
    benchmark "Ramp load test over 5 minutes"
    pass "No error rate increase above 1% at target RPS"

  violation high
  overridable yes


constraint payload-size [metric]
  "API responses must not exceed size limits"

  metric "Response body size"
  threshold < 500KB

  verification
    environment all
    benchmark "Assert response Content-Length on representative queries"
    pass "No response exceeds threshold"

  violation medium
  overridable yes


# Efficiency rules

constraint no-n-plus-one [rule]
  "No endpoint may issue unbounded database calls"

  rule
    No endpoint may issue more than a fixed number of database
    or service calls regardless of result set size. If a loop
    contains a query, it must be refactored to batch or join.

  verification
    static "Query count per request path does not scale with input size"
    runtime "Request with 1 item and 100 items issues same number of DB calls"

  violation high
  overridable no


constraint no-sequential-where-parallel [rule]
  "Independent async operations must execute concurrently"

  rule
    Independent async operations must use parallel execution,
    not sequential await chains.

  verification
    static "Code review for sequential awaits on independent operations"

  violation medium
  overridable yes
```

## Full Example: security.nfr

```
nfr security v1.0.0
title "Security Requirements"

description
  Defines the security constraints for the system including
  data isolation, input validation, and access control.

motivation
  Security constraints are system invariants. Most are not
  overridable — they represent the minimum safety floor.


constraint tenant-isolation [rule]
  "All data access must be scoped by tenant identifier"

  rule
    All data access must be scoped by tenant or repository
    identifier. No query path may return data belonging to
    a different tenant.

  verification
    static "Every database query and search query includes tenant filter"
    runtime "Authenticate as tenant A, attempt to access tenant B data, assert zero results or 403"

  violation critical
  overridable no


constraint no-secrets-in-code [rule]
  "No credentials or tokens in source code"

  rule
    No credentials, API keys, tokens, or connection strings
    in source code or configuration files committed to version
    control. All secrets via environment variables or secrets
    manager.

  verification
    static "Secret scanning on every commit with regex patterns for common key formats"
    runtime "Grep codebase for known secret patterns"

  violation critical
  overridable no


constraint input-validation [rule]
  "All external input must be validated at the boundary"

  rule
    All external input must be validated at the boundary. No
    raw user input reaches database queries, system commands,
    or downstream services without validation and sanitization.

  verification
    static "Every API endpoint has input schema validation"
    runtime "Submit malformed payloads including SQL injection, XSS, and oversized input, assert rejection"

  violation critical
  overridable no


constraint auth-on-every-endpoint [rule]
  "Every API endpoint must enforce authentication"

  rule
    Every API endpoint must enforce authentication. No endpoint
    is accessible without valid credentials unless explicitly
    marked as public with justification.

  verification
    runtime "Call every endpoint without auth header, assert 401 except documented public endpoints"

  violation critical
  overridable no
```

---

## Design Notes

### Why `metric` vs `rule` and not a unified type

Metrics have thresholds you measure against — they produce benchmark results. Rules have invariants you check — they produce pass/fail structural checks. The verification shape is fundamentally different: metrics need environment + benchmark + pass criteria; rules need static/runtime checks. Unifying them would mean every constraint carries fields it doesn't use.

### Why verification is prose, not executable

Verification fields are generation hints — specific enough for a competent agent to write the test, but not an executable test DSL. Making them executable would require embedding a separate language inside the NFR DSL. The quality bar is: could an agent write the test from this description alone, without further clarification? If yes, the verification is specific enough.

### Why `environment all` is valid for metrics

The metric/rule split is about verification shape, not about where it runs. A constraint like `payload-size < 500KB` has a numeric threshold you measure against — that's a metric. The fact that it can be checked in any environment doesn't make it a rule. Rules have invariant statements and static/runtime checks; metrics have thresholds and benchmarks.

### Why conditional thresholds are not in the grammar

When a constraint has different thresholds for different contexts (reads vs writes, fast mode vs exhaustive mode), the NFR file declares the most permissive default. FR behavior-level references narrow the threshold for specific contexts. This keeps the NFR grammar simple and pushes domain-specific conditional logic to the functional spec, which is where that knowledge belongs.

---
---

# NFR Cross-Reference Syntax — FR-Side Grammar Extension

**Status:** Locked
**Version:** 1.0.0
**Purpose:** Defines how functional `.spec` files reference NFR `.nfr` constraints, and the cross-validation rules minter enforces.

---

## Overview

FR specs reference NFRs at two locations:

1. **Spec-level** — in the file header, applies to the whole spec
2. **Behavior-level** — inside a behavior block, applies to that behavior only

All references emit tests. No reference is just context loading.

---

## Reference Syntax

A single NFR reference takes one of three forms:

```
<category>                                    # whole-file reference
<category>#<constraint-name>                  # specific anchor reference
<category>#<constraint-name> <operator> <value>  # anchor with threshold override
```

- **`<category>`** — one of the 7 NFR categories. Resolves to the `.nfr` file declaring that category.
- **`#<constraint-name>`** — anchor into the file. Must match a `constraint` name in the resolved `.nfr` file.
- **`<operator> <value>`** — threshold override. Same syntax as `threshold` in the NFR file. Only valid on metric constraints marked `overridable yes`.

---

## Spec-Level References (Level 1 & Level 2)

A new optional `nfr` section in the FR spec header. Placed **after `motivation`, before the first `behavior`**.

### Grammar

```
nfr
  <reference>
  <reference>
  ...
```

The `nfr` keyword on its own line, followed by indented references (2-space indent). One reference per line.

### Level 1 — Whole File

```
nfr
  security
```

References the entire `security.nfr` file. Test generation emits a test for **every constraint** in that file, scoped to the whole spec.

Use sparingly. This is appropriate when the spec genuinely touches all constraints in a category (e.g., a data-access spec referencing all security constraints). For most specs, Level 2 is more precise.

### Level 2 — Specific Anchor

```
nfr
  security
  reliability#completeness
```

References every constraint in `security.nfr` (Level 1) plus the specific `completeness` constraint from `reliability.nfr` (Level 2). Test generation emits targeted tests for each.

### Rules

- The `nfr` section is optional. Omit it when the spec has no NFR references.
- Spec-level references **cannot** have threshold overrides. Overrides are behavior-level only.
- Whole-file and anchor references can be mixed in the same `nfr` section.
- A category can appear as both a whole-file reference and anchor references — the whole-file reference already includes everything, so anchors from the same category are redundant but not an error.

---

## Behavior-Level References (Level 3)

A new optional `nfr` section inside behavior blocks. Placed **after the quoted description, before `given`**.

### Grammar

```
behavior <name> [<category>]
  "<description>"

  nfr
    <category>#<constraint-name>
    <category>#<constraint-name> <operator> <value>

  given
    ...
```

The `nfr` keyword at behavior indent level (2 spaces), followed by references at content indent level (4 spaces). One reference per line.

### Anchor Reference (no override)

```
behavior cdc-indexing [happy_path]
  "Document changes are indexed within the freshness window"

  nfr
    performance#data-freshness

  given
    ...
```

Pins the `data-freshness` constraint to this behavior. Test generation emits a test for that constraint scoped to this behavior, using the default threshold from `performance.nfr`.

### Anchor Reference with Override

```
behavior fast-mode-search [happy_path]
  "Agent returns results in fast mode within tightened latency bound"

  nfr
    performance#api-response-time < 500ms
    reliability#completeness >= 100%

  given
    ...
```

Overrides the default threshold. The NFR file might say `threshold < 1s`, but this behavior demands `< 500ms`. Test generation uses the overridden value.

### Rules

- Behavior-level references **must** use anchor syntax (`category#constraint`). Whole-file references are not allowed at the behavior level — they're too broad for a single behavior.
- Overrides are optional. A behavior can pin a constraint without overriding its threshold.
- Overrides are only valid on `[metric]` constraints. Rule constraints have no threshold to override.
- The override operator must match the direction of the original threshold (see Cross-Validation Rules).

---

## Containment Rule

**Every category referenced at the behavior level must also appear in the spec-level `nfr` section.**

This ensures the spec explicitly declares its NFR dependencies up front. A behavior cannot silently pull in an NFR category that isn't visible in the header.

Valid:
```
nfr
  performance
  reliability

behavior fast-mode [happy_path]
  "..."
  nfr
    performance#api-response-time < 500ms    # performance is in spec-level nfr
    reliability#completeness >= 100%          # reliability is in spec-level nfr
  given
    ...
```

Invalid:
```
nfr
  performance

behavior fast-mode [happy_path]
  "..."
  nfr
    reliability#completeness >= 100%          # ERROR: reliability not in spec-level nfr
  given
    ...
```

The spec-level reference can be either whole-file (`reliability`) or anchor (`reliability#completeness`) — either satisfies the containment rule.

---

## FR Grammar — Updated Parse Order

The `nfr` section adds one optional step at each level:

### Spec-level

```
spec <name> v<version>
title "<title>"

description
  <free text>

motivation
  <free text>

nfr                              ← NEW, optional
  <references>

behavior <name> [<category>]
  ...

depends on <spec-name> >= <version>
```

Position: after `motivation`, before the first `behavior`. If absent, the spec has no NFR references.

### Behavior-level

```
behavior <name> [<category>]
  "<description>"

  nfr                            ← NEW, optional
    <references>

  given
    ...

  when ...

  then ...
```

Position: after the quoted description, before `given`. If absent, the behavior has no behavior-level NFR references.

---

## Cross-Validation Rules

These are enforced by `minter validate` when NFR references are present. All validation is mechanical — no judgment required.

### Reference Resolution

1. **Category exists.** Every referenced category must resolve to a `.nfr` file with that category declaration. Missing file = validation error.
2. **Anchor exists.** Every `#constraint-name` must match a `constraint` name in the resolved `.nfr` file. Missing anchor = validation error.
3. **Containment.** Every category used in a behavior-level `nfr` must also appear in the spec-level `nfr` section. Violation = validation error.

### Override Validation

4. **Overridable check.** Overrides are only allowed on constraints marked `overridable yes`. Overriding `overridable no` = validation error.
5. **Metric only.** Overrides are only valid on `[metric]` constraints. Overriding a `[rule]` constraint = validation error (rules have no threshold).
6. **Same operator.** The override operator must match the original threshold operator. If the NFR says `threshold < 1s`, the override must also use `<`. Mismatched operator = validation error.
7. **Stricter value.** The override must be stricter than the default. For `<` / `<=`: override value must be smaller. For `>` / `>=`: override value must be larger. For `==`: override must equal the default (effectively no change allowed). Relaxed override = validation error.

### Structural

8. **No spec-level overrides.** Threshold overrides in the spec-level `nfr` section = validation error.
9. **No behavior-level whole-file references.** Whole-file references (`performance` without `#anchor`) in a behavior-level `nfr` section = validation error.

---

## Full Example

```
spec agentic-search v1.0.0
title "Agentic Search"

description
  Search capability for the agent system. Supports fast mode
  for interactive queries and exhaustive mode for complete
  result sets.

motivation
  Search is the primary user-facing operation. Performance
  and data completeness directly impact user trust.

nfr
  security
  performance
  reliability#completeness

# Fast path

behavior fast-mode-search [happy_path]
  "Agent returns results in fast mode within tightened latency"

  nfr
    performance#api-response-time < 500ms
    reliability#completeness

  given
    The search index contains 10K+ documents
    Agent receives a search request with mode "fast"

  when search
    mode = "fast"
    query = "contract renewal"

  then returns search results
    assert results is_present
    assert result_count >= 1

  then emits process_exit
    assert code == 0


# Exhaustive path

behavior exhaustive-mode-search [happy_path]
  "Agent returns complete results in exhaustive mode"

  nfr
    performance#api-response-time < 2s
    reliability#completeness >= 100%

  given
    The search index contains 10K+ documents
    Agent receives a search request with mode "exhaustive"

  when search
    mode = "exhaustive"
    query = "contract renewal"

  then returns search results
    assert results is_present
    assert completeness == "100%"

  then emits process_exit
    assert code == 0


# Indexing

behavior cdc-indexing [happy_path]
  "Document changes appear in search within freshness window"

  nfr
    performance#data-freshness

  given
    A document is saved in the source system

  when index-change
    document_id = "doc-123"

  then side_effect
    assert document is searchable within freshness threshold

  then emits process_exit
    assert code == 0


depends on search-index >= 1.0.0
```

**Tests emitted from this spec:**

| Source | Test |
|--------|------|
| `nfr security` (spec-level, whole file) | One test per constraint in `security.nfr` — tenant isolation, input validation, auth, etc. |
| `nfr reliability#completeness` (spec-level, anchor) | Completeness test scoped to the whole spec |
| `nfr performance` (spec-level, whole file) | One test per constraint in `performance.nfr` — latency, throughput, etc. at default thresholds |
| `fast-mode-search` + `performance#api-response-time < 500ms` | Latency benchmark scoped to fast mode, threshold overridden to 500ms |
| `fast-mode-search` + `reliability#completeness` | Completeness test scoped to fast mode, default threshold |
| `exhaustive-mode-search` + `performance#api-response-time < 2s` | Latency benchmark scoped to exhaustive mode, threshold overridden to 2s |
| `exhaustive-mode-search` + `reliability#completeness >= 100%` | Completeness test scoped to exhaustive mode, threshold overridden to 100% |
| `cdc-indexing` + `performance#data-freshness` | Freshness benchmark scoped to CDC behavior, default threshold |

---

## Design Notes

### Why block syntax instead of repeated `nfr` lines

The `depends on` section uses repeated keywords (`depends on X`, `depends on Y`). The `nfr` section uses a block with indented lines instead. Reason: NFR references can be numerous (3-5 per spec is common), and the block groups them visually as a declaration of quality contracts. It also mirrors how `description` and `motivation` work — keyword, then indented content.

### Why containment is enforced

The containment rule (behavior-level categories must appear in spec-level) serves two purposes:

1. **Discoverability.** Reading just the header tells you every NFR category this spec touches. No need to scan all behaviors.
2. **Explicit dependency declaration.** Same principle as `depends on` for FR deps — you declare what you depend on. This enables `minter graph` to show NFR dependencies without parsing every behavior.

### Why whole-file references are spec-level only

A whole-file reference says "every constraint in this category applies." That's a statement about the spec as a whole — it doesn't make sense scoped to a single behavior. A behavior should reference specific constraints it cares about.

### Why overrides must match operator direction

If an NFR says `threshold < 1s`, overriding with `> 500ms` is nonsensical — it changes the meaning from "must be less than" to "must be greater than." Overrides tighten the same constraint, they don't redefine it. Same operator ensures the override and default are on the same axis.

### Why `==` overrides are effectively no-ops

An `==` threshold means the value must be exactly X. There's no "stricter" direction — you can only match it. So overriding `== 100%` with `== 100%` is valid but pointless, and overriding with any other value is invalid. This is by design — `==` constraints are rigid.
