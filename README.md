# minter

A CLI tool for validating structured specification files. Parses a custom `.spec` DSL that defines behavioral contracts (given/when/then), enforces semantic rules, resolves dependency graphs across specs, and provides a watch mode for instant feedback during authoring.

## Installation

**Requirements:** Rust 1.85+

```bash
git clone git@github.com:arnaudlewis/minter.git
cd minter
cargo install --path .
```

Verify the installation:

```bash
minter --version
```

## Commands

### `validate` — Validate spec files

```
minter validate [OPTIONS] <FILES>...
```

Validate one or more `.spec` files or entire directories. Directories are scanned recursively — specs organized in subdirectories are automatically discovered.

```bash
# Single file
minter validate specs/validation/user-auth.spec

# Multiple files
minter validate specs/validation/auth.spec specs/caching/cache.spec

# Entire directory (recursive)
minter validate specs/
```

**Output:**

```
✓ user-auth v1.0.0 (3 behaviors)
✓ payment v2.1.0 (7 behaviors)
```

Errors are printed to stderr with line numbers:

```
specs/broken.spec: line 12: Expected 'when' section before 'then'
```

#### `--deps` — Resolve and validate dependencies

When a spec declares `depends on`, use `--deps` to resolve the full dependency tree. Dependencies are resolved by name from any `.spec` file in the directory tree — specs in different subdirectories can depend on each other.

```bash
minter validate --deps specs/payment.spec
```

```
✓ payment v2.1.0 (7 behaviors)
├── ✓ user-auth v1.0.0 (3 behaviors)
└── ✓ stripe-api v3.2.1 (8 behaviors)
    └── user-auth v1.0.0 (already shown)
```

With `--deps`, minter also maintains a graph cache in `.minter/graph.json` at your working directory. On subsequent runs, unchanged specs are skipped based on content hashing — only modified files and their dependents are re-validated.

**Exit codes:**

| Code | Meaning |
|------|---------|
| `0`  | All specs valid |
| `1`  | One or more validation failures |

### `watch` — Live validation on file changes

```
minter watch <DIR>
```

Starts a long-running process that monitors a directory (including subdirectories) and re-validates on every file change.

```bash
minter watch specs/
```

```
✓ user-auth v1.0.0 (3 behaviors)
✓ payment v2.1.0 (7 behaviors)
watching specs/
```

When you edit and save a file:

```
changed: payment.spec
✓ payment v2.1.0 (7 behaviors)
```

When a file has errors:

```
changed: payment.spec
✗ payment
specs/payment.spec: line 42: Expected 'when' section before 'then'
```

Watch mode uses colored output to help you scan results at a glance:

| Color  | Used for |
|--------|----------|
| Green  | `✓` success checkmark |
| Red    | `✗` failure cross, deleted file events |
| Yellow | Changed file events |
| Cyan   | Watching banner, new file events |

The dependency graph is kept hot in memory. Rapid successive saves are debounced into a single validation cycle. Press `Ctrl+C` to stop — the graph is saved to `.minter/graph.json` before exit.

## The `.spec` format

A `.spec` file describes a unit of behavior using a structured, indentation-sensitive DSL.

### Minimal example

```
spec user-auth v1.0.0
title "User Authentication"

description
  Handles user login and session creation.

motivation
  A clear spec ensures consistent auth behavior.

behavior login-with-email [happy_path]
  "User logs in with correct credentials"

  given
    @user = User { id: "550e8400", email: "alice@example.com" }
    The user exists in the database

  when authenticate
    email = "alice@example.com"
    password = "secret123"

  then returns session
    assert id is_present
    assert user_id == @user.id

  then emits process_exit
    assert code == 0


behavior login-invalid-password [error_case]
  "User provides incorrect password"

  given
    The user exists in the database

  when authenticate
    email = "alice@example.com"
    password = "wrong"

  then emits stderr
    assert output contains "invalid credentials"

  then emits process_exit
    assert code == 1

depends on session-store >= 1.0.0
```

### Structure

Every `.spec` file follows this order:

```
spec <name> v<version>          # required, kebab-case name, semver version
title "<title>"                 # required, quoted string

description                     # required, followed by indented text
  ...

motivation                      # required, followed by indented text
  ...

behavior <name> [<category>]    # one or more behaviors
  "<description>"
  given ...
  when ...
  then ...

depends on <spec> >= <version>  # optional, at the end
```

Lines starting with `#` are comments. Blank lines are ignored.

### Behaviors

Each behavior has a name, a category, and a quoted description.

**Categories** (one of):

- `happy_path` — Success scenarios (at least one required per spec)
- `error_case` — Expected error conditions
- `edge_case` — Boundary or unusual conditions

### Given (preconditions)

Prose statements and/or alias declarations:

```
given
  The system is ready                                         # prose
  @user = User { id: "550e8400", name: "Alice" }              # alias
```

Aliases define test fixtures. Entity names must start with an uppercase letter. Properties are comma-separated `key: "value"` pairs.

### When (action)

The action name follows `when` on the same line. Inputs are indented below:

```
when create_item
  name = "test"
  owner_id = @user.id          # reference an alias field
```

### Then (postconditions)

Multiple `then` blocks per behavior are allowed.

**Kinds:**

```
then returns <description>      # return value
then emits <target>             # output (stdout, stderr, process_exit, or custom)
then side_effect                # database / system mutations
then                            # plain postcondition
```

### Assertions

Inside a `then` block:

```
assert name == "test"                    # equality
assert created_by == @user.id            # reference equality
assert id is_present                     # presence
assert output contains "done"            # substring / membership
assert count in_range 1..100             # range
assert email matches_pattern "^.+@.+$"   # regex
assert count >= 2                        # comparison
assert the system is in a valid state    # prose (no operator)
```

### Dependencies

Declared at the end of the file. Version constraint uses `>=`:

```
depends on user-auth >= 1.0.0
depends on session-store >= 2.0.0
```

Dependencies are resolved by name from any `.spec` file in the directory tree when using `--deps`. Spec names must be globally unique across the tree.

## Directory layout

Specs can be organized into subdirectories. All commands (`validate`, `validate --deps`, `watch`) scan recursively.

```
specs/
  cli.spec
  validation/
    validate-spec.spec
    validate-dependencies.spec
    validate-display.spec
    dsl-format.spec
  caching/
    graph-cache.spec
    incremental-validation.spec
  watch/
    watch-mode.spec
```

Spec names must be globally unique across the entire tree. If two `.spec` files in different subdirectories share the same stem name, minter reports an error with both file paths.

## Semantic rules

Beyond syntax, minter enforces:

| Rule | Description |
|------|-------------|
| Kebab-case name | Spec name must be lowercase with hyphens, no leading/trailing/double hyphens |
| Valid semver | Version must be valid semantic versioning |
| Unique behavior names | No duplicate behavior names within a spec |
| At least one happy_path | Every spec must have at least one `happy_path` behavior |
| Unique aliases | No duplicate alias names within a behavior's given section |
| Alias refs resolve | All `@alias.field` references must point to a declared alias |

## Graph cache

When using `--deps` or `watch`, minter maintains a `.minter/graph.json` file at your current working directory (not inside the specs directory). This cache stores content hashes and dependency edges so that unchanged specs can be skipped on subsequent runs.

- Created automatically on first `--deps` run
- Updated when spec files change
- Auto-rebuilt if corrupted or schema-incompatible
- Ignored when running `validate` without `--deps`

Add `.minter` to your `.gitignore`.

## Running tests

```bash
cargo test
```

## License

MIT
