# Authoring Guide

How to write good specs — granularity, decomposition, smell detection, and NFR design.

---

## Right granularity

A behavior describes one user-observable outcome. If the description needs "and", split it into two behaviors. Aim for 1–5 assertions per postcondition block. Each behavior should be self-contained with clear pass/fail criteria.

**Too coarse — split when:**
- Description contains multiple verbs joined by "and"
- Given section has more than 5 preconditions
- More than 8 assertions in a single postcondition block
- Multiple distinct actions implied but only one expressed

**Too fine — merge when:**
- Specifies implementation detail (technology choices, internal algorithms)
- Delivers no observable user or agent value on its own
- Tests a single field validation that belongs as an assertion inside a larger behavior

---

## Project type calibration

| Project type | Granularity |
|---|---|
| API | Endpoint + verb level (field validation as assertions, not separate behaviors) |
| CLI | Command/subcommand level |
| UI | Interaction level (not screen, not component) |
| Worker | Event/message level |
| Agent | Reasoning/action level |

---

## When to split a spec

Split when a spec exceeds roughly 15 behaviors or covers multiple bounded contexts. Each spec should map to one capability.

## When to merge behaviors

Merge when two behaviors share the same given/when/then structure and differ only in a single input value. Use a parameterized behavior with representative examples instead.

---

## Work type classification

| Work type | Spec change |
|-----------|-------------|
| New feature | New spec with all behaviors |
| Enhancement | Add behaviors to existing spec, bump minor version |
| Bug fix | Add an `error_case` behavior reproducing the bug |
| Refactor | No spec change (behavior preserved) |
| Deprecation | Mark behaviors removed, bump major version |

---

## Phase collapse rule

When a system has sequential internal phases (validate → process → cleanup), do not write one behavior per phase. Instead:

1. Identify the end-state observable to the user
2. Write one behavior describing that end-state
3. Internal phases become implementation, not spec

The `when` trigger is the completion event, not each phase.

```
# Bad: internal phases exposed as behaviors
behavior paginate-generates-batches [happy_path]
behavior worker-processes-batch [happy_path]
behavior purge-deletes-stale [happy_path]

# Good: user-observable outcomes
behavior all-documents-indexed [happy_path]
behavior stale-documents-removed [happy_path]
```

---

## Writing specs from code

When inferring specs from existing source code:

1. Read the code to understand the domain — then stop reading code
2. Ask: "What does a user or caller observe before and after?"
3. Write behaviors answering only that question
4. If a behavior mentions an internal component name (Lambda, queue, database table, cache key, worker function), it is a smell — rewrite it

Technical constraints found in code (retry counts, timeout values, ID formats, token limits, model names) belong in NFR files, not FR behaviors.

---

## Entity format

Entity names (`@repo`, `@user`, `@input`) aid readability — keep them. Entity field shapes risk prescribing the data model. Rules:

- Use domain-meaningful field names (`id`, `repositoryId`), not implementation names (`mongoId`, `coreId`, `executionArn`)
- Keep entity shapes minimal — only fields referenced in `when` or `then`
- If a field name maps to a specific technology, abstract it:

```
# Bad
@user = User { repositoryMongoId: "repo-789" }

# Good
@user = User { id: "repo-789" }
```

---

## Smell detection

Detect these smells before writing tests. A smell in the spec will produce a misleading test.

### Ambiguity

Signal: subjective language, ambiguous pronouns, vague modifiers.
Action: rewrite with measurable criteria and explicit subjects.

### Non-verifiability

Signal: cannot define a concrete test for the assertion.
Action: sharpen into testable criteria or move to a guideline.

### NASA forbidden words

Signal: flexible, easy, sufficient, safe, adequate, user-friendly, fast, portable, robust, or any `-ly`/`-ize` adverb or verb.
Action: replace with a quantified constraint or measurable threshold. Move to an NFR if it crosses the line.

### Compound behavior

