# NFR Format (.nfr)

The `.nfr` file format defines non-functional quality constraints for one of seven categories.

**Indentation:** 2 spaces. Tab characters are rejected.

---

## File structure

```
nfr <category> v<version>
title "<title>"

description
  <free text>

motivation
  <free text>

constraint <name> [metric]     # one or more constraints required
  ...

constraint <name> [rule]
  ...
```

---

## Header

```
nfr performance v1.0.0
title "Performance Requirements"
```

- **category** — must be one of the 7 valid categories (see below).
- **version** — valid semver. The `v` prefix is required.
- **title** — double-quoted human-readable name.

---

## NFR categories

Exactly seven categories are valid. Category names are fixed; only the constraints within them change.

| Category | Scope |
|----------|-------|
| `performance` | Latency, throughput, resource usage, response time |
| `reliability` | Availability, consistency, fault tolerance, recovery |
| `security` | Isolation, authentication, authorization, encryption |
| `observability` | Logging, metrics, tracing, alerting, health checks |
| `scalability` | Load limits, growth paths, concurrency, ceilings |
| `cost` | Infrastructure budgets, per-unit economics, efficiency |
| `operability` | Deployment, IaC, rollback, maintenance, CI/CD |

Using a category not in this list is a parse error.

---

## Constraints

```
constraint <name> [<type>]
```

- **name** — kebab-case. No leading, trailing, or consecutive hyphens.
- **type** — `metric` or `rule`.

Every constraint has four required parts: description, body (metric or rule), violation severity, and overridable flag.

---

## Metric constraints

A metric constraint defines a measurable threshold.

```
constraint api-response-time [metric]
  "API endpoints must respond within acceptable latency"

  metric "p95 response latency"
  threshold < 500ms

  verification
    environment staging, production
    benchmark "Run k6 load test at 100 RPS for 5 minutes"
    dataset "Standard user fixtures"
    pass "3-of-5 runs meet threshold"

  violation high
  overridable yes
```

| Field | Required | Description |
|-------|----------|-------------|
| `metric "<text>"` | Yes | What is being measured |
| `threshold <op> <value>` | Yes | The pass/fail threshold |
| `verification` block | Yes | How to verify the metric |

**Threshold operators:** `<`, `>`, `<=`, `>=`, `==`

**Verification block fields:**

| Field | Required | Notes |
|-------|----------|-------|
| `environment <env>, ...` | Yes | Comma-separated environments |
| `benchmark "<text>"` | Yes (at least one) | Benchmark procedure description |
| `dataset "<text>"` | No | Data requirements |
| `pass "<text>"` | Yes (at least one) | Pass criteria |

Metric constraints can be overridden at behavior level when `overridable yes`. See [nfr-cross-reference.md](nfr-cross-reference.md).

---

## Rule constraints

A rule constraint defines a binary pass/fail policy.

```
constraint tls-required [rule]
  "All endpoints must use TLS"

  rule
    Every HTTP endpoint must redirect to HTTPS or
    reject plain HTTP connections.

  verification
    static "Check nginx/load balancer config for TLS termination"
    runtime "Attempt plain HTTP connection and verify rejection"

  violation critical
  overridable no
```

| Field | Required | Description |
|-------|----------|-------------|
| `rule` block | Yes | Free-text rule statement |
| `verification` block | Yes | How to verify the rule |

**Verification block fields:**

| Field | Required | Notes |
|-------|----------|-------|
| `static "<text>"` | At least one static or runtime | Static analysis check |
| `runtime "<text>"` | At least one static or runtime | Runtime check |

Rule constraints cannot be overridden. Set `overridable no`.

---

## Violation severity

Required for every constraint.

| Severity | When to use |
|----------|-------------|
| `critical` | System is unsafe or non-compliant without it |
| `high` | Significant user or business impact |
| `medium` | Moderate impact, should be fixed in normal cycle |
| `low` | Minor, can be addressed opportunistically |

---

## Overridable flag

Required for every constraint.

| Value | Meaning |
|-------|---------|
| `yes` | Behaviors may provide a stricter threshold override |
| `no` | Threshold cannot be overridden |

Only metric constraints make sense as `overridable yes`. Rule constraints should always be `overridable no` — a rule is either satisfied or it isn't.

---

## Annotated example

```
nfr performance v1.0.0
title "Performance Requirements"

description
  Latency and throughput targets for API endpoints.

motivation
  Ensures consistent user experience under load.


constraint api-response-time [metric]
  "API endpoints must respond within acceptable latency"

  metric "p95 response latency"
  threshold < 500ms

  verification
    environment staging, production
    benchmark "Run k6 load test at 100 RPS for 5 minutes"
    dataset "Standard user fixtures"
    pass "3-of-5 runs meet threshold"

  violation high
  overridable yes


constraint tls-required [rule]
  "All endpoints must use TLS"

  rule
    Every HTTP endpoint must redirect to HTTPS or
    reject plain HTTP.

  verification
    static "Check load balancer config for TLS termination"
    runtime "Attempt plain HTTP connection and verify rejection"

  violation critical
  overridable no
```

---

## Semantic rules

| Rule | Description |
|------|-------------|
| `valid-semver` | Version must be valid semver |
| `unique-constraint-names` | No duplicate constraint names within a file |
| `kebab-case-constraint-name` | Constraint names must be valid kebab-case |

---

See also:
- [spec-format.md](spec-format.md) — the `.spec` grammar
- [nfr-cross-reference.md](nfr-cross-reference.md) — how specs reference NFR constraints
- [authoring-guide.md](authoring-guide.md#nfr-design) — NFR design guide
- `minter format nfr` — print the grammar reference in the terminal
