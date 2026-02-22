# specval — Spec Validator CLI

Validates .spec files — parses the DSL, runs semantic rules, reports pass/fail. The deterministic gate in a spec-driven development pipeline.

## Why This Exists

Specs are the source of truth in spec-driven development. Code is compiled output. But specs are written by LLMs (non-deterministic), and consumed by LLMs (non-deterministic). The only way to guarantee consistency is a deterministic validation gate between spec-writer and downstream agents.

`specval` is that gate.

```
Human intent → .spec (DSL) → specval validate (deterministic) → downstream agents read .spec
```

There is no intermediate JSON. The .spec DSL file is the artifact that flows through the entire pipeline. LLMs read it natively — it's concise and human-friendly by design. Validation happens on the parsed model internally; the output is pass/fail, not a second file format.

## Core Principle: One Primitive

The spec format has exactly one primitive: **behavioral specs that depend on other behavioral specs**.

There is no type system — data shapes are specs whose behaviors describe what valid instances look like. There is no error catalog — error behavior is expressed directly as behaviors. There is no constraint system — invariants decompose into behaviors with specific preconditions and assertions.

Everything is given/when/then. One concept to learn, one concept to validate, one concept to generate tests from.

## Usage

```bash
specval validate feature.spec             # validate one spec
specval validate specs/*.spec             # validate multiple specs
```

Exit code 0 = valid. Exit code 1 = invalid with clear error messages including line numbers.

## The Spec Format

### Top-level Structure

```
spec <name> v<version>
title "<human title>"

description
  <multiline text>

motivation
  <multiline text>

---

behavior <name> [<category>]
  "<description>"
  ...

---

depends on <spec-name> >= <version>
```

| Section | Required | Purpose |
|---------|----------|---------|
| `spec` | yes | Name (kebab-case) and semver version |
| `title` | yes | Human-readable title |
| `description` | yes | What this feature does |
| `motivation` | yes | Why this feature exists |
| `behaviors` | yes | Given/when/then definitions (the core) |
| `dependencies` | no | Other specs this depends on |

### Behaviors (the core — drives test generation)

Each behavior is a testable unit: given preconditions, when action, then expected observable outcomes.

```
behavior create-note-success [happy_path]
  "Successfully create a new note with title and body"

  given
    An authenticated user with a workspace
    @the_user = User { id: "550e8400-...", workspace_id: "660e8400-..." }

  when create_note
    title = "Meeting Notes"
    user_id = @the_user.id

  then emits response
    assert id is_present
    assert title == "Meeting Notes"
    assert created_by == @the_user.id

  then side_effect
    assert Note entity created with title == "Meeting Notes"
```

**Behavior categories:** `happy_path`, `error_case`, `edge_case`

**Postcondition kinds:**
- `returns` — asserts on return value
- `side_effect` — asserts on state changes
- `emits` — asserts on observable outputs (stdout, stderr, process_exit, events, responses)

**Assertion kinds:** `equals`, `equals_ref` (from alias), `is_present`, `matches_pattern`, `contains`, `in_range`

**Referential chaining:** `given` sets up entities with `@alias` → `when` inputs reference via `@alias.field` → `then` assertions reference via `@alias.field`. This enables mechanical test generation: setup → action → assert.

### Dependencies (specs depend on specs)

Dependencies link specs together. A dependency is a reference to another behavioral spec. The test generator reads the dependency's behaviors to understand how to satisfy preconditions.

```
depends on user-auth >= 1.0.0
```

Example: `create-note.spec` depends on `user-auth.spec`. The given section says "An authenticated user exists." The test generator reads `user-auth.spec` to find the `authenticate-success` behavior and understands how to set up that precondition.

Data shapes emerge from behavioral specs rather than being declared as types:

| Traditional approach | Spec-driven approach |
|---------------------|---------------------|
| `type User { id: uuid, email: string }` | `user-entity.spec` with behaviors defining what a valid user looks like |
| `type Note { id: uuid, title: string }` | `note-entity.spec` with behaviors on creation, validation, etc. |
| Consuming spec imports `User` type | Consuming spec `depends on user-entity >= 1.0.0` |

The test generator derives the data shape from the dependency's behaviors — no schema duplication, no type system to maintain.

## Validation

### Layer 1: Parse (DSL syntax)
The DSL parser rejects: missing required sections, unknown keywords, malformed behavior blocks, unclosed quotes, invalid behavior categories.

All parse errors include line numbers pointing to the .spec source.

### Layer 2: Semantic (7 rules)
1. Aliases are unique within each behavior
2. Alias references (`@alias.field`) resolve to declared aliases
3. Behavior names are unique across the spec
4. `spec` version is valid semver
5. `spec` name is kebab-case (`^[a-z][a-z0-9]*(-[a-z0-9]+)*$`)
6. At least one `happy_path` behavior exists
7. Behavior categories are one of: `happy_path`, `error_case`, `edge_case`

Parse errors stop validation — semantic rules only run on successfully parsed specs.

## Tech Stack

- **Language:** Rust
- **DSL parsing:** pest (PEG grammar)
- **CLI:** clap (derive)
- **Error reporting:** miette (fancy diagnostics with line numbers)
- **Error types:** thiserror

## Project Structure

```
specval/
├── Cargo.toml
├── src/
│   ├── main.rs                  # CLI entry point
│   ├── lib.rs                   # Re-exports
│   ├── model/
│   │   ├── mod.rs
│   │   ├── spec.rs              # Top-level Spec
│   │   ├── identity.rs          # Identity (name, version, title)
│   │   ├── context.rs           # Context (description, motivation)
│   │   ├── behavior.rs          # Behavior, Precondition, Action, Postcondition, Assertion
│   │   └── dependency.rs        # Dependency
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── grammar.pest         # PEG grammar definition
│   │   ├── ast.rs               # Abstract syntax tree nodes
│   │   └── compiler.rs          # AST → Spec model
│   ├── validate/
│   │   ├── mod.rs
│   │   └── semantic.rs          # Cross-field validation (7 rules)
│   └── output/
│       ├── mod.rs
│       └── reporter.rs          # Text output with line numbers
├── specs/
│   └── validate-command.spec    # The spec for specval itself (dogfood)
└── tests/
    ├── valid_specs/
    ├── invalid_specs/
    └── integration.rs
```

## Implementation Order

1. Install Rust via rustup
2. Scaffold project with `cargo new specval`
3. Model layer (behavior.rs → simple modules → spec.rs)
4. PEG grammar + parser (grammar.pest → AST → Spec model)
5. Semantic validation (7 rules)
6. CLI (clap, exit codes, output with line numbers)
7. Test fixtures (valid + invalid .spec files)
8. Tests (unit + integration)

## Relationship to Agent Team Redesign

This tool is the first primitive of the v3.0 agent team. Once it exists:

- The **spec-writer agent** writes .spec files → `specval validate` gates them
- The **test-generator agent** reads validated .spec files → consistent input = consistent tests
- The **implementer agent** receives .spec + tests → builds code that passes tests
- The **benchmark system** measures spec conformance via test pass rates

The spec format defined here IS the contract between all agents. The tool enforces it.