Signal: "and" joining distinct actions in a behavior description.
Action: split into separate behaviors, one action each.

### Missing error cases

Signal: only `happy_path` behaviors, no `error_case`.
Action: add `error_case` and `edge_case` behaviors for each happy path.

### Implicit preconditions

Signal: the `when` block assumes state not declared in `given`.
Action: make every precondition explicit in the `given` block.

### Prose assertion overuse

Signal: more than 20% of assertions are prose (untyped).
Action: convert to typed assertion operators where possible.

### Missing NFR references

Signal: data access with no security NFR, API endpoint with no performance NFR.
Action: add appropriate NFR constraint references.

### Implementation leakage

Signal: the spec describes HOW, not WHAT — "uses HashMap", "calls REST endpoint", "writes to S3".

Apply two tests to each behavior:

**Observer Test** — ask "who observes this outcome?"
- Valid observers: API caller, end user, ops team (via dashboards, which become NFRs).
- Invalid observers: internal function, downstream queue consumer, database trigger, another microservice.
- If only internal components observe the outcome, collapse it into a user-observable outcome, move it to an NFR, or drop it.

**Swap Test** — ask "if I replaced the technology, would this behavior break?"
- "SQS fan-out completes" → swap to HTTP → breaks → SMELL
- "All documents are searchable after upload" → swap anything → holds → GOOD

If swapping the implementation breaks the behavior description, it is a leak. Rewrite at the behavioral level — describe observable outcomes only.

### Phase leak

Signal: sequential behaviors matching internal processing phases.
Action: collapse into one behavior per user-observable end-state.

---

## NFR design

### Seven categories

Not every project needs all seven. The categories are fixed — only the constraints within them change.

| Category | When it applies |
|----------|-----------------|
| `performance` | Data access (read/write), API endpoints, user-facing operations |
| `reliability` | State changes, external service calls |
| `security` | Data access, any endpoint or operation |
| `observability` | Logging/metrics emission |
| `scalability` | Batch/bulk operations, load-sensitive paths |
| `cost` | Batch/bulk operations, per-unit resource usage |
| `operability` | Infrastructure changes, deployment automation |

### Two constraint types

**Metric** — quantitative, threshold-based. Has a measurable value and a pass/fail threshold (e.g., `latency p95 < 500ms`). Can be overridden at behavior level when `overridable yes`.

**Rule** — structural, binary pass/fail. An invariant that holds or doesn't (e.g., "all data access scoped by repository_id"). Cannot be overridden.

Every constraint must include a verification method specific enough to generate a test.

### FR/NFR decision tree

```
Is this a constraint on HOW the system works?
  YES → Does it apply to a single behavior?
    YES → NFR constraint pinned to that behavior (behavior-level)
    NO  → NFR constraint at spec level
  NO → Is the outcome observable by the API caller or end user?
    YES → FR behavior
    NO  → Either NFR or drop it
```

Requirements with both FR and NFR characteristics (auth, encryption, accessibility) should be decomposed: the behavior goes in the FR spec, the quality constraint goes in the NFR file.

### Coverage gap detection

When a functional spec involves these concerns, flag missing NFRs:

| Concern | Missing NFR |
|---------|-------------|
| Data access (read/write) | security (isolation, auth) |
| API endpoints | performance (latency budgets) |
| State changes | reliability (consistency, recovery) |
| External service calls | reliability (fault tolerance) |
| User-facing operations | performance (response time) |
| Infrastructure changes | operability (deployment, rollback) |
| Batch/bulk operations | cost (limits), scalability (concurrency) |
| Logging/metrics emission | observability (structured logging) |

---

See also:
- [spec-format.md](spec-format.md) — full grammar reference
- [nfr-format.md](nfr-format.md) — NFR grammar reference
- [methodology.md](methodology.md) — five-phase workflow
- `minter guide authoring` — authoring guide in the terminal
- `minter guide smells` — smell detection reference in the terminal
- `minter guide nfr` — NFR design guide in the terminal
