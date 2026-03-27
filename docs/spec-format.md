# Spec Format (.spec)

The `.spec` file format defines functional behavioral contracts using a structured, indentation-sensitive DSL.

**Indentation:** 2 spaces. Tab characters are rejected.

---

## File structure

Sections must appear in this exact order:

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

depends on <spec-name> >= <version>    # zero or more, at end
```

---

## Header

```
spec user-auth v1.0.0
title "User Authentication"
```

- **name** — kebab-case: lowercase ASCII alphanumeric and hyphens. No leading, trailing, or consecutive hyphens.
- **version** — valid semver (e.g., `1.0.0`, `2.1.0-beta.1`). The `v` prefix is required in the file.
- **title** — double-quoted human-readable name.

---

## Description and motivation

```
description
  Handles user login and session creation for
  all client types.

motivation
  A clear spec ensures consistent auth behavior
  across web, mobile, and API clients.
```

Both sections are required. Each body line is indented 2 spaces. Lines are joined with newlines; leading/trailing whitespace is trimmed per line.

---

## NFR section (spec-level, optional)

```
nfr
  performance                    # whole-file: all constraints apply
  security#tls-required          # anchor: one specific constraint
```

The spec-level `nfr` section declares which NFR categories are in scope for this spec. It acts as a table of contents — every behavior-level NFR reference must appear here (the containment rule).

Two reference forms:

| Form | Meaning |
|------|---------|
| `<category>` | Whole-file — imports all constraints from the `.nfr` file |
| `<category>#<constraint>` | Anchor — targets one named constraint |

Overrides are not allowed at spec level (parse error). See [nfr-cross-reference.md](nfr-cross-reference.md).

---

## Behaviors

```
behavior <name> [<category>]
  "<description>"

  nfr                                        # optional
    <category>#<constraint>
    <category>#<constraint> <op> <value>     # override

  given
    <preconditions>

  when <action>
    <inputs>

  then returns <type>
    assert <field> <op> <value>

  then emits <channel>
    assert <field> <op> <value>

  then side_effect
    assert <prose>
```

At least one behavior is required. At least one must be `happy_path`.

### Behavior categories

| Category | Description |
|----------|-------------|
| `happy_path` | Expected successful outcome. At least one required. |
| `error_case` | Expected error condition. |
| `edge_case` | Boundary or unusual condition. |

### Behavior-level NFR section (optional)

Appears after the description, before `given`. Only anchor references allowed — whole-file references are a parse error at behavior level.

```
  nfr
    performance#api-response-time              # pin constraint
    performance#api-response-time < 200ms      # override threshold
```

See [nfr-cross-reference.md](nfr-cross-reference.md) for override rules.

### Given section

Declares preconditions. Two forms:

**Prose precondition:**

```
  given
    The user exists in the database
    The session store is available
```

**Alias declaration** — defines a named test fixture:

```
  given
    @user = User { id: "550e8400", email: "alice@example.com", role: "member" }
    The user is not suspended
```

Alias syntax: `@<name> = <Entity> { <key>: <value>, ... }`. Entity type name must start with uppercase. Properties are comma-separated `key: value` pairs.

### When section

The action under test:

```
  when authenticate
    email = "alice@example.com"
    password = "secret123"
```

Inputs can reference alias fields:

```
  when authenticate
    email = @user.email
    password = "secret123"
```

Alias references use `@<alias>.<field>`. Both parts must be non-empty and the alias must be declared in `given`.

### Then section

At least one `then` block required. Multiple allowed.

```
  then returns session
    assert id is_present
    assert user_id == @user.id

  then emits user-logged-in
    assert user_id is_present
    assert timestamp is_present

  then side_effect
    assert the audit log records the login event
```

Three postcondition kinds:

| Kind | When to use |
|------|-------------|
| `then returns <type>` | The action returns a value |
| `then emits <channel>` | The action emits an event |
| `then side_effect` | The action produces a side effect |

### Assertion operators

| Operator | Example |
|----------|---------|
| `==` | `assert status == "active"` |
| `==` (alias ref) | `assert user_id == @user.id` |
| `is_present` | `assert session_token is_present` |
| `contains` | `assert message contains "invalid credentials"` |
| `in_range` | `assert score in_range 0..100` |
| `matches_pattern` | `assert id matches_pattern "^[a-f0-9]{8}"` |
| `>=` | `assert version >= "2.0.0"` |
| prose | `assert the email queue receives one message` |

Prose assertions (no recognized operator) are valid but should be kept below 20% of total assertions. See [authoring-guide.md](authoring-guide.md#smells).

---

## Dependencies

```
depends on session-store >= 1.0.0
depends on notification-service >= 2.1.0
```

Zero or more. Must appear at the end of the file, after all behaviors. Each dependency specifies a spec name and a minimum version. Dependency resolution uses sibling `.spec` files in the same directory.

Resolution rules:
- Cycle detection: a spec that depends on itself (directly or transitively) is an error.
- Depth limit: resolution stops at depth 256.

---

## Annotated example

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

---

See also:
- [nfr-format.md](nfr-format.md) — the `.nfr` grammar
- [nfr-cross-reference.md](nfr-cross-reference.md) — NFR binding and overrides
- [authoring-guide.md](authoring-guide.md) — how to write good behaviors
- `minter format spec` — print the grammar reference in the terminal
