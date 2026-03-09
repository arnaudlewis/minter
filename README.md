# minter

The deterministic validation gate for spec-driven development.

Minter parses a custom `.spec` DSL that defines behavioral contracts (given/when/then) and a `.nfr` DSL that defines non-functional requirements as measurable constraints. It enforces semantic rules, resolves dependency graphs, cross-validates NFR references, and provides watch mode for instant feedback during authoring.

The spec format has exactly one primitive: **behavioral specs that depend on other behavioral specs**. There is no type system -- data shapes are specs whose behaviors describe what valid instances look like. There is no error catalog -- error behavior is expressed directly as behaviors. Everything is given/when/then. One concept to learn, one concept to validate, one concept to generate tests from.

```
Human intent --> .spec (DSL) --> minter validate (deterministic) --> downstream agents read .spec
```

## Install

### Homebrew (macOS and Linux)

```bash
brew install arnaudlewis/tap/minter
```

### Manual download

Download the latest binary for your platform from [Releases](https://github.com/arnaudlewis/minter-releases/releases), extract, and place `minter` and `minter-mcp` on your `PATH`.

Verify:

```bash
minter --version
```

## Quick Start

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

Scan project files for `@minter` tags in comments and cross-reference them against the spec graph to produce a behavior coverage report. Tags placed in test files declare which spec behaviors a test covers.

Fully covered specs are collapsed to a single summary line by default. Use `--verbose` to expand all specs and show individual behaviors.

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
// @minter:<type> <behavior> [<behavior>...]    — behavioral test
// @minter:benchmark #<category>#<constraint>   — NFR benchmark
```

Valid types: `unit`, `integration`, `e2e`, `benchmark`. Tags are placed in comments (`//` or `#` style) above test blocks.

```typescript
// @minter:e2e report-full-coverage report-partial-coverage
describe("coverage report", () => { /* ... */ });

// @minter:benchmark #performance#validation-latency
bench("validate 100 specs", || { /* ... */ });
```

**Output:**

```
Behavior Coverage
  ✓ a v1.0.0  3/3 [unit, e2e]

b v2.0.0
  ✓ do-thing [unit]
  ✗ do-other uncovered

Summary: 4/5 behaviors covered (80%)
  unit: 2
  e2e: 1
```

Fully covered specs (like `a`) collapse to one line. Specs with gaps (like `b`) expand to show every behavior.

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

## License

Proprietary. See [LICENSE](LICENSE).
