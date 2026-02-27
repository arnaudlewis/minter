# Minter DSL Reference

This is the authoritative formal reference for both the Functional Requirement (.spec) and Non-Functional Requirement (.nfr) DSL formats parsed and validated by minter.

---

## Table of Contents

- [FR (.spec) Format](#fr-spec-format)
  - [File Structure](#file-structure)
  - [Spec Header](#spec-header)
  - [Title](#title)
  - [Description and Motivation](#description-and-motivation)
  - [NFR Section (Spec-Level)](#nfr-section-spec-level)
  - [Behaviors](#behaviors)
  - [Behavior Categories](#behavior-categories)
  - [Behavior Description](#behavior-description)
  - [NFR Section (Behavior-Level)](#nfr-section-behavior-level)
  - [Given Section](#given-section)
  - [When Section](#when-section)
  - [Then Section](#then-section)
  - [Assertion Operators](#assertion-operators)
  - [Dependencies](#dependencies)
- [NFR (.nfr) Format](#nfr-nfr-format)
  - [NFR File Structure](#nfr-file-structure)
  - [NFR Header](#nfr-header)
  - [NFR Categories](#nfr-categories)
  - [Constraints](#constraints)
  - [Metric Constraints](#metric-constraints)
  - [Rule Constraints](#rule-constraints)
  - [Violation Severity](#violation-severity)
  - [Overridable](#overridable)
- [NFR Cross-Reference Rules](#nfr-cross-reference-rules)
  - [Spec-Level References](#spec-level-references)
  - [Behavior-Level References](#behavior-level-references)
  - [Override Rules](#override-rules)
  - [Value Normalization](#value-normalization)
  - [Containment Rule](#containment-rule)
- [Validation Rules Summary](#validation-rules-summary)
  - [Parse Rules](#parse-rules)
  - [FR Semantic Rules](#fr-semantic-rules)
  - [NFR Semantic Rules](#nfr-semantic-rules)
  - [Cross-Reference Rules](#cross-reference-rules)

---

## FR (.spec) Format

### File Structure

A `.spec` file has the following top-level structure, in this exact order:

```
spec <name> v<version>
title "<title>"

description
  <free text>

motivation
  <free text>

nfr                                    # optional
  <references>

behavior <name> [<category>]           # one or more
  ...

depends on <spec-name> >= <version>    # zero or more
```

All sections must appear in the order shown. Blank lines and comment lines (starting with `#`) are allowed between sections and are ignored by the parser.

### Spec Header

```
spec <name> v<version>
```

- `<name>` -- the spec identifier. Must be valid kebab-case (lowercase ASCII alphanumeric + hyphens, no leading/trailing/double hyphens).
- `v<version>` -- the version. The `v` prefix is stripped during parsing. The remaining string must be valid [semver](https://semver.org/) (e.g., `1.0.0`, `0.2.1-alpha.1`).

Example:

```
spec user-registration v1.0.0
```

### Title

```
title "<title>"
```

A double-quoted string. The title is a human-readable name for the spec.

Example:

```
title "User Registration"
```

### Description and Motivation

```
description
  <free text>

motivation
  <free text>
```

Both are required sections. The keyword appears alone on its own line, followed by indented body lines (2-space indent). All indented lines are collected and joined with newlines, with leading/trailing whitespace trimmed from each line.

Example:

```
description
  Allows new users to create an account by
  providing an email address and password.

motivation
  Self-service registration reduces onboarding
  friction and support costs.
```

### NFR Section (Spec-Level)

```
nfr
  <category>                     # whole-file reference
  <category>#<constraint>        # anchor reference
```

Optional. Appears after `motivation` and before `behavior` declarations. The `nfr` keyword appears alone on its own line, followed by indented reference lines.

Two reference forms are supported:

| Form | Meaning |
|------|---------|
| `<category>` | Whole-file reference -- imports all constraints from the category's `.nfr` file |
| `<category>#<constraint>` | Anchor reference -- targets a single named constraint |

Overrides are **not allowed** at spec level (parse error).

Example:

```
nfr
  performance
  security#tls-required
```

### Behaviors

```
behavior <name> [<category>]
```

At least one behavior is required. A behavior represents a single user-observable outcome.

The behavior block contains, in order:

1. **Description** (required) -- quoted string
2. **NFR section** (optional) -- behavior-level NFR references
3. **Given section** (required) -- preconditions
4. **When section** (required) -- the action under test
5. **Then section(s)** (required) -- one or more postcondition blocks

### Behavior Categories

The category is specified in square brackets after the behavior name:

| Category | Description |
|----------|-------------|
| `happy_path` | The expected successful outcome |
| `error_case` | An expected error condition |
| `edge_case` | A boundary or unusual condition |

At least one `happy_path` behavior is required per spec (semantic rule).

### Behavior Description

```
behavior register-user [happy_path]
  "The system creates a new user account and returns a confirmation"
```

A double-quoted string on the line immediately after the behavior header (indented by 2 spaces). This is the human-readable description of what the behavior does.

### NFR Section (Behavior-Level)

```
  nfr
    <category>#<constraint>                         # anchor reference
    <category>#<constraint> <operator> <value>       # override
```

Optional. Appears after the behavior description and before `given`. The `nfr` keyword appears alone on an indented line, followed by further-indented reference lines.

At behavior level:

- Only **anchor references** are allowed (must include `#`). Whole-file references produce a parse error.
- Optional **override** syntax appends an operator and value after the reference.

Example:

```
  nfr
    performance#api-response-time < 200ms
    security#tls-required
```

### Given Section

```
  given
    <precondition text>
    @<alias> = <Entity> { <key>: <value>, ... }
```

Required. The `given` keyword appears alone on an indented line. Body lines are indented under it.

Two precondition forms:

**Prose** -- free-text description of a precondition:

```
  given
    The user has a valid session
    The database is available
```

**Alias declaration** -- defines a named test fixture:

```
  given
    @user = User { email: "alice@example.com", role: "admin" }
```

Alias syntax:
- `@<alias>` -- the alias name, prefixed with `@`
- `<Entity>` -- the entity type name (must start with an uppercase letter)
- `{ <key>: <value>, ... }` -- comma-separated key-value properties, enclosed in braces

Values in alias properties may be quoted or unquoted. Properties are parsed as `key: value` pairs separated by commas.

### When Section

```
  when <action-name>
    <input> = <value>
    <input> = @<alias>.<field>
```

Required. The `when` keyword followed by the action name on the same line. Inputs are indented below.

Two input forms:

**Literal value**:

```
  when register-user
    email = "alice@example.com"
    password = "secret123"
```

**Alias reference** -- references a field from a `given` alias:

```
  when register-user
    email = @user.email
```

Alias references use the format `@<alias>.<field>`. Both the alias name and field name must be non-empty.

### Then Section

```
  then returns <type>
    assert <field> <operator> <value>

  then emits <channel>
    assert <field> <operator> <value>

  then side_effect
    assert <prose description>
```

At least one `then` block is required per behavior. Multiple `then` blocks are allowed.

Three postcondition kinds:

| Kind | Syntax | Description |
|------|--------|-------------|
| `returns` | `then returns <type>` | The action returns a value of the given type |
| `emits` | `then emits <channel>` | The action emits an event on the given channel |
| `side_effect` | `then side_effect` | The action produces a side effect |

Each `then` block contains zero or more `assert` lines.

### Assertion Operators

Seven assertion operators are supported:

| Operator | Syntax | Description |
|----------|--------|-------------|
| `==` | `assert <field> == <value>` | Field equals the literal value |
| `==` (ref) | `assert <field> == @<alias>.<field>` | Field equals an alias field value |
| `is_present` | `assert <field> is_present` | Field exists and is non-null |
| `contains` | `assert <field> contains <value>` | Field contains the given substring or element |
| `in_range` | `assert <field> in_range <min>..<max>` | Field falls within the inclusive range |
| `matches_pattern` | `assert <field> matches_pattern <pattern>` | Field matches the given pattern |
| `>=` | `assert <field> >= <value>` | Field is greater than or equal to the value |

**Prose assertion** -- any assert line that does not match a known operator pattern is treated as a prose (free-text) assertion:

```
  then side_effect
    assert the user receives a welcome email
```

If a three-token assertion uses an unknown operator, it is rejected with an error listing the valid operators.

### Dependencies

```
depends on <spec-name> >= <version>
```

Zero or more. Must appear after all behavior declarations, at the end of the file. Each dependency specifies a spec name and a minimum version constraint using the `>=` operator.

Example:

```
depends on auth-service >= 1.0.0
depends on notification-service >= 2.1.0
```

Dependency resolution uses the following rules:
- Cycle detection: if a spec appears in its own dependency chain, a cycle error is reported.
- Depth limit: resolution stops at depth 256 with an error.
- Dependencies are resolved against sibling `.spec` files in the same directory.

---

## NFR (.nfr) Format

### NFR File Structure

A `.nfr` file has the following top-level structure, in this exact order:

```
nfr <category> v<version>
title "<title>"

description
  <free text>

motivation
  <free text>

constraint <name> [metric]            # one or more
  ...

constraint <name> [rule]              # one or more
  ...
```

At least one constraint is required.

### NFR Header

```
nfr <category> v<version>
```

- `<category>` -- must be one of the 7 valid NFR categories (see below).
- `v<version>` -- the `v` prefix is stripped. Must be valid semver.

Example:

```
nfr performance v1.0.0
```

### NFR Categories

There are exactly 7 valid NFR categories:

| Category | Scope |
|----------|-------|
| `performance` | Latency, throughput, resource usage, response time |
| `reliability` | Availability, consistency, fault tolerance, recovery |
| `security` | Isolation, authentication, authorization, encryption |
| `observability` | Logging, metrics, tracing, alerting, health checks |
| `scalability` | Load limits, growth paths, concurrency, ceilings |
| `cost` | Infrastructure budgets, per-unit economics, efficiency |
| `operability` | Deployment, IaC, rollback, maintenance, CI/CD |

Using a category not in this list produces a parse error.

### Constraints

```
constraint <name> [<type>]
```

- `<name>` -- the constraint identifier. Must be valid kebab-case (semantic rule).
- `<type>` -- either `metric` or `rule`. Using any other type produces a parse error.

Each constraint has:
1. **Description** -- a double-quoted string on the next line
2. **Body** -- either a metric body or a rule body (determined by the type)
3. **Violation severity** -- required
4. **Overridable flag** -- required

### Metric Constraints

```
constraint api-response-time [metric]
  "P95 API response time limit"

  metric "p95-latency"
  threshold < 500ms

  verification
    environment staging, production
    benchmark "Load test with 1000 concurrent users"
    dataset "Standard test dataset"
    pass "3 of 5 runs must meet threshold"

  violation high
  overridable yes
```

Metric constraint body fields:

| Field | Syntax | Required | Description |
|-------|--------|----------|-------------|
| `metric` | `metric "<description>"` | Yes | What is being measured |
| `threshold` | `threshold <operator> <value>` | Yes | The pass/fail threshold |
| `verification` | Block with indented fields | Yes | How to verify the metric |

**Threshold operators**: `<`, `>`, `<=`, `>=`, `==`

Using `!=` or any other operator produces a parse error listing the valid operators.

**Verification block** (metric):

| Field | Syntax | Required | Multiple allowed |
|-------|--------|----------|------------------|
| `environment` | `environment <env1>, <env2>, ...` | Yes | No (comma-separated list) |
| `benchmark` | `benchmark "<description>"` | Yes (at least one) | Yes |
| `dataset` | `dataset "<description>"` | No | Yes |
| `pass` | `pass "<criteria>"` | Yes (at least one) | Yes |

Verification fields are indented by 4 spaces (2 levels of indentation).

### Rule Constraints

```
constraint tls-required [rule]
  "All API traffic must use TLS 1.2+"

  rule
    All inbound and outbound API connections
    must use TLS 1.2 or higher.

  verification
    static "Certificate chain validation in CI"
    runtime "TLS version check in health endpoint"

  violation critical
  overridable no
```

Rule constraint body fields:

| Field | Syntax | Required | Description |
|-------|--------|----------|-------------|
| `rule` | Block with indented text | Yes | The rule statement as free text |
| `verification` | Block with indented fields | Yes | How to verify the rule |

The `rule` keyword appears alone on its own line. The rule body consists of indented lines (4-space indent) that are collected and joined with newlines.

**Verification block** (rule):

| Field | Syntax | Multiple allowed |
|-------|--------|------------------|
| `static` | `static "<description>"` | Yes |
| `runtime` | `runtime "<description>"` | Yes |

At least one `static` or `runtime` entry is required.

### Violation Severity

```
violation <severity>
```

Required for every constraint. Valid severity values:

| Severity |
|----------|
| `critical` |
| `high` |
| `medium` |
| `low` |

Using any other value produces a parse error.

### Overridable

```
overridable <yes|no>
```

Required for every constraint. Controls whether behavior-level references can override this constraint's threshold.

- `yes` -- behaviors may provide a stricter threshold override
- `no` -- the constraint threshold cannot be overridden

---

## NFR Cross-Reference Rules

Cross-reference validation runs in deep mode only (directory validation or single-file with `--deep`). It checks that all NFR references in a `.spec` file resolve to actual constraints in `.nfr` files.

### Spec-Level References

```
nfr
  performance                       # whole-file reference
  performance#api-response-time     # anchor reference
```

At spec level, both whole-file and anchor references are allowed. Overrides are **not** allowed (parse error).

- A **whole-file reference** (`category`) imports all constraints from the corresponding `.nfr` file.
- An **anchor reference** (`category#constraint`) targets a single named constraint.

### Behavior-Level References

```
  nfr
    performance#api-response-time                   # anchor only
    performance#api-response-time < 200ms           # with override
```

At behavior level:

- Only **anchor references** are allowed. Whole-file references produce a parse error.
- An optional **override** may follow the anchor reference, consisting of an operator and a value.

### Override Rules

There are 5 override validation rules, checked in order:

1. **Overridable check** -- The target constraint must have `overridable yes`. If `overridable no`, the override is rejected.

2. **Metric only** -- Only metric constraints support overrides. Attempting to override a rule constraint is rejected.

3. **Operator match** -- The override operator must match the original threshold operator exactly. For example, if the constraint uses `<`, the override must also use `<`.

4. **Stricter value** -- The override value must be stricter than the default threshold:
   - For `<` and `<=`: the override value must be **less than** the original value.
   - For `>` and `>=`: the override value must be **greater than** the original value.
   - For `==`: the override value must **equal** the original value.

5. **Behavior-level only** -- Overrides are only allowed in behavior-level `nfr` sections, never at spec level (this is enforced at parse time).

### Value Normalization

When comparing override values against default thresholds, values are normalized to a canonical numeric form:

| Unit suffix | Normalization |
|-------------|---------------|
| `ms` | Stripped, value used as-is (milliseconds) |
| `s` | Converted to milliseconds (value * 1000) |
| `%` | Stripped, value used as-is (percentage) |
| `KB` | Stripped, value used as-is (kilobytes) |
| `MB` | Converted to kilobytes (value * 1000) |
| `GB` | Converted to kilobytes (value * 1000000) |
| (none) | Bare number parsed as-is |

Examples:
- `500ms` normalizes to `500`
- `2s` normalizes to `2000` (milliseconds)
- `1MB` normalizes to `1000` (kilobytes)
- `2GB` normalizes to `2000000` (kilobytes)
- `99.9%` normalizes to `99.9`

### Containment Rule

Every NFR category referenced at behavior level must also appear in the spec-level `nfr` section. This rule ensures the spec-level section acts as a table of contents for all NFR categories in scope.

For example, if a behavior references `performance#api-response-time`, then `performance` (or `performance#api-response-time` or any other `performance` anchor) must appear in the spec-level `nfr` section.

The containment rule is checked by FR semantic validation and always runs (not just in deep mode).

---

## Validation Rules Summary

### Parse Rules

These rules are enforced during parsing of both `.spec` and `.nfr` files:

| Rule | Description |
|------|-------------|
| No tab indentation | Tab characters at the start of any line are rejected. Use 2-space indentation. |
| Non-empty input | Empty files are rejected. |
| Section order | Sections must appear in the defined order. |
| Required sections | All required sections must be present (description, motivation, at least one behavior/constraint). |
| Quoted strings | Title and description strings must be properly quoted with double quotes. |
| Valid behavior category | Must be `happy_path`, `error_case`, or `edge_case`. |
| Valid constraint type | Must be `metric` or `rule`. |
| Valid NFR category | Must be one of the 7 valid categories. |
| Valid threshold operator | Must be `<`, `>`, `<=`, `>=`, or `==`. |
| Valid violation severity | Must be `critical`, `high`, `medium`, or `low`. |
| Valid overridable value | Must be `yes` or `no`. |
| No spec-level overrides | Override syntax in the spec-level `nfr` section is a parse error. |
| No behavior-level whole-file refs | Whole-file references (without `#`) in behavior-level `nfr` sections are a parse error. |
| Metric verification completeness | Metric verification requires `environment`, at least one `benchmark`, and at least one `pass`. |
| Rule verification completeness | Rule verification requires at least one `static` or `runtime` entry. |
| Assertion format | `assert` must be followed by a field and operator. Bare `assert` or `assert ==` are rejected. |
| Alias declaration format | Must follow `@<alias> = <Entity> { ... }` with entity starting with an uppercase letter. |
| Alias reference format | Must follow `@<alias>.<field>` with both parts non-empty. |
| Dependency format | Must follow `depends on <name> >= <version>`. |

### FR Semantic Rules

These rules are checked after successful parsing of a `.spec` file:

| Rule ID | Description |
|---------|-------------|
| `kebab-case-name` | Spec name must be valid kebab-case: lowercase ASCII alphanumeric + hyphens, no leading/trailing/double hyphens. |
| `valid-semver` | Version must be valid semver (e.g., `1.0.0`). The `v` prefix is stripped before validation. |
| `unique-behavior-names` | No two behaviors in the same spec may share a name. |
| `at-least-one-happy-path` | The spec must contain at least one behavior with category `happy_path`. |
| `nfr-containment` | Every NFR category referenced at behavior level must also appear in the spec-level `nfr` section. Only checked when the spec has a non-empty spec-level `nfr` section. |
| `unique-aliases` | Within a single behavior, no two alias declarations may share a name. |
| `alias-refs-resolve` | Every alias reference (`@alias.field`) in `when` inputs and `then` assertions must refer to an alias declared in the behavior's `given` section. |

### NFR Semantic Rules

These rules are checked after successful parsing of a `.nfr` file:

| Rule ID | Description |
|---------|-------------|
| `valid-semver` | Version must be valid semver. |
| `unique-constraint-names` | No two constraints in the same NFR file may share a name. |
| `kebab-case-constraint-name` | Every constraint name must be valid kebab-case. |

### Cross-Reference Rules

These 9 rules are checked during deep validation when NFR `.nfr` files are available:

| # | Rule | Description |
|---|------|-------------|
| 1 | Category must exist | Every referenced NFR category must correspond to a `.nfr` file that declares that category. |
| 2 | Anchor must exist | Every anchor reference (`category#constraint`) must match a constraint name in the corresponding `.nfr` file. |
| 3 | Containment | Behavior-level NFR categories must appear in the spec-level `nfr` section (FR semantic rule). |
| 4 | Overridable check | An override is only allowed if the target constraint has `overridable yes`. |
| 5 | Metric only | Overrides are only allowed on metric constraints, not rule constraints. |
| 6 | Operator match | The override operator must exactly match the original threshold operator. |
| 7 | Stricter value | The override value must be strictly tighter than the default threshold. |
| 8 | No overrides at spec level | Override syntax in the spec-level `nfr` section is a parse error. |
| 9 | No whole-file refs at behavior level | Whole-file references (without `#`) in behavior-level `nfr` sections are a parse error. |

Rules 8 and 9 are enforced at parse time. Rules 1-2 and 4-7 are enforced during cross-reference validation. Rule 3 is enforced during FR semantic validation.

---

## Complete Examples

### Minimal .spec file

```
spec hello-world v1.0.0
title "Hello World"

description
  A minimal spec with one behavior.

motivation
  Demonstrates the basic spec structure.

behavior greet-user [happy_path]
  "The system returns a greeting message"

  given
    The user provides their name

  when greet
    name = "Alice"

  then returns Greeting
    assert message == "Hello, Alice!"
```

### Full-featured .spec file

```
spec user-registration v2.0.0
title "User Registration"

description
  Allows new users to create accounts.

motivation
  Self-service registration reduces support costs.

nfr
  performance
  security#tls-required

behavior register-new-user [happy_path]
  "The system creates a new user and returns confirmation"

  nfr
    performance#api-response-time < 200ms

  given
    @user = User { email: "alice@example.com", role: "member" }
    The email is not already registered

  when register
    email = @user.email
    password = "secure-password"

  then returns UserConfirmation
    assert id is_present
    assert email == @user.email
    assert status == "active"

  then emits user-created
    assert user_id is_present

behavior reject-duplicate-email [error_case]
  "The system rejects registration with an existing email"

  given
    The email is already registered

  when register
    email = "existing@example.com"
    password = "password"

  then returns Error
    assert code == "DUPLICATE_EMAIL"
    assert message contains "already registered"

depends on auth-service >= 1.0.0
```

### Minimal .nfr file

```
nfr performance v1.0.0
title "Performance Requirements"

description
  Defines latency and throughput constraints.

motivation
  Ensures acceptable user experience under load.

constraint api-response-time [metric]
  "P95 API response time"

  metric "p95-latency"
  threshold < 500ms

  verification
    environment staging, production
    benchmark "1000 concurrent users for 5 minutes"
    pass "3 of 5 runs meet threshold"

  violation high
  overridable yes
```

### .nfr file with both constraint types

```
nfr security v1.0.0
title "Security Requirements"

description
  Security policies for the platform.

motivation
  Protect user data and ensure compliance.

constraint tls-required [rule]
  "All API traffic must use TLS 1.2+"

  rule
    All inbound and outbound API connections
    must use TLS 1.2 or higher. Plain HTTP
    connections must be rejected.

  verification
    static "Certificate chain validation in CI"
    runtime "TLS version check in health endpoint"

  violation critical
  overridable no

constraint auth-latency [metric]
  "Authentication request latency"

  metric "p99-auth-latency"
  threshold < 200ms

  verification
    environment staging
    benchmark "Auth flow under load"
    dataset "1000 test credentials"
    pass "All runs under threshold"

  violation high
  overridable yes
```
