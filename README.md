[![Latest Release](https://img.shields.io/github/v/release/arnaudlewis/minter?label=version&color=blue)](https://github.com/arnaudlewis/minter/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/arnaudlewis/minter/total?color=green)](https://github.com/arnaudlewis/minter/releases)
[![Homebrew](https://img.shields.io/badge/homebrew-arnaudlewis%2Ftap%2Fminter-orange)](https://github.com/arnaudlewis/homebrew-tap)
![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)

# <img src="assets/logo-static.svg" alt="" width="28" height="28" /> minter

The deterministic validation gate for spec-driven development.

Minter is a CLI that parses `.spec` and `.nfr` files — a structured DSL for defining behavioral contracts and non-functional requirements. It validates syntax and semantics, resolves dependency graphs, cross-validates NFR references, and gives you instant feedback while authoring.

The spec format has exactly one primitive: **behavioral specs that depend on other behavioral specs**. There is no type system — data shapes are specs whose behaviors describe what valid instances look like. There is no error catalog — error behavior is expressed directly as behaviors. Everything is given/when/then. One concept to learn, one concept to validate, one concept to generate tests from.

```
Human intent --> .spec (DSL) --> minter validate (deterministic) --> downstream agents read .spec
```

## Install

```bash
brew install arnaudlewis/tap/minter
```

This installs both `minter` (CLI) and `minter-mcp` (MCP server for AI agents).

<details>
<summary>Manual download</summary>

Download the archive for your platform from the [latest release](https://github.com/arnaudlewis/minter/releases/latest), extract it, and place `minter` and `minter-mcp` on your `PATH`.

Each archive contains: `minter`, `minter-mcp`, `LICENSE`, and `README.md`. SHA-256 checksums are in `SHA256SUMS.txt`.

</details>

## Setup MCP for Claude Code

```bash
claude mcp add minter minter-mcp
```

The MCP server embeds the spec-driven methodology directly into your AI agent's workflow. It includes a built-in authoring guide that teaches agents how to think in behaviors, structure NFR constraints, and follow the one-primitive philosophy — so every spec an agent writes follows the same rigor as one you'd write yourself.

Tools available to the agent: validate, inspect, scaffold, format, graph, coverage, initialize project, and methodology guide (covering workflow, authoring, smells, NFR design, context management, and coverage tagging).

## Getting started

### Build specs with your agent

Once the MCP is set up, your agent already knows the methodology, the DSL grammar, and every validation rule. You don't need to learn the format first — just describe what you want to build.

**Learn the methodology:**

> "Read the minter methodology guide and explain spec-driven development to me."

**Initialize a project:**

> "Initialize a minter spec project in this repo."

**Break down a feature into specs:**

> "I want to build a user authentication system with registration, login, and password reset. Help me break this down into behavioral specs."

**Add non-functional requirements:**

> "Create performance and security NFR files for my project. API responses should be under 200ms and all endpoints need authentication."

**Validate and explore:**

> "Validate all specs in my project and show me the dependency graph."

**Extend an existing spec:**

> "Read the user-auth spec and add edge cases for rate limiting and expired tokens."

The agent handles scaffolding, formatting, cross-reference validation, and dependency resolution through the MCP tools. You focus on what the system should do — the agent handles the DSL.

### CLI quick start

**1. Scaffold a spec**

```bash
minter scaffold spec > specs/my-feature.spec
```

**2. Validate it**

```bash
minter validate specs/my-feature.spec
```

```
✓ my-feature v0.1.0 (1 behavior)
```

**3. Explore the dependency graph**

```bash
minter graph specs/
```

```
3 specs, 11 behaviors, 2 NFR categories, 5 constraints

my-feature v0.1.0 (1 behavior)
├── user-auth v1.0.0 (3 behaviors)
└── [nfr] performance v1.0.0 (2 constraints)
```

## Example

The [`examples/`](examples/) directory contains a complete spec project — a task management API with user authentication, CRUD behaviors, and performance NFR bindings.

```
examples/specs/
├── user-auth.spec           # 4 behaviors — register, login, security NFR anchors
├── task-management.spec     # 5 behaviors — depends on user-auth, whole-file performance NFR
└── nfr/
    ├── performance.nfr      # 3 constraints — response time, throughput, bounded queries
    └── security.nfr         # 2 constraints — password hashing, brute-force protection
```

```bash
minter graph examples/specs/
```

```
task-management v1.0.0 (5 behaviors)
├── user-auth v1.0.0 (4 behaviors)
│   └── [nfr] security v1.0.0 (2 constraints)
│       ├── #brute-force-protection
│       └── #password-hashing
└── [nfr] performance v1.0.0 (3 constraints)
```

See [`examples/README.md`](examples/README.md) for the full walkthrough.

## Commands

| Command | Purpose |
|---------|---------|
| `validate` | Validate spec files and directories |
| `watch` | Live validation on file changes |
| `format` | Display DSL grammar reference |
| `scaffold` | Generate skeleton files |
| `inspect` | Display structured metadata |
| `guide` | Development reference guides |
| `coverage` | Behavior coverage report |
| `graph` | Dependency graph visualization |

### `validate` -- Validate spec files

```
minter validate [--deep] <FILES>...
```

Validate one or more `.spec` or `.nfr` files, or entire directories. Directories are scanned recursively and are always validated in deep mode (dependency resolution + NFR cross-validation).

```bash
minter validate specs/user-auth.spec           # single file
minter validate specs/auth.spec specs/pay.spec  # multiple files
minter validate specs/                          # entire directory (always deep)
```

**Output:**

```
✓ user-auth v1.0.0 (3 behaviors)
✓ payment v2.1.0 (7 behaviors)
```

Errors print to stderr with line numbers:

```
specs/broken.spec: line 12: Expected 'when' section before 'then'
```

**`--deep` flag:** When validating individual files, use `--deep` to resolve the full dependency tree and cross-validate NFR references against `.nfr` files in the directory.

```bash
minter validate --deep specs/payment.spec
```

```
✓ payment v2.1.0 (7 behaviors)
├── ✓ user-auth v1.0.0 (3 behaviors)
└── ✓ stripe-api v3.2.1 (8 behaviors)
    └── user-auth v1.0.0 (already shown)
2 dependencies resolved
```

Deep mode maintains a graph cache at `.minter/graph.json`. On subsequent runs, unchanged specs are skipped based on SHA-256 content hashing.

| Exit code | Meaning |
|-----------|---------|
| `0` | All specs valid |
| `1` | One or more validation failures |

### `watch` -- Live validation on file changes

```
minter watch <PATH>
```

Watch a file or directory for `.spec` and `.nfr` file changes and re-validate incrementally. Supports both single-file watching and directory watching (recursive).

```bash
minter watch specs/
```

```
✓ user-auth v1.0.0 (3 behaviors)
✓ payment v2.1.0 (7 behaviors)
watching specs/
```

On file change:

```
changed: payment.spec
✓ payment v2.1.0 (7 behaviors)
```

Watch mode uses colored output (suppressed by `NO_COLOR` env var):

| Color | Meaning |
|-------|---------|
| Green | Success checkmark |
| Red | Failure cross, deleted files |
| Yellow | Changed files |
| Cyan | Watching banner, new files |

The dependency graph is maintained in memory. Rapid saves are debounced (300ms). When watching a single file, dependents are automatically re-validated. Press `Ctrl+C` to stop -- the graph is saved before exit.

### `format` -- Display DSL grammar reference

```
minter format <TYPE>
```

Print the full grammar reference for a spec type.

| Type | Description |
|------|-------------|
| `spec` | Functional requirement grammar (behaviors, given/when/then) |
| `nfr` | Non-functional requirement grammar (constraints, metric/rule) |

```bash
minter format spec
minter format nfr
```

### `scaffold` -- Generate skeleton files

```
minter scaffold <TYPE> [CATEGORY]
```

Generate a ready-to-edit template on stdout.

| Usage | Description |
|-------|-------------|
| `minter scaffold spec` | FR spec template with example behavior |
| `minter scaffold nfr <category>` | NFR spec template for the given category |

Valid NFR categories: `performance`, `reliability`, `security`, `observability`, `scalability`, `cost`, `operability`.

```bash
minter scaffold spec > specs/my-feature.spec
minter scaffold nfr performance > specs/performance.nfr
```

### `inspect` -- Display structured metadata

```
minter inspect <FILE>
```

Parse a `.spec` or `.nfr` file and display structured metadata: name, version, behavior/constraint counts, category distribution, dependencies, and assertion types.

```bash
minter inspect specs/user-auth.spec
```

```
user-auth v1.0.0
title: User Authentication

3 behaviors
  error_case: 1
  happy_path: 2

dependencies:
  session-store >= 1.0.0

assertion types:
  equals: 3
  is_present: 1
```

For `.nfr` files:

```bash
minter inspect specs/performance.nfr
```

```
performance v1.0.0
title: Performance Requirements

2 constraints
  metric: 1
  rule: 1

category: performance

no dependencies
```

### `guide` -- Development reference guides

```
minter guide [TOPIC]
```

Print a condensed reference guide for a specific topic. Run `minter guide` without arguments to list available topics with descriptions.

| Topic | Description |
|-------|-------------|
| `methodology` | Full spec-driven development methodology |
| `workflow` | Five-phase development workflow |
| `authoring` | Spec authoring: granularity, decomposition, entity format |
| `smells` | Requirements smell detection (ambiguity, Observer Test, Swap Test) |
| `nfr` | NFR design: categories, constraints, FR/NFR decision tree |
| `context` | Context management protocol for lazy loading specs |
| `coverage` | Coverage tagging guide for linking tests to spec behaviors |

```bash
minter guide                # List available topics
minter guide methodology    # Full methodology reference
minter guide workflow       # Quick workflow phase reference
minter guide coverage       # How to tag tests for coverage tracking
```

### `coverage` -- Behavior coverage report

```
minter coverage [--scan <DIR>...] [--format <FORMAT>] [--verbose] <SPEC_PATH>
```

Scan project files for `@minter` tags in comments and cross-reference them against the spec graph to produce a behavior coverage report.

```bash
minter coverage specs/                          # Scan cwd for tags against all specs
minter coverage specs/my-feature.spec           # Coverage for a single spec
minter coverage specs/ --scan tests/            # Only scan tests/ directory
minter coverage specs/ --scan tests/ --scan e2e/  # Multiple scan directories
minter coverage specs/ --format json            # Machine-readable JSON output
minter coverage specs/ --verbose                # Expand all specs (show individual behaviors)
```

**Tag format:**

```
// @​minter:<type> <behavior> [<behavior>...]    — behavioral test
// @​minter:benchmark #<category>#<constraint>   — NFR benchmark
```

Valid types: `unit`, `integration`, `e2e`, `benchmark`. Tags work in `//` and `#` style comments.

```typescript
// @​minter:e2e login-user login-wrong-password
describe("authentication", () => { /* ... */ });

// @​minter:benchmark #performance#api-response-time
bench("create task latency", () => { /* ... */ });
```

**Output:**

Fully covered specs collapse to one line; specs with gaps expand to show individual behaviors:

```
Behavior Coverage
  ✓ user-auth v1.0.0  4/4 [e2e]

task-management v1.0.0
  ✓ create-task [e2e]
  ✓ list-tasks [e2e]
  ✓ complete-task [e2e]
  ✗ create-task-unauthenticated uncovered
  ✗ complete-nonexistent-task uncovered
```

NFR coverage is derived automatically from the spec graph — if a covered behavior references an NFR constraint, that constraint has indirect coverage. No configuration file is required. The scanner respects `.gitignore`.

| Exit code | Meaning |
|-----------|---------|
| `0` | All behaviors covered, no tag errors |
| `1` | Uncovered behaviors or tag validation errors |

### `graph` -- Dependency graph visualization

```
minter graph [--impacted <NAME>] <DIR>
```

Build and display the full dependency graph for all specs in a directory, including NFR references.

```bash
minter graph specs/
```

```
3 specs, 11 behaviors, 2 NFR categories, 5 constraints

checkout v1.0.0 (4 behaviors)
├── payment v2.1.0 (7 behaviors)
│   └── user-auth v1.0.0 (3 behaviors)
└── [nfr] performance v1.0.0 (2 constraints)
    ├── #api-response-time
    └── #db-query-time
```

The summary header shows total spec, behavior, NFR category, and constraint counts.

**`--impacted` flag:** Show which specs would be affected if a given spec or NFR category changes (reverse dependency analysis via BFS).

```bash
minter graph --impacted user-auth specs/
```

```
impacted by user-auth v1.0.0 (3 behaviors)
├── checkout v1.0.0 (4 behaviors)
└── payment v2.1.0 (7 behaviors)
```

Works for NFR categories too:

```bash
minter graph --impacted performance specs/
```

```
impacted by [nfr] performance v1.0.0 (2 constraints)
└── checkout v1.0.0 (4 behaviors)
```

## The `.spec` format

A `.spec` file describes a unit of behavior using a structured, indentation-sensitive DSL (2-space indent; tabs are rejected).

```
spec user-auth v1.0.0
title "User Authentication"

description
  Handles user login and session creation.

motivation
  A clear spec ensures consistent auth behavior.

nfr
  performance
  security#tls-required

behavior login-with-email [happy_path]
  "User logs in with correct credentials"

  nfr
    performance#api-response-time < 500ms

  given
    @user = User { id: "550e8400", email: "alice@example.com" }
    The user exists in the database

  when authenticate
    email = "alice@example.com"
    password = "secret123"

  then returns session
    assert id is_present
    assert user_id == @user.id

behavior login-invalid-password [error_case]
  "User provides incorrect password"

  given
    The user exists in the database

  when authenticate
    email = "alice@example.com"
    password = "wrong"

  then emits stderr
    assert output contains "invalid credentials"

depends on session-store >= 1.0.0
```

**Structure order:** `spec` header, `title`, `description`, `motivation`, optional `nfr` section, one or more `behavior` blocks, optional `depends on` declarations.

**Behavior categories:** `happy_path` (at least one required), `error_case`, `edge_case`.

**Assertion operators:** `==`, `is_present`, `contains`, `in_range`, `matches_pattern`, `>=`, or prose (no operator).

For the complete grammar, run `minter format spec` or see [docs/reference.md](docs/reference.md).

## The `.nfr` format

A `.nfr` file defines non-functional quality constraints for one of seven categories: `performance`, `reliability`, `security`, `observability`, `scalability`, `cost`, `operability`.

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

  violation critical
  overridable yes


constraint tls-required [rule]
  "All endpoints must use TLS"

  rule
    Every HTTP endpoint must redirect to HTTPS or reject plain HTTP.

  verification
    static "Check nginx/load balancer config for TLS termination"
    runtime "Attempt plain HTTP connection and verify rejection"

  violation critical
  overridable no
```

**Constraint types:** `metric` (measurable threshold with operator) and `rule` (binary pass/fail policy).

**Threshold operators:** `<`, `>`, `<=`, `>=`, `==`.

**Severity levels:** `critical`, `high`, `medium`, `low`.

For the complete grammar, run `minter format nfr` or see [docs/reference.md](docs/reference.md).

## NFR cross-references

Specs reference NFR constraints via an `nfr` section at two levels.

### Spec-level references

Apply constraints to all behaviors. Supports whole-file and anchor references:

```
nfr
  performance                    # whole-file: imports all constraints
  security#tls-required          # anchor: targets one constraint
```

### Behavior-level references

Pin a specific constraint to one behavior. Supports optional overrides:

```
behavior fast-checkout [happy_path]
  "Checkout completes quickly"

  nfr
    performance#api-response-time < 200ms
```

### Override rules

A behavior-level override tightens a metric constraint's threshold. All four conditions must hold:

1. The constraint is marked `overridable yes`
2. The constraint type is `metric` (not `rule`)
3. The override operator matches the original threshold operator
4. The override value is stricter than the default

### Containment rule

Every NFR category referenced at behavior-level must also appear in the spec-level `nfr` section. This ensures the spec header acts as a table of contents for NFR categories in scope.

## Semantic rules

Beyond syntax, minter enforces these rules during validation:

### Functional requirement (.spec) rules

| Rule | Description |
|------|-------------|
| `kebab-case-name` | Spec name must be lowercase alphanumeric with hyphens, no leading/trailing/double hyphens |
| `valid-semver` | Version must be valid semantic versioning (e.g. `1.0.0`) |
| `unique-behavior-names` | No duplicate behavior names within a spec |
| `at-least-one-happy-path` | Every spec must have at least one `happy_path` behavior |
| `unique-aliases` | No duplicate alias names within a behavior's `given` section |
| `alias-refs-resolve` | All `@alias.field` references in `when` and `then` must point to a declared alias |
| `nfr-containment` | Behavior-level NFR categories must appear in the spec-level `nfr` section |

### Non-functional requirement (.nfr) rules

| Rule | Description |
|------|-------------|
| `valid-semver` | Version must be valid semantic versioning |
| `unique-constraint-names` | No duplicate constraint names within an NFR file |
| `kebab-case-constraint-name` | Constraint names must be valid kebab-case |

### Cross-reference rules (deep mode only)

| Rule | Description |
|------|-------------|
| Category exists | Referenced NFR category must have a corresponding `.nfr` file |
| Anchor exists | Referenced constraint name must exist in the NFR file |
| Override requires overridable | Constraint must be marked `overridable yes` |
| Override requires metric | Only metric constraints can be overridden (not rule) |
| Operator must match | Override operator must match the original threshold operator |
| Value must be stricter | Override value must be stricter than the default threshold |

Value normalization for strictness comparison: `s` to `ms` (x1000), `MB` to `KB` (x1000), `GB` to `KB` (x1000000). Bare numbers and `%` are compared directly.

## Graph cache

Minter maintains a graph cache at `.minter/graph.json` in the current working directory (schema version 3). The cache stores:

- SHA-256 content hashes for each spec and NFR file
- Version, behavior count, dependency edges, and NFR category references
- Validation status per spec

**Behavior:**

- Created automatically on first `--deep` validation, directory validation, or `watch`
- Unchanged specs are skipped on subsequent runs (hash-based change detection)
- NFR changes invalidate all specs that reference the changed category
- Stale entries (deleted files) are pruned automatically
- Auto-rebuilt from scratch if corrupted or schema-incompatible
- Written atomically (temp file + rename)

Add `.minter/` to your `.gitignore`.

## MCP integration

Minter ships a separate `minter-mcp` binary that exposes minter's capabilities as an [MCP](https://modelcontextprotocol.io/) server over stdio. This allows AI agents and IDEs to validate specs, scaffold files, inspect metadata, and explore the dependency graph programmatically.

**Tools provided:** `validate`, `inspect`, `scaffold`, `format`, `graph`, `coverage`, `initialize_minter`, `guide`.

The MCP server supports both file-path and inline-content modes for validation and inspection, making it usable in environments where specs are not yet written to disk.

```jsonc
// Example: Claude Desktop or Cursor MCP config
{
  "mcpServers": {
    "minter": {
      "command": "minter-mcp"
    }
  }
}
```

## Supported platforms

| Platform | Target |
|----------|--------|
| macOS ARM64 (Apple Silicon) | `aarch64-apple-darwin` |
| macOS x86_64 (Intel) | `x86_64-apple-darwin` |
| Linux x86_64 | `x86_64-unknown-linux-gnu` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` |
| Windows x86_64 | `x86_64-pc-windows-msvc` |

## Changelog

See [Releases](https://github.com/arnaudlewis/minter/releases) for version history and changelogs.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, testing, and contribution guidelines.

## License

MIT. See [LICENSE](LICENSE).
